-- R9.78: tenant-admin maker/checker lifecycle for typed real-POP NOC grants.
-- Separate from legacy exact-Site grants; no physical/network capability.
BEGIN;
CREATE TABLE ipat_platform.noc_real_pop_grant_requests(
 request_id uuid PRIMARY KEY,
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 target_issuer text NOT NULL,
 target_subject text NOT NULL,
 target_role text NOT NULL DEFAULT 'noc_engineer' CHECK(target_role='noc_engineer'),
 pop_code text NOT NULL CHECK(pop_code ~ '^[A-Za-z0-9_.-]{1,128}$'),
 requested_expires_at timestamptz NOT NULL,
 requested_by_issuer text NOT NULL,
 requested_by_subject text NOT NULL,
 state text NOT NULL DEFAULT 'PENDING' CHECK(state IN('PENDING','APPROVED','REJECTED')),
 reviewed_by_issuer text,
 reviewed_by_subject text,
 created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 reviewed_at timestamptz,
 CONSTRAINT noc_request_target_membership FOREIGN KEY(
   tenant_id,target_issuer,target_subject,target_role)
   REFERENCES ipat_platform.identity_memberships(tenant_id,issuer,subject,role),
 CONSTRAINT noc_request_target_pop FOREIGN KEY(tenant_id,pop_code)
   REFERENCES ipat_ops.tenant_pops(tenant_id,code),
 CONSTRAINT noc_request_review_shape CHECK(
   (state='PENDING' AND reviewed_by_issuer IS NULL AND reviewed_by_subject IS NULL AND reviewed_at IS NULL)
   OR (state<>'PENDING' AND reviewed_by_issuer IS NOT NULL AND reviewed_by_subject IS NOT NULL AND reviewed_at IS NOT NULL)),
 CONSTRAINT noc_request_distinct_review CHECK(
   reviewed_by_issuer IS NULL OR requested_by_issuer<>reviewed_by_issuer
   OR requested_by_subject<>reviewed_by_subject),
 CONSTRAINT noc_request_future_expiry CHECK(requested_expires_at>created_at)
);
ALTER TABLE ipat_platform.noc_real_pop_grant_requests OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.noc_real_pop_grant_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.noc_real_pop_grant_requests FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.noc_real_pop_grant_requests FROM PUBLIC,ipat_app_runtime,
 ipat_tenant_api_login,ipat_oidc_session_issuer_login;
CREATE UNIQUE INDEX noc_one_pending_request_per_target_pop ON
 ipat_platform.noc_real_pop_grant_requests(tenant_id,target_issuer,target_subject,pop_code)
 WHERE state='PENDING';

