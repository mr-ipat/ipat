-- R9.58 Operator-initiated firmware lifecycle, metadata only until independently
-- qualified vendor-specific worker is deployed. Requires canonical 0001-0016.
BEGIN;
CREATE TABLE ipat_ops.firmware_artifacts (
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 id uuid NOT NULL,
 vendor text NOT NULL CHECK (vendor ~ '^[A-Za-z0-9_.-]{1,64}$'),
 exact_model text NOT NULL CHECK (length(exact_model) BETWEEN 1 AND 128),
 target_version text NOT NULL CHECK (length(target_version) BETWEEN 1 AND 128),
 size_bytes bigint NOT NULL CHECK (size_bytes BETWEEN 1024 AND 1073741824),
 sha256 text NOT NULL CHECK (sha256 ~ '^[a-f0-9]{64}$'),
 -- Private object-storage pointer only; image bytes and signed release
 -- manifest are NEVER persisted or downloaded by this SQL function.
 object_ref text NOT NULL CHECK (length(object_ref) BETWEEN 75 AND 255 AND
   object_ref LIKE 'artifact://tenant/' || tenant_id::text || '/firmware/%' AND
   object_ref ~ '^artifact://tenant/[a-f0-9-]+/firmware/[A-Za-z0-9/_-]+$'),
 vendor_release_ref text NOT NULL CHECK (length(vendor_release_ref) BETWEEN 12 AND 180
   AND vendor_release_ref ~ '^[A-Za-z0-9_.:/() -]+$'),
 artifact_state text NOT NULL DEFAULT 'METADATA_STAGED' CHECK (artifact_state='METADATA_STAGED'),
 uploaded_by_issuer text NOT NULL, uploaded_by_subject text NOT NULL,
 registered_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,id)
);
ALTER TABLE ipat_ops.firmware_artifacts OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.firmware_artifacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.firmware_artifacts FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.firmware_artifacts FROM PUBLIC, ipat_app_runtime;

CREATE TABLE ipat_ops.firmware_changes (
 tenant_id uuid NOT NULL, id uuid NOT NULL, device_id uuid NOT NULL, artifact_id uuid NOT NULL,
 request_id uuid NOT NULL, reason text NOT NULL CHECK (length(reason) BETWEEN 12 AND 180
   AND reason ~ '^[A-Za-z0-9 _.,:/()-]+$'),
 window_start timestamptz NOT NULL, window_end timestamptz NOT NULL,
 state text NOT NULL DEFAULT 'AWAITING_EVIDENCE'
   CHECK (state IN ('AWAITING_EVIDENCE','APPROVED','REJECTED','EXECUTION_REQUESTED')),
 requested_by_issuer text NOT NULL, requested_by_subject text NOT NULL,
 created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 reviewed_by_issuer text, reviewed_by_subject text, reviewed_at timestamptz,
 execution_requested_by_issuer text, execution_requested_by_subject text,
 execution_requested_at timestamptz,
 PRIMARY KEY(tenant_id,id), UNIQUE(tenant_id,request_id),
 FOREIGN KEY(tenant_id,device_id) REFERENCES ipat_ops.managed_devices(tenant_id,id),
 FOREIGN KEY(tenant_id,artifact_id) REFERENCES ipat_ops.firmware_artifacts(tenant_id,id),
 CHECK(window_end > window_start AND window_end <= window_start+interval '8 hours')
);
-- One pending upgrade intent per device (separate future worker controls
-- locking/retries and reconciles actual device state, never multiple flashes).
CREATE UNIQUE INDEX firmware_one_active_change_per_device
 ON ipat_ops.firmware_changes(tenant_id,device_id)
 WHERE state IN('AWAITING_EVIDENCE','APPROVED','EXECUTION_REQUESTED');
ALTER TABLE ipat_ops.firmware_changes OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.firmware_changes ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.firmware_changes FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.firmware_changes FROM PUBLIC, ipat_app_runtime;

