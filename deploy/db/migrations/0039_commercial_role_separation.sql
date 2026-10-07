-- R10.09: commercial tenant role vocabulary + PPPoE maker/checker separation.
-- Append-only migration. No physical RouterOS execution capability is added.
BEGIN;

ALTER TABLE ipat_platform.identity_memberships
  DROP CONSTRAINT identity_memberships_role_check;
ALTER TABLE ipat_platform.identity_memberships
  ADD CONSTRAINT identity_memberships_role_check CHECK(role IN(
    'tenant_admin','system_admin','security_admin','noc_manager','noc_engineer',
    'provisioning_officer','helpdesk','field_technician','auditor'));

-- Preserve every pre-existing role's scope semantics. New system/security/
-- provisioning roles are tenant-wide; NOC manager/field technician require an
-- exact POP/Site grant just like existing scoped operational roles.
CREATE OR REPLACE FUNCTION ipat_platform.lookup_active_membership(
    p_issuer text, p_subject text, p_tenant uuid, p_role text, p_pop text
) RETURNS TABLE (approved_by text, expires_at timestamptz, tenant_slug text)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, ipat_platform
AS $ipat_sql$
    SELECT m.approved_by, m.expires_at, t.tenant_slug
    FROM ipat_platform.identity_memberships AS m
    JOIN ipat_platform.tenants AS t
      ON t.id = m.tenant_id AND t.state = 'active'
    WHERE m.issuer = p_issuer AND m.subject = p_subject
      AND m.tenant_id = p_tenant AND m.role = p_role
      AND m.revoked_at IS NULL
      AND m.created_at <= statement_timestamp()
      AND m.expires_at > statement_timestamp()
      AND length(m.approved_by) > 0
      AND p_issuer IS NOT NULL AND p_subject IS NOT NULL
      AND p_tenant IS NOT NULL AND p_role IS NOT NULL
      AND p_role IN(
        'tenant_admin','system_admin','security_admin','noc_manager','noc_engineer',
        'provisioning_officer','helpdesk','field_technician','auditor')
      AND (
        (p_role IN('tenant_admin','system_admin','security_admin','provisioning_officer')
          AND p_pop IS NULL)
        OR
        (p_role IN('noc_manager','noc_engineer','helpdesk','field_technician','auditor')
          AND p_pop IS NOT NULL
          AND EXISTS (
            SELECT 1 FROM ipat_platform.identity_pop_grants AS g
            WHERE g.tenant_id = m.tenant_id AND g.issuer = m.issuer
              AND g.subject = m.subject AND g.role = m.role
              AND g.pop_id = p_pop
          ))
      )
$ipat_sql$;


