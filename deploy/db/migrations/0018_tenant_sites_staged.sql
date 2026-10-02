-- R9.64 commercial-path tenant Site master. Two-phase migration:
-- 0018 stages the master and immediately constrains NEW managed devices.
-- Existing managed_devices rows are NEVER silently assigned to an invented Site.
-- 0019 validates all legacy references only after independently confirmed
-- tenant_admin creates/imports their correct Site records.
-- No user password, device command, public route, or production DB install.
BEGIN;
CREATE TABLE ipat_ops.tenant_sites (
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 code text NOT NULL CHECK(code ~ '^[A-Za-z0-9_.-]{1,128}$'),
 display_name text NOT NULL CHECK(length(display_name) BETWEEN 1 AND 120
   AND display_name=btrim(display_name) AND display_name !~ '[[:cntrl:]]'),
 revision bigint NOT NULL DEFAULT 1 CHECK(revision BETWEEN 1 AND 9223372036854775806),
 created_by_issuer text NOT NULL, created_by_subject text NOT NULL,
 updated_by_issuer text NOT NULL, updated_by_subject text NOT NULL,
 created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,code)
);
ALTER TABLE ipat_ops.tenant_sites OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.tenant_sites ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.tenant_sites FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.tenant_sites FROM PUBLIC, ipat_app_runtime;

