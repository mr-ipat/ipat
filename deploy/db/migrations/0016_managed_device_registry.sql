-- R9.57 production-path metadata-only Device Registry.
-- Requires 0001-0015. No network access, worker execution or credentials are granted.
BEGIN;
CREATE TABLE ipat_ops.managed_devices (
  tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
  id uuid NOT NULL,
  request_id uuid NOT NULL,
  pop_id text NOT NULL CHECK (pop_id ~ '^[A-Za-z0-9_.-]{1,128}$'),
  display_name text NOT NULL CHECK (length(display_name) BETWEEN 1 AND 120 AND display_name = btrim(display_name)),
  device_kind text NOT NULL CHECK (device_kind IN ('olt','ont','router')),
  vendor text NOT NULL CHECK (vendor ~ '^[A-Za-z0-9_.-]{1,64}$'),
  intended_model text CHECK (intended_model IS NULL OR length(intended_model) BETWEEN 1 AND 128),
  management_transport text NOT NULL CHECK (management_transport IN ('ssh','snmp','routeros_api_ssl','cwmp','usp')),
  management_host text CHECK (management_host IS NULL OR
    (length(management_host) BETWEEN 3 AND 253 AND management_host ~ '^[A-Za-z0-9][A-Za-z0-9.:-]*$')),
  management_port integer CHECK (management_port IS NULL OR management_port BETWEEN 1 AND 65535),
  -- Vault reference only. Never a credential, URL containing password, or raw CLI transcript.
  secret_ref text CHECK (secret_ref IS NULL OR
    (length(secret_ref) BETWEEN 55 AND 255 AND
     secret_ref LIKE 'vault://tenant/' || tenant_id::text || '/%' AND
     secret_ref ~ '^vault://tenant/[a-f0-9-]+/[A-Za-z0-9/_-]+$')),
  lifecycle_state text NOT NULL DEFAULT 'SAVED' CHECK (lifecycle_state = 'SAVED'),
  added_by_issuer text NOT NULL, added_by_subject text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
  PRIMARY KEY (tenant_id,id), UNIQUE (tenant_id,request_id),
  CHECK ((management_transport IN ('cwmp','usp') AND management_host IS NULL AND management_port IS NULL)
     OR (management_transport NOT IN ('cwmp','usp') AND management_host IS NOT NULL AND management_port IS NOT NULL)),
  CHECK ((management_transport = 'routeros_api_ssl' AND device_kind = 'router')
     OR management_transport <> 'routeros_api_ssl')
);
ALTER TABLE ipat_ops.managed_devices OWNER TO ipat_schema_owner;
CREATE INDEX managed_devices_tenant_pop ON ipat_ops.managed_devices(tenant_id,pop_id,id);
ALTER TABLE ipat_ops.managed_devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.managed_devices FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.managed_devices FROM PUBLIC, ipat_app_runtime;
CREATE TABLE ipat_ops.managed_device_audit (
  tenant_id uuid NOT NULL, device_id uuid NOT NULL, event text NOT NULL DEFAULT 'METADATA_SAVED'
    CHECK (event = 'METADATA_SAVED'),
  occurred_at timestamptz NOT NULL DEFAULT statement_timestamp(),
  PRIMARY KEY(tenant_id,device_id),
  FOREIGN KEY(tenant_id,device_id) REFERENCES ipat_ops.managed_devices(tenant_id,id)
);
ALTER TABLE ipat_ops.managed_device_audit OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.managed_device_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.managed_device_audit FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.managed_device_audit FROM PUBLIC, ipat_app_runtime;

CREATE ROLE ipat_device_registry_owner NOLOGIN NOSUPERUSER NOCREATEDB
  NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_device_registry_exec NOLOGIN NOSUPERUSER NOCREATEDB
  NOCREATEROLE NOREPLICATION NOBYPASSRLS;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_device_registry_owner, ipat_device_registry_exec;
GRANT USAGE ON SCHEMA ipat_ops TO ipat_device_registry_owner;
GRANT SELECT ON ipat_platform.tenants TO ipat_device_registry_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(text,text,uuid,text,text)
  TO ipat_device_registry_owner;
GRANT SELECT,INSERT ON ipat_ops.managed_devices,ipat_ops.managed_device_audit
  TO ipat_device_registry_owner;
CREATE POLICY managed_device_owner_read ON ipat_ops.managed_devices
  FOR SELECT TO ipat_device_registry_owner USING (true);
CREATE POLICY managed_device_owner_insert ON ipat_ops.managed_devices
  FOR INSERT TO ipat_device_registry_owner WITH CHECK (true);
CREATE POLICY managed_device_audit_owner_read ON ipat_ops.managed_device_audit
  FOR SELECT TO ipat_device_registry_owner USING (true);
