-- R9.94 least-privilege domain verification split.
-- Existing DNS verifier is narrowed to ownership-only. A distinct ingress
-- verifier may advance only routing -> TLS -> active in strict DB order.
BEGIN;

CREATE ROLE ipat_domain_ingress_verifier
  NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE ROLE ipat_domain_ingress_verifier_login
  LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
  IN ROLE ipat_domain_ingress_verifier;

-- Remove the old broad lifecycle entry point from the DNS ownership verifier.
REVOKE EXECUTE ON FUNCTION ipat_platform.record_tenant_domain_check(uuid,text,text,text)
  FROM ipat_domain_verifier, ipat_domain_verifier_login;

CREATE FUNCTION ipat_platform.record_tenant_domain_ownership_check(
  p_domain_id uuid,
  p_event text,
  p_evidence_sha256 text,
  p_error_code text
)
RETURNS text
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $body$
BEGIN
  IF p_event NOT IN ('ownership_verified','check_failed') THEN
    RETURN NULL;
  END IF;
  IF p_event='check_failed' AND NOT EXISTS (
    SELECT 1 FROM ipat_platform.tenant_domains d
    WHERE d.id=p_domain_id AND d.domain_type='custom_domain'
      AND d.activation_state='pending_dns' AND d.disabled_at IS NULL
  ) THEN
    RETURN NULL;
  END IF;
  RETURN ipat_platform.record_tenant_domain_check(
    p_domain_id,p_event,p_evidence_sha256,p_error_code
  );
END
$body$;
ALTER FUNCTION ipat_platform.record_tenant_domain_ownership_check(uuid,text,text,text)
  OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_tenant_domain_ownership_check(uuid,text,text,text)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.record_tenant_domain_ownership_check(uuid,text,text,text)
  TO ipat_domain_verifier;

CREATE FUNCTION ipat_platform.record_tenant_domain_ingress_check(
  p_domain_id uuid,
  p_event text,
  p_evidence_sha256 text,
  p_error_code text
)
RETURNS text
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $body$
BEGIN
  IF p_event NOT IN ('routing_ready','tls_ready','activate','check_failed') THEN
    RETURN NULL;
  END IF;
  IF p_event='check_failed' AND NOT EXISTS (
    SELECT 1 FROM ipat_platform.tenant_domains d
    WHERE d.id=p_domain_id AND d.domain_type='custom_domain'
      AND d.activation_state IN ('ownership_verified','routing_ready','tls_ready')
      AND d.disabled_at IS NULL
  ) THEN
    RETURN NULL;
  END IF;
  RETURN ipat_platform.record_tenant_domain_check(
    p_domain_id,p_event,p_evidence_sha256,p_error_code
  );
END
$body$;
ALTER FUNCTION ipat_platform.record_tenant_domain_ingress_check(uuid,text,text,text)
  OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_tenant_domain_ingress_check(uuid,text,text,text)
  FROM PUBLIC;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_domain_ingress_verifier;
GRANT EXECUTE ON FUNCTION ipat_platform.record_tenant_domain_ingress_check(uuid,text,text,text)
  TO ipat_domain_ingress_verifier;

CREATE FUNCTION ipat_platform.list_tenant_domain_ingress_checks(
    p_limit integer
)
RETURNS TABLE (
    id uuid,
    hostname text,
    routing_mode text,
    verification_name text,
    verification_value text,
    activation_state text
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $function$
    SELECT d.id,d.hostname,d.routing_mode,d.verification_name,
           d.verification_value,d.activation_state
    FROM ipat_platform.tenant_domains d
    JOIN ipat_platform.tenants t ON t.id=d.tenant_id
    WHERE t.state='active'
      AND d.domain_type='custom_domain'
      AND d.verification_state='verified'
      AND d.verification_method='dns_txt'
      AND d.ownership_verified_at IS NOT NULL
      AND d.disabled_at IS NULL
      AND d.routing_mode='a_record'
      AND d.verification_name='_ipat-verify.' || d.hostname
      AND d.verification_value LIKE 'ipat-domain=%'
      AND d.activation_state IN ('ownership_verified','routing_ready','tls_ready')
    ORDER BY d.ownership_verified_at,d.id
    LIMIT CASE WHEN p_limit BETWEEN 1 AND 100 THEN p_limit ELSE 0 END
$function$;

ALTER FUNCTION ipat_platform.list_tenant_domain_ingress_checks(integer)
  OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_domain_ingress_checks(integer)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_domain_ingress_checks(integer)
  TO ipat_domain_ingress_verifier;

-- Neither verifier gets raw tenant-domain table access.
REVOKE ALL ON ipat_platform.tenant_domains
  FROM ipat_domain_ingress_verifier, ipat_domain_ingress_verifier_login;

COMMIT;
