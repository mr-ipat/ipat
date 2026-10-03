-- R9.76 independent tenant POP master. Never infer historical POP from Site
-- code and NEVER silently broaden legacy NOC identity_pop_grants.
BEGIN;
CREATE TABLE ipat_ops.tenant_pops(
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 code text NOT NULL CHECK(code ~ '^[A-Za-z0-9_.-]{1,128}$'),
 display_name text NOT NULL CHECK(length(display_name) BETWEEN 1 AND 120
   AND display_name=btrim(display_name) AND display_name !~ '[[:cntrl:]]'),
 revision bigint NOT NULL DEFAULT 1 CHECK(revision BETWEEN 1 AND 9223372036854775806),
 created_by_issuer text NOT NULL, created_by_subject text NOT NULL,
 updated_by_issuer text NOT NULL,updated_by_subject text NOT NULL,
 created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 PRIMARY KEY(tenant_id,code)
);
ALTER TABLE ipat_ops.tenant_pops OWNER TO ipat_schema_owner;
ALTER TABLE ipat_ops.tenant_pops ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.tenant_pops FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.tenant_pops FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_login;
CREATE TABLE ipat_ops.tenant_pop_events(
 id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
 tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
 pop_code text NOT NULL,
 event text NOT NULL CHECK(event IN('POP_CREATED','POP_RENAMED','POP_DELETED','SITE_POP_ASSIGNED')),
 site_code text, old_pop_code text,new_pop_code text,
 revision bigint NOT NULL CHECK(revision>0),
 actor_issuer text NOT NULL,actor_subject text NOT NULL,
 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
ALTER TABLE ipat_ops.tenant_pop_events OWNER TO ipat_schema_owner;
CREATE INDEX tenant_pop_events_tenant_order ON ipat_ops.tenant_pop_events(tenant_id,id);
ALTER TABLE ipat_ops.tenant_pop_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.tenant_pop_events FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.tenant_pop_events FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_login;
-- Nullable, explicitly assigned independent relationship. No fake backfill.
ALTER TABLE ipat_ops.tenant_sites
 ADD COLUMN parent_pop_code text,
 ADD CONSTRAINT tenant_sites_parent_tenant_pop
 FOREIGN KEY(tenant_id,parent_pop_code)
 REFERENCES ipat_ops.tenant_pops(tenant_id,code) NOT VALID;
CREATE ROLE ipat_pop_registry_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_pop_registry_exec NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_pop_registry_owner,ipat_pop_registry_exec;
GRANT USAGE ON SCHEMA ipat_ops TO ipat_pop_registry_owner;
GRANT SELECT ON ipat_platform.tenants TO ipat_pop_registry_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
 text,text,uuid,text,text) TO ipat_pop_registry_owner;
GRANT SELECT,INSERT,UPDATE(display_name,revision,updated_by_issuer,updated_by_subject,updated_at),DELETE
 ON ipat_ops.tenant_pops TO ipat_pop_registry_owner;
GRANT SELECT,INSERT ON ipat_ops.tenant_pop_events TO ipat_pop_registry_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_ops.tenant_pop_events_id_seq TO ipat_pop_registry_owner;
GRANT SELECT,UPDATE(parent_pop_code,revision,updated_by_issuer,updated_by_subject,updated_at)
 ON ipat_ops.tenant_sites TO ipat_pop_registry_owner;
GRANT SELECT ON ipat_ops.managed_devices TO ipat_pop_registry_owner;
CREATE POLICY pop_owner_read ON ipat_ops.tenant_pops
 FOR SELECT TO ipat_pop_registry_owner USING(true);
CREATE POLICY pop_owner_insert ON ipat_ops.tenant_pops
 FOR INSERT TO ipat_pop_registry_owner WITH CHECK(true);
CREATE POLICY pop_owner_update ON ipat_ops.tenant_pops
 FOR UPDATE TO ipat_pop_registry_owner USING(true) WITH CHECK(true);
CREATE POLICY pop_owner_delete ON ipat_ops.tenant_pops
 FOR DELETE TO ipat_pop_registry_owner USING(true);
