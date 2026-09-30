-- R9.19 production-shaped tenant-domain activation lifecycle.
-- Separates customer request authority from automated verifier authority.
BEGIN;

ALTER TABLE ipat_platform.tenant_domains
  ADD COLUMN activation_state text NOT NULL DEFAULT 'pending_dns'
    CHECK (activation_state IN (
      'pending_dns','ownership_verified','routing_ready','tls_ready','active','disabled'
    )),
  ADD COLUMN ownership_verified_at timestamptz,
  ADD COLUMN routing_ready_at timestamptz,
  ADD COLUMN tls_ready_at timestamptz,
  ADD COLUMN activated_at timestamptz,
  ADD COLUMN last_checked_at timestamptz,
  ADD COLUMN last_error_code text,
  ADD COLUMN last_evidence_sha256 text;

UPDATE ipat_platform.tenant_domains
SET activation_state = CASE
      WHEN verification_state='disabled' THEN 'disabled'
      WHEN verification_state='verified' THEN 'ownership_verified'
      ELSE 'pending_dns'
    END,
    ownership_verified_at = CASE
      WHEN verification_state='verified' THEN verified_at
      ELSE NULL
    END;

ALTER TABLE ipat_platform.tenant_domains
  ADD CONSTRAINT tenant_domain_activation_consistency CHECK (
    (activation_state='pending_dns'
      AND ownership_verified_at IS NULL
      AND routing_ready_at IS NULL AND tls_ready_at IS NULL AND activated_at IS NULL)
    OR
    (activation_state='ownership_verified'
      AND ownership_verified_at IS NOT NULL
      AND routing_ready_at IS NULL AND tls_ready_at IS NULL AND activated_at IS NULL)
    OR
    (activation_state='routing_ready'
      AND ownership_verified_at IS NOT NULL
      AND routing_ready_at IS NOT NULL
      AND tls_ready_at IS NULL AND activated_at IS NULL)
    OR
    (activation_state='tls_ready'
      AND ownership_verified_at IS NOT NULL
      AND routing_ready_at IS NOT NULL
      AND tls_ready_at IS NOT NULL AND activated_at IS NULL)
    OR
    (activation_state='active'
      AND ownership_verified_at IS NOT NULL
      AND routing_ready_at IS NOT NULL
      AND tls_ready_at IS NOT NULL AND activated_at IS NOT NULL)
    OR activation_state='disabled'
  ),
  ADD CONSTRAINT tenant_domain_evidence_digest_format CHECK (
    last_evidence_sha256 IS NULL OR last_evidence_sha256 ~ '^[0-9a-f]{64}$'
  ),
  ADD CONSTRAINT tenant_domain_error_code_format CHECK (
    last_error_code IS NULL OR (
      length(last_error_code) BETWEEN 3 AND 96
      AND last_error_code ~ '^[A-Z0-9_]+$'
    )
  );

CREATE ROLE ipat_domain_verifier
  NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE ROLE ipat_domain_verifier_login
  LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
  IN ROLE ipat_domain_verifier;

CREATE OR REPLACE FUNCTION ipat_platform.record_tenant_domain_check(
  p_domain_id uuid,
  p_event text,
  p_evidence_sha256 text,
  p_error_code text
)
RETURNS text
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $$
DECLARE
  v_state text;
