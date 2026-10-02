-- R9.69 platform company lifecycle, commercial metadata and tenant-admin invitation.
-- Platform owner remains platform-scoped and NEVER automatically becomes a
-- tenant member or receives tenant Site/Device/secret privileges.
BEGIN;

CREATE TABLE ipat_platform.tenant_profiles (
 tenant_id uuid PRIMARY KEY REFERENCES ipat_platform.tenants(id) ON DELETE CASCADE,
 display_name text NOT NULL CHECK(length(display_name) BETWEEN 1 AND 120 AND display_name=btrim(display_name) AND display_name !~ '[[:cntrl:]]'),
 plan_code text NOT NULL CHECK(plan_code ~ '^[a-z0-9][a-z0-9_-]{0,31}$'),
 device_quota integer NOT NULL CHECK(device_quota BETWEEN 1 AND 1000000),
 subscriber_quota integer NOT NULL CHECK(subscriber_quota BETWEEN 1 AND 100000000),
 branding_name text CHECK(branding_name IS NULL OR (length(branding_name) BETWEEN 1 AND 80 AND branding_name=btrim(branding_name))),
 branding_primary_hex text CHECK(branding_primary_hex IS NULL OR branding_primary_hex ~ '^#[0-9A-Fa-f]{6}$'),
 revision bigint NOT NULL DEFAULT 1 CHECK(revision BETWEEN 1 AND 9223372036854775806),
 created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 updated_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
ALTER TABLE ipat_platform.tenant_profiles OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.tenant_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.tenant_profiles FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.tenant_profiles FROM PUBLIC,ipat_app_runtime;

CREATE TABLE ipat_platform.tenant_admin_invitations (
 id uuid PRIMARY KEY,
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id) ON DELETE CASCADE,
 invited_issuer text NOT NULL CHECK(length(invited_issuer) BETWEEN 10 AND 512 AND invited_issuer LIKE 'https://%' AND invited_issuer !~ '[[:space:]]'),
 invited_subject text NOT NULL CHECK(length(invited_subject) BETWEEN 1 AND 128 AND invited_subject ~ '^[a-zA-Z0-9_:/.-]+$'),
 state text NOT NULL DEFAULT 'PENDING' CHECK(state IN('PENDING','ACCEPTED','REVOKED','EXPIRED')),
 invited_by_issuer text NOT NULL,
 invited_by_subject text NOT NULL,
 created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 expires_at timestamptz NOT NULL,
 accepted_at timestamptz,
 revoked_at timestamptz,
 UNIQUE(tenant_id,invited_issuer,invited_subject,state),
 CHECK(expires_at>created_at AND expires_at<=created_at+interval '7 days'),
 CHECK((state='PENDING' AND accepted_at IS NULL AND revoked_at IS NULL)
    OR (state='ACCEPTED' AND accepted_at IS NOT NULL AND revoked_at IS NULL)
    OR (state='REVOKED' AND revoked_at IS NOT NULL AND accepted_at IS NULL)
    OR (state='EXPIRED' AND accepted_at IS NULL))
);
ALTER TABLE ipat_platform.tenant_admin_invitations OWNER TO ipat_schema_owner;
CREATE INDEX tenant_admin_invitations_target ON ipat_platform.tenant_admin_invitations(invited_issuer,invited_subject,state,expires_at);
ALTER TABLE ipat_platform.tenant_admin_invitations ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.tenant_admin_invitations FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.tenant_admin_invitations FROM PUBLIC,ipat_app_runtime;

