-- R9.17 production-shaped custom-domain enrollment metadata.
-- Dashboard registration is tenant-admin scoped through sealed functions.
-- DNS verification itself remains a separate network/evidence step.
BEGIN;

ALTER TABLE ipat_platform.tenant_domains
  ADD COLUMN routing_mode text
    CHECK (routing_mode IS NULL OR routing_mode IN ('a_record','cname','nameserver')),
  ADD COLUMN verification_name text,
  ADD COLUMN verification_value text,
  ADD COLUMN requested_by_issuer text,
  ADD COLUMN requested_by_subject text,
  ADD COLUMN requested_at timestamptz;

ALTER TABLE ipat_platform.tenant_domains
  ADD CONSTRAINT tenant_domains_custom_request_metadata CHECK (
    domain_type <> 'custom_domain'
    OR routing_mode IS NULL
    OR (
      verification_method = 'dns_txt'
      AND verification_name = '_ipat-verify.' || hostname
      AND verification_value LIKE 'ipat-domain=%'
      AND length(verification_value) BETWEEN 20 AND 160
      AND requested_by_issuer IS NOT NULL
      AND requested_by_subject IS NOT NULL
      AND requested_at IS NOT NULL
    )
  );

CREATE ROLE ipat_domain_enrollment_owner
  NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_domain_admin
  NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;

-- Credential material is deployment-owned and MUST NOT be stored in Git.
CREATE ROLE ipat_domain_admin_login
  LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
  IN ROLE ipat_domain_admin;

GRANT USAGE ON SCHEMA ipat_platform TO ipat_domain_enrollment_owner, ipat_domain_admin;
GRANT SELECT,INSERT,UPDATE ON ipat_platform.tenant_domains
  TO ipat_domain_enrollment_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
  text,text,uuid,text,text
) TO ipat_domain_enrollment_owner;

CREATE OR REPLACE FUNCTION ipat_platform.request_tenant_custom_domain(
  p_issuer text,
  p_subject text,
  p_tenant uuid,
  p_domain_id uuid,
  p_hostname text,
  p_routing_mode text,
  p_verification_value text
)
RETURNS uuid
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
DECLARE
  v_id uuid;
BEGIN
  IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
     OR p_domain_id IS NULL OR p_hostname IS NULL
     OR p_routing_mode NOT IN ('a_record','cname','nameserver')
     OR p_verification_value IS NULL
     OR p_verification_value NOT LIKE 'ipat-domain=%'
     OR length(p_verification_value) NOT BETWEEN 20 AND 160 THEN
    RETURN NULL;
  END IF;

  IF NOT EXISTS (
    SELECT 1
    FROM ipat_platform.lookup_active_membership(
      p_issuer,p_subject,p_tenant,'tenant_admin',NULL
    )
  ) THEN
    RETURN NULL;
  END IF;

  INSERT INTO ipat_platform.tenant_domains(
    id,tenant_id,hostname,domain_type,verification_state,verification_method,
    routing_mode,verification_name,verification_value,
    requested_by_issuer,requested_by_subject,requested_at
  )
  VALUES(
    p_domain_id,p_tenant,p_hostname,'custom_domain','pending','dns_txt',
    p_routing_mode,'_ipat-verify.' || p_hostname,p_verification_value,
    p_issuer,p_subject,clock_timestamp()
  )
  ON CONFLICT (hostname) DO NOTHING
  RETURNING id INTO v_id;

  RETURN v_id;
END
$$;
ALTER FUNCTION ipat_platform.request_tenant_custom_domain(
  text,text,uuid,uuid,text,text,text
) OWNER TO ipat_domain_enrollment_owner;
REVOKE ALL ON FUNCTION ipat_platform.request_tenant_custom_domain(
  text,text,uuid,uuid,text,text,text
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.request_tenant_custom_domain(
  text,text,uuid,uuid,text,text,text
) TO ipat_domain_admin;

CREATE OR REPLACE FUNCTION ipat_platform.list_tenant_domains_for_member(
  p_issuer text,
  p_subject text,
  p_tenant uuid
)
RETURNS TABLE(
  id uuid,
  hostname text,
  domain_type text,
  verification_state text,
  verification_method text,
  routing_mode text,
  verification_name text,
  verification_value text,
  verified_at timestamptz,
  disabled_at timestamptz,
  created_at timestamptz
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
  SELECT d.id,d.hostname,d.domain_type,d.verification_state,
         d.verification_method,d.routing_mode,d.verification_name,
         d.verification_value,d.verified_at,d.disabled_at,d.created_at
  FROM ipat_platform.tenant_domains d
  WHERE d.tenant_id = p_tenant
    AND EXISTS (
      SELECT 1
      FROM ipat_platform.lookup_active_membership(
        p_issuer,p_subject,p_tenant,'tenant_admin',NULL
      )
    )
  ORDER BY d.created_at DESC,d.id
  LIMIT 100
$$;
ALTER FUNCTION ipat_platform.list_tenant_domains_for_member(
  text,text,uuid
) OWNER TO ipat_domain_enrollment_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_domains_for_member(
  text,text,uuid
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_domains_for_member(
  text,text,uuid
) TO ipat_domain_admin;

CREATE OR REPLACE FUNCTION ipat_platform.disable_tenant_custom_domain(
  p_issuer text,
  p_subject text,
  p_tenant uuid,
  p_domain_id uuid
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
DECLARE
  v_changed integer;
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM ipat_platform.lookup_active_membership(
      p_issuer,p_subject,p_tenant,'tenant_admin',NULL
    )
  ) THEN
    RETURN false;
  END IF;

  UPDATE ipat_platform.tenant_domains
  SET verification_state='disabled', disabled_at=clock_timestamp()
  WHERE id=p_domain_id
    AND tenant_id=p_tenant
    AND domain_type='custom_domain'
    AND verification_state <> 'disabled';

  GET DIAGNOSTICS v_changed = ROW_COUNT;
  RETURN v_changed = 1;
END
$$;
ALTER FUNCTION ipat_platform.disable_tenant_custom_domain(
  text,text,uuid,uuid
) OWNER TO ipat_domain_enrollment_owner;
REVOKE ALL ON FUNCTION ipat_platform.disable_tenant_custom_domain(
  text,text,uuid,uuid
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.disable_tenant_custom_domain(
  text,text,uuid,uuid
) TO ipat_domain_admin;

REVOKE ALL ON ipat_platform.tenant_domains FROM ipat_domain_admin;
REVOKE ALL ON ipat_platform.tenant_domains FROM ipat_domain_admin_login;

COMMIT;