CREATE TABLE ipat_ops.firmware_evidence (
 tenant_id uuid NOT NULL, change_id uuid NOT NULL,
 evidence_kind text NOT NULL CHECK(evidence_kind IN
   ('VENDOR_RELEASE','DEVICE_IDENTITY','BACKUP_RESTORE','IMPACT_BASELINE','RECOVERY_PATH','ADAPTER_QUALIFIED')),
 evidence_sha256 text NOT NULL CHECK(evidence_sha256 ~ '^[a-f0-9]{64}$'),
 -- For DEVICE_IDENTITY, recorded ONLY from a separately authenticated
 -- adapter attestation; never populated by the operator form.
 observed_vendor text, observed_model text, observed_version text,
 attested_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,change_id,evidence_kind),
 FOREIGN KEY(tenant_id,change_id) REFERENCES ipat_ops.firmware_changes(tenant_id,id),
 CHECK(evidence_kind <> 'DEVICE_IDENTITY' OR
       (observed_vendor IS NOT NULL AND observed_model IS NOT NULL AND
        observed_version IS NOT NULL AND length(observed_version) BETWEEN 1 AND 128))
);
ALTER TABLE ipat_ops.firmware_evidence OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.firmware_evidence ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.firmware_evidence FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.firmware_evidence FROM PUBLIC, ipat_app_runtime;

CREATE TABLE ipat_ops.firmware_change_events (
 id bigint GENERATED ALWAYS AS IDENTITY,
 tenant_id uuid NOT NULL, change_id uuid NOT NULL,
 event text NOT NULL CHECK(event IN('REQUESTED','EVIDENCE_ATTESTED','APPROVED','REJECTED','EXECUTION_REQUESTED')),
 actor_kind text NOT NULL CHECK(actor_kind IN('OPERATOR','CHECKER','SERVICE')),
 actor_ref text NOT NULL CHECK(length(actor_ref) BETWEEN 3 AND 256),
 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 PRIMARY KEY(id), FOREIGN KEY(tenant_id,change_id) REFERENCES ipat_ops.firmware_changes(tenant_id,id)
);
ALTER TABLE ipat_ops.firmware_change_events OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.firmware_change_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.firmware_change_events FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.firmware_change_events FROM PUBLIC, ipat_app_runtime;

CREATE ROLE ipat_fw_workflow_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_fw_api_execute NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE ROLE ipat_fw_attestor_execute NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_fw_workflow_owner, ipat_fw_api_execute, ipat_fw_attestor_execute;
GRANT USAGE ON SCHEMA ipat_ops TO ipat_fw_workflow_owner;
GRANT SELECT ON ipat_platform.tenants TO ipat_fw_workflow_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(text,text,uuid,text,text) TO ipat_fw_workflow_owner;
GRANT SELECT ON ipat_ops.managed_devices TO ipat_fw_workflow_owner;
GRANT SELECT,INSERT ON ipat_ops.firmware_artifacts TO ipat_fw_workflow_owner;
GRANT SELECT,INSERT,UPDATE(state,reviewed_by_issuer,reviewed_by_subject,reviewed_at,
 execution_requested_by_issuer,execution_requested_by_subject,execution_requested_at)
 ON ipat_ops.firmware_changes TO ipat_fw_workflow_owner;
GRANT SELECT,INSERT ON ipat_ops.firmware_evidence,ipat_ops.firmware_change_events TO ipat_fw_workflow_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_ops.firmware_change_events_id_seq TO ipat_fw_workflow_owner;
CREATE POLICY fw_owner_managed_device_read ON ipat_ops.managed_devices FOR SELECT TO ipat_fw_workflow_owner USING(true);
DO $p$
DECLARE tab text;
BEGIN
 FOREACH tab IN ARRAY ARRAY['firmware_artifacts','firmware_changes','firmware_evidence','firmware_change_events'] LOOP
  EXECUTE format('CREATE POLICY fw_owner_read ON ipat_ops.%I FOR SELECT TO ipat_fw_workflow_owner USING(true)',tab);
  EXECUTE format('CREATE POLICY fw_owner_insert ON ipat_ops.%I FOR INSERT TO ipat_fw_workflow_owner WITH CHECK(true)',tab);
 END LOOP;
END $p$;
CREATE POLICY fw_owner_change_update ON ipat_ops.firmware_changes FOR UPDATE TO ipat_fw_workflow_owner USING(true) WITH CHECK(true);

