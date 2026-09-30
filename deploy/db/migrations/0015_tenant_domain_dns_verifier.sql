-- R9.55 DNS ownership-verifier work queue.
-- The runtime verifier can enumerate only pending custom-domain TXT challenges
-- through this sealed function; it still has no direct tenant_domains access.
BEGIN;

CREATE FUNCTION ipat_platform.list_tenant_domain_ownership_checks(
  p_limit integer
)
RETURNS TABLE(
  fqdn text,
  verification_name text,
  verification_value text
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
  SELECT d.fqdn,d.verification_name,d.verification_value
  FROM ipat_platform.tenant_domains d
  WHERE d.domain_kind='custom_domain'
    AND d.state='pending'
    AND d.ownership_verified_at IS NULL
    AND d.verification_method='dns_txt'
    AND d.verification_name IS NOT NULL
    AND d.verification_value IS NOT NULL
  ORDER BY d.requested_at,d.fqdn
  LIMIT CASE
    WHEN p_limit BETWEEN 1 AND 100 THEN p_limit
    ELSE 0
  END
$$;

ALTER FUNCTION ipat_platform.list_tenant_domain_ownership_checks(integer)
  OWNER TO ipat_domain_verifier_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_domain_ownership_checks(integer)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_domain_ownership_checks(integer)
  TO ipat_domain_verifier;

COMMIT;