CREATE POLICY pop_event_owner_read ON ipat_ops.tenant_pop_events
 FOR SELECT TO ipat_pop_registry_owner USING(true);
CREATE POLICY pop_event_owner_insert ON ipat_ops.tenant_pop_events
 FOR INSERT TO ipat_pop_registry_owner WITH CHECK(true);
CREATE POLICY pop_registry_site_read ON ipat_ops.tenant_sites
 FOR SELECT TO ipat_pop_registry_owner USING(true);
CREATE POLICY pop_registry_site_update ON ipat_ops.tenant_sites
 FOR UPDATE TO ipat_pop_registry_owner USING(true) WITH CHECK(true);
CREATE POLICY pop_registry_device_read ON ipat_ops.managed_devices
 FOR SELECT TO ipat_pop_registry_owner USING(true);

CREATE FUNCTION ipat_platform.create_tenant_pop(
 p_issuer text,p_subject text,p_tenant uuid,p_code text,p_name text)
 RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
 OR p_code IS NULL OR p_name IS NULL
 OR p_code !~ '^[A-Za-z0-9_.-]{1,128}$'
 OR length(p_name) NOT BETWEEN 1 AND 120 OR p_name<>btrim(p_name)
 OR p_name ~ '[[:cntrl:]]'
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 THEN RETURN NULL; END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended(
  p_tenant::text||'/pop/'||p_code,0));
 IF EXISTS(SELECT 1 FROM ipat_ops.tenant_pops p
  WHERE p.tenant_id=p_tenant AND p.code=p_code)
 OR EXISTS(SELECT 1 FROM ipat_ops.tenant_pop_events e
  WHERE e.tenant_id=p_tenant AND e.pop_code=p_code
  AND e.event='POP_CREATED') THEN RETURN NULL; END IF;
 INSERT INTO ipat_ops.tenant_pops(
  tenant_id,code,display_name,created_by_issuer,created_by_subject,
  updated_by_issuer,updated_by_subject)
 VALUES(p_tenant,p_code,p_name,p_issuer,p_subject,p_issuer,p_subject);
 INSERT INTO ipat_ops.tenant_pop_events(
  tenant_id,pop_code,event,revision,actor_issuer,actor_subject)
 VALUES(p_tenant,p_code,'POP_CREATED',1,p_issuer,p_subject);
 RETURN p_code;
EXCEPTION WHEN unique_violation OR check_violation OR foreign_key_violation THEN
 RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.create_tenant_pop(text,text,uuid,text,text)
 OWNER TO ipat_pop_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.create_tenant_pop(text,text,uuid,text,text)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.create_tenant_pop(text,text,uuid,text,text)
 TO ipat_pop_registry_exec;

CREATE FUNCTION ipat_platform.list_tenant_pops(
 p_issuer text,p_subject text,p_tenant uuid,p_after text)
 RETURNS TABLE(code text,display_name text,revision bigint,assigned_sites bigint)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT p.code,p.display_name,p.revision,
  (SELECT count(*) FROM ipat_ops.tenant_sites s
   WHERE s.tenant_id=p.tenant_id AND s.parent_pop_code=p.code)::bigint
 FROM ipat_ops.tenant_pops p
 WHERE p.tenant_id=p_tenant AND (p_after IS NULL OR p.code>p_after)
 AND EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 ORDER BY p.code LIMIT 101
$body$;
ALTER FUNCTION ipat_platform.list_tenant_pops(text,text,uuid,text)
 OWNER TO ipat_pop_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_pops(text,text,uuid,text)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_pops(text,text,uuid,text)
 TO ipat_pop_registry_exec;

