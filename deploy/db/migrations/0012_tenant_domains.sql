-- R9.52 production-path tenant hostname registry.
-- Hostname resolves routing context only; it NEVER grants tenant authorization.
BEGIN;
CREATE TABLE ipat_platform.tenant_domains (
    tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
    fqdn text PRIMARY KEY,
    domain_kind text NOT NULL CHECK
        (domain_kind IN ('platform_subdomain','custom_domain')),
    verification_method text NOT NULL CHECK
        (verification_method IN ('platform_parent','dns_txt')),
    state text NOT NULL DEFAULT 'pending' CHECK
        (state IN ('pending','verified','suspended','revoked')),
    verified_at timestamptz,
    tls_ready boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    CHECK (length(fqdn) BETWEEN 3 AND 253),
    CHECK (fqdn = lower(fqdn)),
    CHECK (fqdn ~ '^[a-z0-9][a-z0-9.-]*[a-z0-9]$'),
    CHECK (position('.' in fqdn) > 1),
    CHECK (fqdn NOT LIKE '%..%'),
    CHECK (fqdn NOT LIKE '%.-%' AND fqdn NOT LIKE '%-.%'),
    CHECK (state <> 'verified' OR verified_at IS NOT NULL)
);
ALTER TABLE ipat_platform.tenant_domains OWNER TO ipat_schema_owner;
CREATE UNIQUE INDEX tenant_domains_one_platform_subdomain
    ON ipat_platform.tenant_domains(tenant_id)
    WHERE domain_kind = 'platform_subdomain' AND state <> 'revoked';
ALTER TABLE ipat_platform.tenant_domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.tenant_domains FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.tenant_domains FROM PUBLIC, ipat_app_runtime;

CREATE ROLE ipat_domain_lookup_owner NOLOGIN NOSUPERUSER NOCREATEDB
    NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_domain_query NOLOGIN NOSUPERUSER NOCREATEDB
    NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform
    TO ipat_domain_lookup_owner, ipat_domain_query;
GRANT SELECT ON ipat_platform.tenants, ipat_platform.tenant_domains
    TO ipat_domain_lookup_owner;

CREATE POLICY tenant_domain_lookup_select
    ON ipat_platform.tenant_domains FOR SELECT
    TO ipat_domain_lookup_owner USING (true);
CREATE FUNCTION ipat_platform.resolve_verified_tenant_domain(p_fqdn text)
RETURNS TABLE (tenant_id uuid, tenant_slug text, fqdn text)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $ipat_sql$
    SELECT t.id, t.tenant_slug, d.fqdn
    FROM ipat_platform.tenant_domains AS d
    JOIN ipat_platform.tenants AS t ON t.id = d.tenant_id
    WHERE d.fqdn = p_fqdn
      AND p_fqdn IS NOT NULL
      AND length(p_fqdn) BETWEEN 3 AND 253
      AND p_fqdn = lower(p_fqdn)
      AND d.state = 'verified'
      AND d.verified_at IS NOT NULL
      AND d.tls_ready = true
      AND t.state = 'active'
$ipat_sql$;
ALTER FUNCTION ipat_platform.resolve_verified_tenant_domain(text)
    OWNER TO ipat_domain_lookup_owner;
REVOKE ALL ON FUNCTION ipat_platform.resolve_verified_tenant_domain(text)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.resolve_verified_tenant_domain(text)
    TO ipat_domain_query;

-- Runtime service logins are provisioned outside migrations and inherit only
-- ipat_domain_query. DNS/TLS verification writes remain platform-admin actions.
COMMIT;
