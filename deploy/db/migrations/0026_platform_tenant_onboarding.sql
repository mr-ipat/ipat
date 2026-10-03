-- R9.73: nonpublic platform-owner tenant RESERVATION only.
-- This never provisions MFA, active subscriptions, DNS, or tenant admin access.
-- Only a separately authenticated platform service may receive the executor role.
BEGIN;
CREATE ROLE ipat_platform_onboard_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_platform_onboard_exec NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_platform_onboard_owner,ipat_platform_onboard_exec;
GRANT SELECT ON ipat_platform.platform_principals TO ipat_platform_onboard_owner;
CREATE POLICY platform_onboard_principal_read ON ipat_platform.platform_principals
 FOR SELECT TO ipat_platform_onboard_owner USING(true);
GRANT SELECT,INSERT ON ipat_platform.tenants TO ipat_platform_onboard_owner;

CREATE TABLE ipat_platform.platform_tenant_reservations(
 request_id uuid PRIMARY KEY,
 tenant_id uuid NOT NULL UNIQUE REFERENCES ipat_platform.tenants(id),
 tenant_slug text NOT NULL UNIQUE,
 requested_by_issuer text NOT NULL,
 requested_by_subject text NOT NULL,
 created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 CONSTRAINT reservation_slug_match CHECK(
   tenant_slug ~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$'
   AND length(tenant_slug) BETWEEN 1 AND 63)
);
ALTER TABLE ipat_platform.platform_tenant_reservations OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.platform_tenant_reservations ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.platform_tenant_reservations FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.platform_tenant_reservations FROM PUBLIC,ipat_app_runtime,
 ipat_tenant_api_exec,ipat_oidc_session_issuer_login;
GRANT SELECT,INSERT ON ipat_platform.platform_tenant_reservations
 TO ipat_platform_onboard_owner;
CREATE POLICY platform_onboard_reservation_read ON ipat_platform.platform_tenant_reservations
 FOR SELECT TO ipat_platform_onboard_owner USING(true);
CREATE POLICY platform_onboard_reservation_insert ON ipat_platform.platform_tenant_reservations
 FOR INSERT TO ipat_platform_onboard_owner WITH CHECK(true);

CREATE FUNCTION ipat_platform.reserve_suspended_tenant(
 p_issuer text,p_subject text,p_request uuid,p_tenant uuid,p_slug text
) RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE prior ipat_platform.platform_tenant_reservations%ROWTYPE;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_request IS NULL
    OR p_tenant IS NULL OR p_slug IS NULL
    OR length(p_slug) NOT BETWEEN 1 AND 63
    OR p_slug !~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$'
    OR NOT EXISTS (
      SELECT 1 FROM ipat_platform.platform_principals p
      WHERE p.issuer=p_issuer AND p.subject=p_subject
        AND p.role='platform_owner' AND p.revoked_at IS NULL
        AND p.created_at<=statement_timestamp()
        AND p.expires_at>statement_timestamp()
        AND length(btrim(p.approved_by))>0
    ) THEN RETURN NULL; END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended('ipat-platform/tenant-request/'||p_request::text,0));
 SELECT * INTO prior FROM ipat_platform.platform_tenant_reservations
 WHERE request_id=p_request;
 IF FOUND THEN
  -- Preserve idempotency only for EXACT same request, issuer and subject.
  IF prior.tenant_id=p_tenant AND prior.tenant_slug=p_slug
     AND prior.requested_by_issuer=p_issuer
     AND prior.requested_by_subject=p_subject THEN
   RETURN prior.tenant_id;
  END IF;
  RETURN NULL;
 END IF;
 -- No active tenant, implicit custom domain, automatic membership or DNS.
 INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
 VALUES(p_tenant,p_slug,'suspended');
 INSERT INTO ipat_platform.platform_tenant_reservations
 (request_id,tenant_id,tenant_slug,requested_by_issuer,requested_by_subject)
 VALUES(p_request,p_tenant,p_slug,p_issuer,p_subject);
 RETURN p_tenant;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN
 RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.reserve_suspended_tenant(text,text,uuid,uuid,text)
 OWNER TO ipat_platform_onboard_owner;
REVOKE ALL ON FUNCTION ipat_platform.reserve_suspended_tenant(text,text,uuid,uuid,text)
 FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_exec,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.reserve_suspended_tenant(text,text,uuid,uuid,text)
 TO ipat_platform_onboard_exec;

CREATE FUNCTION ipat_platform.list_reserved_tenants_for_platform_owner(
 p_issuer text,p_subject text
) RETURNS TABLE(tenant_id uuid,tenant_slug text,tenant_state text,reserved_at timestamptz)
LANGUAGE sql STABLE SECURITY DEFINER SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT t.id,t.tenant_slug,t.state,r.created_at
 FROM ipat_platform.platform_tenant_reservations r
 JOIN ipat_platform.tenants t ON t.id=r.tenant_id
 WHERE EXISTS(
  SELECT 1 FROM ipat_platform.platform_principals p
  WHERE p.issuer=p_issuer AND p.subject=p_subject
  AND p.role='platform_owner' AND p.revoked_at IS NULL
  AND p.created_at<=statement_timestamp() AND p.expires_at>statement_timestamp()
  AND length(btrim(p.approved_by))>0
 ) ORDER BY r.created_at DESC,t.id LIMIT 100
$body$;
ALTER FUNCTION ipat_platform.list_reserved_tenants_for_platform_owner(text,text)
 OWNER TO ipat_platform_onboard_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_reserved_tenants_for_platform_owner(text,text)
 FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_exec,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.list_reserved_tenants_for_platform_owner(text,text)
 TO ipat_platform_onboard_exec;
COMMIT;