CREATE FUNCTION ipat_platform.rename_tenant_pop(
 p_issuer text,p_subject text,p_tenant uuid,p_code text,p_name text,
 p_expected bigint) RETURNS bigint
 LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE updated bigint;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR
 p_code IS NULL OR p_expected IS NULL OR p_expected<1 OR p_name IS NULL
 OR length(p_name) NOT BETWEEN 1 AND 120 OR p_name<>btrim(p_name)
 OR p_name ~ '[[:cntrl:]]'
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
 p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN NULL; END IF;
 UPDATE ipat_ops.tenant_pops SET display_name=p_name,revision=revision+1,
 updated_by_issuer=p_issuer,updated_by_subject=p_subject,
 updated_at=statement_timestamp()
 WHERE tenant_id=p_tenant AND code=p_code AND revision=p_expected
 RETURNING revision INTO updated;
 IF updated IS NULL THEN RETURN NULL; END IF;
 INSERT INTO ipat_ops.tenant_pop_events(
 tenant_id,pop_code,event,revision,actor_issuer,actor_subject)
 VALUES(p_tenant,p_code,'POP_RENAMED',updated,p_issuer,p_subject);
 RETURN updated;
END $body$;
ALTER FUNCTION ipat_platform.rename_tenant_pop(text,text,uuid,text,text,bigint)
 OWNER TO ipat_pop_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.rename_tenant_pop(text,text,uuid,text,text,bigint)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.rename_tenant_pop(text,text,uuid,text,text,bigint)
 TO ipat_pop_registry_exec;

CREATE FUNCTION ipat_platform.assign_tenant_site_to_pop(
 p_issuer text,p_subject text,p_tenant uuid,p_site text,p_pop text,
 p_expected bigint) RETURNS bigint
 LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE site ipat_ops.tenant_sites%ROWTYPE; updated bigint;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
 OR p_site IS NULL OR p_site !~ '^[A-Za-z0-9_.-]{1,128}$'
 OR p_expected IS NULL OR p_expected<1
 OR (p_pop IS NOT NULL AND p_pop !~ '^[A-Za-z0-9_.-]{1,128}$')
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
 p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN NULL; END IF;
 SELECT * INTO site FROM ipat_ops.tenant_sites
 WHERE tenant_id=p_tenant AND code=p_site FOR UPDATE;
 IF NOT FOUND OR site.revision<>p_expected THEN RETURN NULL; END IF;
 IF p_pop IS NOT NULL AND NOT EXISTS(SELECT 1 FROM ipat_ops.tenant_pops p
 WHERE p.tenant_id=p_tenant AND p.code=p_pop) THEN RETURN NULL; END IF;
 IF site.parent_pop_code IS NOT DISTINCT FROM p_pop THEN
 RETURN site.revision; END IF;
 UPDATE ipat_ops.tenant_sites SET parent_pop_code=p_pop,
 revision=revision+1,updated_by_issuer=p_issuer,
 updated_by_subject=p_subject,updated_at=statement_timestamp()
 WHERE tenant_id=p_tenant AND code=p_site AND revision=p_expected
 RETURNING revision INTO updated;
 IF updated IS NULL THEN RETURN NULL; END IF;
 INSERT INTO ipat_ops.tenant_pop_events(
 tenant_id,pop_code,event,site_code,old_pop_code,new_pop_code,
 revision,actor_issuer,actor_subject)
 VALUES(p_tenant,COALESCE(p_pop,site.parent_pop_code),'SITE_POP_ASSIGNED',
 p_site,site.parent_pop_code,p_pop,updated,p_issuer,p_subject);
 RETURN updated;
EXCEPTION WHEN foreign_key_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.assign_tenant_site_to_pop(
 text,text,uuid,text,text,bigint) OWNER TO ipat_pop_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.assign_tenant_site_to_pop(
 text,text,uuid,text,text,bigint) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.assign_tenant_site_to_pop(
 text,text,uuid,text,text,bigint) TO ipat_pop_registry_exec;

CREATE FUNCTION ipat_platform.delete_unused_tenant_pop(
 p_issuer text,p_subject text,p_tenant uuid,p_code text,p_expected bigint)
 RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE deleted bigint;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
 OR p_code IS NULL OR p_code !~ '^[A-Za-z0-9_.-]{1,128}$'
 OR p_expected IS NULL OR p_expected<1
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
 p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) THEN RETURN false; END IF;
 DELETE FROM ipat_ops.tenant_pops p WHERE p.tenant_id=p_tenant
 AND p.code=p_code AND p.revision=p_expected AND NOT EXISTS(
  SELECT 1 FROM ipat_ops.tenant_sites s WHERE s.tenant_id=p.tenant_id
  AND s.parent_pop_code=p.code)
 RETURNING p.revision INTO deleted;
 IF deleted IS NULL THEN RETURN false; END IF;
 INSERT INTO ipat_ops.tenant_pop_events(
 tenant_id,pop_code,event,revision,actor_issuer,actor_subject)
 VALUES(p_tenant,p_code,'POP_DELETED',deleted,p_issuer,p_subject);
 RETURN true;
