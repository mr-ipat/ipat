-- R10.07: durable tenant-scoped PPPoE batch DRY-RUN + maker/checker.
-- Deliberately NO physical RouterOS execution function exists in this migration.
-- Raw PPPoE passwords are forbidden; only tenant-scoped Vault references may be
-- carried for create/update intents and are never returned through read APIs.
BEGIN;

CREATE TABLE ipat_ops.pppoe_batch_plans(
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 id uuid NOT NULL,
 request_id uuid NOT NULL,
 router_id uuid NOT NULL,
 site_code text NOT NULL CHECK(site_code ~ '^[A-Za-z0-9_.-]{1,128}$'),
 pop_code text NOT NULL CHECK(pop_code ~ '^[A-Za-z0-9_.-]{1,128}$'),
 idempotency_key text NOT NULL CHECK(
   length(idempotency_key) BETWEEN 1 AND 128
   AND idempotency_key ~ '^[A-Za-z0-9_.:-]+$'),
 plan_digest text NOT NULL CHECK(plan_digest ~ '^[0-9a-f]{64}$'),
 item_count integer NOT NULL CHECK(item_count BETWEEN 1 AND 128),
 state text NOT NULL DEFAULT 'awaiting_approval'
   CHECK(state IN('awaiting_approval','approved','rejected')),
 requested_by_issuer text NOT NULL,
 requested_by_subject text NOT NULL,
 requested_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 reviewed_by_issuer text,
 reviewed_by_subject text,
 reviewed_at timestamptz,
 approval_expires_at timestamptz,
 rate_limit_per_minute integer NOT NULL DEFAULT 20
   CHECK(rate_limit_per_minute BETWEEN 1 AND 60),
 execution_allowed boolean NOT NULL DEFAULT false CHECK(execution_allowed=false),
 physical_readback_verified boolean NOT NULL DEFAULT false
   CHECK(physical_readback_verified=false),
 basis text NOT NULL DEFAULT 'subscriber360_declared_not_router_readback'
   CHECK(basis='subscriber360_declared_not_router_readback'),
 PRIMARY KEY(tenant_id,id),
 UNIQUE(tenant_id,request_id),
 UNIQUE(tenant_id,idempotency_key),
 FOREIGN KEY(tenant_id,router_id) REFERENCES ipat_ops.managed_devices(tenant_id,id),
 FOREIGN KEY(tenant_id,site_code) REFERENCES ipat_ops.tenant_sites(tenant_id,code),
 FOREIGN KEY(tenant_id,pop_code) REFERENCES ipat_ops.tenant_pops(tenant_id,code),
 CHECK(
   (state='awaiting_approval' AND reviewed_by_issuer IS NULL
    AND reviewed_by_subject IS NULL AND reviewed_at IS NULL
    AND approval_expires_at IS NULL)
   OR
   (state='approved' AND reviewed_by_issuer IS NOT NULL
    AND reviewed_by_subject IS NOT NULL AND reviewed_at IS NOT NULL
    AND approval_expires_at IS NOT NULL
    AND (reviewed_by_issuer,reviewed_by_subject)
      IS DISTINCT FROM (requested_by_issuer,requested_by_subject))
   OR
   (state='rejected' AND reviewed_by_issuer IS NOT NULL
    AND reviewed_by_subject IS NOT NULL AND reviewed_at IS NOT NULL
    AND approval_expires_at IS NULL
    AND (reviewed_by_issuer,reviewed_by_subject)
      IS DISTINCT FROM (requested_by_issuer,requested_by_subject))
 )
);
ALTER TABLE ipat_ops.pppoe_batch_plans OWNER TO ipat_schema_owner;