-- Audits persist after a permitted unused Site deletion. Do not FK the audit
-- to a deletable Site or require a privileged caller to truncate its history.
CREATE TABLE ipat_ops.tenant_site_events (
 id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 site_code text NOT NULL,
 event text NOT NULL CHECK(event IN('SITE_CREATED','SITE_RENAMED','SITE_DELETED')),
 revision bigint NOT NULL CHECK(revision>0),
 actor_issuer text NOT NULL, actor_subject text NOT NULL,
 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
ALTER TABLE ipat_ops.tenant_site_events OWNER TO ipat_schema_owner;
CREATE INDEX tenant_site_events_tenant_order ON ipat_ops.tenant_site_events(tenant_id,id);
ALTER TABLE ipat_ops.tenant_site_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.tenant_site_events FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.tenant_site_events FROM PUBLIC,ipat_app_runtime;

CREATE ROLE ipat_site_registry_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_site_registry_exec NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_site_registry_owner,ipat_site_registry_exec;
GRANT USAGE ON SCHEMA ipat_ops TO ipat_site_registry_owner;
GRANT SELECT ON ipat_platform.tenants TO ipat_site_registry_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(text,text,uuid,text,text)
 TO ipat_site_registry_owner;
GRANT SELECT,INSERT,UPDATE(display_name,revision,updated_by_issuer,updated_by_subject,updated_at),DELETE
 ON ipat_ops.tenant_sites TO ipat_site_registry_owner;
GRANT SELECT,INSERT ON ipat_ops.tenant_site_events TO ipat_site_registry_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_ops.tenant_site_events_id_seq TO ipat_site_registry_owner;
GRANT SELECT ON ipat_ops.managed_devices TO ipat_site_registry_owner;
CREATE POLICY site_owner_read ON ipat_ops.tenant_sites
 FOR SELECT TO ipat_site_registry_owner USING(true);
CREATE POLICY site_owner_insert ON ipat_ops.tenant_sites
 FOR INSERT TO ipat_site_registry_owner WITH CHECK(true);
CREATE POLICY site_owner_update ON ipat_ops.tenant_sites
 FOR UPDATE TO ipat_site_registry_owner USING(true) WITH CHECK(true);
CREATE POLICY site_owner_delete ON ipat_ops.tenant_sites
 FOR DELETE TO ipat_site_registry_owner USING(true);
CREATE POLICY site_event_owner_read ON ipat_ops.tenant_site_events
 FOR SELECT TO ipat_site_registry_owner USING(true);
CREATE POLICY site_event_owner_insert ON ipat_ops.tenant_site_events
 FOR INSERT TO ipat_site_registry_owner WITH CHECK(true);
CREATE POLICY site_owner_managed_count ON ipat_ops.managed_devices
 FOR SELECT TO ipat_site_registry_owner USING(true);

-- The existing R9.57 registry owner never gains Site mutation rights; its
-- present registration routine is constrained by this composite foreign key.
ALTER TABLE ipat_ops.managed_devices
 ADD CONSTRAINT managed_devices_registered_site_fk
 FOREIGN KEY(tenant_id,pop_id) REFERENCES ipat_ops.tenant_sites(tenant_id,code)
 NOT VALID;

-- No user header/Host is read inside SQL. Caller must pass a trusted signed
-- issuer/subject/tenant verified against the request's trusted Host by BFF.
-- Every operation ALSO checks a current active DB membership at execution.
CREATE FUNCTION ipat_platform.create_tenant_site(
 p_issuer text,p_subject text,p_tenant uuid,p_code text,p_name text)
 RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_code IS NULL
   OR p_name IS NULL OR p_code !~ '^[A-Za-z0-9_.-]{1,128}$'
   OR length(p_name) NOT BETWEEN 1 AND 120 OR p_name<>btrim(p_name)
   OR p_name ~ '[[:cntrl:]]'
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN NULL;
 END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended(p_tenant::text||'/site/'||p_code,0));
 IF EXISTS(SELECT 1 FROM ipat_ops.tenant_sites WHERE tenant_id=p_tenant AND code=p_code)
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_ops.tenant_sites(tenant_id,code,display_name,created_by_issuer,
   created_by_subject,updated_by_issuer,updated_by_subject)
 VALUES(p_tenant,p_code,p_name,p_issuer,p_subject,p_issuer,p_subject);
 INSERT INTO ipat_ops.tenant_site_events(tenant_id,site_code,event,revision,actor_issuer,actor_subject)
 VALUES(p_tenant,p_code,'SITE_CREATED',1,p_issuer,p_subject);
 RETURN p_code;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.create_tenant_site(text,text,uuid,text,text)
 OWNER TO ipat_site_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.create_tenant_site(text,text,uuid,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.create_tenant_site(text,text,uuid,text,text)
 TO ipat_site_registry_exec;

CREATE FUNCTION ipat_platform.rename_tenant_site(
 p_issuer text,p_subject text,p_tenant uuid,p_code text,p_name text,p_expected bigint)
 RETURNS bigint LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE old ipat_ops.tenant_sites%ROWTYPE;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_code IS NULL
   OR p_name IS NULL OR p_expected IS NULL OR p_expected<1
   OR length(p_name) NOT BETWEEN 1 AND 120 OR p_name<>btrim(p_name)
   OR p_name ~ '[[:cntrl:]]'
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN NULL; END IF;
 SELECT * INTO old FROM ipat_ops.tenant_sites
 WHERE tenant_id=p_tenant AND code=p_code FOR UPDATE;
 IF NOT FOUND OR old.revision<>p_expected OR old.revision>=9223372036854775806
 THEN RETURN NULL; END IF;
 UPDATE ipat_ops.tenant_sites SET display_name=p_name,revision=old.revision+1,
   updated_by_issuer=p_issuer,updated_by_subject=p_subject,
   updated_at=statement_timestamp() WHERE tenant_id=p_tenant AND code=p_code;
 INSERT INTO ipat_ops.tenant_site_events(tenant_id,site_code,event,revision,actor_issuer,actor_subject)
 VALUES(p_tenant,p_code,'SITE_RENAMED',old.revision+1,p_issuer,p_subject);
 RETURN old.revision+1;
END $body$;
ALTER FUNCTION ipat_platform.rename_tenant_site(text,text,uuid,text,text,bigint)
 OWNER TO ipat_site_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.rename_tenant_site(text,text,uuid,text,text,bigint) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.rename_tenant_site(text,text,uuid,text,text,bigint)
 TO ipat_site_registry_exec;

CREATE FUNCTION ipat_platform.delete_tenant_site(
 p_issuer text,p_subject text,p_tenant uuid,p_code text,p_expected bigint)
 RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE old ipat_ops.tenant_sites%ROWTYPE;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_code IS NULL
   OR p_expected IS NULL OR p_expected<1
   OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN false; END IF;
 SELECT * INTO old FROM ipat_ops.tenant_sites
 WHERE tenant_id=p_tenant AND code=p_code FOR UPDATE;
 IF NOT FOUND OR old.revision<>p_expected
   OR EXISTS(SELECT 1 FROM ipat_ops.managed_devices
       WHERE tenant_id=p_tenant AND pop_id=p_code)
 THEN RETURN false; END IF;
 DELETE FROM ipat_ops.tenant_sites WHERE tenant_id=p_tenant AND code=p_code;
 INSERT INTO ipat_ops.tenant_site_events(tenant_id,site_code,event,revision,actor_issuer,actor_subject)
 VALUES(p_tenant,p_code,'SITE_DELETED',old.revision,p_issuer,p_subject);
 RETURN true;
EXCEPTION WHEN foreign_key_violation THEN RETURN false;
END $body$;
ALTER FUNCTION ipat_platform.delete_tenant_site(text,text,uuid,text,bigint)
 OWNER TO ipat_site_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.delete_tenant_site(text,text,uuid,text,bigint) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.delete_tenant_site(text,text,uuid,text,bigint)
 TO ipat_site_registry_exec;

-- A NOC reader must supply their currently granted exact POP. Admin may
-- list all own Sites with cursor pagination. No cross-tenant counts leak.
CREATE FUNCTION ipat_platform.list_tenant_sites(
 p_issuer text,p_subject text,p_tenant uuid,p_pop text,p_after text)
 RETURNS TABLE(code text,display_name text,revision bigint,assigned_devices bigint)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT s.code,s.display_name,s.revision,
    (SELECT count(*) FROM ipat_ops.managed_devices d
     WHERE d.tenant_id=s.tenant_id AND d.pop_id=s.code)::bigint
 FROM ipat_ops.tenant_sites s
 WHERE s.tenant_id=p_tenant AND (p_pop IS NULL OR s.code=p_pop)
   AND (p_after IS NULL OR s.code>p_after)
   AND (EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
         p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
      OR (p_pop IS NOT NULL AND EXISTS(
         SELECT 1 FROM ipat_platform.lookup_active_membership(
           p_issuer,p_subject,p_tenant,'noc_engineer',p_pop))))
 ORDER BY s.code LIMIT 101
$body$;
ALTER FUNCTION ipat_platform.list_tenant_sites(text,text,uuid,text,text)
 OWNER TO ipat_site_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_sites(text,text,uuid,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_sites(text,text,uuid,text,text)
 TO ipat_site_registry_exec;

-- All new managed records must reference an existing tenant-matched Site
-- even while legacy rows are awaiting separately reviewed 0019 validation.
-- Existing orphan rows are visible ONLY in a privileged preflight, never auto-fixed.
COMMIT;
