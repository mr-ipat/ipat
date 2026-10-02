-- R9.65 tenant Device metadata CRUD. Requires 0001-0019 in order.
-- Not deployed on the actual owner VPS. Never sends device commands.
-- Archive is NOT equipment disconnect, secret revocation or physical delete.
BEGIN;
DO $require_validated_site$
BEGIN
 IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE
   conrelid='ipat_ops.managed_devices'::regclass AND
   conname='managed_devices_registered_site_fk' AND convalidated)
 THEN RAISE EXCEPTION 'TENANT_SITE_MIGRATION_0019_MUST_BE_VALIDATED'; END IF;
END $require_validated_site$;

-- Every separate Site instance has an immutable identity. Archived devices
-- keep its ID/code as a HISTORICAL snapshot, NOT an active FK; deleting and
-- recreating the same Site code cannot silently alter historical identity.
ALTER TABLE ipat_ops.tenant_sites
 ADD COLUMN site_instance_id uuid NOT NULL DEFAULT gen_random_uuid(),
 ADD CONSTRAINT tenant_sites_instance_id_unique UNIQUE(tenant_id,site_instance_id);
ALTER TABLE ipat_ops.managed_devices
 ALTER COLUMN pop_id DROP NOT NULL;
ALTER TABLE ipat_ops.managed_devices
 ADD COLUMN archived_site_code text CHECK (archived_site_code IS NULL OR
      archived_site_code ~ '^[A-Za-z0-9_.-]{1,128}$'),
 ADD COLUMN archived_site_instance_id uuid,
 ADD COLUMN archived_site_name text CHECK (archived_site_name IS NULL OR
      (length(archived_site_name) BETWEEN 1 AND 120)),
 ADD COLUMN metadata_revision bigint NOT NULL DEFAULT 1
   CHECK (metadata_revision BETWEEN 1 AND 9223372036854775806),
 ADD COLUMN archived_at timestamptz,
 DROP CONSTRAINT managed_devices_lifecycle_state_check,
 ADD CONSTRAINT managed_device_metadata_archive_state CHECK
   ((lifecycle_state='SAVED' AND archived_at IS NULL AND
       pop_id IS NOT NULL AND archived_site_code IS NULL AND
       archived_site_instance_id IS NULL AND archived_site_name IS NULL)
     OR (lifecycle_state='ARCHIVED' AND archived_at IS NOT NULL AND
       pop_id IS NULL AND archived_site_code IS NOT NULL AND
       archived_site_instance_id IS NOT NULL AND archived_site_name IS NOT NULL));

-- Prior one-time 0016 audit stays immutable. New append-only event stream
-- can record every human-issued edit/archive without direct runtime access.
CREATE TABLE ipat_ops.managed_device_events (
 id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
 tenant_id uuid NOT NULL,device_id uuid NOT NULL,
 event text NOT NULL CHECK(event IN('METADATA_EDITED','METADATA_ARCHIVED')),
 previous_revision bigint NOT NULL CHECK(previous_revision>=1),
 new_revision bigint NOT NULL CHECK(new_revision=previous_revision+1),
 previous_site text NOT NULL,new_site text,
 previous_site_instance_id uuid,
 CHECK ((event='METADATA_EDITED' AND new_site IS NOT NULL) OR
        (event='METADATA_ARCHIVED' AND new_site IS NULL AND
          previous_site_instance_id IS NOT NULL)),
 actor_issuer text NOT NULL,actor_subject text NOT NULL,
 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 FOREIGN KEY(tenant_id,device_id) REFERENCES ipat_ops.managed_devices(tenant_id,id)
);
ALTER TABLE ipat_ops.managed_device_events OWNER TO ipat_schema_owner;
CREATE INDEX managed_device_events_tenant_device ON ipat_ops.managed_device_events(tenant_id,device_id,id);
ALTER TABLE ipat_ops.managed_device_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.managed_device_events FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.managed_device_events FROM PUBLIC,ipat_app_runtime;
GRANT SELECT,INSERT ON ipat_ops.managed_device_events TO ipat_managed_registry_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_ops.managed_device_events_id_seq TO ipat_managed_registry_owner;
CREATE POLICY managed_device_events_owner_read ON ipat_ops.managed_device_events
 FOR SELECT TO ipat_managed_registry_owner USING(true);