-- The registered artifact is a reference, NOT a successfully uploaded or
-- vendor-authenticated binary. Independently authenticated storage service
-- must first verify content SHA256 and vendor release outside this function.
CREATE FUNCTION ipat_platform.stage_firmware_artifact(
 p_issuer text,p_subject text,p_tenant uuid,p_id uuid,p_vendor text,p_model text,
 p_version text,p_size bigint,p_sha256 text,p_object_ref text,p_vendor_release text
) RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $fw$
DECLARE existing ipat_ops.firmware_artifacts%ROWTYPE;
BEGIN
 IF NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
    p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN NULL; END IF;
 SELECT * INTO existing FROM ipat_ops.firmware_artifacts WHERE tenant_id=p_tenant AND id=p_id;
 IF FOUND THEN
  IF existing.uploaded_by_issuer=p_issuer AND existing.uploaded_by_subject=p_subject
    AND existing.vendor=p_vendor AND existing.exact_model=p_model AND existing.target_version=p_version
    AND existing.size_bytes=p_size AND existing.sha256=p_sha256
    AND existing.object_ref=p_object_ref AND existing.vendor_release_ref=p_vendor_release
  THEN RETURN p_id; END IF;
  RETURN NULL;
 END IF;
 INSERT INTO ipat_ops.firmware_artifacts(tenant_id,id,vendor,exact_model,target_version,
  size_bytes,sha256,object_ref,vendor_release_ref,uploaded_by_issuer,uploaded_by_subject)
 VALUES(p_tenant,p_id,p_vendor,p_model,p_version,p_size,p_sha256,p_object_ref,p_vendor_release,
  p_issuer,p_subject);
 RETURN p_id;
EXCEPTION WHEN check_violation OR unique_violation OR foreign_key_violation THEN RETURN NULL;
END $fw$;
ALTER FUNCTION ipat_platform.stage_firmware_artifact(text,text,uuid,uuid,text,text,text,bigint,text,text,text) OWNER TO ipat_fw_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.stage_firmware_artifact(text,text,uuid,uuid,text,text,text,bigint,text,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.stage_firmware_artifact(text,text,uuid,uuid,text,text,text,bigint,text,text,text) TO ipat_fw_api_execute;

CREATE FUNCTION ipat_platform.propose_firmware_change(
 p_issuer text,p_subject text,p_tenant uuid,p_id uuid,p_device uuid,p_artifact uuid,
 p_request uuid,p_reason text,p_window_start timestamptz,p_window_end timestamptz
) RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $fw$
DECLARE prior ipat_ops.firmware_changes%ROWTYPE;
BEGIN
 IF p_tenant IS NULL OR p_id IS NULL OR p_device IS NULL OR p_artifact IS NULL OR p_request IS NULL
    OR p_window_start IS NULL OR p_window_end IS NULL
    OR p_window_start <= statement_timestamp() OR p_window_start > statement_timestamp()+interval '30 days'
    OR p_window_end <= p_window_start OR p_window_end > p_window_start+interval '8 hours'
    OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
      p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN NULL; END IF;
 -- A display-intended model alone NEVER authorizes firmware execution.
 IF NOT EXISTS(SELECT 1 FROM ipat_ops.managed_devices d JOIN ipat_ops.firmware_artifacts a
   ON a.tenant_id=d.tenant_id WHERE d.tenant_id=p_tenant AND d.id=p_device AND a.id=p_artifact
   AND d.vendor=a.vendor AND d.intended_model=a.exact_model) THEN RETURN NULL; END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended(p_tenant::text || p_request::text,0));
 SELECT * INTO prior FROM ipat_ops.firmware_changes WHERE tenant_id=p_tenant AND request_id=p_request;
 IF FOUND THEN
  IF prior.device_id=p_device AND prior.artifact_id=p_artifact
   AND prior.requested_by_issuer=p_issuer AND prior.requested_by_subject=p_subject
   AND prior.reason=p_reason AND prior.window_start=p_window_start AND prior.window_end=p_window_end
  THEN RETURN prior.id; END IF;
  RETURN NULL;
 END IF;
 INSERT INTO ipat_ops.firmware_changes(tenant_id,id,device_id,artifact_id,request_id,
  reason,window_start,window_end,requested_by_issuer,requested_by_subject)
 VALUES(p_tenant,p_id,p_device,p_artifact,p_request,p_reason,p_window_start,p_window_end,
  p_issuer,p_subject);
 INSERT INTO ipat_ops.firmware_change_events(tenant_id,change_id,event,actor_kind,actor_ref)
 VALUES(p_tenant,p_id,'REQUESTED','OPERATOR',p_issuer || '#' || p_subject);
 RETURN p_id;