CREATE TABLE ipat_platform.platform_tenant_events (
 id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 event text NOT NULL CHECK(event IN('TENANT_CREATED','PROFILE_UPDATED','TENANT_SUSPENDED','TENANT_RESUMED','TENANT_ADMIN_INVITED','TENANT_ADMIN_INVITE_REVOKED','TENANT_ADMIN_JOINED')),
 actor_issuer text NOT NULL,actor_subject text NOT NULL,
 correlation_id uuid NOT NULL,
 reason text CHECK(reason IS NULL OR (length(reason) BETWEEN 8 AND 240 AND reason=btrim(reason) AND reason !~ '[[:cntrl:]]')),
 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
ALTER TABLE ipat_platform.platform_tenant_events OWNER TO ipat_schema_owner;
CREATE INDEX platform_tenant_events_tenant_order ON ipat_platform.platform_tenant_events(tenant_id,id);
ALTER TABLE ipat_platform.platform_tenant_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.platform_tenant_events FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.platform_tenant_events FROM PUBLIC,ipat_app_runtime;

CREATE ROLE ipat_platform_lifecycle_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_platform_admin_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE ROLE ipat_tenant_invite_accept_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_platform_lifecycle_owner,ipat_platform_admin_exec,ipat_tenant_invite_accept_exec;
GRANT SELECT ON ipat_platform.platform_principals TO ipat_platform_lifecycle_owner;
GRANT SELECT,INSERT,UPDATE ON ipat_platform.tenants,ipat_platform.tenant_profiles,ipat_platform.tenant_admin_invitations TO ipat_platform_lifecycle_owner;
GRANT SELECT,INSERT ON ipat_platform.platform_tenant_events TO ipat_platform_lifecycle_owner;
GRANT SELECT,INSERT ON ipat_platform.identity_memberships TO ipat_platform_lifecycle_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_platform.platform_tenant_events_id_seq TO ipat_platform_lifecycle_owner;
CREATE POLICY platform_lifecycle_principal_read ON ipat_platform.platform_principals FOR SELECT TO ipat_platform_lifecycle_owner USING(true);
CREATE POLICY platform_lifecycle_profile_all ON ipat_platform.tenant_profiles FOR ALL TO ipat_platform_lifecycle_owner USING(true) WITH CHECK(true);
CREATE POLICY platform_lifecycle_invite_all ON ipat_platform.tenant_admin_invitations FOR ALL TO ipat_platform_lifecycle_owner USING(true) WITH CHECK(true);
CREATE POLICY platform_lifecycle_event_all ON ipat_platform.platform_tenant_events FOR ALL TO ipat_platform_lifecycle_owner USING(true) WITH CHECK(true);
CREATE POLICY platform_lifecycle_membership_read ON ipat_platform.identity_memberships FOR SELECT TO ipat_platform_lifecycle_owner USING(true);
CREATE POLICY platform_lifecycle_membership_insert ON ipat_platform.identity_memberships FOR INSERT TO ipat_platform_lifecycle_owner WITH CHECK(true);

CREATE FUNCTION ipat_platform.current_platform_owner(p_issuer text,p_subject text)
RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT EXISTS(SELECT 1 FROM ipat_platform.platform_principals p
   WHERE p.issuer=p_issuer AND p.subject=p_subject AND p.role='platform_owner'
     AND p.revoked_at IS NULL AND p.expires_at>clock_timestamp())
$body$;
ALTER FUNCTION ipat_platform.current_platform_owner(text,text) OWNER TO ipat_platform_lifecycle_owner;
REVOKE ALL ON FUNCTION ipat_platform.current_platform_owner(text,text) FROM PUBLIC;

CREATE FUNCTION ipat_platform.create_company_tenant(
 p_issuer text,p_subject text,p_tenant uuid,p_correlation uuid,p_slug text,p_display text,p_plan text,p_device_quota integer,p_subscriber_quota integer)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
BEGIN
 IF NOT ipat_platform.current_platform_owner(p_issuer,p_subject) OR p_tenant IS NULL OR p_correlation IS NULL
   OR p_slug !~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$' OR length(p_slug)>63
   OR p_display IS NULL OR length(p_display) NOT BETWEEN 1 AND 120 OR p_display<>btrim(p_display) OR p_display ~ '[[:cntrl:]]'
   OR p_plan !~ '^[a-z0-9][a-z0-9_-]{0,31}$' OR p_device_quota NOT BETWEEN 1 AND 1000000 OR p_subscriber_quota NOT BETWEEN 1 AND 100000000
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES(p_tenant,p_slug,'active');
 INSERT INTO ipat_platform.tenant_profiles(tenant_id,display_name,plan_code,device_quota,subscriber_quota)
 VALUES(p_tenant,p_display,p_plan,p_device_quota,p_subscriber_quota);
 INSERT INTO ipat_platform.platform_tenant_events(tenant_id,event,actor_issuer,actor_subject,correlation_id)
 VALUES(p_tenant,'TENANT_CREATED',p_issuer,p_subject,p_correlation);
 RETURN p_tenant;
EXCEPTION WHEN unique_violation OR check_violation OR foreign_key_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.create_company_tenant(text,text,uuid,uuid,text,text,text,integer,integer) OWNER TO ipat_platform_lifecycle_owner;
REVOKE ALL ON FUNCTION ipat_platform.create_company_tenant(text,text,uuid,uuid,text,text,text,integer,integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.create_company_tenant(text,text,uuid,uuid,text,text,text,integer,integer) TO ipat_platform_admin_exec;

CREATE FUNCTION ipat_platform.update_company_profile(
 p_issuer text,p_subject text,p_tenant uuid,p_correlation uuid,p_expected bigint,p_display text,p_plan text,p_device_quota integer,p_subscriber_quota integer,p_brand text,p_hex text)
RETURNS bigint LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE old bigint;
BEGIN
 IF NOT ipat_platform.current_platform_owner(p_issuer,p_subject) OR p_expected<1 OR p_correlation IS NULL
  OR p_display IS NULL OR length(p_display) NOT BETWEEN 1 AND 120 OR p_display<>btrim(p_display) OR p_display ~ '[[:cntrl:]]'
  OR p_plan !~ '^[a-z0-9][a-z0-9_-]{0,31}$' OR p_device_quota NOT BETWEEN 1 AND 1000000 OR p_subscriber_quota NOT BETWEEN 1 AND 100000000
  OR (p_brand IS NOT NULL AND (length(p_brand) NOT BETWEEN 1 AND 80 OR p_brand<>btrim(p_brand)))
  OR (p_hex IS NOT NULL AND p_hex !~ '^#[0-9A-Fa-f]{6}$') THEN RETURN NULL; END IF;
 SELECT revision INTO old FROM ipat_platform.tenant_profiles WHERE tenant_id=p_tenant FOR UPDATE;
 IF NOT FOUND OR old<>p_expected THEN RETURN NULL; END IF;
 UPDATE ipat_platform.tenant_profiles SET display_name=p_display,plan_code=p_plan,device_quota=p_device_quota,
  subscriber_quota=p_subscriber_quota,branding_name=p_brand,branding_primary_hex=p_hex,revision=old+1,updated_at=clock_timestamp()
 WHERE tenant_id=p_tenant;
 INSERT INTO ipat_platform.platform_tenant_events(tenant_id,event,actor_issuer,actor_subject,correlation_id)
 VALUES(p_tenant,'PROFILE_UPDATED',p_issuer,p_subject,p_correlation);
 RETURN old+1;
END $body$;
ALTER FUNCTION ipat_platform.update_company_profile(text,text,uuid,uuid,bigint,text,text,integer,integer,text,text) OWNER TO ipat_platform_lifecycle_owner;
REVOKE ALL ON FUNCTION ipat_platform.update_company_profile(text,text,uuid,uuid,bigint,text,text,integer,integer,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.update_company_profile(text,text,uuid,uuid,bigint,text,text,integer,integer,text,text) TO ipat_platform_admin_exec;

CREATE FUNCTION ipat_platform.set_company_tenant_state(
 p_issuer text,p_subject text,p_tenant uuid,p_correlation uuid,p_state text,p_reason text)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE changed integer; ev text;
BEGIN
 IF NOT ipat_platform.current_platform_owner(p_issuer,p_subject) OR p_correlation IS NULL OR p_state NOT IN('active','suspended')
   OR p_reason IS NULL OR length(p_reason) NOT BETWEEN 8 AND 240 OR p_reason<>btrim(p_reason) OR p_reason ~ '[[:cntrl:]]' THEN RETURN false; END IF;
 UPDATE ipat_platform.tenants SET state=p_state WHERE id=p_tenant AND state<>p_state; GET DIAGNOSTICS changed=ROW_COUNT;
 IF changed<>1 THEN RETURN false; END IF; ev=CASE WHEN p_state='suspended' THEN 'TENANT_SUSPENDED' ELSE 'TENANT_RESUMED' END;
 INSERT INTO ipat_platform.platform_tenant_events(tenant_id,event,actor_issuer,actor_subject,correlation_id,reason)
 VALUES(p_tenant,ev,p_issuer,p_subject,p_correlation,p_reason); RETURN true;
END $body$;
ALTER FUNCTION ipat_platform.set_company_tenant_state(text,text,uuid,uuid,text,text) OWNER TO ipat_platform_lifecycle_owner;
REVOKE ALL ON FUNCTION ipat_platform.set_company_tenant_state(text,text,uuid,uuid,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.set_company_tenant_state(text,text,uuid,uuid,text,text) TO ipat_platform_admin_exec;

CREATE FUNCTION ipat_platform.invite_company_tenant_admin(
 p_issuer text,p_subject text,p_tenant uuid,p_invite uuid,p_correlation uuid,p_target_issuer text,p_target_subject text,p_expires timestamptz)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
BEGIN
 IF NOT ipat_platform.current_platform_owner(p_issuer,p_subject) OR p_invite IS NULL OR p_correlation IS NULL
   OR p_target_issuer IS NULL OR p_target_subject IS NULL OR p_expires<=clock_timestamp() OR p_expires>clock_timestamp()+interval '7 days'
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.tenants WHERE id=p_tenant AND state='active')
   OR EXISTS(SELECT 1 FROM ipat_platform.identity_memberships WHERE tenant_id=p_tenant AND issuer=p_target_issuer AND subject=p_target_subject AND role='tenant_admin' AND revoked_at IS NULL AND expires_at>clock_timestamp())
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_platform.tenant_admin_invitations(id,tenant_id,invited_issuer,invited_subject,invited_by_issuer,invited_by_subject,expires_at)
 VALUES(p_invite,p_tenant,p_target_issuer,p_target_subject,p_issuer,p_subject,p_expires);
 INSERT INTO ipat_platform.platform_tenant_events(tenant_id,event,actor_issuer,actor_subject,correlation_id)
 VALUES(p_tenant,'TENANT_ADMIN_INVITED',p_issuer,p_subject,p_correlation); RETURN p_invite;
EXCEPTION WHEN unique_violation OR check_violation OR foreign_key_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.invite_company_tenant_admin(text,text,uuid,uuid,uuid,text,text,timestamptz) OWNER TO ipat_platform_lifecycle_owner;
REVOKE ALL ON FUNCTION ipat_platform.invite_company_tenant_admin(text,text,uuid,uuid,uuid,text,text,timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.invite_company_tenant_admin(text,text,uuid,uuid,uuid,text,text,timestamptz) TO ipat_platform_admin_exec;

CREATE FUNCTION ipat_platform.accept_company_tenant_admin_invitation(
 p_issuer text,p_subject text,p_invite uuid,p_correlation uuid)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE inv ipat_platform.tenant_admin_invitations%ROWTYPE;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_invite IS NULL OR p_correlation IS NULL THEN RETURN NULL; END IF;
 SELECT * INTO inv FROM ipat_platform.tenant_admin_invitations WHERE id=p_invite FOR UPDATE;
 IF NOT FOUND OR inv.state<>'PENDING' OR inv.expires_at<=clock_timestamp() OR inv.invited_issuer<>p_issuer OR inv.invited_subject<>p_subject
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.tenants WHERE id=inv.tenant_id AND state='active') THEN RETURN NULL; END IF;
 INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at)
 VALUES(inv.tenant_id,p_issuer,p_subject,'tenant_admin','platform_invite:'||p_invite::text,clock_timestamp()+interval '30 days')
 ON CONFLICT(tenant_id,issuer,subject,role) DO NOTHING;
 IF NOT FOUND THEN RETURN NULL; END IF;
 UPDATE ipat_platform.tenant_admin_invitations SET state='ACCEPTED',accepted_at=clock_timestamp() WHERE id=p_invite;
 INSERT INTO ipat_platform.platform_tenant_events(tenant_id,event,actor_issuer,actor_subject,correlation_id)
 VALUES(inv.tenant_id,'TENANT_ADMIN_JOINED',p_issuer,p_subject,p_correlation); RETURN inv.tenant_id;
END $body$;
ALTER FUNCTION ipat_platform.accept_company_tenant_admin_invitation(text,text,uuid,uuid) OWNER TO ipat_platform_lifecycle_owner;
REVOKE ALL ON FUNCTION ipat_platform.accept_company_tenant_admin_invitation(text,text,uuid,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.accept_company_tenant_admin_invitation(text,text,uuid,uuid) TO ipat_tenant_invite_accept_exec;

CREATE FUNCTION ipat_platform.list_company_tenants(p_issuer text,p_subject text)
RETURNS TABLE(tenant_id uuid,tenant_slug text,state text,display_name text,plan_code text,device_quota integer,subscriber_quota integer,branding_name text,branding_primary_hex text,revision bigint)
LANGUAGE sql STABLE SECURITY DEFINER SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT t.id,t.tenant_slug,t.state,p.display_name,p.plan_code,p.device_quota,p.subscriber_quota,p.branding_name,p.branding_primary_hex,p.revision
 FROM ipat_platform.tenants t JOIN ipat_platform.tenant_profiles p ON p.tenant_id=t.id
 WHERE ipat_platform.current_platform_owner(p_issuer,p_subject) ORDER BY t.tenant_slug LIMIT 1000
$body$;
ALTER FUNCTION ipat_platform.list_company_tenants(text,text) OWNER TO ipat_platform_lifecycle_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_company_tenants(text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_company_tenants(text,text) TO ipat_platform_admin_exec;

COMMIT;
