-- R9.80: distinct durable PLATFORM-owner browser session and guarded reservation.
-- Provisioning a console hostname or issuing a signed/fresh-MFA session is
-- EXTERNAL to this migration. Never attach these roles to a tenant login.
BEGIN;
CREATE TABLE ipat_platform.platform_console_hosts(
 hostname text PRIMARY KEY CHECK(
  length(hostname) BETWEEN 4 AND 253
  AND hostname=lower(hostname)
  AND hostname ~ '^[a-z0-9]([a-z0-9.-]*[a-z0-9])$'
  AND hostname LIKE '%.%' AND position('..' in hostname)=0),
 verified_at timestamptz NOT NULL,
 tls_ready_at timestamptz NOT NULL,
 disabled_at timestamptz
);
ALTER TABLE ipat_platform.platform_console_hosts OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.platform_console_hosts ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.platform_console_hosts FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.platform_console_hosts FROM PUBLIC,ipat_app_runtime,
 ipat_tenant_api_exec,ipat_tenant_api_login,ipat_oidc_session_issuer_login;

CREATE TABLE ipat_platform.platform_browser_sessions(
 id uuid PRIMARY KEY,
 issuer text NOT NULL,
 subject text NOT NULL,
 hostname text NOT NULL REFERENCES ipat_platform.platform_console_hosts(hostname),
 cookie_sha256 text NOT NULL UNIQUE CHECK(cookie_sha256 ~ '^[0-9a-f]{64}$'),
 csrf_sha256 text NOT NULL CHECK(csrf_sha256 ~ '^[0-9a-f]{64}$' AND csrf_sha256<>cookie_sha256),
 issued_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 last_seen_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 expires_at timestamptz NOT NULL,
 revoked_at timestamptz,
 CONSTRAINT platform_session_exact_principal FOREIGN KEY(issuer,subject)
   REFERENCES ipat_platform.platform_principals(issuer,subject),
 CONSTRAINT platform_session_expiry CHECK(
  expires_at>issued_at AND expires_at<=issued_at+interval '10 minutes')
);
ALTER TABLE ipat_platform.platform_browser_sessions OWNER TO ipat_schema_owner;
CREATE INDEX platform_browser_session_current ON
 ipat_platform.platform_browser_sessions(issuer,subject,expires_at)
 WHERE revoked_at IS NULL;
ALTER TABLE ipat_platform.platform_browser_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.platform_browser_sessions FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.platform_browser_sessions FROM PUBLIC,ipat_app_runtime,
 ipat_tenant_api_exec,ipat_tenant_api_login,ipat_oidc_session_issuer_login;

CREATE ROLE ipat_platform_session_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
 NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_platform_session_issue_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
 NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_platform_session_api_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
 NOREPLICATION NOBYPASSRLS NOINHERIT;
-- Login identities start without a password. A separate deployment must
-- provision approved peer/cert auth and an actual independent MFA verifier.
CREATE ROLE ipat_platform_session_issuer_login LOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT
 IN ROLE ipat_platform_session_issue_exec;
CREATE ROLE ipat_platform_session_api_login LOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT
 IN ROLE ipat_platform_session_api_exec;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_platform_session_owner,
 ipat_platform_session_issue_exec,ipat_platform_session_api_exec,
 ipat_platform_session_issuer_login,ipat_platform_session_api_login;
GRANT SELECT ON ipat_platform.platform_principals,
 ipat_platform.platform_console_hosts TO ipat_platform_session_owner;
GRANT SELECT,INSERT,UPDATE(last_seen_at,revoked_at)
 ON ipat_platform.platform_browser_sessions TO ipat_platform_session_owner;
CREATE POLICY platform_browser_principal_owner_read
 ON ipat_platform.platform_principals FOR SELECT TO ipat_platform_session_owner USING(true);
CREATE POLICY platform_browser_host_owner_read
 ON ipat_platform.platform_console_hosts FOR SELECT TO ipat_platform_session_owner USING(true);
CREATE POLICY platform_browser_session_owner_all
 ON ipat_platform.platform_browser_sessions FOR ALL TO ipat_platform_session_owner
 USING(true) WITH CHECK(true);