EXCEPTION WHEN check_violation OR unique_violation OR foreign_key_violation THEN RETURN NULL;
END $fw$;
ALTER FUNCTION ipat_platform.propose_firmware_change(text,text,uuid,uuid,uuid,uuid,uuid,text,timestamptz,timestamptz) OWNER TO ipat_fw_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.propose_firmware_change(text,text,uuid,uuid,uuid,uuid,uuid,text,timestamptz,timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.propose_firmware_change(text,text,uuid,uuid,uuid,uuid,uuid,text,timestamptz,timestamptz) TO ipat_fw_api_execute;

-- Future independently provisioned service role only: browser membership,
-- admin approval or an operator-supplied checkbox can NEVER mint evidence.
CREATE FUNCTION ipat_platform.attest_firmware_evidence(
 p_tenant uuid,p_change uuid,p_kind text,p_sha256 text,
 p_observed_vendor text,p_observed_model text,p_observed_version text
) RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $fw$
DECLARE c ipat_ops.firmware_changes%ROWTYPE; a ipat_ops.firmware_artifacts%ROWTYPE;
BEGIN
 SELECT * INTO c FROM ipat_ops.firmware_changes WHERE tenant_id=p_tenant AND id=p_change FOR UPDATE;
 IF NOT FOUND OR c.state<>'AWAITING_EVIDENCE' THEN RETURN false; END IF;
 SELECT * INTO a FROM ipat_ops.firmware_artifacts WHERE tenant_id=p_tenant AND id=c.artifact_id;
 IF p_kind NOT IN('VENDOR_RELEASE','DEVICE_IDENTITY','BACKUP_RESTORE',
   'IMPACT_BASELINE','RECOVERY_PATH','ADAPTER_QUALIFIED')
   OR p_sha256 IS NULL OR p_sha256 !~ '^[a-f0-9]{64}$'
   OR (p_kind='VENDOR_RELEASE' AND p_sha256<>a.sha256)
   OR (p_kind='DEVICE_IDENTITY' AND (p_observed_vendor IS DISTINCT FROM a.vendor
       OR p_observed_model IS DISTINCT FROM a.exact_model
       OR p_observed_version IS NULL OR length(p_observed_version) NOT BETWEEN 1 AND 128))
 THEN RETURN false; END IF;
 INSERT INTO ipat_ops.firmware_evidence(tenant_id,change_id,evidence_kind,evidence_sha256,
  observed_vendor,observed_model,observed_version)
 VALUES(p_tenant,p_change,p_kind,p_sha256,p_observed_vendor,p_observed_model,p_observed_version)
 ON CONFLICT (tenant_id,change_id,evidence_kind) DO NOTHING;
 IF NOT FOUND THEN RETURN false; END IF;
 INSERT INTO ipat_ops.firmware_change_events(tenant_id,change_id,event,actor_kind,actor_ref)
 VALUES(p_tenant,p_change,'EVIDENCE_ATTESTED','SERVICE',p_kind);
 RETURN true;
EXCEPTION WHEN check_violation OR foreign_key_violation THEN RETURN false;
END $fw$;
ALTER FUNCTION ipat_platform.attest_firmware_evidence(uuid,uuid,text,text,text,text,text) OWNER TO ipat_fw_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.attest_firmware_evidence(uuid,uuid,text,text,text,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.attest_firmware_evidence(uuid,uuid,text,text,text,text,text) TO ipat_fw_attestor_execute;

CREATE FUNCTION ipat_platform.review_firmware_change(
 p_issuer text,p_subject text,p_tenant uuid,p_change uuid,p_approve boolean
) RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $fw$
DECLARE c ipat_ops.firmware_changes%ROWTYPE;
BEGIN
 IF p_approve IS NULL OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_issuer,p_subject,p_tenant,'security_admin',NULL)) THEN RETURN NULL; END IF;
 SELECT * INTO c FROM ipat_ops.firmware_changes WHERE tenant_id=p_tenant AND id=p_change FOR UPDATE;
 IF NOT FOUND OR c.state<>'AWAITING_EVIDENCE'
  OR (c.requested_by_issuer=p_issuer AND c.requested_by_subject=p_subject)
 THEN RETURN NULL; END IF;
 -- Reject always possible for current independent reviewer. Approval requires
 -- real separately attested gates, not a form with checked boxes.
 IF p_approve AND (SELECT count(*) FROM ipat_ops.firmware_evidence
     WHERE tenant_id=p_tenant AND change_id=p_change) <> 6 THEN RETURN NULL; END IF;
 UPDATE ipat_ops.firmware_changes SET state=CASE WHEN p_approve THEN 'APPROVED' ELSE 'REJECTED' END,
  reviewed_by_issuer=p_issuer,reviewed_by_subject=p_subject,reviewed_at=statement_timestamp()
 WHERE tenant_id=p_tenant AND id=p_change;
 INSERT INTO ipat_ops.firmware_change_events(tenant_id,change_id,event,actor_kind,actor_ref)
 VALUES(p_tenant,p_change,CASE WHEN p_approve THEN 'APPROVED' ELSE 'REJECTED' END,
  'CHECKER',p_issuer || '#' || p_subject);
 RETURN CASE WHEN p_approve THEN 'APPROVED' ELSE 'REJECTED' END;