CREATE TABLE ipat_platform.noc_real_pop_grant_events(
 id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 request_id uuid,
 target_issuer text NOT NULL,
 target_subject text NOT NULL,
 pop_code text NOT NULL,
 event text NOT NULL CHECK(event IN('REQUESTED','APPROVED','REJECTED','REVOKED')),
 actor_issuer text NOT NULL,
 actor_subject text NOT NULL,
 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
ALTER TABLE ipat_platform.noc_real_pop_grant_events OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.noc_real_pop_grant_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.noc_real_pop_grant_events FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.noc_real_pop_grant_events FROM PUBLIC,ipat_app_runtime,
 ipat_tenant_api_login,ipat_oidc_session_issuer_login;

CREATE ROLE ipat_noc_grant_workflow_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_noc_grant_workflow_exec NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT;
GRANT USAGE ON SCHEMA ipat_platform,ipat_ops TO ipat_noc_grant_workflow_owner;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_noc_grant_workflow_exec;
GRANT SELECT ON ipat_platform.tenants,ipat_platform.identity_memberships
 TO ipat_noc_grant_workflow_owner;
GRANT SELECT ON ipat_ops.tenant_pops TO ipat_noc_grant_workflow_owner;
GRANT SELECT,INSERT,UPDATE(state,reviewed_by_issuer,reviewed_by_subject,reviewed_at)
 ON ipat_platform.noc_real_pop_grant_requests TO ipat_noc_grant_workflow_owner;
GRANT SELECT,INSERT,UPDATE(requested_by_issuer,requested_by_subject,
 approved_by_issuer,approved_by_subject,created_at,expires_at,revoked_at)
 ON ipat_platform.noc_real_pop_grants TO ipat_noc_grant_workflow_owner;
GRANT SELECT,INSERT ON ipat_platform.noc_real_pop_grant_events
 TO ipat_noc_grant_workflow_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_platform.noc_real_pop_grant_events_id_seq
 TO ipat_noc_grant_workflow_owner;
CREATE POLICY noc_workflow_member_read ON ipat_platform.identity_memberships
 FOR SELECT TO ipat_noc_grant_workflow_owner USING(true);
CREATE POLICY noc_workflow_pop_read ON ipat_ops.tenant_pops
 FOR SELECT TO ipat_noc_grant_workflow_owner USING(true);
CREATE POLICY noc_workflow_request_read ON ipat_platform.noc_real_pop_grant_requests
 FOR SELECT TO ipat_noc_grant_workflow_owner USING(true);
CREATE POLICY noc_workflow_request_insert ON ipat_platform.noc_real_pop_grant_requests
 FOR INSERT TO ipat_noc_grant_workflow_owner WITH CHECK(true);
CREATE POLICY noc_workflow_request_update ON ipat_platform.noc_real_pop_grant_requests
 FOR UPDATE TO ipat_noc_grant_workflow_owner USING(true) WITH CHECK(true);
CREATE POLICY noc_workflow_grant_read ON ipat_platform.noc_real_pop_grants
 FOR SELECT TO ipat_noc_grant_workflow_owner USING(true);
CREATE POLICY noc_workflow_grant_insert ON ipat_platform.noc_real_pop_grants
 FOR INSERT TO ipat_noc_grant_workflow_owner WITH CHECK(true);
CREATE POLICY noc_workflow_grant_update ON ipat_platform.noc_real_pop_grants
 FOR UPDATE TO ipat_noc_grant_workflow_owner USING(true) WITH CHECK(true);
CREATE POLICY noc_workflow_event_read ON ipat_platform.noc_real_pop_grant_events
 FOR SELECT TO ipat_noc_grant_workflow_owner USING(true);
CREATE POLICY noc_workflow_event_insert ON ipat_platform.noc_real_pop_grant_events
 FOR INSERT TO ipat_noc_grant_workflow_owner WITH CHECK(true);
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(text,text,uuid,text,text)
 TO ipat_noc_grant_workflow_owner;

CREATE FUNCTION ipat_platform.request_noc_real_pop_grant(
 p_actor_issuer text,p_actor_subject text,p_tenant uuid,p_request uuid,
 p_target_issuer text,p_target_subject text,p_pop text,p_expires timestamptz)
 RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE prior ipat_platform.noc_real_pop_grant_requests%ROWTYPE;
BEGIN
 IF p_actor_issuer IS NULL OR p_actor_subject IS NULL OR p_tenant IS NULL
 OR p_request IS NULL OR p_target_issuer IS NULL OR p_target_subject IS NULL
 OR p_pop IS NULL OR p_pop !~ '^[A-Za-z0-9_.-]{1,128}$'
 OR p_expires IS NULL OR p_expires<=statement_timestamp()+interval '5 minutes'
 OR p_expires>statement_timestamp()+interval '90 days'
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_actor_issuer,p_actor_subject,p_tenant,'tenant_admin',NULL))
 OR NOT EXISTS(
   SELECT 1 FROM ipat_platform.identity_memberships m
   JOIN ipat_platform.tenants t ON t.id=m.tenant_id AND t.state='active'
   WHERE m.tenant_id=p_tenant AND m.issuer=p_target_issuer
    AND m.subject=p_target_subject AND m.role='noc_engineer'
    AND m.revoked_at IS NULL AND m.created_at<=statement_timestamp()
    AND m.expires_at>statement_timestamp() AND length(btrim(m.approved_by))>0)
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.tenant_pops p
   WHERE p.tenant_id=p_tenant AND p.code=p_pop)
 THEN RETURN NULL; END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended(p_tenant::text||'/noc-request/'||p_request::text,0));
 SELECT * INTO prior FROM ipat_platform.noc_real_pop_grant_requests
 WHERE request_id=p_request;
 IF FOUND THEN
   IF prior.tenant_id=p_tenant AND prior.target_issuer=p_target_issuer
    AND prior.target_subject=p_target_subject AND prior.pop_code=p_pop
    AND prior.requested_expires_at=p_expires
    AND prior.requested_by_issuer=p_actor_issuer
    AND prior.requested_by_subject=p_actor_subject
   THEN RETURN p_request; END IF;
   RETURN NULL;
 END IF;
 IF EXISTS(SELECT 1 FROM ipat_platform.noc_real_pop_grants g
  WHERE g.tenant_id=p_tenant AND g.issuer=p_target_issuer
   AND g.subject=p_target_subject AND g.pop_code=p_pop
   AND g.revoked_at IS NULL AND g.expires_at>statement_timestamp())
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_platform.noc_real_pop_grant_requests(
  request_id,tenant_id,target_issuer,target_subject,pop_code,requested_expires_at,
  requested_by_issuer,requested_by_subject)
 VALUES(p_request,p_tenant,p_target_issuer,p_target_subject,p_pop,p_expires,
  p_actor_issuer,p_actor_subject);
 INSERT INTO ipat_platform.noc_real_pop_grant_events(
  tenant_id,request_id,target_issuer,target_subject,pop_code,event,actor_issuer,actor_subject)
 VALUES(p_tenant,p_request,p_target_issuer,p_target_subject,p_pop,'REQUESTED',
  p_actor_issuer,p_actor_subject);
 RETURN p_request;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN
 RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.request_noc_real_pop_grant(
 text,text,uuid,uuid,text,text,text,timestamptz) OWNER TO ipat_noc_grant_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.request_noc_real_pop_grant(
 text,text,uuid,uuid,text,text,text,timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.request_noc_real_pop_grant(
 text,text,uuid,uuid,text,text,text,timestamptz) TO ipat_noc_grant_workflow_exec;

CREATE FUNCTION ipat_platform.review_noc_real_pop_grant(
 p_checker_issuer text,p_checker_subject text,p_tenant uuid,p_request uuid,p_approve boolean)
 RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE r ipat_platform.noc_real_pop_grant_requests%ROWTYPE;
DECLARE g ipat_platform.noc_real_pop_grants%ROWTYPE;
DECLARE final_state text;
BEGIN
 IF p_checker_issuer IS NULL OR p_checker_subject IS NULL OR p_tenant IS NULL
 OR p_request IS NULL OR p_approve IS NULL
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_checker_issuer,p_checker_subject,p_tenant,'tenant_admin',NULL))
 THEN RETURN NULL; END IF;
 SELECT * INTO r FROM ipat_platform.noc_real_pop_grant_requests
 WHERE request_id=p_request AND tenant_id=p_tenant FOR UPDATE;
 IF NOT FOUND OR r.state<>'PENDING'
 OR (r.requested_by_issuer=p_checker_issuer AND r.requested_by_subject=p_checker_subject)
 THEN RETURN NULL; END IF;
 IF p_approve THEN
  IF r.requested_expires_at<=statement_timestamp()
  OR NOT EXISTS(
   SELECT 1 FROM ipat_platform.identity_memberships m
   JOIN ipat_platform.tenants t ON t.id=m.tenant_id AND t.state='active'
   WHERE m.tenant_id=p_tenant AND m.issuer=r.target_issuer
    AND m.subject=r.target_subject AND m.role='noc_engineer'
    AND m.revoked_at IS NULL AND m.created_at<=statement_timestamp()
    AND m.expires_at>statement_timestamp() AND length(btrim(m.approved_by))>0)
  OR NOT EXISTS(SELECT 1 FROM ipat_ops.tenant_pops p
    WHERE p.tenant_id=p_tenant AND p.code=r.pop_code)
  THEN RETURN NULL; END IF;
  SELECT * INTO g FROM ipat_platform.noc_real_pop_grants
   WHERE tenant_id=p_tenant AND issuer=r.target_issuer
    AND subject=r.target_subject AND pop_code=r.pop_code FOR UPDATE;
  IF FOUND AND g.revoked_at IS NULL AND g.expires_at>statement_timestamp()
  THEN RETURN NULL; END IF;
  IF FOUND THEN
   UPDATE ipat_platform.noc_real_pop_grants SET
    requested_by_issuer=r.requested_by_issuer,
    requested_by_subject=r.requested_by_subject,
    approved_by_issuer=p_checker_issuer,
    approved_by_subject=p_checker_subject,
    created_at=statement_timestamp(),
    expires_at=r.requested_expires_at,
    revoked_at=NULL
   WHERE tenant_id=p_tenant AND issuer=r.target_issuer
    AND subject=r.target_subject AND pop_code=r.pop_code;
  ELSE
   INSERT INTO ipat_platform.noc_real_pop_grants(
    tenant_id,issuer,subject,role,pop_code,requested_by_issuer,requested_by_subject,
    approved_by_issuer,approved_by_subject,expires_at)
   VALUES(p_tenant,r.target_issuer,r.target_subject,'noc_engineer',r.pop_code,
    r.requested_by_issuer,r.requested_by_subject,p_checker_issuer,p_checker_subject,
    r.requested_expires_at);
  END IF;
  final_state='APPROVED';
 ELSE
  final_state='REJECTED';
 END IF;
 UPDATE ipat_platform.noc_real_pop_grant_requests SET
  state=final_state,reviewed_by_issuer=p_checker_issuer,
  reviewed_by_subject=p_checker_subject,reviewed_at=statement_timestamp()
 WHERE request_id=p_request;
 INSERT INTO ipat_platform.noc_real_pop_grant_events(
  tenant_id,request_id,target_issuer,target_subject,pop_code,event,actor_issuer,actor_subject)
 VALUES(p_tenant,p_request,r.target_issuer,r.target_subject,r.pop_code,final_state,
  p_checker_issuer,p_checker_subject);
 RETURN final_state;
