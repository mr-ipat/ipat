-- R10.11: two-person Platform Owner company activation + initial tenant-admin
-- identity binding + one exact pending customer domain. No DNS/TLS is activated.
BEGIN;

CREATE ROLE ipat_company_activation_owner
  NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;

GRANT USAGE ON SCHEMA ipat_platform TO ipat_company_activation_owner;
GRANT SELECT ON ipat_platform.platform_principals TO ipat_company_activation_owner;
GRANT SELECT,UPDATE ON ipat_platform.tenants TO ipat_company_activation_owner;
GRANT SELECT,INSERT ON ipat_platform.identity_memberships TO ipat_company_activation_owner;
GRANT SELECT,INSERT ON ipat_platform.tenant_domains TO ipat_company_activation_owner;
GRANT SELECT ON ipat_platform.platform_tenant_reservations TO ipat_company_activation_owner;

CREATE POLICY company_activation_principal_read
 ON ipat_platform.platform_principals FOR SELECT
 TO ipat_company_activation_owner USING(true);
CREATE POLICY company_activation_tenant_read
 ON ipat_platform.tenants FOR SELECT
 TO ipat_company_activation_owner USING(true);
CREATE POLICY company_activation_tenant_update
 ON ipat_platform.tenants FOR UPDATE
 TO ipat_company_activation_owner USING(true) WITH CHECK(true);
CREATE POLICY company_activation_membership_read
 ON ipat_platform.identity_memberships FOR SELECT
 TO ipat_company_activation_owner USING(true);
CREATE POLICY company_activation_membership_insert
 ON ipat_platform.identity_memberships FOR INSERT
 TO ipat_company_activation_owner WITH CHECK(true);
CREATE POLICY company_activation_domain_read
 ON ipat_platform.tenant_domains FOR SELECT
 TO ipat_company_activation_owner USING(true);
CREATE POLICY company_activation_domain_insert
 ON ipat_platform.tenant_domains FOR INSERT
 TO ipat_company_activation_owner WITH CHECK(true);
CREATE POLICY company_activation_reservation_read
 ON ipat_platform.platform_tenant_reservations FOR SELECT
 TO ipat_company_activation_owner USING(true);

CREATE TABLE ipat_platform.platform_tenant_activation_requests(
 request_id uuid PRIMARY KEY,
 tenant_id uuid NOT NULL UNIQUE REFERENCES ipat_platform.tenants(id),
 domain_id uuid NOT NULL UNIQUE,
 customer_hostname text NOT NULL UNIQUE,
 routing_mode text NOT NULL CHECK(routing_mode IN('a_record','cname','nameserver')),
 admin_issuer text NOT NULL CHECK(length(admin_issuer) BETWEEN 10 AND 512
   AND admin_issuer LIKE 'https://%' AND admin_issuer !~ '[[:space:]]'),
 admin_subject text NOT NULL CHECK(length(admin_subject) BETWEEN 1 AND 128
   AND admin_subject ~ '^[a-zA-Z0-9_:/.-]+$'),
 admin_expires_at timestamptz NOT NULL,
 evidence_sha256 text NOT NULL CHECK(evidence_sha256 ~ '^[0-9a-f]{64}$'),
 state text NOT NULL DEFAULT 'REQUESTED' CHECK(state IN('REQUESTED','APPROVED','REJECTED')),
 requested_by_issuer text NOT NULL,
 requested_by_subject text NOT NULL,
 requested_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 reviewed_by_issuer text,
 reviewed_by_subject text,
 reviewed_at timestamptz,
 CONSTRAINT company_activation_domain_shape CHECK(
   customer_hostname=lower(customer_hostname)
   AND length(customer_hostname) BETWEEN 4 AND 253
   AND customer_hostname ~ '^[a-z0-9]([a-z0-9.-]*[a-z0-9])$'
   AND customer_hostname LIKE '%.%'
   AND position('..' in customer_hostname)=0
   AND customer_hostname !~ '\.(invalid|test|example|localhost|local)$'),
 CONSTRAINT company_activation_review_consistency CHECK(
   (state='REQUESTED' AND reviewed_by_issuer IS NULL AND reviewed_by_subject IS NULL AND reviewed_at IS NULL)
   OR
   (state IN('APPROVED','REJECTED') AND reviewed_by_issuer IS NOT NULL
     AND reviewed_by_subject IS NOT NULL AND reviewed_at IS NOT NULL))
);
ALTER TABLE ipat_platform.platform_tenant_activation_requests OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.platform_tenant_activation_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.platform_tenant_activation_requests FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.platform_tenant_activation_requests
 FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_exec,ipat_oidc_session_issuer_login,
 ipat_platform_session_api_exec,ipat_platform_session_issue_exec;