CREATE POLICY managed_device_audit_owner_insert ON ipat_ops.managed_device_audit
  FOR INSERT TO ipat_device_registry_owner WITH CHECK (true);

-- The application must pass a server-verified issuer/subject/tenant, not headers.
-- DB independently requires current tenant_admin membership, even on replay.
CREATE FUNCTION ipat_platform.register_managed_device(
 p_issuer text,p_subject text,p_tenant uuid,p_device_id uuid,p_request_id uuid,
 p_pop text,p_name text,p_kind text,p_vendor text,p_model text,p_transport text,
 p_host text,p_port integer,p_secret_ref text
) RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE saved ipat_ops.managed_devices%ROWTYPE;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
    OR p_device_id IS NULL OR p_request_id IS NULL OR p_pop IS NULL
    OR p_name IS NULL OR p_kind IS NULL OR p_vendor IS NULL OR p_transport IS NULL
    OR NOT EXISTS (SELECT 1 FROM ipat_platform.lookup_active_membership(
      p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN
   RETURN NULL;
 END IF;
 -- Reject conflicting replays even when concurrently racing on the same request.
 PERFORM pg_advisory_xact_lock(hashtextextended(p_tenant::text || p_request_id::text, 0));
 SELECT * INTO saved FROM ipat_ops.managed_devices
    WHERE tenant_id=p_tenant AND request_id=p_request_id;
 IF FOUND THEN
   IF saved.added_by_issuer=p_issuer AND saved.added_by_subject=p_subject
      AND saved.pop_id=p_pop AND saved.display_name=p_name
      AND saved.device_kind=p_kind AND saved.vendor=p_vendor
      AND saved.intended_model IS NOT DISTINCT FROM p_model
      AND saved.management_transport=p_transport
      AND saved.management_host IS NOT DISTINCT FROM p_host
      AND saved.management_port IS NOT DISTINCT FROM p_port
      AND saved.secret_ref IS NOT DISTINCT FROM p_secret_ref
   THEN RETURN saved.id; END IF;
   RETURN NULL;
 END IF;
 INSERT INTO ipat_ops.managed_devices(
    tenant_id,id,request_id,pop_id,display_name,device_kind,vendor,
    intended_model,management_transport,management_host,management_port,
    secret_ref,added_by_issuer,added_by_subject
 ) VALUES(p_tenant,p_device_id,p_request_id,p_pop,p_name,p_kind,p_vendor,
    p_model,p_transport,p_host,p_port,p_secret_ref,p_issuer,p_subject);
 INSERT INTO ipat_ops.managed_device_audit(tenant_id,device_id)
 VALUES(p_tenant,p_device_id);
 RETURN p_device_id;
EXCEPTION WHEN check_violation OR unique_violation OR foreign_key_violation THEN
 -- Never leak tenant-specific uniqueness, device IDs, or constraints to caller.
 RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.register_managed_device(
 text,text,uuid,uuid,uuid,text,text,text,text,text,text,text,integer,text)
 OWNER TO ipat_device_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.register_managed_device(
 text,text,uuid,uuid,uuid,text,text,text,text,text,text,text,integer,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.register_managed_device(
 text,text,uuid,uuid,uuid,text,text,text,text,text,text,text,integer,text)
 TO ipat_device_registry_exec;

-- Deliberately excludes management endpoint and vault ref from read results.
-- NOC reader requires exact POP; tenant admin may list all assigned POPs.
CREATE FUNCTION ipat_platform.list_managed_devices(
 p_issuer text,p_subject text,p_tenant uuid,p_pop text
) RETURNS TABLE(id uuid,pop_id text,display_name text,device_kind text,
 vendor text,intended_model text,management_transport text,lifecycle_state text,
 created_at timestamptz)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT d.id,d.pop_id,d.display_name,d.device_kind,d.vendor,
        d.intended_model,d.management_transport,d.lifecycle_state,d.created_at
 FROM ipat_ops.managed_devices d
 WHERE d.tenant_id=p_tenant AND (p_pop IS NULL OR d.pop_id=p_pop)
  AND (EXISTS (SELECT 1 FROM ipat_platform.lookup_active_membership(
       p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
    OR (p_pop IS NOT NULL AND EXISTS (
       SELECT 1 FROM ipat_platform.lookup_active_membership(
       p_issuer,p_subject,p_tenant,'noc_engineer',p_pop))))
 ORDER BY d.created_at DESC,d.id LIMIT 100
$body$;
ALTER FUNCTION ipat_platform.list_managed_devices(text,text,uuid,text)
 OWNER TO ipat_device_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_managed_devices(text,text,uuid,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_managed_devices(text,text,uuid,text)
 TO ipat_device_registry_exec;
-- Runtime service login is provisioned outside this migration only after BFF gates.
COMMIT;