END $body$;
ALTER FUNCTION ipat_platform.review_noc_real_pop_grant(
 text,text,uuid,uuid,boolean) OWNER TO ipat_noc_grant_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.review_noc_real_pop_grant(
 text,text,uuid,uuid,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.review_noc_real_pop_grant(
 text,text,uuid,uuid,boolean) TO ipat_noc_grant_workflow_exec;

CREATE FUNCTION ipat_platform.revoke_noc_real_pop_grant(
 p_actor_issuer text,p_actor_subject text,p_tenant uuid,
 p_target_issuer text,p_target_subject text,p_pop text)
 RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE changed uuid;
BEGIN
 IF NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_actor_issuer,p_actor_subject,p_tenant,'tenant_admin',NULL))
 THEN RETURN false; END IF;
 UPDATE ipat_platform.noc_real_pop_grants SET revoked_at=statement_timestamp()
 WHERE tenant_id=p_tenant AND issuer=p_target_issuer
  AND subject=p_target_subject AND pop_code=p_pop
  AND revoked_at IS NULL AND expires_at>statement_timestamp()
 RETURNING tenant_id INTO changed;
 IF changed IS NULL THEN RETURN false; END IF;
 INSERT INTO ipat_platform.noc_real_pop_grant_events(
  tenant_id,target_issuer,target_subject,pop_code,event,actor_issuer,actor_subject)
 VALUES(p_tenant,p_target_issuer,p_target_subject,p_pop,'REVOKED',
  p_actor_issuer,p_actor_subject);
 RETURN true;
