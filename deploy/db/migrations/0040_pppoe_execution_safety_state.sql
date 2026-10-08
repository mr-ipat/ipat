-- R10.10: physical PPPoE execution SAFETY STATE only.
-- This migration adds no RouterOS network adapter and no generic command path.
-- A plan may be armed only after current physical readback/recovery evidence,
-- current role separation and a <=5 minute MFA-derived browser session.
BEGIN;

-- Existing R10.07 hard-false columns become derived safety state. They can
-- only be changed by the sealed owner functions below; raw API roles retain
-- zero table rights.
ALTER TABLE ipat_ops.pppoe_batch_plans
  DROP CONSTRAINT pppoe_batch_plans_execution_allowed_check,
  DROP CONSTRAINT pppoe_batch_plans_physical_readback_verified_check,
  DROP CONSTRAINT pppoe_batch_plans_basis_check;
ALTER TABLE ipat_ops.pppoe_batch_plans
  ADD CONSTRAINT pppoe_execution_allowed_shape CHECK(
    NOT execution_allowed OR (
      state='approved' AND physical_readback_verified
      AND approval_expires_at IS NOT NULL)),
  ADD CONSTRAINT pppoe_readback_basis_shape CHECK(
    (NOT physical_readback_verified
      AND basis='subscriber360_declared_not_router_readback')
    OR
    (physical_readback_verified
      AND basis='routeros_api_ssl_exact_readback'));

CREATE TABLE ipat_ops.pppoe_router_readiness(
 tenant_id uuid NOT NULL,
 router_id uuid NOT NULL,
 evidence_sha256 text NOT NULL CHECK(evidence_sha256 ~ '^[0-9a-f]{64}$'),
 recovery_sha256 text NOT NULL CHECK(recovery_sha256 ~ '^[0-9a-f]{64}$'),
 device_metadata_revision bigint NOT NULL CHECK(device_metadata_revision>=1),
 routeros_version text NOT NULL CHECK(
   length(routeros_version) BETWEEN 1 AND 80
   AND routeros_version=btrim(routeros_version)
   AND routeros_version !~ '[[:cntrl:]]'),
 transport text NOT NULL CHECK(transport='routeros_api_ssl'),
 observed_at timestamptz NOT NULL,
 recovery_tested_at timestamptz NOT NULL,
 valid_until timestamptz NOT NULL,
 recorded_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,router_id),
 FOREIGN KEY(tenant_id,router_id)
  REFERENCES ipat_ops.managed_devices(tenant_id,id),
 CHECK(observed_at<=recorded_at+interval '5 seconds'),
 CHECK(recovery_tested_at<=recorded_at+interval '5 seconds'),
 CHECK(valid_until>recorded_at AND valid_until<=recorded_at+interval '30 minutes')
);
ALTER TABLE ipat_ops.pppoe_router_readiness OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.pppoe_router_readiness ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_router_readiness FORCE ROW LEVEL SECURITY;

CREATE TABLE ipat_ops.pppoe_execution_attempts(
 tenant_id uuid NOT NULL,
 id uuid NOT NULL,
 plan_id uuid NOT NULL,
 router_id uuid NOT NULL,
 state text NOT NULL DEFAULT 'armed'
   CHECK(state IN('armed','claimed','unknown_reconcile_required',
                  'readback_verified','rolled_back','failed_safe')),
 armed_by_issuer text NOT NULL,
 armed_by_subject text NOT NULL,
 armed_session_id uuid NOT NULL,
 armed_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 readiness_evidence_sha256 text NOT NULL CHECK(readiness_evidence_sha256 ~ '^[0-9a-f]{64}$'),
 recovery_sha256 text NOT NULL CHECK(recovery_sha256 ~ '^[0-9a-f]{64}$'),
 PRIMARY KEY(tenant_id,id),
 UNIQUE(tenant_id,plan_id),
 FOREIGN KEY(tenant_id,plan_id)
   REFERENCES ipat_ops.pppoe_batch_plans(tenant_id,id),
 FOREIGN KEY(tenant_id,router_id)
   REFERENCES ipat_ops.managed_devices(tenant_id,id),
 CHECK(updated_at>=armed_at)
);
ALTER TABLE ipat_ops.pppoe_execution_attempts OWNER TO ipat_schema_owner;
CREATE UNIQUE INDEX pppoe_one_active_attempt_per_router
 ON ipat_ops.pppoe_execution_attempts(tenant_id,router_id)
 WHERE state IN('armed','claimed','unknown_reconcile_required');
ALTER TABLE ipat_ops.pppoe_execution_attempts ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_execution_attempts FORCE ROW LEVEL SECURITY;

