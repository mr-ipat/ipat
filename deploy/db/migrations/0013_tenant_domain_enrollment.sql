-- R9.53 production-path custom-domain enrollment on top of R9.52.
-- Tenant Admin may request/list/revoke its own custom domains through sealed
-- functions only. DNS verification and TLS activation remain separate gates.
BEGIN;

ALTER TABLE ipat_platform.tenant_domains
  ADD COLUMN routing_mode text
    CHECK (routing_mode IS NULL OR routing_mode IN ('a_record','cname','nameserver')),
  ADD COLUMN verification_name text,
  ADD COLUMN verification_value text,
  ADD COLUMN requested_by_issuer text,
  ADD COLUMN requested_by_subject text,
  ADD COLUMN requested_at timestamptz,
  ADD COLUMN ownership_verified_at timestamptz,
  ADD COLUMN routing_ready boolean NOT NULL DEFAULT false;

-- Preserve semantics of any pre-R9.53 row that was already explicitly marked
-- verified+TLS-ready by the earlier R9.52 verification workflow.
UPDATE ipat_platform.tenant_domains
SET ownership_verified_at = verified_at,
    routing_ready = tls_ready
WHERE domain_kind = 'custom_domain'
  AND state = 'verified'
  AND verified_at IS NOT NULL
  AND tls_ready = true;

ALTER TABLE ipat_platform.tenant_domains
  ADD CONSTRAINT tenant_domains_custom_request_metadata CHECK (
    domain_kind <> 'custom_domain'
    OR routing_mode IS NULL
    OR (
      verification_method = 'dns_txt'
      AND verification_name = '_ipat-verify.' || fqdn
      AND verification_value LIKE 'ipat-domain=%'
      AND length(verification_value) BETWEEN 20 AND 160
      AND requested_by_issuer IS NOT NULL
      AND requested_by_subject IS NOT NULL
      AND requested_at IS NOT NULL
    )
  ),
  ADD CONSTRAINT tenant_domains_verified_custom_gates CHECK (
    domain_kind <> 'custom_domain'
    OR state <> 'verified'
    OR (
      ownership_verified_at IS NOT NULL
      AND routing_ready = true
      AND tls_ready = true
    )
  );

CREATE ROLE ipat_domain_enrollment_owner NOLOGIN NOSUPERUSER NOCREATEDB
  NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_domain_admin NOLOGIN NOSUPERUSER NOCREATEDB
  NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;

GRANT USAGE ON SCHEMA ipat_platform
  TO ipat_domain_enrollment_owner, ipat_domain_admin;
GRANT SELECT,INSERT,UPDATE ON ipat_platform.tenant_domains
  TO ipat_domain_enrollment_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
  text,text,uuid,text,text
) TO ipat_domain_enrollment_owner;

-- FORCE RLS remains active. Only this non-login function owner gets policies;
-- runtime capability roles cannot SELECT/INSERT/UPDATE the table directly.
CREATE POLICY tenant_domain_enrollment_select
  ON ipat_platform.tenant_domains FOR SELECT
  TO ipat_domain_enrollment_owner USING (true);
CREATE POLICY tenant_domain_enrollment_insert
  ON ipat_platform.tenant_domains FOR INSERT
  TO ipat_domain_enrollment_owner WITH CHECK (true);
CREATE POLICY tenant_domain_enrollment_update
  ON ipat_platform.tenant_domains FOR UPDATE
  TO ipat_domain_enrollment_owner USING (true) WITH CHECK (true);

CREATE FUNCTION ipat_platform.request_tenant_custom_domain(
  p_issuer text,
  p_subject text,
  p_tenant uuid,
  p_fqdn text,
  p_routing_mode text,
  p_verification_value text
)
RETURNS text
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
DECLARE
  v_fqdn text;
BEGIN
  IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
     OR p_fqdn IS NULL
     OR p_fqdn <> lower(p_fqdn)
     OR length(p_fqdn) NOT BETWEEN 3 AND 253
     OR p_fqdn !~ '^[a-z0-9][a-z0-9.-]*[a-z0-9]$'
     OR position('.' in p_fqdn) <= 1
     OR p_fqdn LIKE '%..%'
     OR p_fqdn LIKE '%.-%'
     OR p_fqdn LIKE '%-.%'
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
    tenant_id,fqdn,domain_kind,verification_method,state,verified_at,tls_ready,
    routing_mode,verification_name,verification_value,
    requested_by_issuer,requested_by_subject,requested_at,
    ownership_verified_at,routing_ready
  )
  VALUES(
    p_tenant,p_fqdn,'custom_domain','dns_txt','pending',NULL,false,
    p_routing_mode,'_ipat-verify.' || p_fqdn,p_verification_value,
    p_issuer,p_subject,clock_timestamp(),NULL,false
  )
  ON CONFLICT (fqdn) DO NOTHING
  RETURNING fqdn INTO v_fqdn;

  RETURN v_fqdn;
END
$$;
ALTER FUNCTION ipat_platform.request_tenant_custom_domain(
  text,text,uuid,text,text,text
) OWNER TO ipat_domain_enrollment_owner;
REVOKE ALL ON FUNCTION ipat_platform.request_tenant_custom_domain(
  text,text,uuid,text,text,text
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.request_tenant_custom_domain(
  text,text,uuid,text,text,text
) TO ipat_domain_admin;

CREATE FUNCTION ipat_platform.list_tenant_domains_for_member(
  p_issuer text,
  p_subject text,
  p_tenant uuid
)
RETURNS TABLE(
  fqdn text,
  domain_kind text,
  state text,
  verification_method text,
  routing_mode text,
  verification_name text,
  verification_value text,
  ownership_verified_at timestamptz,
  routing_ready boolean,
  tls_ready boolean,
  verified_at timestamptz,
  created_at timestamptz
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
  SELECT d.fqdn,d.domain_kind,d.state,d.verification_method,
         d.routing_mode,d.verification_name,d.verification_value,
         d.ownership_verified_at,d.routing_ready,d.tls_ready,
         d.verified_at,d.created_at
  FROM ipat_platform.tenant_domains d
  WHERE d.tenant_id = p_tenant
    AND EXISTS (
      SELECT 1
      FROM ipat_platform.lookup_active_membership(
        p_issuer,p_subject,p_tenant,'tenant_admin',NULL
      )
    )
  ORDER BY d.created_at DESC,d.fqdn
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

CREATE FUNCTION ipat_platform.revoke_tenant_custom_domain(
  p_issuer text,
  p_subject text,
  p_tenant uuid,
  p_fqdn text
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
  SET state='revoked',
      tls_ready=false,
      routing_ready=false
  WHERE fqdn=p_fqdn
    AND tenant_id=p_tenant
    AND domain_kind='custom_domain'
    AND state <> 'revoked';

  GET DIAGNOSTICS v_changed = ROW_COUNT;
  RETURN v_changed = 1;
END
$$;
ALTER FUNCTION ipat_platform.revoke_tenant_custom_domain(
  text,text,uuid,text
) OWNER TO ipat_domain_enrollment_owner;
REVOKE ALL ON FUNCTION ipat_platform.revoke_tenant_custom_domain(
  text,text,uuid,text
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.revoke_tenant_custom_domain(
  text,text,uuid,text
) TO ipat_domain_admin;

-- No LOGIN role is created here. Deployment provisions a dedicated login
-- outside migrations and grants only ipat_domain_admin after auth review.
COMMIT;
