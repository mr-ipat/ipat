-- R9.56 pending DNS TXT ownership queue on canonical R9.53 UUID schema.
-- Background verifier can list only pending active-tenant custom challenges.
-- SQL cannot attest DNS; only the isolated verifier process performs the lookup.
BEGIN;

CREATE OR REPLACE FUNCTION ipat_platform.list_tenant_domain_ownership_checks(
    p_limit integer
)
RETURNS TABLE (
    id uuid,
    hostname text,
    verification_name text,
    verification_value text
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $function$
    SELECT d.id,d.hostname,d.verification_name,d.verification_value
    FROM ipat_platform.tenant_domains d
    JOIN ipat_platform.tenants t ON t.id = d.tenant_id
    WHERE t.state = 'active'
      AND d.domain_type = 'custom_domain'
      AND d.verification_method = 'dns_txt'
      AND d.verification_state = 'pending'
      AND d.activation_state = 'pending_dns'
      AND d.ownership_verified_at IS NULL
      AND d.disabled_at IS NULL
      AND d.verification_name = '_ipat-verify.' || d.hostname
      AND d.verification_value LIKE 'ipat-domain=%'
      AND d.requested_at IS NOT NULL
    ORDER BY d.requested_at, d.id
    LIMIT CASE WHEN p_limit BETWEEN 1 AND 100 THEN p_limit ELSE 0 END
$function$;

ALTER FUNCTION ipat_platform.list_tenant_domain_ownership_checks(integer)
    OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_domain_ownership_checks(integer)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_domain_ownership_checks(integer)
    TO ipat_domain_verifier;

COMMIT;