GRANT SELECT,INSERT,UPDATE ON ipat_platform.platform_tenant_activation_requests
 TO ipat_company_activation_owner;
CREATE POLICY company_activation_request_all
 ON ipat_platform.platform_tenant_activation_requests FOR ALL
 TO ipat_company_activation_owner USING(true) WITH CHECK(true);

CREATE FUNCTION ipat_platform.request_company_activation(
 p_issuer text,p_subject text,p_request uuid,p_tenant uuid,p_domain uuid,
 p_hostname text,p_routing_mode text,p_admin_issuer text,p_admin_subject text,
 p_admin_expires timestamptz,p_evidence_sha text)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE existing ipat_platform.platform_tenant_activation_requests%ROWTYPE;
DECLARE now_at timestamptz:=clock_timestamp();
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_request IS NULL OR p_tenant IS NULL
   OR p_domain IS NULL OR p_hostname IS NULL OR p_routing_mode NOT IN('a_record','cname','nameserver')
   OR p_admin_issuer IS NULL OR p_admin_subject IS NULL OR p_admin_expires IS NULL
   OR p_evidence_sha !~ '^[0-9a-f]{64}$'
   OR p_admin_issuer !~ '^https://[^[:space:]]{2,}$'
   OR p_admin_subject !~ '^[a-zA-Z0-9_:/.-]{1,128}$'
   OR p_admin_expires<=now_at+interval '1 hour' OR p_admin_expires>now_at+interval '90 days'
   OR p_hostname<>lower(p_hostname) OR length(p_hostname) NOT BETWEEN 4 AND 253
   OR p_hostname !~ '^[a-z0-9]([a-z0-9.-]*[a-z0-9])$'
   OR p_hostname NOT LIKE '%.%' OR position('..' in p_hostname)>0
   OR p_hostname ~ '\.(invalid|test|example|localhost|local)$'
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.platform_principals p
      WHERE p.issuer=p_issuer AND p.subject=p_subject AND p.role='platform_owner'
       AND p.revoked_at IS NULL AND p.created_at<=now_at AND p.expires_at>now_at
       AND length(btrim(p.approved_by))>0)
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.platform_tenant_reservations r
      JOIN ipat_platform.tenants t ON t.id=r.tenant_id
      WHERE r.tenant_id=p_tenant AND t.state='suspended')
 THEN RETURN NULL; END IF;

 PERFORM pg_advisory_xact_lock(hashtextextended('company-activation/'||p_request::text,0));
 SELECT * INTO existing FROM ipat_platform.platform_tenant_activation_requests WHERE request_id=p_request;
 IF FOUND THEN
  IF existing.tenant_id=p_tenant AND existing.domain_id=p_domain
    AND existing.customer_hostname=p_hostname AND existing.routing_mode=p_routing_mode
    AND existing.admin_issuer=p_admin_issuer AND existing.admin_subject=p_admin_subject
    AND existing.admin_expires_at=p_admin_expires AND existing.evidence_sha256=p_evidence_sha
    AND existing.requested_by_issuer=p_issuer AND existing.requested_by_subject=p_subject
  THEN RETURN existing.request_id; END IF;
  RETURN NULL;
 END IF;

 INSERT INTO ipat_platform.platform_tenant_activation_requests(
   request_id,tenant_id,domain_id,customer_hostname,routing_mode,
   admin_issuer,admin_subject,admin_expires_at,evidence_sha256,
   requested_by_issuer,requested_by_subject)
 VALUES(p_request,p_tenant,p_domain,p_hostname,p_routing_mode,
   p_admin_issuer,p_admin_subject,p_admin_expires,p_evidence_sha,p_issuer,p_subject);
 RETURN p_request;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.request_company_activation(
 text,text,uuid,uuid,uuid,text,text,text,text,timestamptz,text)
 OWNER TO ipat_company_activation_owner;
