-- R9.98 exact per-customer activation result for restricted ingress verifier.
-- This provides no raw tenant table access and cannot change lifecycle state.
BEGIN;

CREATE FUNCTION ipat_platform.get_tenant_domain_activation_result(
  p_domain_id uuid
)
RETURNS TABLE (
  id uuid,
  hostname text,
  verification_name text,
  verification_value text,
  activation_state text,
  last_error_code text,
  activated_at timestamptz
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $function$
  SELECT d.id,d.hostname,d.verification_name,d.verification_value,
         d.activation_state,d.last_error_code,d.activated_at
  FROM ipat_platform.tenant_domains d
  JOIN ipat_platform.tenants t ON t.id=d.tenant_id
  WHERE d.id=p_domain_id
    AND t.state='active'
    AND d.domain_type='custom_domain'
    AND d.verification_state='verified'
    AND d.verification_method='dns_txt'
    AND d.ownership_verified_at IS NOT NULL
    AND d.disabled_at IS NULL
    AND d.routing_mode='a_record'
    AND d.verification_name='_ipat-verify.' || d.hostname
    AND d.verification_value LIKE 'ipat-domain=%'
    AND d.activation_state IN ('ownership_verified','routing_ready','tls_ready','active')
$function$;

ALTER FUNCTION ipat_platform.get_tenant_domain_activation_result(uuid)
  OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.get_tenant_domain_activation_result(uuid)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.get_tenant_domain_activation_result(uuid)
  TO ipat_domain_ingress_verifier;

COMMIT;
