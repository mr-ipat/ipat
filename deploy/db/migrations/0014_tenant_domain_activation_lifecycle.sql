-- R9.54 ordered custom-domain activation lifecycle on the R9.52/R9.53 schema.
-- DNS/TLS evidence advances one stage at a time. No browser or tenant-admin role
-- may mark verification/routing/TLS/activation complete directly.
BEGIN;

ALTER TABLE ipat_platform.tenant_domains
  ADD COLUMN lifecycle_last_checked_at timestamptz,
  ADD COLUMN lifecycle_evidence_code text,
  ADD COLUMN lifecycle_last_error_code text,
  ADD CONSTRAINT tenant_domains_lifecycle_evidence_code CHECK (
    lifecycle_evidence_code IS NULL
    OR (
      length(lifecycle_evidence_code) BETWEEN 3 AND 96
      AND lifecycle_evidence_code ~ '^[A-Z0-9_.:-]+$'
    )
  ),
  ADD CONSTRAINT tenant_domains_lifecycle_error_code CHECK (
    lifecycle_last_error_code IS NULL
    OR (
      length(lifecycle_last_error_code) BETWEEN 3 AND 96
      AND lifecycle_last_error_code ~ '^[A-Z0-9_.:-]+$'
    )
  );

CREATE ROLE ipat_domain_verifier_owner NOLOGIN NOSUPERUSER NOCREATEDB
  NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_domain_verifier NOLOGIN NOSUPERUSER NOCREATEDB
  NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;

GRANT USAGE ON SCHEMA ipat_platform
  TO ipat_domain_verifier_owner, ipat_domain_verifier;
GRANT SELECT,UPDATE ON ipat_platform.tenant_domains
  TO ipat_domain_verifier_owner;

CREATE POLICY tenant_domain_verifier_select
  ON ipat_platform.tenant_domains FOR SELECT
  TO ipat_domain_verifier_owner USING (true);
CREATE POLICY tenant_domain_verifier_update
  ON ipat_platform.tenant_domains FOR UPDATE
  TO ipat_domain_verifier_owner USING (true) WITH CHECK (true);

CREATE FUNCTION ipat_platform.advance_tenant_domain_lifecycle(
  p_fqdn text,
  p_target text,
  p_evidence_code text
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
DECLARE
  v_changed integer;
BEGIN
  IF p_fqdn IS NULL
     OR p_target NOT IN ('ownership_verified','routing_ready','tls_ready','active')
     OR p_evidence_code IS NULL
     OR length(p_evidence_code) NOT BETWEEN 3 AND 96
     OR p_evidence_code !~ '^[A-Z0-9_.:-]+$' THEN
    RETURN false;
  END IF;

  IF p_target = 'ownership_verified' THEN
    UPDATE ipat_platform.tenant_domains
       SET ownership_verified_at = clock_timestamp(),
           lifecycle_last_checked_at = clock_timestamp(),
           lifecycle_evidence_code = p_evidence_code,
           lifecycle_last_error_code = NULL
     WHERE fqdn = p_fqdn
       AND domain_kind = 'custom_domain'
       AND state = 'pending'
       AND ownership_verified_at IS NULL
       AND routing_ready = false
       AND tls_ready = false;

  ELSIF p_target = 'routing_ready' THEN
    UPDATE ipat_platform.tenant_domains
       SET routing_ready = true,
           lifecycle_last_checked_at = clock_timestamp(),
           lifecycle_evidence_code = p_evidence_code,
           lifecycle_last_error_code = NULL
     WHERE fqdn = p_fqdn
       AND domain_kind = 'custom_domain'
       AND state = 'pending'
       AND ownership_verified_at IS NOT NULL
       AND routing_ready = false
       AND tls_ready = false;

  ELSIF p_target = 'tls_ready' THEN
    UPDATE ipat_platform.tenant_domains
       SET tls_ready = true,
           lifecycle_last_checked_at = clock_timestamp(),
           lifecycle_evidence_code = p_evidence_code,
           lifecycle_last_error_code = NULL
     WHERE fqdn = p_fqdn
       AND domain_kind = 'custom_domain'
       AND state = 'pending'
       AND ownership_verified_at IS NOT NULL
       AND routing_ready = true
       AND tls_ready = false;

  ELSE
    UPDATE ipat_platform.tenant_domains
       SET state = 'verified',
           verified_at = clock_timestamp(),
           lifecycle_last_checked_at = clock_timestamp(),
           lifecycle_evidence_code = p_evidence_code,
           lifecycle_last_error_code = NULL
     WHERE fqdn = p_fqdn
       AND domain_kind = 'custom_domain'
       AND state = 'pending'
       AND ownership_verified_at IS NOT NULL
       AND routing_ready = true
       AND tls_ready = true
       AND verified_at IS NULL;
  END IF;

  GET DIAGNOSTICS v_changed = ROW_COUNT;
  RETURN v_changed = 1;
END
$$;
ALTER FUNCTION ipat_platform.advance_tenant_domain_lifecycle(text,text,text)
  OWNER TO ipat_domain_verifier_owner;
REVOKE ALL ON FUNCTION ipat_platform.advance_tenant_domain_lifecycle(text,text,text)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.advance_tenant_domain_lifecycle(text,text,text)
  TO ipat_domain_verifier;

CREATE FUNCTION ipat_platform.record_tenant_domain_lifecycle_error(
  p_fqdn text,
  p_error_code text
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
DECLARE
  v_changed integer;
BEGIN
  IF p_fqdn IS NULL
     OR p_error_code IS NULL
     OR length(p_error_code) NOT BETWEEN 3 AND 96
     OR p_error_code !~ '^[A-Z0-9_.:-]+$' THEN
    RETURN false;
  END IF;

  UPDATE ipat_platform.tenant_domains
     SET lifecycle_last_checked_at = clock_timestamp(),
         lifecycle_last_error_code = p_error_code
   WHERE fqdn = p_fqdn
     AND domain_kind = 'custom_domain'
     AND state = 'pending';

  GET DIAGNOSTICS v_changed = ROW_COUNT;
  RETURN v_changed = 1;
END
$$;
ALTER FUNCTION ipat_platform.record_tenant_domain_lifecycle_error(text,text)
  OWNER TO ipat_domain_verifier_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_tenant_domain_lifecycle_error(text,text)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.record_tenant_domain_lifecycle_error(text,text)
  TO ipat_domain_verifier;

COMMIT;