EXCEPTION WHEN foreign_key_violation THEN RETURN false;
END $body$;
ALTER FUNCTION ipat_platform.delete_unused_tenant_pop(
 text,text,uuid,text,bigint) OWNER TO ipat_pop_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.delete_unused_tenant_pop(
 text,text,uuid,text,bigint) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.delete_unused_tenant_pop(
 text,text,uuid,text,bigint) TO ipat_pop_registry_exec;

-- Admin-only parallel list exposes association without broadening legacy NOC
-- whose historical POP grant currently maps to an exact Site code.
CREATE FUNCTION ipat_platform.list_tenant_sites_with_parent_pop(
 p_issuer text,p_subject text,p_tenant uuid,p_site_filter text,p_after text)
 RETURNS TABLE(code text,display_name text,revision bigint,
  assigned_devices bigint,parent_pop_code text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT s.code,s.display_name,s.revision,
 (SELECT count(*) FROM ipat_ops.managed_devices d
 WHERE d.tenant_id=s.tenant_id AND d.pop_id=s.code)::bigint,
 s.parent_pop_code
 FROM ipat_ops.tenant_sites s
 WHERE s.tenant_id=p_tenant AND (p_site_filter IS NULL OR s.code=p_site_filter)
 AND (p_after IS NULL OR s.code>p_after)
 AND EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 ORDER BY s.code LIMIT 101
$body$;
ALTER FUNCTION ipat_platform.list_tenant_sites_with_parent_pop(
 text,text,uuid,text,text) OWNER TO ipat_pop_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_sites_with_parent_pop(
 text,text,uuid,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_sites_with_parent_pop(
 text,text,uuid,text,text) TO ipat_pop_registry_exec;

-- Atomic Site creation with explicit existing POP. No Site appears if
-- the chosen parent is missing, wrong-tenant or concurrently deleted.
GRANT EXECUTE ON FUNCTION ipat_platform.create_tenant_site(
 text,text,uuid,text,text) TO ipat_pop_registry_owner;
CREATE FUNCTION ipat_platform.create_tenant_site_with_pop(
 p_issuer text,p_subject text,p_tenant uuid,p_site text,p_name text,p_pop text)
 RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE created text;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
 OR (p_pop IS NOT NULL AND (p_pop !~ '^[A-Za-z0-9_.-]{1,128}$'
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.tenant_pops
 WHERE tenant_id=p_tenant AND code=p_pop)))
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
 p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 THEN RETURN NULL; END IF;
 SELECT ipat_platform.create_tenant_site(
 p_issuer,p_subject,p_tenant,p_site,p_name) INTO created;
 IF created IS NULL THEN RETURN NULL; END IF;
 IF p_pop IS NOT NULL THEN
 UPDATE ipat_ops.tenant_sites SET parent_pop_code=p_pop
 WHERE tenant_id=p_tenant AND code=created;
 INSERT INTO ipat_ops.tenant_pop_events(
 tenant_id,pop_code,event,site_code,new_pop_code,revision,
 actor_issuer,actor_subject)
 VALUES(p_tenant,p_pop,'SITE_POP_ASSIGNED',created,p_pop,1,p_issuer,p_subject);
 END IF;
 RETURN created;
EXCEPTION WHEN foreign_key_violation THEN RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.create_tenant_site_with_pop(
 text,text,uuid,text,text,text) OWNER TO ipat_pop_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.create_tenant_site_with_pop(
 text,text,uuid,text,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.create_tenant_site_with_pop(
 text,text,uuid,text,text,text) TO ipat_pop_registry_exec;

GRANT ipat_pop_registry_exec TO ipat_tenant_api_exec;
COMMIT;
