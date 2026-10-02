-- R9.67 durable Host-bound tenant browser sessions for the commercial path.
-- Requires 0001,0003,0004,0012-0015. Session secrets themselves are NEVER
-- stored in PostgreSQL: only lower-case SHA-256 digests are persisted.
-- No HTTP route, IdP client secret, tenant grant or public listener is created.
BEGIN;

-- tenant_domains originally has id PK and tenant index; a compound unique key
-- is required for the session FK so a session can never bind a foreign tenant
-- to an otherwise valid domain id.
CREATE UNIQUE INDEX IF NOT EXISTS tenant_domains_tenant_id_id_uq
 ON ipat_platform.tenant_domains(tenant_id,id);

CREATE TABLE ipat_platform.tenant_browser_sessions (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
  domain_id uuid NOT NULL REFERENCES ipat_platform.tenant_domains(id),
  issuer text NOT NULL CHECK(length(issuer) BETWEEN 10 AND 512 AND issuer LIKE 'https://%' AND issuer !~ '[[:space:]]'),
  subject text NOT NULL CHECK(length(subject) BETWEEN 1 AND 128 AND subject ~ '^[a-zA-Z0-9_:/.-]+$'),
  cookie_sha256 text NOT NULL UNIQUE CHECK(cookie_sha256 ~ '^[0-9a-f]{64}$'),
  csrf_sha256 text NOT NULL CHECK(csrf_sha256 ~ '^[0-9a-f]{64}$' AND csrf_sha256 <> cookie_sha256),
  issued_at timestamptz NOT NULL DEFAULT clock_timestamp(),
  last_seen_at timestamptz NOT NULL DEFAULT clock_timestamp(),
  expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  CHECK(expires_at > issued_at AND expires_at <= issued_at + interval '15 minutes'),
  CHECK(revoked_at IS NULL OR revoked_at >= issued_at),
  CONSTRAINT session_domain_same_tenant_fk FOREIGN KEY(tenant_id,domain_id)
    REFERENCES ipat_platform.tenant_domains(tenant_id,id)
);
ALTER TABLE ipat_platform.tenant_browser_sessions OWNER TO ipat_schema_owner;
CREATE INDEX tenant_browser_sessions_subject_current
 ON ipat_platform.tenant_browser_sessions(tenant_id,issuer,subject,expires_at)
 WHERE revoked_at IS NULL;
ALTER TABLE ipat_platform.tenant_browser_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.tenant_browser_sessions FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.tenant_browser_sessions FROM PUBLIC,ipat_app_runtime;

CREATE ROLE ipat_browser_session_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_browser_session_issue_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE ROLE ipat_browser_session_auth_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_browser_session_owner,ipat_browser_session_issue_exec,ipat_browser_session_auth_exec;
GRANT SELECT ON ipat_platform.tenants,ipat_platform.tenant_domains,ipat_platform.identity_memberships
 TO ipat_browser_session_owner;
GRANT SELECT,INSERT,UPDATE(last_seen_at,revoked_at) ON ipat_platform.tenant_browser_sessions TO ipat_browser_session_owner;
CREATE POLICY browser_session_owner_all ON ipat_platform.tenant_browser_sessions
 FOR ALL TO ipat_browser_session_owner USING(true) WITH CHECK(true);
-- Existing identity/domain tables are FORCE RLS. Give this exact function owner
-- read-only policies; API execution roles still cannot inspect either table.
CREATE POLICY browser_session_owner_identity_read ON ipat_platform.identity_memberships
 FOR SELECT TO ipat_browser_session_owner USING(true);
CREATE POLICY browser_session_owner_domain_read ON ipat_platform.tenant_domains
 FOR SELECT TO ipat_browser_session_owner USING(true);