REVOKE ALL ON FUNCTION ipat_platform.request_company_activation(
 text,text,uuid,uuid,uuid,text,text,text,text,timestamptz,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.request_company_activation(
 text,text,uuid,uuid,uuid,text,text,text,text,timestamptz,text)
 TO ipat_platform_session_owner;

CREATE FUNCTION ipat_platform.review_company_activation(
 p_issuer text,p_subject text,p_request uuid,p_approve boolean)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE req ipat_platform.platform_tenant_activation_requests%ROWTYPE;
DECLARE now_at timestamptz:=clock_timestamp();
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_request IS NULL OR p_approve IS NULL
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.platform_principals p
      WHERE p.issuer=p_issuer AND p.subject=p_subject AND p.role='platform_owner'
       AND p.revoked_at IS NULL AND p.created_at<=now_at AND p.expires_at>now_at
       AND length(btrim(p.approved_by))>0)
 THEN RETURN false; END IF;

 SELECT * INTO req FROM ipat_platform.platform_tenant_activation_requests
  WHERE request_id=p_request FOR UPDATE;
 IF NOT FOUND OR req.state<>'REQUESTED'
   OR (req.requested_by_issuer,req.requested_by_subject)=(p_issuer,p_subject)
   OR req.admin_expires_at<=now_at+interval '30 minutes'
 THEN RETURN false; END IF;

 IF NOT p_approve THEN
   UPDATE ipat_platform.platform_tenant_activation_requests SET
    state='REJECTED',reviewed_by_issuer=p_issuer,reviewed_by_subject=p_subject,reviewed_at=now_at
    WHERE request_id=p_request;
   RETURN true;
 END IF;

 IF NOT EXISTS(SELECT 1 FROM ipat_platform.tenants t
    WHERE t.id=req.tenant_id AND t.state='suspended')
   OR EXISTS(SELECT 1 FROM ipat_platform.identity_memberships m
      WHERE m.tenant_id=req.tenant_id AND m.revoked_at IS NULL AND m.expires_at>now_at)
   OR EXISTS(SELECT 1 FROM ipat_platform.tenant_domains d
      WHERE d.tenant_id=req.tenant_id AND d.disabled_at IS NULL)
 THEN RETURN false; END IF;

 INSERT INTO ipat_platform.identity_memberships(
   tenant_id,issuer,subject,role,approved_by,expires_at)
 VALUES(req.tenant_id,req.admin_issuer,req.admin_subject,'tenant_admin',
   left(p_subject,128),req.admin_expires_at);

 INSERT INTO ipat_platform.tenant_domains(
   id,tenant_id,hostname,domain_type,verification_state,verification_method,
   routing_mode,verification_name,verification_value,
   requested_by_issuer,requested_by_subject,requested_at,activation_state)
 VALUES(req.domain_id,req.tenant_id,req.customer_hostname,'custom_domain','pending','dns_txt',
   req.routing_mode,'_ipat-verify.'||req.customer_hostname,'ipat-domain='||req.domain_id::text,
   req.requested_by_issuer,req.requested_by_subject,req.requested_at,'pending_dns');

 UPDATE ipat_platform.tenants SET state='active' WHERE id=req.tenant_id;
 UPDATE ipat_platform.platform_tenant_activation_requests SET
  state='APPROVED',reviewed_by_issuer=p_issuer,reviewed_by_subject=p_subject,reviewed_at=now_at
  WHERE request_id=p_request;
 RETURN true;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN RETURN false;
END $body$;
ALTER FUNCTION ipat_platform.review_company_activation(text,text,uuid,boolean)
 OWNER TO ipat_company_activation_owner;