CREATE POLICY managed_device_events_owner_insert ON ipat_ops.managed_device_events
 FOR INSERT TO ipat_managed_registry_owner WITH CHECK(true);
GRANT UPDATE(display_name,pop_id,intended_model,management_host,management_port,
  metadata_revision,lifecycle_state,archived_at,
  archived_site_code,archived_site_instance_id,archived_site_name)
 ON ipat_ops.managed_devices TO ipat_managed_registry_owner;
CREATE POLICY managed_device_owner_update ON ipat_ops.managed_devices
 FOR UPDATE TO ipat_managed_registry_owner USING(true) WITH CHECK(true);
-- A firmware change with unresolved impact blocks ALL metadata editing and
-- even metadata-only archive. No row may disappear from recovery inventory.
-- Only this NOLOGIN owner needs to read the prior active Site once to
-- capture a durable tombstone identity before releasing its live FK.
GRANT SELECT ON ipat_ops.tenant_sites TO ipat_managed_registry_owner;
CREATE POLICY device_registry_site_snapshot_read ON ipat_ops.tenant_sites
 FOR SELECT TO ipat_managed_registry_owner USING(true);
GRANT SELECT ON ipat_ops.firmware_changes TO ipat_managed_registry_owner;
CREATE POLICY fw_managed_registry_read ON ipat_ops.firmware_changes
 FOR SELECT TO ipat_managed_registry_owner USING(true);

