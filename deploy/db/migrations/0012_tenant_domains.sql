-- IPAT tenant-domain control plane.
-- Production-shaped schema: hostname selects public tenant bootstrap context only.
-- It MUST NOT grant user membership, role, POP scope, device access, or secret access.
BEGIN;

CREATE ROLE ipat_domain_reader
  NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;

-- Authentication material is never stored in Git/migration. This LOGIN role
-- starts without a password and is usable only when deployment configures an
-- approved local peer/cert/SCRAM authentication path outside source control.
CREATE ROLE ipat_domain_reader_login
  LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
  IN ROLE ipat_domain_reader;

CREATE TABLE ipat_platform.tenant_domains (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id) ON DELETE CASCADE,
  hostname text NOT NULL UNIQUE,
  domain_type text NOT NULL CHECK (domain_type IN ('managed_subdomain','custom_domain')),
  verification_state text NOT NULL DEFAULT 'pending'
    CHECK (verification_state IN ('pending','verified','disabled')),
  verification_method text NOT NULL
    CHECK (verification_method IN ('platform_managed','dns_txt')),
  verified_at timestamptz,
  disabled_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  CHECK (
    length(hostname) BETWEEN 3 AND 253
    AND hostname = lower(hostname)
    AND hostname !~ '[:/[:space:]]'
    AND hostname ~ '^[a-z0-9]([a-z0-9.-]*[a-z0-9])$'
    AND position('..' in hostname) = 0
  ),
  CHECK (
    (verification_state = 'verified' AND verified_at IS NOT NULL AND disabled_at IS NULL)
    OR (verification_state = 'pending' AND verified_at IS NULL AND disabled_at IS NULL)
    OR (verification_state = 'disabled' AND disabled_at IS NOT NULL)
  ),
  CHECK (
    (domain_type = 'managed_subdomain' AND verification_method = 'platform_managed')
    OR (domain_type = 'custom_domain' AND verification_method = 'dns_txt')
  )
);
ALTER TABLE ipat_platform.tenant_domains OWNER TO ipat_schema_owner;

REVOKE ALL ON ipat_platform.tenant_domains FROM PUBLIC;
REVOKE ALL ON ipat_platform.tenant_domains FROM ipat_app_runtime;
CREATE INDEX tenant_domains_tenant_idx ON ipat_platform.tenant_domains(tenant_id);

CREATE OR REPLACE FUNCTION ipat_platform.resolve_active_tenant_domain(p_hostname text)
RETURNS TABLE (
  tenant_id uuid,
  tenant_slug text,
  hostname text,
  domain_type text
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
  SELECT t.id, t.tenant_slug, d.hostname, d.domain_type
  FROM ipat_platform.tenant_domains d
  JOIN ipat_platform.tenants t ON t.id = d.tenant_id
  WHERE d.hostname = p_hostname
    AND d.verification_state = 'verified'
    AND d.disabled_at IS NULL
    AND t.state = 'active'
  LIMIT 1
$$;
ALTER FUNCTION ipat_platform.resolve_active_tenant_domain(text) OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.resolve_active_tenant_domain(text) FROM PUBLIC;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_domain_reader;
GRANT EXECUTE ON FUNCTION ipat_platform.resolve_active_tenant_domain(text) TO ipat_domain_reader;

COMMIT;