CREATE TABLE ipat_ops.pppoe_execution_items(
 tenant_id uuid NOT NULL,
 attempt_id uuid NOT NULL,
 plan_id uuid NOT NULL,
 ordinal integer NOT NULL CHECK(ordinal BETWEEN 1 AND 128),
 state text NOT NULL DEFAULT 'pending'
   CHECK(state IN('pending','claimed','unknown_reconcile_required',
                  'readback_verified','rolled_back','failed_safe')),
 worker_id text,
 claimed_at timestamptz,
 claim_expires_at timestamptz,
 outcome_sha256 text CHECK(outcome_sha256 IS NULL OR outcome_sha256 ~ '^[0-9a-f]{64}$'),
 updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,attempt_id,ordinal),
 FOREIGN KEY(tenant_id,attempt_id)
   REFERENCES ipat_ops.pppoe_execution_attempts(tenant_id,id),
 FOREIGN KEY(tenant_id,plan_id,ordinal)
   REFERENCES ipat_ops.pppoe_batch_items(tenant_id,plan_id,ordinal),
 CHECK(
   (state='pending' AND worker_id IS NULL AND claimed_at IS NULL
      AND claim_expires_at IS NULL AND outcome_sha256 IS NULL)
   OR
   (state='claimed' AND worker_id IS NOT NULL AND claimed_at IS NOT NULL
      AND claim_expires_at IS NOT NULL AND claim_expires_at>claimed_at
      AND outcome_sha256 IS NULL)
   OR
   (state='unknown_reconcile_required'
      AND worker_id IS NOT NULL AND claimed_at IS NOT NULL)
   OR
   (state IN('readback_verified','rolled_back','failed_safe')
      AND worker_id IS NOT NULL AND claimed_at IS NOT NULL
      AND outcome_sha256 IS NOT NULL)
 )
);
ALTER TABLE ipat_ops.pppoe_execution_items OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.pppoe_execution_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_execution_items FORCE ROW LEVEL SECURITY;

CREATE TABLE ipat_ops.pppoe_execution_audit(
 tenant_id uuid NOT NULL,
 attempt_id uuid NOT NULL,
 sequence bigint GENERATED BY DEFAULT AS IDENTITY,
 event text NOT NULL CHECK(event IN(
  'ARMED','ITEM_CLAIMED','ITEM_UNKNOWN','ITEM_READBACK_VERIFIED',
  'ITEM_ROLLED_BACK','ITEM_FAILED_SAFE','ATTEMPT_COMPLETED',
  'CLAIM_EXPIRED_TO_UNKNOWN','RECONCILED')),
 actor text NOT NULL CHECK(length(actor) BETWEEN 1 AND 160),
 evidence_sha256 text CHECK(evidence_sha256 IS NULL OR evidence_sha256 ~ '^[0-9a-f]{64}$'),
 occurred_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,attempt_id,sequence),
 FOREIGN KEY(tenant_id,attempt_id)
   REFERENCES ipat_ops.pppoe_execution_attempts(tenant_id,id)
);
ALTER TABLE ipat_ops.pppoe_execution_audit OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.pppoe_execution_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_execution_audit FORCE ROW LEVEL SECURITY;

REVOKE ALL ON ipat_ops.pppoe_router_readiness,
 ipat_ops.pppoe_execution_attempts,ipat_ops.pppoe_execution_items,
 ipat_ops.pppoe_execution_audit
 FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_login,
 ipat_oidc_session_issuer_login;

CREATE ROLE ipat_pppoe_exec_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
 NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_pppoe_readback_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
 NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_pppoe_worker_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
 NOREPLICATION NOBYPASSRLS NOINHERIT;

GRANT USAGE ON SCHEMA ipat_platform,ipat_ops TO
 ipat_pppoe_exec_owner,ipat_pppoe_readback_exec,ipat_pppoe_worker_exec;
GRANT EXECUTE ON FUNCTION ipat_platform.pppoe_batch_create_capability(
 text,text,uuid) TO ipat_pppoe_exec_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.pppoe_batch_review_capability(
 text,text,uuid) TO ipat_pppoe_exec_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
 text,text,uuid,text,text) TO ipat_pppoe_exec_owner;

CREATE FUNCTION ipat_platform.pppoe_execution_arm_capability(
 p_issuer text,p_subject text,p_tenant uuid)
RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $$
 SELECT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'system_admin',NULL))
$$;
ALTER FUNCTION ipat_platform.pppoe_execution_arm_capability(text,text,uuid)
 OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.pppoe_execution_arm_capability(
 text,text,uuid) FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.pppoe_execution_arm_capability(
 text,text,uuid) TO ipat_tenant_api_exec,ipat_pppoe_exec_owner,ipat_pppoe_plan_owner;

-- System Admin must be able to inspect already-reviewed plans without gaining
-- maker/reviewer rights. Keep the existing result shape; can_review remains
-- Security-Admin-specific.
CREATE OR REPLACE FUNCTION ipat_platform.list_pppoe_batch_dry_runs(
 p_issuer text,p_subject text,p_tenant uuid)