-- Browser session establishment is deliberately role-agnostic within the
-- approved commercial membership vocabulary. Authorization remains per-route.
CREATE OR REPLACE FUNCTION ipat_platform.issue_tenant_browser_session(
 p_issuer text,p_subject text,p_tenant uuid,p_domain uuid,p_session uuid,
 p_cookie_sha text,p_csrf_sha text,p_expires timestamptz)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE v_now timestamptz:=clock_timestamp();
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_domain IS NULL OR p_session IS NULL
   OR p_cookie_sha !~ '^[0-9a-f]{64}$' OR p_csrf_sha !~ '^[0-9a-f]{64}$' OR p_cookie_sha=p_csrf_sha
   OR p_expires<=v_now OR p_expires>v_now+interval '15 minutes'
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.tenants t WHERE t.id=p_tenant AND t.state='active')
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.tenant_domains d WHERE d.id=p_domain AND d.tenant_id=p_tenant
       AND d.verification_state='verified' AND d.activation_state='active' AND d.disabled_at IS NULL)
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.identity_memberships m
       WHERE m.tenant_id=p_tenant AND m.issuer=p_issuer AND m.subject=p_subject
         AND m.role IN('tenant_admin','system_admin','security_admin','noc_manager','noc_engineer',
                       'provisioning_officer','helpdesk','field_technician','auditor')
         AND m.revoked_at IS NULL AND m.created_at<=v_now AND m.expires_at>v_now
         AND length(btrim(m.approved_by))>0)
 THEN RETURN NULL; END IF;
 IF (SELECT count(*) FROM ipat_platform.tenant_browser_sessions s
      WHERE s.tenant_id=p_tenant AND s.issuer=p_issuer AND s.subject=p_subject
        AND s.revoked_at IS NULL AND s.expires_at>v_now AND s.last_seen_at>v_now-interval '5 minutes') >= 8
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_platform.tenant_browser_sessions(
   id,tenant_id,domain_id,issuer,subject,cookie_sha256,csrf_sha256,issued_at,last_seen_at,expires_at)
 VALUES(p_session,p_tenant,p_domain,p_issuer,p_subject,p_cookie_sha,p_csrf_sha,v_now,v_now,p_expires);
 RETURN p_session;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.issue_tenant_browser_session(
 text,text,uuid,uuid,uuid,text,text,timestamptz) OWNER TO ipat_browser_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.issue_tenant_browser_session(
 text,text,uuid,uuid,uuid,text,text,timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.issue_tenant_browser_session(
 text,text,uuid,uuid,uuid,text,text,timestamptz) TO ipat_browser_session_issue_exec;

CREATE OR REPLACE FUNCTION ipat_platform.authenticate_tenant_browser_session(
 p_cookie_sha text,p_hostname text,p_csrf_sha text,p_mutation boolean)
RETURNS TABLE(tenant_id uuid,domain_id uuid,issuer text,subject text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE v_now timestamptz:=clock_timestamp(); v ipat_platform.tenant_browser_sessions%ROWTYPE;
BEGIN
 IF p_cookie_sha !~ '^[0-9a-f]{64}$' OR p_hostname IS NULL
   OR length(p_hostname) NOT BETWEEN 3 AND 253 OR p_hostname<>lower(p_hostname)
   OR p_hostname !~ '^[a-z0-9]([a-z0-9.-]*[a-z0-9])$' OR position('..' in p_hostname)>0
   OR p_mutation IS NULL
   OR (p_mutation AND (p_csrf_sha IS NULL OR p_csrf_sha !~ '^[0-9a-f]{64}$'))
 THEN RETURN; END IF;
 SELECT s.* INTO v FROM ipat_platform.tenant_browser_sessions s
 JOIN ipat_platform.tenant_domains d ON d.id=s.domain_id AND d.tenant_id=s.tenant_id
 JOIN ipat_platform.tenants t ON t.id=s.tenant_id
 WHERE s.cookie_sha256=p_cookie_sha AND d.hostname=p_hostname
   AND d.verification_state='verified' AND d.activation_state='active' AND d.disabled_at IS NULL
   AND t.state='active' AND s.revoked_at IS NULL AND s.expires_at>v_now
   AND s.last_seen_at>v_now-interval '5 minutes'
   AND (NOT p_mutation OR s.csrf_sha256=p_csrf_sha)
   AND EXISTS(SELECT 1 FROM ipat_platform.identity_memberships m
       WHERE m.tenant_id=s.tenant_id AND m.issuer=s.issuer AND m.subject=s.subject
         AND m.role IN('tenant_admin','system_admin','security_admin','noc_manager','noc_engineer',
                       'provisioning_officer','helpdesk','field_technician','auditor')
         AND m.revoked_at IS NULL AND m.created_at<=v_now AND m.expires_at>v_now
         AND length(btrim(m.approved_by))>0)
 FOR UPDATE OF s;
 IF NOT FOUND THEN RETURN; END IF;
 UPDATE ipat_platform.tenant_browser_sessions s SET last_seen_at=v_now WHERE s.id=v.id;
 tenant_id:=v.tenant_id; domain_id:=v.domain_id; issuer:=v.issuer; subject:=v.subject;
 RETURN NEXT;
END $body$;
ALTER FUNCTION ipat_platform.authenticate_tenant_browser_session(
 text,text,text,boolean) OWNER TO ipat_browser_session_owner;
REVOKE ALL ON FUNCTION ipat_platform.authenticate_tenant_browser_session(
 text,text,text,boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.authenticate_tenant_browser_session(
 text,text,text,boolean) TO ipat_browser_session_auth_exec;

CREATE FUNCTION ipat_platform.pppoe_batch_create_capability(
 p_issuer text,p_subject text,p_tenant uuid)
RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $$
 SELECT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'provisioning_officer',NULL))
$$;
ALTER FUNCTION ipat_platform.pppoe_batch_create_capability(text,text,uuid)
 OWNER TO ipat_pppoe_plan_owner;
REVOKE ALL ON FUNCTION ipat_platform.pppoe_batch_create_capability(text,text,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.pppoe_batch_create_capability(text,text,uuid)
 TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.pppoe_batch_review_capability(
 p_issuer text,p_subject text,p_tenant uuid)
RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $$
 SELECT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'security_admin',NULL))
$$;
ALTER FUNCTION ipat_platform.pppoe_batch_review_capability(text,text,uuid)
 OWNER TO ipat_pppoe_plan_owner;
REVOKE ALL ON FUNCTION ipat_platform.pppoe_batch_review_capability(text,text,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.pppoe_batch_review_capability(text,text,uuid)
 TO ipat_tenant_api_exec;

CREATE OR REPLACE FUNCTION ipat_platform.pppoe_batch_capability(
 p_issuer text,p_subject text,p_tenant uuid)
RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $$
 SELECT ipat_platform.pppoe_batch_create_capability(p_issuer,p_subject,p_tenant)
     OR ipat_platform.pppoe_batch_review_capability(p_issuer,p_subject,p_tenant)
$$;

CREATE OR REPLACE FUNCTION ipat_platform.create_pppoe_batch_dry_run(
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
  OR NOT ipat_platform.pppoe_batch_create_capability(p_issuer,p_subject,p_tenant)
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

CREATE OR REPLACE FUNCTION ipat_platform.review_pppoe_batch_dry_run(
 p_issuer text,p_subject text,p_tenant uuid,p_plan uuid,p_approve boolean)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform,ipat_ops AS $$
DECLARE p ipat_ops.pppoe_batch_plans%ROWTYPE;
DECLARE now_at timestamptz:=statement_timestamp();
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_plan IS NULL
   OR p_approve IS NULL
   OR NOT ipat_platform.pppoe_batch_review_capability(p_issuer,p_subject,p_tenant)
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
 WHERE p.tenant_id=p_tenant
  AND ipat_platform.pppoe_batch_capability(p_issuer,p_subject,p_tenant)
 ORDER BY p.requested_at DESC,p.id DESC LIMIT 100
$$;

-- Existing list_pppoe_batch_items remains safe because pppoe_batch_capability()
-- is now restricted to only the two PPPoE workflow roles.
COMMIT;