CREATE TABLE ipat_ops.pppoe_batch_items(
 tenant_id uuid NOT NULL,
 plan_id uuid NOT NULL,
 ordinal integer NOT NULL CHECK(ordinal BETWEEN 1 AND 128),
 subscriber_id text NOT NULL CHECK(
   length(subscriber_id) BETWEEN 1 AND 128
   AND subscriber_id ~ '^[A-Za-z0-9_.-]+$'),
 action text NOT NULL CHECK(action IN('create','update','disable')),
 before_username text,
 desired_username text,
 profile_name text,
 secret_ref text,
 PRIMARY KEY(tenant_id,plan_id,ordinal),
 UNIQUE(tenant_id,plan_id,subscriber_id),
 FOREIGN KEY(tenant_id,plan_id)
   REFERENCES ipat_ops.pppoe_batch_plans(tenant_id,id) ON DELETE RESTRICT,
 FOREIGN KEY(tenant_id,subscriber_id)
   REFERENCES ipat_ops.subscribers(tenant_id,customer_ref),
 CHECK(before_username IS NULL OR (
   length(before_username) BETWEEN 1 AND 128
   AND before_username=btrim(before_username)
   AND before_username !~ '[[:cntrl:][:space:]]')),
 CHECK(desired_username IS NULL OR (
   length(desired_username) BETWEEN 1 AND 128
   AND desired_username=btrim(desired_username)
   AND desired_username !~ '[[:cntrl:][:space:]]')),
 CHECK(profile_name IS NULL OR (
   length(profile_name) BETWEEN 1 AND 64
   AND profile_name ~ '^[A-Za-z0-9_.:-]+$')),
 CHECK(secret_ref IS NULL OR (
   length(secret_ref) BETWEEN 55 AND 255
   AND secret_ref LIKE 'vault://tenant/' || tenant_id::text || '/pppoe/%'
   AND secret_ref ~ '^vault://tenant/[a-f0-9-]+/pppoe/[A-Za-z0-9/_-]+$')),
 CHECK(
   (action='create' AND before_username IS NULL
     AND desired_username IS NOT NULL AND profile_name IS NOT NULL
     AND secret_ref IS NOT NULL)
   OR
   (action='update' AND before_username IS NOT NULL
     AND desired_username IS NOT NULL AND profile_name IS NOT NULL
     AND secret_ref IS NOT NULL)
   OR
   (action='disable' AND before_username IS NOT NULL
     AND desired_username IS NULL AND profile_name IS NULL
     AND secret_ref IS NULL)
 )
);
ALTER TABLE ipat_ops.pppoe_batch_items OWNER TO ipat_schema_owner;

CREATE UNIQUE INDEX pppoe_batch_unique_desired_username
 ON ipat_ops.pppoe_batch_items(tenant_id,plan_id,desired_username)
 WHERE desired_username IS NOT NULL;

CREATE TABLE ipat_ops.pppoe_batch_audit(
 tenant_id uuid NOT NULL,
 plan_id uuid NOT NULL,
 sequence smallint NOT NULL CHECK(sequence BETWEEN 1 AND 2),
 event text NOT NULL CHECK(event IN('DRY_RUN_CREATED','APPROVED','REJECTED')),
 actor_issuer text NOT NULL,
 actor_subject text NOT NULL,
 occurred_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,plan_id,sequence),
 FOREIGN KEY(tenant_id,plan_id)
   REFERENCES ipat_ops.pppoe_batch_plans(tenant_id,id)
);
ALTER TABLE ipat_ops.pppoe_batch_audit OWNER TO ipat_schema_owner;

ALTER TABLE ipat_ops.pppoe_batch_plans ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_batch_plans FORCE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_batch_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_batch_items FORCE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_batch_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.pppoe_batch_audit FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.pppoe_batch_plans,ipat_ops.pppoe_batch_items,
 ipat_ops.pppoe_batch_audit FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_login,
 ipat_oidc_session_issuer_login;

CREATE ROLE ipat_pppoe_plan_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
 NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform,ipat_ops TO ipat_pppoe_plan_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
 text,text,uuid,text,text) TO ipat_pppoe_plan_owner;
GRANT SELECT ON ipat_platform.tenants TO ipat_pppoe_plan_owner;
GRANT SELECT ON ipat_ops.managed_devices,ipat_ops.subscribers,
 ipat_ops.tenant_sites,ipat_ops.tenant_pops TO ipat_pppoe_plan_owner;
-- All source tables FORCE RLS. The SECURITY DEFINER plan owner gets explicit
-- read-only policies; tenant/API logins still have no raw table privileges.
CREATE POLICY pppoe_plan_managed_read ON ipat_ops.managed_devices
 FOR SELECT TO ipat_pppoe_plan_owner USING(true);