END $fw$;
ALTER FUNCTION ipat_platform.review_firmware_change(text,text,uuid,uuid,boolean) OWNER TO ipat_fw_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.review_firmware_change(text,text,uuid,uuid,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.review_firmware_change(text,text,uuid,uuid,boolean) TO ipat_fw_api_execute;

-- This is the operator's EXPLICIT execute request. It DOES NOT directly
-- dispatch firmware; the future verified adapter worker must independently
-- enforce target/attestation freshness, approval, current time and rollback.
CREATE FUNCTION ipat_platform.request_firmware_execution(
 p_issuer text,p_subject text,p_tenant uuid,p_change uuid
) RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $fw$
DECLARE c ipat_ops.firmware_changes%ROWTYPE;
BEGIN
 IF NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN NULL; END IF;
 SELECT * INTO c FROM ipat_ops.firmware_changes WHERE tenant_id=p_tenant AND id=p_change FOR UPDATE;
 IF NOT FOUND OR c.state<>'APPROVED' OR c.window_end <= statement_timestamp()
  OR c.window_start > statement_timestamp()+interval '7 days'
  OR c.reviewed_by_issuer IS NULL OR c.reviewed_by_subject IS NULL
  OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
    c.reviewed_by_issuer,c.reviewed_by_subject,p_tenant,'security_admin',NULL))
  OR EXISTS(SELECT 1 FROM ipat_ops.firmware_evidence e WHERE e.tenant_id=p_tenant AND e.change_id=p_change
    AND ((e.evidence_kind='DEVICE_IDENTITY' AND e.attested_at < statement_timestamp()-interval '30 minutes')
      OR (e.evidence_kind<>'DEVICE_IDENTITY' AND e.attested_at < statement_timestamp()-interval '24 hours')))
  OR (SELECT count(*) FROM ipat_ops.firmware_evidence WHERE tenant_id=p_tenant AND change_id=p_change)<>6
 THEN RETURN NULL; END IF;
 UPDATE ipat_ops.firmware_changes SET state='EXECUTION_REQUESTED',
  execution_requested_by_issuer=p_issuer,execution_requested_by_subject=p_subject,
  execution_requested_at=statement_timestamp()
 WHERE tenant_id=p_tenant AND id=p_change;
 INSERT INTO ipat_ops.firmware_change_events(tenant_id,change_id,event,actor_kind,actor_ref)
 VALUES(p_tenant,p_change,'EXECUTION_REQUESTED','OPERATOR',p_issuer || '#' || p_subject);
 RETURN 'EXECUTION_REQUESTED';
END $fw$;
ALTER FUNCTION ipat_platform.request_firmware_execution(text,text,uuid,uuid) OWNER TO ipat_fw_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.request_firmware_execution(text,text,uuid,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.request_firmware_execution(text,text,uuid,uuid) TO ipat_fw_api_execute;

-- List excludes object pointer, raw hashes, endpoints and any device secret.
CREATE FUNCTION ipat_platform.list_firmware_changes(
 p_issuer text,p_subject text,p_tenant uuid
) RETURNS TABLE(id uuid,device_id uuid,target_version text,state text,window_start timestamptz,window_end timestamptz)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $fw$
 SELECT c.id,c.device_id,a.target_version,c.state,c.window_start,c.window_end
 FROM ipat_ops.firmware_changes c JOIN ipat_ops.firmware_artifacts a
   ON a.tenant_id=c.tenant_id AND a.id=c.artifact_id
 WHERE c.tenant_id=p_tenant AND (
  EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
  OR EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(p_issuer,p_subject,p_tenant,'security_admin',NULL)))
 ORDER BY c.created_at DESC,c.id LIMIT 100;
$fw$;
ALTER FUNCTION ipat_platform.list_firmware_changes(text,text,uuid) OWNER TO ipat_fw_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_firmware_changes(text,text,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_firmware_changes(text,text,uuid) TO ipat_fw_api_execute;
-- No LOGIN roles, object storage access, network endpoint or physical executor
-- are created by this migration; operator intent alone cannot flash equipment.
COMMIT;