-- R9.14 fixed lifecycle bug: after 0014, 'verified' only proves ownership.
-- A Host MUST NOT resolve into application tenant context until routing + TLS
-- are independently attested and activation_state is exactly active.
CREATE OR REPLACE FUNCTION ipat_platform.resolve_active_tenant_domain(p_hostname text)
RETURNS TABLE(tenant_id uuid,tenant_slug text,hostname text,domain_type text)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT t.id,t.tenant_slug,d.hostname,d.domain_type
 FROM ipat_platform.tenant_domains d JOIN ipat_platform.tenants t ON t.id=d.tenant_id
 WHERE d.hostname=p_hostname
   AND d.verification_state='verified'
   AND d.activation_state='active'
   AND d.disabled_at IS NULL
   AND t.state='active'
 LIMIT 1
$body$;
ALTER FUNCTION ipat_platform.resolve_active_tenant_domain(text) OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.resolve_active_tenant_domain(text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.resolve_active_tenant_domain(text) TO ipat_domain_reader;

CREATE FUNCTION ipat_platform.resolve_active_tenant_domain_binding(p_hostname text)
RETURNS TABLE(domain_id uuid,tenant_id uuid,hostname text)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT d.id,d.tenant_id,d.hostname
 FROM ipat_platform.tenant_domains d JOIN ipat_platform.tenants t ON t.id=d.tenant_id
 WHERE d.hostname=p_hostname
   AND d.verification_state='verified'
   AND d.activation_state='active'
   AND d.disabled_at IS NULL
   AND t.state='active'
 LIMIT 1
$body$;
ALTER FUNCTION ipat_platform.resolve_active_tenant_domain_binding(text) OWNER TO ipat_browser_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.resolve_active_tenant_domain_binding(text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.resolve_active_tenant_domain_binding(text) TO ipat_browser_session_issue_exec;

-- The caller is a dedicated OIDC verifier service. It may invoke this ONLY
-- after cryptographic ID/access pair + nonce + fresh MFA verification. The DB
-- independently requires current membership and exact active Host/domain.
CREATE FUNCTION ipat_platform.issue_tenant_browser_session(
 p_issuer text,p_subject text,p_tenant uuid,p_domain uuid,p_session uuid,
 p_cookie_sha text,p_csrf_sha text,p_expires timestamptz)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE v_now timestamptz:=clock_timestamp();
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_domain IS NULL OR p_session IS NULL
   OR p_cookie_sha !~ '^[0-9a-f]{64}$' OR p_csrf_sha !~ '^[0-9a-f]{64}$' OR p_cookie_sha=p_csrf_sha
   OR p_expires<=v_now OR p_expires>v_now+interval '15 minutes'
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.tenants t WHERE t.id=p_tenant AND t.state='active')
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.tenant_domains d WHERE d.id=p_domain AND d.tenant_id=p_tenant
       AND d.verification_state='verified' AND d.activation_state='active' AND d.disabled_at IS NULL)
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.identity_memberships m
       WHERE m.tenant_id=p_tenant AND m.issuer=p_issuer AND m.subject=p_subject
         AND m.role IN('tenant_admin','noc_engineer','helpdesk','auditor')
         AND m.revoked_at IS NULL AND m.expires_at>v_now)
 THEN RETURN NULL; END IF;
 IF (SELECT count(*) FROM ipat_platform.tenant_browser_sessions s
      WHERE s.tenant_id=p_tenant AND s.issuer=p_issuer AND s.subject=p_subject
        AND s.revoked_at IS NULL AND s.expires_at>v_now AND s.last_seen_at>v_now-interval '5 minutes') >= 8
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_platform.tenant_browser_sessions(
   id,tenant_id,domain_id,issuer,subject,cookie_sha256,csrf_sha256,issued_at,last_seen_at,expires_at)
 VALUES(p_session,p_tenant,p_domain,p_issuer,p_subject,p_cookie_sha,p_csrf_sha,v_now,v_now,p_expires);
 RETURN p_session;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.issue_tenant_browser_session(text,text,uuid,uuid,uuid,text,text,timestamptz)
 OWNER TO ipat_browser_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.issue_tenant_browser_session(text,text,uuid,uuid,uuid,text,text,timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.issue_tenant_browser_session(text,text,uuid,uuid,uuid,text,text,timestamptz)
 TO ipat_browser_session_issue_exec;

-- Host is mandatory and is the tenant selector. There is deliberately NO
-- tenant parameter here: a browser cannot submit X-Tenant or choose tenant id.
-- Mutation requests additionally require the independent CSRF digest.
CREATE FUNCTION ipat_platform.authenticate_tenant_browser_session(
 p_cookie_sha text,p_hostname text,p_csrf_sha text,p_mutation boolean)
RETURNS TABLE(tenant_id uuid,domain_id uuid,issuer text,subject text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE v_now timestamptz:=clock_timestamp(); v ipat_platform.tenant_browser_sessions%ROWTYPE;
BEGIN
 IF p_cookie_sha !~ '^[0-9a-f]{64}$' OR p_hostname IS NULL
   OR length(p_hostname) NOT BETWEEN 3 AND 253 OR p_hostname<>lower(p_hostname)
   OR p_hostname !~ '^[a-z0-9]([a-z0-9.-]*[a-z0-9])$' OR position('..' in p_hostname)>0
   OR p_mutation IS NULL
   OR (p_mutation AND (p_csrf_sha IS NULL OR p_csrf_sha !~ '^[0-9a-f]{64}$'))
 THEN RETURN; END IF;
 SELECT s.* INTO v FROM ipat_platform.tenant_browser_sessions s
 JOIN ipat_platform.tenant_domains d ON d.id=s.domain_id AND d.tenant_id=s.tenant_id
 JOIN ipat_platform.tenants t ON t.id=s.tenant_id
 WHERE s.cookie_sha256=p_cookie_sha AND d.hostname=p_hostname
   AND d.verification_state='verified' AND d.activation_state='active' AND d.disabled_at IS NULL
   AND t.state='active' AND s.revoked_at IS NULL AND s.expires_at>v_now
   AND s.last_seen_at>v_now-interval '5 minutes'
   AND (NOT p_mutation OR s.csrf_sha256=p_csrf_sha)
   AND EXISTS(SELECT 1 FROM ipat_platform.identity_memberships m
       WHERE m.tenant_id=s.tenant_id AND m.issuer=s.issuer AND m.subject=s.subject
         AND m.role IN('tenant_admin','noc_engineer','helpdesk','auditor')
         AND m.revoked_at IS NULL AND m.expires_at>v_now)
 FOR UPDATE OF s;
 IF NOT FOUND THEN RETURN; END IF;
 UPDATE ipat_platform.tenant_browser_sessions s SET last_seen_at=v_now WHERE s.id=v.id;
 tenant_id:=v.tenant_id; domain_id:=v.domain_id; issuer:=v.issuer; subject:=v.subject;
 RETURN NEXT;
END $body$;
ALTER FUNCTION ipat_platform.authenticate_tenant_browser_session(text,text,text,boolean)
 OWNER TO ipat_browser_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.authenticate_tenant_browser_session(text,text,text,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.authenticate_tenant_browser_session(text,text,text,boolean)
 TO ipat_browser_session_auth_exec;

CREATE FUNCTION ipat_platform.revoke_tenant_browser_session(p_cookie_sha text,p_hostname text)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE changed integer;
BEGIN
 IF p_cookie_sha !~ '^[0-9a-f]{64}$' OR p_hostname IS NULL THEN RETURN false; END IF;
 UPDATE ipat_platform.tenant_browser_sessions s SET revoked_at=clock_timestamp()
 FROM ipat_platform.tenant_domains d
 WHERE s.cookie_sha256=p_cookie_sha AND d.id=s.domain_id AND d.tenant_id=s.tenant_id
   AND d.hostname=p_hostname AND s.revoked_at IS NULL;
 GET DIAGNOSTICS changed=ROW_COUNT; RETURN changed=1;
END $body$;
ALTER FUNCTION ipat_platform.revoke_tenant_browser_session(text,text) OWNER TO ipat_browser_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.revoke_tenant_browser_session(text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.revoke_tenant_browser_session(text,text) TO ipat_browser_session_auth_exec;

COMMIT;