CREATE POLICY pppoe_plan_subscriber_read ON ipat_ops.subscribers
 FOR SELECT TO ipat_pppoe_plan_owner USING(true);
CREATE POLICY pppoe_plan_site_read ON ipat_ops.tenant_sites
 FOR SELECT TO ipat_pppoe_plan_owner USING(true);
CREATE POLICY pppoe_plan_pop_read ON ipat_ops.tenant_pops
 FOR SELECT TO ipat_pppoe_plan_owner USING(true);
GRANT SELECT,INSERT,UPDATE ON ipat_ops.pppoe_batch_plans TO ipat_pppoe_plan_owner;
GRANT SELECT,INSERT ON ipat_ops.pppoe_batch_items,ipat_ops.pppoe_batch_audit
 TO ipat_pppoe_plan_owner;
CREATE POLICY pppoe_plan_owner_all ON ipat_ops.pppoe_batch_plans
 FOR ALL TO ipat_pppoe_plan_owner USING(true) WITH CHECK(true);
CREATE POLICY pppoe_item_owner_all ON ipat_ops.pppoe_batch_items
 FOR ALL TO ipat_pppoe_plan_owner USING(true) WITH CHECK(true);
CREATE POLICY pppoe_audit_owner_all ON ipat_ops.pppoe_batch_audit
 FOR ALL TO ipat_pppoe_plan_owner USING(true) WITH CHECK(true);

CREATE FUNCTION ipat_platform.pppoe_batch_capability(
 p_issuer text,p_subject text,p_tenant uuid)
RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $$
 SELECT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
$$;
ALTER FUNCTION ipat_platform.pppoe_batch_capability(text,text,uuid)
 OWNER TO ipat_pppoe_plan_owner;