-- Platform session owner may call existing reviewed reservation SECURITY
-- DEFINER functions, but neither the browser login nor the tenant login gets
-- the raw reservation execution privilege.
GRANT EXECUTE ON FUNCTION ipat_platform.reserve_suspended_tenant(
 text,text,uuid,uuid,text) TO ipat_platform_session_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.list_reserved_tenants_for_platform_owner(
 text,text) TO ipat_platform_session_owner;

CREATE FUNCTION ipat_platform.issue_platform_browser_session(
 p_issuer text,p_subject text,p_host text,p_id uuid,
 p_cookie_sha text,p_csrf_sha text,p_expires timestamptz)
 RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE at_time timestamptz:=clock_timestamp();
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_host IS NULL OR p_id IS NULL
  OR p_cookie_sha IS NULL OR p_csrf_sha IS NULL
  OR p_cookie_sha !~ '^[0-9a-f]{64}$' OR p_csrf_sha !~ '^[0-9a-f]{64}$'
  OR p_cookie_sha=p_csrf_sha
  OR p_expires IS NULL OR p_expires<=at_time
  OR p_expires>at_time+interval '10 minutes'
  OR NOT EXISTS(SELECT 1 FROM ipat_platform.platform_principals p
    WHERE p.issuer=p_issuer AND p.subject=p_subject AND p.role='platform_owner'
    AND p.revoked_at IS NULL AND p.created_at<=at_time AND p.expires_at>at_time
    AND length(btrim(p.approved_by))>0)
  OR NOT EXISTS(SELECT 1 FROM ipat_platform.platform_console_hosts h
    WHERE h.hostname=p_host AND h.disabled_at IS NULL
    AND h.verified_at<=at_time AND h.tls_ready_at<=at_time)
 THEN RETURN NULL; END IF;
 -- Dedicated issuer may issue AFTER independently verified real OIDC MFA.
 -- A database-side principal/host check alone cannot validate a human MFA.
 PERFORM pg_advisory_xact_lock(hashtextextended(
  p_issuer||'/'||p_subject||'/platform-session',0));
 IF (SELECT count(*) FROM ipat_platform.platform_browser_sessions s
   WHERE s.issuer=p_issuer AND s.subject=p_subject
   AND s.revoked_at IS NULL AND s.expires_at>at_time
   AND s.last_seen_at>at_time-interval '4 minutes')>=4
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_platform.platform_browser_sessions(
  id,issuer,subject,hostname,cookie_sha256,csrf_sha256,
  issued_at,last_seen_at,expires_at)
 VALUES(p_id,p_issuer,p_subject,p_host,p_cookie_sha,p_csrf_sha,
  at_time,at_time,p_expires);
 RETURN p_id;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN
 RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.issue_platform_browser_session(
 text,text,text,uuid,text,text,timestamptz) OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.issue_platform_browser_session(
 text,text,text,uuid,text,text,timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.issue_platform_browser_session(
 text,text,text,uuid,text,text,timestamptz) TO ipat_platform_session_issue_exec;

CREATE FUNCTION ipat_platform.authenticate_platform_browser_session(
 p_cookie_sha text,p_host text,p_csrf_sha text,p_mutation boolean)
 RETURNS TABLE(issuer text,subject text)
 LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE at_time timestamptz:=clock_timestamp();
DECLARE current_session ipat_platform.platform_browser_sessions%ROWTYPE;
BEGIN
 IF p_cookie_sha IS NULL OR p_cookie_sha !~ '^[0-9a-f]{64}$'
  OR p_host IS NULL OR length(p_host) NOT BETWEEN 4 AND 253
  OR p_host<>lower(p_host) OR position('..' in p_host)>0
  OR p_mutation IS NULL
  OR (p_mutation AND (p_csrf_sha IS NULL OR p_csrf_sha !~ '^[0-9a-f]{64}$'))
 THEN RETURN; END IF;
 SELECT s.* INTO current_session
 FROM ipat_platform.platform_browser_sessions s
 JOIN ipat_platform.platform_console_hosts h ON h.hostname=s.hostname
 JOIN ipat_platform.platform_principals p ON p.issuer=s.issuer AND p.subject=s.subject
 WHERE s.cookie_sha256=p_cookie_sha AND s.hostname=p_host
  AND h.verified_at<=at_time AND h.tls_ready_at<=at_time AND h.disabled_at IS NULL
  AND p.role='platform_owner' AND p.revoked_at IS NULL
  AND p.created_at<=at_time AND p.expires_at>at_time
  AND length(btrim(p.approved_by))>0
  AND s.revoked_at IS NULL AND s.expires_at>at_time
  AND s.last_seen_at>at_time-interval '4 minutes'
  AND (NOT p_mutation OR s.csrf_sha256=p_csrf_sha)
 FOR UPDATE OF s;
 IF NOT FOUND THEN RETURN; END IF;
 UPDATE ipat_platform.platform_browser_sessions s SET last_seen_at=at_time
 WHERE s.id=current_session.id;
 issuer:=current_session.issuer;subject:=current_session.subject; RETURN NEXT;
END $body$;
ALTER FUNCTION ipat_platform.authenticate_platform_browser_session(
 text,text,text,boolean) OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.authenticate_platform_browser_session(
 text,text,text,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.authenticate_platform_browser_session(
 text,text,text,boolean) TO ipat_platform_session_api_exec;
-- Execution of the reservation through the independent authenticated SESSION
-- not through browser-submitted issuer or subject.
CREATE FUNCTION ipat_platform.reserve_tenant_from_platform_session(
 p_cookie_sha text,p_host text,p_csrf_sha text,
 p_request uuid,p_tenant uuid,p_slug text)
 RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE verified record;
BEGIN
 SELECT * INTO verified FROM ipat_platform.authenticate_platform_browser_session(
  p_cookie_sha,p_host,p_csrf_sha,true);
 IF NOT FOUND THEN RETURN NULL; END IF;
 RETURN ipat_platform.reserve_suspended_tenant(
  verified.issuer,verified.subject,p_request,p_tenant,p_slug);
END $body$;
ALTER FUNCTION ipat_platform.reserve_tenant_from_platform_session(
 text,text,text,uuid,uuid,text) OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.reserve_tenant_from_platform_session(
 text,text,text,uuid,uuid,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.reserve_tenant_from_platform_session(
 text,text,text,uuid,uuid,text) TO ipat_platform_session_api_exec;

CREATE FUNCTION ipat_platform.list_reservations_from_platform_session(
 p_cookie_sha text,p_host text)
 RETURNS TABLE(tenant_id uuid,tenant_slug text,tenant_state text,reserved_at timestamptz)
 LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE verified record;
BEGIN
 SELECT * INTO verified FROM ipat_platform.authenticate_platform_browser_session(
  p_cookie_sha,p_host,NULL,false);
 IF NOT FOUND THEN RETURN; END IF;
 RETURN QUERY SELECT * FROM ipat_platform.list_reserved_tenants_for_platform_owner(
  verified.issuer,verified.subject);
END $body$;
ALTER FUNCTION ipat_platform.list_reservations_from_platform_session(text,text)
 OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_reservations_from_platform_session(
 text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_reservations_from_platform_session(
 text,text) TO ipat_platform_session_api_exec;

CREATE FUNCTION ipat_platform.revoke_platform_browser_session(
 p_cookie_sha text,p_host text,p_csrf_sha text)
 RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE verified record;count_changed integer;
BEGIN
 SELECT * INTO verified FROM ipat_platform.authenticate_platform_browser_session(
  p_cookie_sha,p_host,p_csrf_sha,true);
 IF NOT FOUND THEN RETURN false; END IF;
 UPDATE ipat_platform.platform_browser_sessions s SET revoked_at=clock_timestamp()
 WHERE s.cookie_sha256=p_cookie_sha AND s.hostname=p_host AND s.revoked_at IS NULL;
 GET DIAGNOSTICS count_changed=ROW_COUNT;
 RETURN count_changed=1;
END $body$;
ALTER FUNCTION ipat_platform.revoke_platform_browser_session(text,text,text)
 OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.revoke_platform_browser_session(text,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.revoke_platform_browser_session(
 text,text,text) TO ipat_platform_session_api_exec;
-- No migrations give the tenant API, legacy OIDC tenant issuer, or public
-- application login the platform session issue/auth/reserve privileges.
COMMIT;