CREATE FUNCTION ipat_platform.edit_managed_device_metadata(
 p_issuer text,p_subject text,p_tenant uuid,p_device uuid,p_expected_revision bigint,
 p_name text,p_site text,p_model text,p_host text,p_port integer)
 RETURNS bigint LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE old ipat_ops.managed_devices%ROWTYPE;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_device IS NULL
    OR p_expected_revision IS NULL OR p_expected_revision<1
    OR p_name IS NULL OR length(p_name) NOT BETWEEN 1 AND 120 OR p_name<>btrim(p_name)
    OR p_name ~ '[[:cntrl:]]'
    OR p_site IS NULL OR p_site !~ '^[A-Za-z0-9_.-]{1,128}$'
    OR (p_model IS NOT NULL AND (length(p_model) NOT BETWEEN 1 AND 128 OR p_model ~ '[[:cntrl:]]'))
    OR NOT EXISTS (SELECT 1 FROM ipat_platform.lookup_active_membership(
      p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 THEN RETURN NULL; END IF;
 SELECT * INTO old FROM ipat_ops.managed_devices
 WHERE tenant_id=p_tenant AND id=p_device FOR UPDATE;
 IF NOT FOUND OR old.lifecycle_state<>'SAVED' OR old.metadata_revision<>p_expected_revision
    OR old.metadata_revision>=9223372036854775806
    OR EXISTS (SELECT 1 FROM ipat_ops.firmware_changes f
       WHERE f.tenant_id=p_tenant AND f.device_id=p_device AND
             f.state IN ('AWAITING_EVIDENCE','APPROVED','EXECUTION_REQUESTED'))
 THEN RETURN NULL; END IF;
 IF ((old.management_transport IN ('cwmp','usp') AND
          (p_host IS NOT NULL OR p_port IS NOT NULL)) OR
     (old.management_transport NOT IN ('cwmp','usp') AND
          (p_host IS NULL OR length(p_host) NOT BETWEEN 3 AND 253 OR
           p_host !~ '^[A-Za-z0-9][A-Za-z0-9.:-]*$' OR
           p_port IS NULL OR p_port NOT BETWEEN 1 AND 65535)))
 THEN RETURN NULL; END IF;
 -- Any reference to a Vault secret means independent physical adoption
 -- MAY have happened. Only change a safe display label in that case; real
 -- endpoint/site/model transfer requires separate revocation/approval flow.
 IF old.secret_ref IS NOT NULL AND
    (old.pop_id<>p_site OR old.intended_model IS DISTINCT FROM p_model OR
     old.management_host IS DISTINCT FROM p_host OR
     old.management_port IS DISTINCT FROM p_port)
 THEN RETURN NULL; END IF;
 -- Site must already be registered under this exact tenant; compound FK
 -- independently rechecks it even during concurrent delete.
 UPDATE ipat_ops.managed_devices SET display_name=p_name,pop_id=p_site,
   intended_model=p_model,management_host=p_host,management_port=p_port,
   metadata_revision=old.metadata_revision+1
 WHERE tenant_id=p_tenant AND id=p_device;
 INSERT INTO ipat_ops.managed_device_events(tenant_id,device_id,event,
    previous_revision,new_revision,previous_site,new_site,actor_issuer,actor_subject)
 VALUES(p_tenant,p_device,'METADATA_EDITED',old.metadata_revision,
    old.metadata_revision+1,old.pop_id,p_site,p_issuer,p_subject);
 RETURN old.metadata_revision+1;
EXCEPTION WHEN foreign_key_violation OR check_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.edit_managed_device_metadata(
 text,text,uuid,uuid,bigint,text,text,text,text,integer)
 OWNER TO ipat_managed_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.edit_managed_device_metadata(
 text,text,uuid,uuid,bigint,text,text,text,text,integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.edit_managed_device_metadata(
 text,text,uuid,uuid,bigint,text,text,text,text,integer)
 TO ipat_managed_registry_exec;

CREATE FUNCTION ipat_platform.archive_managed_device_metadata(
 p_issuer text,p_subject text,p_tenant uuid,p_device uuid,p_expected_revision bigint)
 RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE old ipat_ops.managed_devices%ROWTYPE;
 previous_site ipat_ops.tenant_sites%ROWTYPE;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_device IS NULL
  OR p_expected_revision IS NULL OR p_expected_revision<1
  OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 THEN RETURN false; END IF;
 SELECT * INTO old FROM ipat_ops.managed_devices
 WHERE tenant_id=p_tenant AND id=p_device FOR UPDATE;
 IF NOT FOUND OR old.lifecycle_state<>'SAVED' OR old.metadata_revision<>p_expected_revision
   OR old.metadata_revision>=9223372036854775806 OR old.secret_ref IS NOT NULL
   OR EXISTS (SELECT 1 FROM ipat_ops.firmware_changes f WHERE
       f.tenant_id=p_tenant AND f.device_id=p_device AND
       f.state IN ('AWAITING_EVIDENCE','APPROVED','EXECUTION_REQUESTED'))
 THEN RETURN false; END IF;
 SELECT * INTO previous_site FROM ipat_ops.tenant_sites
  WHERE tenant_id=p_tenant AND code=old.pop_id;
 IF NOT FOUND THEN RETURN false; END IF;
 -- This releases the live compound FK only after retaining immutable
 -- historical Site instance identity. No credential/physical I/O is done.
 UPDATE ipat_ops.managed_devices SET lifecycle_state='ARCHIVED',pop_id=NULL,
   archived_site_code=old.pop_id,
   archived_site_instance_id=previous_site.site_instance_id,
   archived_site_name=previous_site.display_name,
   archived_at=statement_timestamp(),metadata_revision=old.metadata_revision+1
 WHERE tenant_id=p_tenant AND id=p_device;
 INSERT INTO ipat_ops.managed_device_events(tenant_id,device_id,event,
    previous_revision,new_revision,previous_site,new_site,previous_site_instance_id,
    actor_issuer,actor_subject)
 VALUES(p_tenant,p_device,'METADATA_ARCHIVED',old.metadata_revision,
    old.metadata_revision+1,old.pop_id,NULL,previous_site.site_instance_id,p_issuer,p_subject);
 RETURN true;
END $body$;
ALTER FUNCTION ipat_platform.archive_managed_device_metadata(text,text,uuid,uuid,bigint)
 OWNER TO ipat_managed_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.archive_managed_device_metadata(text,text,uuid,uuid,bigint)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.archive_managed_device_metadata(text,text,uuid,uuid,bigint)
 TO ipat_managed_registry_exec;

-- Rich admin-only View: never return vault refs, credentials or audit secrets.
CREATE FUNCTION ipat_platform.get_managed_device_metadata(
 p_issuer text,p_subject text,p_tenant uuid,p_device uuid)
 RETURNS TABLE(id uuid,pop_id text,display_name text,device_kind text,vendor text,
 intended_model text,management_transport text,management_host text,
 management_port integer,lifecycle_state text,metadata_revision bigint,
 created_at timestamptz,archived_at timestamptz,
 archived_site_instance_id uuid, archived_site_name text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT d.id,COALESCE(d.pop_id,d.archived_site_code),d.display_name,d.device_kind,d.vendor,d.intended_model,
    d.management_transport,d.management_host,d.management_port,d.lifecycle_state,
    d.metadata_revision,d.created_at,d.archived_at,
    d.archived_site_instance_id,d.archived_site_name
 FROM ipat_ops.managed_devices d WHERE d.tenant_id=p_tenant AND d.id=p_device
  AND EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
       p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
$body$;
ALTER FUNCTION ipat_platform.get_managed_device_metadata(text,text,uuid,uuid)
 OWNER TO ipat_managed_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.get_managed_device_metadata(text,text,uuid,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.get_managed_device_metadata(text,text,uuid,uuid)
 TO ipat_managed_registry_exec;

-- Admin-only keyset page includes archived tombstones for accountable history;
-- ordinary NOC R9.57 list remains secretless and active-only below.
CREATE FUNCTION ipat_platform.list_managed_devices_admin(
 p_issuer text,p_subject text,p_tenant uuid,p_include_archived boolean,
 p_after_created timestamptz,p_after_id uuid)
 RETURNS TABLE(id uuid,pop_id text,display_name text,device_kind text,vendor text,
 intended_model text,management_transport text,lifecycle_state text,
 metadata_revision bigint,created_at timestamptz)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT d.id,COALESCE(d.pop_id,d.archived_site_code),d.display_name,d.device_kind,d.vendor,d.intended_model,
   d.management_transport,d.lifecycle_state,d.metadata_revision,d.created_at
 FROM ipat_ops.managed_devices d
 WHERE d.tenant_id=p_tenant AND
   (p_include_archived IS TRUE OR d.lifecycle_state='SAVED') AND
   ((p_after_created IS NULL AND p_after_id IS NULL) OR
    (p_after_created IS NOT NULL AND p_after_id IS NOT NULL AND
       (d.created_at,d.id)<(p_after_created,p_after_id)))
   AND EXISTS (SELECT 1 FROM ipat_platform.lookup_active_membership(
      p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 ORDER BY d.created_at DESC,d.id DESC LIMIT 101
$body$;
ALTER FUNCTION ipat_platform.list_managed_devices_admin(text,text,uuid,boolean,timestamptz,uuid)
 OWNER TO ipat_managed_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_managed_devices_admin(text,text,uuid,boolean,timestamptz,uuid)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_managed_devices_admin(text,text,uuid,boolean,timestamptz,uuid)
 TO ipat_managed_registry_exec;

-- An exact register request replay must NOT reactivate an archived row.
-- Existing NOC list excludes archived rows. Admin archive history remains
-- accessible only through separately verified admin-specific functions.
CREATE OR REPLACE FUNCTION ipat_platform.register_managed_device(
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
   IF saved.lifecycle_state='SAVED' AND saved.added_by_issuer=p_issuer AND saved.added_by_subject=p_subject
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
CREATE OR REPLACE FUNCTION ipat_platform.list_managed_devices(
 p_issuer text,p_subject text,p_tenant uuid,p_pop text
) RETURNS TABLE(id uuid,pop_id text,display_name text,device_kind text,
 vendor text,intended_model text,management_transport text,lifecycle_state text,
 created_at timestamptz)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT d.id,d.pop_id,d.display_name,d.device_kind,d.vendor,
        d.intended_model,d.management_transport,d.lifecycle_state,d.created_at
 FROM ipat_ops.managed_devices d
 WHERE d.tenant_id=p_tenant AND d.lifecycle_state='SAVED' AND (p_pop IS NULL OR d.pop_id=p_pop)
  AND (EXISTS (SELECT 1 FROM ipat_platform.lookup_active_membership(
       p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
    OR (p_pop IS NOT NULL AND EXISTS (
       SELECT 1 FROM ipat_platform.lookup_active_membership(
       p_issuer,p_subject,p_tenant,'noc_engineer',p_pop))))
 ORDER BY d.created_at DESC,d.id LIMIT 100
$body$;
COMMIT;