END $body$;
ALTER FUNCTION ipat_platform.revoke_noc_real_pop_grant(
 text,text,uuid,text,text,text) OWNER TO ipat_noc_grant_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.revoke_noc_real_pop_grant(
 text,text,uuid,text,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.revoke_noc_real_pop_grant(
 text,text,uuid,text,text,text) TO ipat_noc_grant_workflow_exec;

CREATE FUNCTION ipat_platform.list_current_noc_members_for_admin(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(issuer text,subject text,expires_at timestamptz)
 LANGUAGE sql STABLE SECURITY DEFINER SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT m.issuer,m.subject,m.expires_at
 FROM ipat_platform.identity_memberships m
 JOIN ipat_platform.tenants t ON t.id=m.tenant_id AND t.state='active'
 WHERE m.tenant_id=p_tenant AND m.role='noc_engineer'
 AND m.revoked_at IS NULL AND m.created_at<=statement_timestamp()
 AND m.expires_at>statement_timestamp() AND length(btrim(m.approved_by))>0
 AND EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 ORDER BY m.subject,m.issuer LIMIT 200
$body$;
ALTER FUNCTION ipat_platform.list_current_noc_members_for_admin(text,text,uuid)
 OWNER TO ipat_noc_grant_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_current_noc_members_for_admin(text,text,uuid)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_current_noc_members_for_admin(text,text,uuid)
 TO ipat_noc_grant_workflow_exec;

CREATE FUNCTION ipat_platform.list_noc_real_pop_access_for_admin(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(kind text,request_id uuid,target_issuer text,target_subject text,
  pop_code text,state text,expires_at timestamptz,requested_by text,reviewed_by text)
 LANGUAGE sql STABLE SECURITY DEFINER SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT x.kind,x.request_id,x.target_issuer,x.target_subject,x.pop_code,
  x.state,x.expires_at,x.requested_by,x.reviewed_by
 FROM (
  SELECT 'request'::text AS kind,r.request_id,r.target_issuer,r.target_subject,
   r.pop_code,r.state,r.requested_expires_at AS expires_at,
   r.requested_by_issuer||'#'||r.requested_by_subject AS requested_by,
   CASE WHEN r.reviewed_by_issuer IS NULL THEN NULL
    ELSE r.reviewed_by_issuer||'#'||r.reviewed_by_subject END AS reviewed_by
  FROM ipat_platform.noc_real_pop_grant_requests r
  WHERE r.tenant_id=p_tenant AND EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
  UNION ALL
  SELECT 'grant'::text,NULL::uuid,g.issuer,g.subject,g.pop_code,
   CASE WHEN g.revoked_at IS NOT NULL THEN 'REVOKED'
        WHEN g.expires_at<=statement_timestamp() THEN 'EXPIRED' ELSE 'ACTIVE' END,
   g.expires_at,g.requested_by_issuer||'#'||g.requested_by_subject,
   g.approved_by_issuer||'#'||g.approved_by_subject
  FROM ipat_platform.noc_real_pop_grants g
  WHERE g.tenant_id=p_tenant AND EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 ) x ORDER BY x.kind,x.target_subject,x.pop_code LIMIT 500
$body$;
ALTER FUNCTION ipat_platform.list_noc_real_pop_access_for_admin(text,text,uuid)
 OWNER TO ipat_noc_grant_workflow_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_noc_real_pop_access_for_admin(text,text,uuid)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_noc_real_pop_access_for_admin(text,text,uuid)
 TO ipat_noc_grant_workflow_exec;

GRANT ipat_noc_grant_workflow_exec TO ipat_tenant_api_exec;
COMMIT;