REVOKE ALL ON FUNCTION ipat_platform.pppoe_batch_capability(text,text,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.pppoe_batch_capability(text,text,uuid)
 TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.create_pppoe_batch_dry_run(
 p_issuer text,p_subject text,p_tenant uuid,p_plan uuid,p_request uuid,
 p_router uuid,p_key text,p_digest text,p_items jsonb)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $$
DECLARE existing ipat_ops.pppoe_batch_plans%ROWTYPE;
DECLARE router_site text;
DECLARE router_pop text;
DECLARE item jsonb;
DECLARE n integer;
DECLARE ord integer:=0;
DECLARE sub_user text;
DECLARE sid text;
DECLARE act text;
DECLARE want_user text;
DECLARE profile text;
DECLARE secret text;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_plan IS NULL
  OR p_request IS NULL OR p_router IS NULL OR p_key IS NULL OR p_digest IS NULL
  OR p_key !~ '^[A-Za-z0-9_.:-]{1,128}$' OR p_digest !~ '^[0-9a-f]{64}$'
  OR jsonb_typeof(p_items)<>'array'
  OR NOT ipat_platform.pppoe_batch_capability(p_issuer,p_subject,p_tenant)
 THEN RETURN NULL; END IF;
 n=jsonb_array_length(p_items);
 IF n NOT BETWEEN 1 AND 128 THEN RETURN NULL; END IF;

 SELECT d.pop_id,s.parent_pop_code INTO router_site,router_pop
 FROM ipat_ops.managed_devices d
 JOIN ipat_ops.tenant_sites s
   ON s.tenant_id=d.tenant_id AND s.code=d.pop_id
 WHERE d.tenant_id=p_tenant AND d.id=p_router AND d.device_kind='router'
  AND d.vendor='MikroTik' AND d.management_transport='routeros_api_ssl'
  AND d.lifecycle_state='SAVED' AND d.archived_at IS NULL
  AND s.parent_pop_code IS NOT NULL;
 IF NOT FOUND THEN RETURN NULL; END IF;

 PERFORM pg_advisory_xact_lock(hashtextextended(
   p_tenant::text||'/'||p_key,0));
 SELECT * INTO existing FROM ipat_ops.pppoe_batch_plans
  WHERE tenant_id=p_tenant AND (request_id=p_request OR idempotency_key=p_key)
  ORDER BY (request_id=p_request) DESC LIMIT 1;
 IF FOUND THEN
  IF existing.id=p_plan AND existing.request_id=p_request
    AND existing.router_id=p_router AND existing.site_code=router_site
    AND existing.pop_code=router_pop AND existing.idempotency_key=p_key
    AND existing.plan_digest=p_digest AND existing.item_count=n
    AND existing.requested_by_issuer=p_issuer
    AND existing.requested_by_subject=p_subject
    AND (SELECT jsonb_agg(jsonb_build_object(
       'subscriber_id',i.subscriber_id,'action',i.action,
       'username',i.desired_username,'profile',i.profile_name,
       'secret_ref',i.secret_ref) ORDER BY i.ordinal)
      FROM ipat_ops.pppoe_batch_items i
      WHERE i.tenant_id=p_tenant AND i.plan_id=existing.id)=p_items
  THEN RETURN existing.id; END IF;
  RETURN NULL;
 END IF;

 INSERT INTO ipat_ops.pppoe_batch_plans(
   tenant_id,id,request_id,router_id,site_code,pop_code,idempotency_key,plan_digest,
   item_count,requested_by_issuer,requested_by_subject)
 VALUES(p_tenant,p_plan,p_request,p_router,router_site,router_pop,p_key,p_digest,n,p_issuer,p_subject);

 FOR item IN SELECT value FROM jsonb_array_elements(p_items)
 LOOP
  ord=ord+1;
  IF jsonb_typeof(item)<>'object'
    OR (SELECT count(*) FROM jsonb_object_keys(item))<>5
    OR EXISTS(SELECT 1 FROM jsonb_object_keys(item) k
      WHERE k NOT IN('subscriber_id','action','username','profile','secret_ref'))
  THEN RAISE EXCEPTION 'invalid item shape'; END IF;
  sid=item->>'subscriber_id';act=item->>'action';
  want_user=NULLIF(item->>'username','');
  profile=NULLIF(item->>'profile','');
  secret=NULLIF(item->>'secret_ref','');
  IF sid IS NULL OR sid !~ '^[A-Za-z0-9_.-]{1,128}$'
    OR act NOT IN('create','update','disable')
  THEN RAISE EXCEPTION 'invalid item'; END IF;

  SELECT s.pppoe_username INTO sub_user FROM ipat_ops.subscribers s
   WHERE s.tenant_id=p_tenant AND s.customer_ref=sid
    AND s.subscriber360_profile AND s.distribution_device_id=p_router;
  IF NOT FOUND THEN RAISE EXCEPTION 'subscriber outside router scope'; END IF;

  IF act='create' THEN
    IF sub_user IS NOT NULL OR want_user IS NULL OR profile IS NULL OR secret IS NULL
    THEN RAISE EXCEPTION 'invalid create diff'; END IF;
  ELSIF act='update' THEN
    IF sub_user IS NULL OR want_user IS NULL OR profile IS NULL OR secret IS NULL
    THEN RAISE EXCEPTION 'invalid update diff'; END IF;
  ELSE
    IF sub_user IS NULL OR want_user IS NOT NULL OR profile IS NOT NULL OR secret IS NOT NULL
    THEN RAISE EXCEPTION 'invalid disable diff'; END IF;
  END IF;

  INSERT INTO ipat_ops.pppoe_batch_items(
   tenant_id,plan_id,ordinal,subscriber_id,action,before_username,
   desired_username,profile_name,secret_ref)
  VALUES(p_tenant,p_plan,ord,sid,act,sub_user,want_user,profile,secret);
 END LOOP;

 INSERT INTO ipat_ops.pppoe_batch_audit(
  tenant_id,plan_id,sequence,event,actor_issuer,actor_subject)
 VALUES(p_tenant,p_plan,1,'DRY_RUN_CREATED',p_issuer,p_subject);
 RETURN p_plan;
EXCEPTION WHEN check_violation OR unique_violation OR foreign_key_violation OR raise_exception THEN
 RETURN NULL;
END $$;
ALTER FUNCTION ipat_platform.create_pppoe_batch_dry_run(
 text,text,uuid,uuid,uuid,uuid,text,text,jsonb) OWNER TO ipat_pppoe_plan_owner;
REVOKE ALL ON FUNCTION ipat_platform.create_pppoe_batch_dry_run(
 text,text,uuid,uuid,uuid,uuid,text,text,jsonb) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.create_pppoe_batch_dry_run(
 text,text,uuid,uuid,uuid,uuid,text,text,jsonb) TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.review_pppoe_batch_dry_run(
 p_issuer text,p_subject text,p_tenant uuid,p_plan uuid,p_approve boolean)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $$
DECLARE p ipat_ops.pppoe_batch_plans%ROWTYPE;
DECLARE now_at timestamptz:=statement_timestamp();
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_plan IS NULL
   OR p_approve IS NULL
   OR NOT ipat_platform.pppoe_batch_capability(p_issuer,p_subject,p_tenant)
 THEN RETURN false; END IF;
 SELECT * INTO p FROM ipat_ops.pppoe_batch_plans
  WHERE tenant_id=p_tenant AND id=p_plan FOR UPDATE;
 IF NOT FOUND OR p.state<>'awaiting_approval'
   OR (p.requested_by_issuer,p.requested_by_subject)=(p_issuer,p_subject)
 THEN RETURN false; END IF;
 UPDATE ipat_ops.pppoe_batch_plans SET
   state=CASE WHEN p_approve THEN 'approved' ELSE 'rejected' END,
   reviewed_by_issuer=p_issuer,reviewed_by_subject=p_subject,reviewed_at=now_at,
   approval_expires_at=CASE WHEN p_approve THEN now_at+interval '30 minutes' ELSE NULL END
 WHERE tenant_id=p_tenant AND id=p_plan;
 INSERT INTO ipat_ops.pppoe_batch_audit(
  tenant_id,plan_id,sequence,event,actor_issuer,actor_subject)
 VALUES(p_tenant,p_plan,2,CASE WHEN p_approve THEN 'APPROVED' ELSE 'REJECTED' END,
   p_issuer,p_subject);
 RETURN true;
END $$;
ALTER FUNCTION ipat_platform.review_pppoe_batch_dry_run(
 text,text,uuid,uuid,boolean) OWNER TO ipat_pppoe_plan_owner;
REVOKE ALL ON FUNCTION ipat_platform.review_pppoe_batch_dry_run(
 text,text,uuid,uuid,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.review_pppoe_batch_dry_run(
 text,text,uuid,uuid,boolean) TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.list_pppoe_batch_dry_runs(
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
    AND (p.requested_by_issuer,p.requested_by_subject)<>(p_issuer,p_subject)
 FROM ipat_ops.pppoe_batch_plans p
 WHERE p.tenant_id=p_tenant
  AND ipat_platform.pppoe_batch_capability(p_issuer,p_subject,p_tenant)
 ORDER BY p.requested_at DESC,p.id DESC LIMIT 100
$$;
ALTER FUNCTION ipat_platform.list_pppoe_batch_dry_runs(text,text,uuid)
 OWNER TO ipat_pppoe_plan_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_pppoe_batch_dry_runs(text,text,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_pppoe_batch_dry_runs(text,text,uuid)
 TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.list_pppoe_batch_items(
 p_issuer text,p_subject text,p_tenant uuid,p_plan uuid)
RETURNS TABLE(
 ordinal integer,subscriber_id text,action text,before_username text,
 desired_username text,profile_name text,has_secret_ref boolean)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $$
 SELECT i.ordinal,i.subscriber_id,i.action,i.before_username,i.desired_username,
   i.profile_name,i.secret_ref IS NOT NULL
 FROM ipat_ops.pppoe_batch_items i
 WHERE i.tenant_id=p_tenant AND i.plan_id=p_plan
  AND ipat_platform.pppoe_batch_capability(p_issuer,p_subject,p_tenant)
 ORDER BY i.ordinal
$$;
ALTER FUNCTION ipat_platform.list_pppoe_batch_items(text,text,uuid,uuid)
 OWNER TO ipat_pppoe_plan_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_pppoe_batch_items(text,text,uuid,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_pppoe_batch_items(text,text,uuid,uuid)
 TO ipat_tenant_api_exec;

-- There is intentionally NO claim/lease/execute/router-write function.
COMMIT;