REVOKE ALL ON FUNCTION ipat_platform.review_company_activation(text,text,uuid,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.review_company_activation(text,text,uuid,boolean)
 TO ipat_platform_session_owner;

CREATE FUNCTION ipat_platform.list_company_activation_requests(
 p_issuer text,p_subject text)
RETURNS TABLE(
 request_id uuid,tenant_id uuid,tenant_slug text,state text,customer_hostname text,
 routing_mode text,admin_issuer text,admin_subject text,admin_expires_at timestamptz,
 evidence_sha256 text,requested_by_subject text,requested_at timestamptz,
 reviewed_by_subject text,reviewed_at timestamptz,verification_name text,verification_value text)
LANGUAGE sql STABLE SECURITY DEFINER SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT a.request_id,a.tenant_id,t.tenant_slug,a.state,a.customer_hostname,a.routing_mode,
  a.admin_issuer,a.admin_subject,a.admin_expires_at,a.evidence_sha256,
  a.requested_by_subject,a.requested_at,a.reviewed_by_subject,a.reviewed_at,
  CASE WHEN a.state='APPROVED' THEN '_ipat-verify.'||a.customer_hostname ELSE NULL END,
  CASE WHEN a.state='APPROVED' THEN 'ipat-domain='||a.domain_id::text ELSE NULL END
 FROM ipat_platform.platform_tenant_activation_requests a
 JOIN ipat_platform.tenants t ON t.id=a.tenant_id
 WHERE EXISTS(SELECT 1 FROM ipat_platform.platform_principals p
   WHERE p.issuer=p_issuer AND p.subject=p_subject AND p.role='platform_owner'
    AND p.revoked_at IS NULL AND p.created_at<=statement_timestamp()
    AND p.expires_at>statement_timestamp() AND length(btrim(p.approved_by))>0)
 ORDER BY a.requested_at DESC,a.request_id LIMIT 100
$body$;
ALTER FUNCTION ipat_platform.list_company_activation_requests(text,text)
 OWNER TO ipat_company_activation_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_company_activation_requests(text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_company_activation_requests(text,text)
 TO ipat_platform_session_owner;

CREATE FUNCTION ipat_platform.request_company_activation_from_platform_session(
 p_cookie_sha text,p_host text,p_csrf_sha text,p_request uuid,p_tenant uuid,p_domain uuid,
 p_hostname text,p_routing_mode text,p_admin_issuer text,p_admin_subject text,
 p_admin_expires timestamptz,p_evidence_sha text)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE v record;
BEGIN
 SELECT * INTO v FROM ipat_platform.authenticate_platform_browser_session(
   p_cookie_sha,p_host,p_csrf_sha,true);
 IF NOT FOUND THEN RETURN NULL; END IF;
 RETURN ipat_platform.request_company_activation(
   v.issuer,v.subject,p_request,p_tenant,p_domain,p_hostname,p_routing_mode,
   p_admin_issuer,p_admin_subject,p_admin_expires,p_evidence_sha);
END $body$;
ALTER FUNCTION ipat_platform.request_company_activation_from_platform_session(
 text,text,text,uuid,uuid,uuid,text,text,text,text,timestamptz,text)
 OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.request_company_activation_from_platform_session(
 text,text,text,uuid,uuid,uuid,text,text,text,text,timestamptz,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.request_company_activation_from_platform_session(
 text,text,text,uuid,uuid,uuid,text,text,text,text,timestamptz,text)
 TO ipat_platform_session_api_exec;

CREATE FUNCTION ipat_platform.review_company_activation_from_platform_session(
 p_cookie_sha text,p_host text,p_csrf_sha text,p_request uuid,p_approve boolean)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE v record;
BEGIN
 SELECT * INTO v FROM ipat_platform.authenticate_platform_browser_session(
   p_cookie_sha,p_host,p_csrf_sha,true);
 IF NOT FOUND THEN RETURN false; END IF;
 RETURN ipat_platform.review_company_activation(v.issuer,v.subject,p_request,p_approve);
END $body$;
ALTER FUNCTION ipat_platform.review_company_activation_from_platform_session(
 text,text,text,uuid,boolean) OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.review_company_activation_from_platform_session(
 text,text,text,uuid,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.review_company_activation_from_platform_session(
 text,text,text,uuid,boolean) TO ipat_platform_session_api_exec;

CREATE FUNCTION ipat_platform.list_company_activation_requests_from_platform_session(
 p_cookie_sha text,p_host text)
RETURNS TABLE(
 request_id uuid,tenant_id uuid,tenant_slug text,state text,customer_hostname text,
 routing_mode text,admin_issuer text,admin_subject text,admin_expires_at timestamptz,
 evidence_sha256 text,requested_by_subject text,requested_at timestamptz,
 reviewed_by_subject text,reviewed_at timestamptz,verification_name text,verification_value text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE v record;
BEGIN
 SELECT * INTO v FROM ipat_platform.authenticate_platform_browser_session(
   p_cookie_sha,p_host,NULL,false);
 IF NOT FOUND THEN RETURN; END IF;
 RETURN QUERY SELECT * FROM ipat_platform.list_company_activation_requests(v.issuer,v.subject);
END $body$;
ALTER FUNCTION ipat_platform.list_company_activation_requests_from_platform_session(text,text)
 OWNER TO ipat_platform_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_company_activation_requests_from_platform_session(text,text)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_company_activation_requests_from_platform_session(text,text)
 TO ipat_platform_session_api_exec;

COMMIT;