BEGIN
  IF p_domain_id IS NULL
     OR p_event NOT IN ('ownership_verified','routing_ready','tls_ready','activate','check_failed')
     OR (p_evidence_sha256 IS NOT NULL AND p_evidence_sha256 !~ '^[0-9a-f]{64}$')
     OR (p_error_code IS NOT NULL AND (
          length(p_error_code) NOT BETWEEN 3 AND 96
          OR p_error_code !~ '^[A-Z0-9_]+$'
        )) THEN
    RETURN NULL;
  END IF;

  SELECT activation_state INTO v_state
  FROM ipat_platform.tenant_domains
  WHERE id=p_domain_id AND domain_type='custom_domain'
  FOR UPDATE;

  IF NOT FOUND OR v_state='disabled' THEN
    RETURN NULL;
  END IF;

  IF p_event='check_failed' THEN
    UPDATE ipat_platform.tenant_domains
    SET last_checked_at=clock_timestamp(),
        last_error_code=COALESCE(p_error_code,'CHECK_FAILED'),
        last_evidence_sha256=p_evidence_sha256
    WHERE id=p_domain_id;
    RETURN v_state;
  END IF;

  IF p_error_code IS NOT NULL OR p_evidence_sha256 IS NULL THEN
    RETURN NULL;
  END IF;

  IF p_event='ownership_verified' AND v_state='pending_dns' THEN
    UPDATE ipat_platform.tenant_domains
    SET activation_state='ownership_verified',
        verification_state='verified',
        verified_at=clock_timestamp(),
        ownership_verified_at=clock_timestamp(),
        last_checked_at=clock_timestamp(),
        last_error_code=NULL,
        last_evidence_sha256=p_evidence_sha256
    WHERE id=p_domain_id;
    RETURN 'ownership_verified';
  ELSIF p_event='routing_ready' AND v_state='ownership_verified' THEN
    UPDATE ipat_platform.tenant_domains
    SET activation_state='routing_ready',
        routing_ready_at=clock_timestamp(),
        last_checked_at=clock_timestamp(),
        last_error_code=NULL,
        last_evidence_sha256=p_evidence_sha256
    WHERE id=p_domain_id;
    RETURN 'routing_ready';
  ELSIF p_event='tls_ready' AND v_state='routing_ready' THEN
    UPDATE ipat_platform.tenant_domains
    SET activation_state='tls_ready',
        tls_ready_at=clock_timestamp(),
        last_checked_at=clock_timestamp(),
        last_error_code=NULL,
        last_evidence_sha256=p_evidence_sha256
    WHERE id=p_domain_id;
    RETURN 'tls_ready';
  ELSIF p_event='activate' AND v_state='tls_ready' THEN
    UPDATE ipat_platform.tenant_domains
    SET activation_state='active',
        activated_at=clock_timestamp(),
        last_checked_at=clock_timestamp(),
        last_error_code=NULL,
        last_evidence_sha256=p_evidence_sha256
    WHERE id=p_domain_id;
    RETURN 'active';
  END IF;

  RETURN NULL;
END
$$;
ALTER FUNCTION ipat_platform.record_tenant_domain_check(uuid,text,text,text)
  OWNER TO ipat_schema_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_tenant_domain_check(uuid,text,text,text)
  FROM PUBLIC;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_domain_verifier;
GRANT EXECUTE ON FUNCTION ipat_platform.record_tenant_domain_check(uuid,text,text,text)
  TO ipat_domain_verifier;

DROP FUNCTION ipat_platform.list_tenant_domains_for_member(text,text,uuid);
CREATE FUNCTION ipat_platform.list_tenant_domains_for_member(
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
  activation_state text,
  ownership_verified_at timestamptz,
  routing_ready_at timestamptz,
  tls_ready_at timestamptz,
  activated_at timestamptz,
  last_checked_at timestamptz,
  last_error_code text,
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
         d.verification_value,d.activation_state,d.ownership_verified_at,
         d.routing_ready_at,d.tls_ready_at,d.activated_at,d.last_checked_at,
         d.last_error_code,d.verified_at,d.disabled_at,d.created_at
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
ALTER FUNCTION ipat_platform.list_tenant_domains_for_member(text,text,uuid)
  OWNER TO ipat_domain_enrollment_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_domains_for_member(text,text,uuid)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_domains_for_member(text,text,uuid)
  TO ipat_domain_admin;

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
  SET verification_state='disabled',
      activation_state='disabled',
      disabled_at=clock_timestamp(),
      last_checked_at=clock_timestamp()
  WHERE id=p_domain_id
    AND tenant_id=p_tenant
    AND domain_type='custom_domain'
    AND activation_state <> 'disabled';

  GET DIAGNOSTICS v_changed = ROW_COUNT;
  RETURN v_changed = 1;
END
$$;
ALTER FUNCTION ipat_platform.disable_tenant_custom_domain(text,text,uuid,uuid)
  OWNER TO ipat_domain_enrollment_owner;
REVOKE ALL ON FUNCTION ipat_platform.disable_tenant_custom_domain(text,text,uuid,uuid)
  FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.disable_tenant_custom_domain(text,text,uuid,uuid)
  TO ipat_domain_admin;

REVOKE ALL ON ipat_platform.tenant_domains FROM ipat_domain_verifier;
REVOKE ALL ON ipat_platform.tenant_domains FROM ipat_domain_verifier_login;

COMMIT;