RETURNS TABLE(
 id uuid,router_id uuid,site_code text,pop_code text,idempotency_key text,plan_digest text,
 item_count integer,state text,requested_by text,reviewed_by text,
 requested_at timestamptz,reviewed_at timestamptz,approval_expires_at timestamptz,
 rate_limit_per_minute integer,execution_allowed boolean,
 physical_readback_verified boolean,basis text,can_review boolean)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $$
 SELECT p.id,p.router_id,p.site_code,p.pop_code,p.idempotency_key,p.plan_digest,p.item_count,
   p.state,p.requested_by_subject,p.reviewed_by_subject,p.requested_at,p.reviewed_at,
   p.approval_expires_at,p.rate_limit_per_minute,p.execution_allowed,
   p.physical_readback_verified,p.basis,
   p.state='awaiting_approval'
    AND ipat_platform.pppoe_batch_review_capability(p_issuer,p_subject,p_tenant)
    AND (p.requested_by_issuer,p.requested_by_subject)<>(p_issuer,p_subject)
 FROM ipat_ops.pppoe_batch_plans p
 WHERE p.tenant_id=p_tenant AND (
   ipat_platform.pppoe_batch_create_capability(p_issuer,p_subject,p_tenant)
   OR ipat_platform.pppoe_batch_review_capability(p_issuer,p_subject,p_tenant)
   OR ipat_platform.pppoe_execution_arm_capability(p_issuer,p_subject,p_tenant))
 ORDER BY p.requested_at DESC,p.id DESC LIMIT 100
$$;
ALTER FUNCTION ipat_platform.list_pppoe_batch_dry_runs(text,text,uuid)
 OWNER TO ipat_pppoe_plan_owner;

CREATE OR REPLACE FUNCTION ipat_platform.list_pppoe_batch_items(
 p_issuer text,p_subject text,p_tenant uuid,p_plan uuid)
RETURNS TABLE(
 ordinal integer,subscriber_id text,action text,before_username text,
 desired_username text,profile_name text,has_secret_ref boolean)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $$
 SELECT i.ordinal,i.subscriber_id,i.action,i.before_username,i.desired_username,
   i.profile_name,i.secret_ref IS NOT NULL
 FROM ipat_ops.pppoe_batch_items i
 WHERE i.tenant_id=p_tenant AND i.plan_id=p_plan AND (
   ipat_platform.pppoe_batch_create_capability(p_issuer,p_subject,p_tenant)
   OR ipat_platform.pppoe_batch_review_capability(p_issuer,p_subject,p_tenant)
   OR ipat_platform.pppoe_execution_arm_capability(p_issuer,p_subject,p_tenant))
 ORDER BY i.ordinal
$$;
ALTER FUNCTION ipat_platform.list_pppoe_batch_items(text,text,uuid,uuid)
 OWNER TO ipat_pppoe_plan_owner;

GRANT SELECT ON ipat_platform.identity_memberships,ipat_platform.tenants,
 ipat_platform.tenant_domains,ipat_platform.tenant_browser_sessions
 TO ipat_pppoe_exec_owner;
GRANT SELECT ON ipat_ops.managed_devices,ipat_ops.pppoe_batch_plans,
 ipat_ops.pppoe_batch_items,ipat_ops.pppoe_router_readiness,
 ipat_ops.pppoe_execution_attempts,ipat_ops.pppoe_execution_items
 TO ipat_pppoe_exec_owner;
GRANT INSERT,UPDATE ON ipat_ops.pppoe_router_readiness,
 ipat_ops.pppoe_execution_attempts,ipat_ops.pppoe_execution_items
 TO ipat_pppoe_exec_owner;
GRANT INSERT ON ipat_ops.pppoe_execution_audit TO ipat_pppoe_exec_owner;
GRANT UPDATE(execution_allowed,physical_readback_verified,basis)
 ON ipat_ops.pppoe_batch_plans TO ipat_pppoe_exec_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_ops.pppoe_execution_audit_sequence_seq
 TO ipat_pppoe_exec_owner;

CREATE POLICY pppoe_exec_identity_read ON ipat_platform.identity_memberships
 FOR SELECT TO ipat_pppoe_exec_owner USING(true);
CREATE POLICY pppoe_exec_domain_read ON ipat_platform.tenant_domains
 FOR SELECT TO ipat_pppoe_exec_owner USING(true);
CREATE POLICY pppoe_exec_session_read ON ipat_platform.tenant_browser_sessions
 FOR SELECT TO ipat_pppoe_exec_owner USING(true);
CREATE POLICY pppoe_exec_device_read ON ipat_ops.managed_devices
 FOR SELECT TO ipat_pppoe_exec_owner USING(true);
CREATE POLICY pppoe_exec_plan_read ON ipat_ops.pppoe_batch_plans
 FOR SELECT TO ipat_pppoe_exec_owner USING(true);
CREATE POLICY pppoe_exec_plan_update ON ipat_ops.pppoe_batch_plans
 FOR UPDATE TO ipat_pppoe_exec_owner USING(true) WITH CHECK(true);
CREATE POLICY pppoe_exec_item_read ON ipat_ops.pppoe_batch_items
 FOR SELECT TO ipat_pppoe_exec_owner USING(true);
CREATE POLICY pppoe_exec_ready_all ON ipat_ops.pppoe_router_readiness
 FOR ALL TO ipat_pppoe_exec_owner USING(true) WITH CHECK(true);
CREATE POLICY pppoe_exec_attempt_all ON ipat_ops.pppoe_execution_attempts
 FOR ALL TO ipat_pppoe_exec_owner USING(true) WITH CHECK(true);
CREATE POLICY pppoe_exec_item_all ON ipat_ops.pppoe_execution_items
 FOR ALL TO ipat_pppoe_exec_owner USING(true) WITH CHECK(true);
CREATE POLICY pppoe_exec_audit_all ON ipat_ops.pppoe_execution_audit
 FOR ALL TO ipat_pppoe_exec_owner USING(true) WITH CHECK(true);

-- Re-authenticate the exact current Host-bound mutation session and then
-- require it to have been issued no more than five minutes ago. The OIDC
-- issuer is allowed to call issue_tenant_browser_session only after verified
-- signed ID/access pair + nonce + fresh MFA, so this preserves that freshness
-- boundary without trusting a browser-supplied timestamp.
CREATE FUNCTION ipat_platform.authenticate_tenant_browser_session_high_risk(
 p_cookie_sha text,p_hostname text,p_csrf_sha text)
RETURNS TABLE(session_id uuid,tenant_id uuid,domain_id uuid,issuer text,subject text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE a record; sid uuid;
BEGIN
 SELECT * INTO a FROM ipat_platform.authenticate_tenant_browser_session(
   p_cookie_sha,p_hostname,p_csrf_sha,true);
 IF NOT FOUND THEN RETURN; END IF;
 SELECT s.id INTO sid
 FROM ipat_platform.tenant_browser_sessions s
 JOIN ipat_platform.tenant_domains d
   ON d.id=s.domain_id AND d.tenant_id=s.tenant_id
 WHERE s.cookie_sha256=p_cookie_sha AND d.hostname=p_hostname
   AND s.tenant_id=a.tenant_id AND s.domain_id=a.domain_id
   AND s.issuer=a.issuer AND s.subject=a.subject
   AND s.revoked_at IS NULL AND s.expires_at>clock_timestamp()
   AND s.issued_at>=clock_timestamp()-interval '5 minutes';
 IF sid IS NULL THEN RETURN; END IF;
 session_id:=sid;tenant_id:=a.tenant_id;domain_id:=a.domain_id;
 issuer:=a.issuer;subject:=a.subject;RETURN NEXT;
END $body$;
ALTER FUNCTION ipat_platform.authenticate_tenant_browser_session_high_risk(
 text,text,text) OWNER TO ipat_browser_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.authenticate_tenant_browser_session_high_risk(
 text,text,text) FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_login,
 ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.authenticate_tenant_browser_session_high_risk(
 text,text,text) TO ipat_browser_session_auth_exec;
GRANT EXECUTE ON FUNCTION ipat_platform.authenticate_tenant_browser_session_high_risk(
 text,text,text) TO ipat_pppoe_exec_owner;

-- Only a future restricted READBACK worker can insert readiness. Human/API
-- roles cannot forge physical readiness.
CREATE FUNCTION ipat_platform.record_pppoe_router_readiness(
 p_tenant uuid,p_router uuid,p_evidence text,p_recovery text,
 p_routeros text,p_observed timestamptz,p_recovery_tested timestamptz,
 p_valid_until timestamptz)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE changed integer;
DECLARE device_revision bigint;
BEGIN
 IF p_tenant IS NULL OR p_router IS NULL
   OR p_evidence !~ '^[0-9a-f]{64}$' OR p_recovery !~ '^[0-9a-f]{64}$'
   OR p_routeros IS NULL OR length(p_routeros) NOT BETWEEN 1 AND 80
   OR p_observed IS NULL OR p_recovery_tested IS NULL OR p_valid_until IS NULL
   OR p_observed>clock_timestamp()+interval '5 seconds'
   OR p_observed<clock_timestamp()-interval '10 minutes'
   OR p_recovery_tested>clock_timestamp()+interval '5 seconds'
   OR p_recovery_tested<clock_timestamp()-interval '24 hours'
   OR p_valid_until<=clock_timestamp()
   OR p_valid_until>clock_timestamp()+interval '30 minutes'
 THEN RETURN false; END IF;
 SELECT d.metadata_revision INTO device_revision
 FROM ipat_ops.managed_devices d
 WHERE d.tenant_id=p_tenant AND d.id=p_router
   AND d.device_kind='router' AND d.vendor='MikroTik'
   AND d.management_transport='routeros_api_ssl'
   AND d.lifecycle_state='SAVED' AND d.archived_at IS NULL;
 IF device_revision IS NULL THEN RETURN false; END IF;
 INSERT INTO ipat_ops.pppoe_router_readiness(
  tenant_id,router_id,evidence_sha256,recovery_sha256,device_metadata_revision,
  routeros_version,transport,observed_at,recovery_tested_at,valid_until)
 VALUES(p_tenant,p_router,p_evidence,p_recovery,device_revision,p_routeros,
  'routeros_api_ssl',p_observed,p_recovery_tested,p_valid_until)
 ON CONFLICT(tenant_id,router_id) DO UPDATE SET
  evidence_sha256=EXCLUDED.evidence_sha256,
  recovery_sha256=EXCLUDED.recovery_sha256,
  device_metadata_revision=EXCLUDED.device_metadata_revision,
  routeros_version=EXCLUDED.routeros_version,
  observed_at=EXCLUDED.observed_at,
  recovery_tested_at=EXCLUDED.recovery_tested_at,
  valid_until=EXCLUDED.valid_until,
  recorded_at=statement_timestamp();
 GET DIAGNOSTICS changed=ROW_COUNT;RETURN changed=1;
END $body$;
ALTER FUNCTION ipat_platform.record_pppoe_router_readiness(
 uuid,uuid,text,text,text,timestamptz,timestamptz,timestamptz)
 OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_pppoe_router_readiness(
 uuid,uuid,text,text,text,timestamptz,timestamptz,timestamptz)
 FROM PUBLIC,ipat_tenant_api_exec,ipat_oidc_session_issuer_login,
 ipat_app_runtime;
GRANT EXECUTE ON FUNCTION ipat_platform.record_pppoe_router_readiness(
 uuid,uuid,text,text,text,timestamptz,timestamptz,timestamptz)
 TO ipat_pppoe_readback_exec;

CREATE FUNCTION ipat_platform.arm_pppoe_batch_execution(
 p_cookie_sha text,p_hostname text,p_csrf_sha text,p_plan uuid,p_attempt uuid)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE a record;
DECLARE p ipat_ops.pppoe_batch_plans%ROWTYPE;
DECLARE r ipat_ops.pppoe_router_readiness%ROWTYPE;
DECLARE now_at timestamptz:=clock_timestamp();
BEGIN
 IF p_cookie_sha IS NULL OR p_hostname IS NULL OR p_csrf_sha IS NULL
   OR p_plan IS NULL OR p_attempt IS NULL
 THEN RETURN NULL; END IF;

 -- Re-authenticate inside the SAME SECURITY DEFINER transaction. The API
 -- process cannot arm by supplying issuer/subject/tenant/session UUID.
 SELECT * INTO a
 FROM ipat_platform.authenticate_tenant_browser_session_high_risk(
   p_cookie_sha,p_hostname,p_csrf_sha);
 IF NOT FOUND OR NOT ipat_platform.pppoe_execution_arm_capability(
      a.issuer,a.subject,a.tenant_id)
 THEN RETURN NULL; END IF;

 SELECT * INTO p FROM ipat_ops.pppoe_batch_plans
 WHERE tenant_id=a.tenant_id AND id=p_plan FOR UPDATE;
 IF NOT FOUND OR p.state<>'approved' OR p.approval_expires_at<=now_at
   OR p.reviewed_at<now_at-interval '10 minutes'
   OR (p.requested_by_issuer,p.requested_by_subject)=(a.issuer,a.subject)
   OR (p.reviewed_by_issuer,p.reviewed_by_subject)=(a.issuer,a.subject)
   OR NOT ipat_platform.pppoe_batch_create_capability(
       p.requested_by_issuer,p.requested_by_subject,a.tenant_id)
 THEN RETURN NULL; END IF;

 SELECT * INTO r FROM ipat_ops.pppoe_router_readiness
 WHERE tenant_id=a.tenant_id AND router_id=p.router_id
   AND observed_at>=p.requested_at
   AND valid_until>now_at
 FOR UPDATE;
 IF NOT FOUND OR NOT EXISTS(
   SELECT 1 FROM ipat_ops.managed_devices d
   WHERE d.tenant_id=a.tenant_id AND d.id=p.router_id
     AND d.metadata_revision=r.device_metadata_revision
     AND d.device_kind='router' AND d.vendor='MikroTik'
     AND d.management_transport='routeros_api_ssl'
     AND d.lifecycle_state='SAVED' AND d.archived_at IS NULL)
 THEN RETURN NULL; END IF;

 PERFORM pg_advisory_xact_lock(hashtextextended(
   a.tenant_id::text||'/'||p.router_id::text||'/pppoe-exec',0));
 IF EXISTS(
   SELECT 1 FROM ipat_ops.pppoe_execution_attempts x
   WHERE x.tenant_id=a.tenant_id AND x.router_id=p.router_id
     AND x.state IN('armed','claimed','unknown_reconcile_required'))
 THEN RETURN NULL; END IF;

 INSERT INTO ipat_ops.pppoe_execution_attempts(
   tenant_id,id,plan_id,router_id,armed_by_issuer,armed_by_subject,
   armed_session_id,readiness_evidence_sha256,recovery_sha256)
 VALUES(a.tenant_id,p_attempt,p_plan,p.router_id,a.issuer,a.subject,
   a.session_id,r.evidence_sha256,r.recovery_sha256);
 INSERT INTO ipat_ops.pppoe_execution_items(
   tenant_id,attempt_id,plan_id,ordinal)
 SELECT a.tenant_id,p_attempt,p_plan,i.ordinal
 FROM ipat_ops.pppoe_batch_items i
 WHERE i.tenant_id=a.tenant_id AND i.plan_id=p_plan
 ORDER BY i.ordinal;
 IF NOT FOUND THEN RAISE EXCEPTION 'empty plan'; END IF;
 UPDATE ipat_ops.pppoe_batch_plans
 SET execution_allowed=true,physical_readback_verified=true,
     basis='routeros_api_ssl_exact_readback'
 WHERE tenant_id=a.tenant_id AND id=p_plan;
 INSERT INTO ipat_ops.pppoe_execution_audit(
   tenant_id,attempt_id,event,actor,evidence_sha256)
 VALUES(a.tenant_id,p_attempt,'ARMED',a.subject,r.evidence_sha256);
 RETURN p_attempt;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation
 THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.arm_pppoe_batch_execution(
 text,text,text,uuid,uuid) OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.arm_pppoe_batch_execution(
 text,text,text,uuid,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.arm_pppoe_batch_execution(
 text,text,text,uuid,uuid) TO ipat_tenant_api_exec;

-- Worker claim returns exactly one next item and enforces per-router
-- serialization plus a non-burst interval from the plan rate limit.
CREATE FUNCTION ipat_platform.claim_next_pppoe_execution_item(
 p_tenant uuid,p_attempt uuid,p_worker text)
RETURNS TABLE(
 ordinal integer,subscriber_id text,action text,before_username text,
 desired_username text,profile_name text,secret_ref text,claim_expires_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE x ipat_ops.pppoe_execution_attempts%ROWTYPE;
DECLARE p ipat_ops.pppoe_batch_plans%ROWTYPE;
DECLARE chosen integer;DECLARE now_at timestamptz:=clock_timestamp();
DECLARE last_claim timestamptz;
BEGIN
 IF p_worker IS NULL OR length(p_worker) NOT BETWEEN 1 AND 128
   OR p_worker !~ '^[A-Za-z0-9_.:-]+$'
 THEN RETURN; END IF;
 SELECT * INTO x FROM ipat_ops.pppoe_execution_attempts
  WHERE tenant_id=p_tenant AND id=p_attempt FOR UPDATE;
 IF NOT FOUND OR x.state NOT IN('armed','claimed') THEN RETURN; END IF;
 SELECT * INTO p FROM ipat_ops.pppoe_batch_plans
  WHERE tenant_id=p_tenant AND id=x.plan_id FOR UPDATE;
 IF NOT FOUND OR NOT p.execution_allowed OR NOT p.physical_readback_verified
   OR p.state<>'approved' OR p.approval_expires_at<=now_at
   OR NOT EXISTS(
      SELECT 1
      FROM ipat_ops.pppoe_router_readiness r
      JOIN ipat_ops.managed_devices d
        ON d.tenant_id=r.tenant_id AND d.id=r.router_id
      WHERE r.tenant_id=p_tenant AND r.router_id=x.router_id
        AND r.valid_until>now_at
        AND d.metadata_revision=r.device_metadata_revision
        AND d.device_kind='router' AND d.vendor='MikroTik'
        AND d.management_transport='routeros_api_ssl'
        AND d.lifecycle_state='SAVED' AND d.archived_at IS NULL)
 THEN RETURN; END IF;

 PERFORM pg_advisory_xact_lock(hashtextextended(
   p_tenant::text||'/'||x.router_id::text||'/pppoe-exec',0));
 IF EXISTS(
   SELECT 1 FROM ipat_ops.pppoe_execution_items i
   WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
     AND i.state IN('claimed','unknown_reconcile_required'))
 THEN RETURN; END IF;
 SELECT max(i.claimed_at) INTO last_claim
 FROM ipat_ops.pppoe_execution_items i
 WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
   AND i.claimed_at IS NOT NULL;
 IF last_claim IS NOT NULL
   AND now_at < last_claim + make_interval(secs => 60.0/p.rate_limit_per_minute)
 THEN RETURN; END IF;

 SELECT i.ordinal INTO chosen FROM ipat_ops.pppoe_execution_items i
  WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt AND i.state='pending'
  ORDER BY i.ordinal LIMIT 1 FOR UPDATE;
 IF chosen IS NULL THEN RETURN; END IF;

 UPDATE ipat_ops.pppoe_execution_items i
 SET state='claimed',worker_id=p_worker,claimed_at=now_at,
     claim_expires_at=now_at+interval '45 seconds',updated_at=now_at
 WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt AND i.ordinal=chosen;
 UPDATE ipat_ops.pppoe_execution_attempts
 SET state='claimed',updated_at=now_at
 WHERE tenant_id=p_tenant AND id=p_attempt;
 INSERT INTO ipat_ops.pppoe_execution_audit(tenant_id,attempt_id,event,actor)
 VALUES(p_tenant,p_attempt,'ITEM_CLAIMED',p_worker);

 RETURN QUERY
 SELECT i.ordinal,b.subscriber_id,b.action,b.before_username,b.desired_username,
        b.profile_name,b.secret_ref,i.claim_expires_at
 FROM ipat_ops.pppoe_execution_items i
 JOIN ipat_ops.pppoe_batch_items b
  ON b.tenant_id=i.tenant_id AND b.plan_id=i.plan_id AND b.ordinal=i.ordinal
 WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt AND i.ordinal=chosen;
END $body$;
ALTER FUNCTION ipat_platform.claim_next_pppoe_execution_item(uuid,uuid,text)
 OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.claim_next_pppoe_execution_item(uuid,uuid,text)
 FROM PUBLIC,ipat_tenant_api_exec,ipat_oidc_session_issuer_login,ipat_app_runtime;
GRANT EXECUTE ON FUNCTION ipat_platform.claim_next_pppoe_execution_item(
 uuid,uuid,text) TO ipat_pppoe_worker_exec;

CREATE FUNCTION ipat_platform.record_pppoe_execution_item_result(
 p_tenant uuid,p_attempt uuid,p_ordinal integer,p_worker text,
 p_result text,p_evidence text)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_ops AS $body$
DECLARE now_at timestamptz:=clock_timestamp();DECLARE changed integer;
DECLARE event_name text;
BEGIN
 IF p_result NOT IN('readback_verified','unknown_reconcile_required',
                    'rolled_back','failed_safe')
   OR p_evidence !~ '^[0-9a-f]{64}$'
 THEN RETURN false; END IF;
 event_name=CASE p_result
  WHEN 'readback_verified' THEN 'ITEM_READBACK_VERIFIED'
  WHEN 'unknown_reconcile_required' THEN 'ITEM_UNKNOWN'
  WHEN 'rolled_back' THEN 'ITEM_ROLLED_BACK'
  ELSE 'ITEM_FAILED_SAFE' END;
 UPDATE ipat_ops.pppoe_execution_items i
 SET state=p_result,outcome_sha256=p_evidence,updated_at=now_at
 WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
   AND i.ordinal=p_ordinal AND i.state='claimed'
   AND i.worker_id=p_worker AND i.claim_expires_at>=now_at;
 GET DIAGNOSTICS changed=ROW_COUNT;
 IF changed<>1 THEN RETURN false; END IF;
 INSERT INTO ipat_ops.pppoe_execution_audit(
  tenant_id,attempt_id,event,actor,evidence_sha256)
 VALUES(p_tenant,p_attempt,event_name,p_worker,p_evidence);

 IF p_result='unknown_reconcile_required' THEN
   UPDATE ipat_ops.pppoe_execution_attempts
    SET state='unknown_reconcile_required',updated_at=now_at
    WHERE tenant_id=p_tenant AND id=p_attempt;
 ELSIF p_result IN('rolled_back','failed_safe') THEN
   UPDATE ipat_ops.pppoe_execution_attempts
    SET state=p_result,updated_at=now_at
    WHERE tenant_id=p_tenant AND id=p_attempt;
   UPDATE ipat_ops.pppoe_batch_plans p SET execution_allowed=false
    FROM ipat_ops.pppoe_execution_attempts x
    WHERE x.tenant_id=p_tenant AND x.id=p_attempt
      AND p.tenant_id=x.tenant_id AND p.id=x.plan_id;
 ELSIF NOT EXISTS(
   SELECT 1 FROM ipat_ops.pppoe_execution_items i
   WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
     AND i.state<>'readback_verified') THEN
   UPDATE ipat_ops.pppoe_execution_attempts
    SET state='readback_verified',updated_at=now_at
    WHERE tenant_id=p_tenant AND id=p_attempt;
   UPDATE ipat_ops.pppoe_batch_plans p SET execution_allowed=false
    FROM ipat_ops.pppoe_execution_attempts x
    WHERE x.tenant_id=p_tenant AND x.id=p_attempt
      AND p.tenant_id=x.tenant_id AND p.id=x.plan_id;
   INSERT INTO ipat_ops.pppoe_execution_audit(
    tenant_id,attempt_id,event,actor,evidence_sha256)
   VALUES(p_tenant,p_attempt,'ATTEMPT_COMPLETED',p_worker,p_evidence);
 ELSE
   UPDATE ipat_ops.pppoe_execution_attempts
    SET state='armed',updated_at=now_at
    WHERE tenant_id=p_tenant AND id=p_attempt;
 END IF;
 RETURN true;
END $body$;
ALTER FUNCTION ipat_platform.record_pppoe_execution_item_result(
 uuid,uuid,integer,text,text,text) OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_pppoe_execution_item_result(
 uuid,uuid,integer,text,text,text)
 FROM PUBLIC,ipat_tenant_api_exec,ipat_oidc_session_issuer_login,ipat_app_runtime;
GRANT EXECUTE ON FUNCTION ipat_platform.record_pppoe_execution_item_result(
 uuid,uuid,integer,text,text,text) TO ipat_pppoe_worker_exec;

-- If a worker disappears after claim, assume ambiguity instead of retrying.
CREATE FUNCTION ipat_platform.expire_pppoe_execution_claims_to_unknown(
 p_tenant uuid,p_attempt uuid,p_worker text)
RETURNS integer LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_ops AS $body$
DECLARE changed integer;
BEGIN
 WITH expired AS(
  UPDATE ipat_ops.pppoe_execution_items i
   SET state='unknown_reconcile_required',outcome_sha256=NULL,
       updated_at=clock_timestamp()
  WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
    AND i.state='claimed' AND i.claim_expires_at<clock_timestamp()
  RETURNING i.ordinal)
 SELECT count(*) INTO changed FROM expired;
 IF changed>0 THEN
  UPDATE ipat_ops.pppoe_execution_attempts SET
   state='unknown_reconcile_required',updated_at=clock_timestamp()
   WHERE tenant_id=p_tenant AND id=p_attempt;
  INSERT INTO ipat_ops.pppoe_execution_audit(
   tenant_id,attempt_id,event,actor,evidence_sha256)
  VALUES(p_tenant,p_attempt,'CLAIM_EXPIRED_TO_UNKNOWN',p_worker,NULL);
 END IF;
 RETURN changed;
END $body$;
ALTER FUNCTION ipat_platform.expire_pppoe_execution_claims_to_unknown(
 uuid,uuid,text) OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.expire_pppoe_execution_claims_to_unknown(
 uuid,uuid,text)
 FROM PUBLIC,ipat_tenant_api_exec,ipat_oidc_session_issuer_login,ipat_app_runtime;
GRANT EXECUTE ON FUNCTION ipat_platform.expire_pppoe_execution_claims_to_unknown(
 uuid,uuid,text) TO ipat_pppoe_worker_exec;

CREATE FUNCTION ipat_platform.reconcile_pppoe_execution_item(
 p_tenant uuid,p_attempt uuid,p_ordinal integer,p_worker text,
 p_resolution text,p_evidence text)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_ops AS $body$
DECLARE changed integer;DECLARE now_at timestamptz:=clock_timestamp();
BEGIN
 IF p_resolution NOT IN('readback_verified','rolled_back','failed_safe')
   OR p_evidence !~ '^[0-9a-f]{64}$' THEN RETURN false; END IF;
 UPDATE ipat_ops.pppoe_execution_items i
 SET state=p_resolution,worker_id=p_worker,outcome_sha256=p_evidence,
     updated_at=now_at
 WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
   AND i.ordinal=p_ordinal AND i.state='unknown_reconcile_required';
 GET DIAGNOSTICS changed=ROW_COUNT;
 IF changed<>1 THEN RETURN false; END IF;
 INSERT INTO ipat_ops.pppoe_execution_audit(
  tenant_id,attempt_id,event,actor,evidence_sha256)
 VALUES(p_tenant,p_attempt,'RECONCILED',p_worker,p_evidence);
 IF EXISTS(
   SELECT 1 FROM ipat_ops.pppoe_execution_items i
   WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
     AND i.state='unknown_reconcile_required') THEN
  RETURN true;
 END IF;
 IF EXISTS(
   SELECT 1 FROM ipat_ops.pppoe_execution_items i
   WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
     AND i.state IN('rolled_back','failed_safe')) THEN
  UPDATE ipat_ops.pppoe_batch_plans p SET execution_allowed=false
   FROM ipat_ops.pppoe_execution_attempts x
   WHERE x.tenant_id=p_tenant AND x.id=p_attempt
     AND p.tenant_id=x.tenant_id AND p.id=x.plan_id;
  UPDATE ipat_ops.pppoe_execution_attempts SET
   state=CASE WHEN EXISTS(
     SELECT 1 FROM ipat_ops.pppoe_execution_items i
     WHERE i.tenant_id=p_tenant AND i.attempt_id=p_attempt
       AND i.state='rolled_back') THEN 'rolled_back' ELSE 'failed_safe' END,
   updated_at=now_at
   WHERE tenant_id=p_tenant AND id=p_attempt;
 ELSE
  UPDATE ipat_ops.pppoe_execution_attempts SET state='armed',updated_at=now_at
   WHERE tenant_id=p_tenant AND id=p_attempt;
 END IF;
 RETURN true;
END $body$;
ALTER FUNCTION ipat_platform.reconcile_pppoe_execution_item(
 uuid,uuid,integer,text,text,text) OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.reconcile_pppoe_execution_item(
 uuid,uuid,integer,text,text,text)
 FROM PUBLIC,ipat_tenant_api_exec,ipat_oidc_session_issuer_login,ipat_app_runtime;
GRANT EXECUTE ON FUNCTION ipat_platform.reconcile_pppoe_execution_item(
 uuid,uuid,integer,text,text,text) TO ipat_pppoe_worker_exec;

CREATE FUNCTION ipat_platform.get_pppoe_execution_status(
 p_issuer text,p_subject text,p_tenant uuid,p_plan uuid)
RETURNS TABLE(
 attempt_id uuid,state text,armed_at timestamptz,updated_at timestamptz,
 item_count integer,pending_count integer,unknown_count integer,
 verified_count integer,physical_execution_adapter_enabled boolean)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT x.id,x.state,x.armed_at,x.updated_at,count(i.ordinal)::integer,
  count(*) FILTER(WHERE i.state='pending')::integer,
  count(*) FILTER(WHERE i.state='unknown_reconcile_required')::integer,
  count(*) FILTER(WHERE i.state='readback_verified')::integer,
  false
 FROM ipat_ops.pppoe_execution_attempts x
 JOIN ipat_ops.pppoe_execution_items i
   ON i.tenant_id=x.tenant_id AND i.attempt_id=x.id
 WHERE x.tenant_id=p_tenant AND x.plan_id=p_plan
   AND (ipat_platform.pppoe_batch_create_capability(p_issuer,p_subject,p_tenant)
     OR ipat_platform.pppoe_batch_review_capability(p_issuer,p_subject,p_tenant)
     OR ipat_platform.pppoe_execution_arm_capability(p_issuer,p_subject,p_tenant))
 GROUP BY x.id,x.state,x.armed_at,x.updated_at
$body$;
ALTER FUNCTION ipat_platform.get_pppoe_execution_status(text,text,uuid,uuid)
 OWNER TO ipat_pppoe_exec_owner;
REVOKE ALL ON FUNCTION ipat_platform.get_pppoe_execution_status(
 text,text,uuid,uuid) FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.get_pppoe_execution_status(
 text,text,uuid,uuid) TO ipat_tenant_api_exec;

COMMIT;
