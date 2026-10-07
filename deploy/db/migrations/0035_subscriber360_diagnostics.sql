-- R10.05: tenant-isolated Subscriber 360 + normalized diagnostic observations.
-- Operator-entered topology is DECLARED, never VERIFIED automatically.
-- No credentials, subscriber secrets or automatic remediation are stored.
BEGIN;

-- Reuse the canonical subscriber table from migration 0001. Do NOT create a
-- competing subscriber master. Historical rows remain valid until explicitly
-- reconciled by the correct tenant.
ALTER TABLE ipat_ops.subscribers
  ADD COLUMN display_name text,
  ADD COLUMN pppoe_username text,
  ADD COLUMN site_code text,
  ADD COLUMN distribution_device_id uuid,
  ADD COLUMN access_device_id uuid,
  ADD COLUMN ont_reference text,
  ADD COLUMN topology_state text NOT NULL DEFAULT 'unknown'
    CHECK(topology_state IN('unknown','declared','verified')),
  ADD COLUMN topology_source_id text,
  ADD COLUMN topology_declared_at timestamptz,
  ADD COLUMN topology_verified_at timestamptz,
  ADD COLUMN topology_evidence_sha256 text,
  ADD COLUMN subscriber360_profile boolean NOT NULL DEFAULT false,
  ADD COLUMN revision bigint NOT NULL DEFAULT 1,
  ADD COLUMN created_by_issuer text,
  ADD COLUMN created_by_subject text,
  ADD COLUMN updated_by_issuer text,
  ADD COLUMN updated_by_subject text,
  ADD COLUMN updated_at timestamptz;
ALTER TABLE ipat_ops.subscribers
  ADD CONSTRAINT subscriber360_shape CHECK(
    NOT subscriber360_profile OR (
      display_name IS NOT NULL
      AND length(display_name) BETWEEN 1 AND 160
      AND display_name=btrim(display_name) AND display_name !~ '[[:cntrl:]]'
      AND site_code IS NOT NULL
      AND distribution_device_id IS NOT NULL
      AND created_by_issuer IS NOT NULL AND created_by_subject IS NOT NULL
      AND updated_by_issuer IS NOT NULL AND updated_by_subject IS NOT NULL
      AND updated_at IS NOT NULL
      AND topology_declared_at IS NOT NULL
      AND revision BETWEEN 1 AND 9223372036854775806
      AND (pppoe_username IS NULL OR (
        length(pppoe_username) BETWEEN 1 AND 128
        AND pppoe_username=btrim(pppoe_username)
        AND pppoe_username !~ '[[:cntrl:][:space:]]'))
      AND (ont_reference IS NULL OR (
        length(ont_reference) BETWEEN 1 AND 128
        AND ont_reference ~ '^[A-Za-z0-9_.:/-]+$'))
      AND (
        (topology_state='declared' AND topology_source_id IS NULL
          AND topology_verified_at IS NULL AND topology_evidence_sha256 IS NULL)
        OR
        (topology_state='verified' AND topology_source_id IS NOT NULL
          AND topology_source_id ~ '^[A-Za-z0-9_.:-]{1,128}$'
          AND topology_verified_at IS NOT NULL
          AND topology_verified_at>=topology_declared_at
          AND topology_evidence_sha256 ~ '^[0-9a-f]{64}$')
      )
    ));
ALTER TABLE ipat_ops.subscribers
  ADD CONSTRAINT subscriber360_site_fk
    FOREIGN KEY(tenant_id,site_code) REFERENCES ipat_ops.tenant_sites(tenant_id,code),
  ADD CONSTRAINT subscriber360_distribution_device_fk
    FOREIGN KEY(tenant_id,distribution_device_id)
    REFERENCES ipat_ops.managed_devices(tenant_id,id),
  ADD CONSTRAINT subscriber360_access_device_fk
    FOREIGN KEY(tenant_id,access_device_id)
    REFERENCES ipat_ops.managed_devices(tenant_id,id);
CREATE UNIQUE INDEX subscribers_tenant_pppoe_unique
  ON ipat_ops.subscribers(tenant_id,pppoe_username)
  WHERE subscriber360_profile AND pppoe_username IS NOT NULL;
CREATE INDEX subscribers_tenant_distribution
  ON ipat_ops.subscribers(tenant_id,distribution_device_id,customer_ref)
  WHERE subscriber360_profile;
-- Retire migration-0001 broad application-table access for the canonical
-- subscriber master. Commercial access is function-bound from this point.
REVOKE ALL ON ipat_ops.subscribers FROM ipat_app_runtime,ipat_tenant_api_login;

CREATE TABLE ipat_ops.diagnostic_observations(
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id),
  distribution_device_id uuid NOT NULL,
  subscriber_id text,
  signal text NOT NULL CHECK(signal IN(
    'distribution_uplink_down','distribution_uplink_healthy',
    'subscriber_unreachable','ont_optical_los','neighbor_ont_healthy',
    'ont_optical_normal','pppoe_authentication_rejected','cwmp_inform_missing')),
  source_id text NOT NULL CHECK(source_id ~ '^[A-Za-z0-9_.:-]{1,128}$'),
  observed_at timestamptz NOT NULL,
  evidence_sha256 text NOT NULL CHECK(evidence_sha256 ~ '^[0-9a-f]{64}$'),
  recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),
  FOREIGN KEY(tenant_id,distribution_device_id)
    REFERENCES ipat_ops.managed_devices(tenant_id,id),
  FOREIGN KEY(tenant_id,subscriber_id)
    REFERENCES ipat_ops.subscribers(tenant_id,customer_ref)
);
ALTER TABLE ipat_ops.diagnostic_observations OWNER TO ipat_schema_owner;
CREATE INDEX diagnostic_observations_path_time
  ON ipat_ops.diagnostic_observations(
    tenant_id,distribution_device_id,observed_at DESC,id DESC);
ALTER TABLE ipat_ops.diagnostic_observations ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_ops.diagnostic_observations FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_ops.diagnostic_observations
 FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_login;

CREATE ROLE ipat_subscriber_diag_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_diag_ingest_exec NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
CREATE ROLE ipat_topology_verify_exec NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_subscriber_diag_owner,
 ipat_diag_ingest_exec,ipat_topology_verify_exec;
GRANT USAGE ON SCHEMA ipat_ops TO ipat_subscriber_diag_owner;
GRANT SELECT ON ipat_platform.tenants TO ipat_subscriber_diag_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
 text,text,uuid,text,text) TO ipat_subscriber_diag_owner;
GRANT EXECUTE ON FUNCTION ipat_platform.current_noc_real_pop_scopes(
 text,text,uuid) TO ipat_subscriber_diag_owner;
GRANT SELECT,INSERT,UPDATE ON ipat_ops.subscribers TO ipat_subscriber_diag_owner;
GRANT SELECT,INSERT ON ipat_ops.diagnostic_observations TO ipat_subscriber_diag_owner;
GRANT USAGE,SELECT ON SEQUENCE ipat_ops.diagnostic_observations_id_seq
 TO ipat_subscriber_diag_owner;
GRANT SELECT ON ipat_ops.tenant_pops,ipat_ops.tenant_sites,ipat_ops.managed_devices
 TO ipat_subscriber_diag_owner;

CREATE POLICY subscriber_diag_subscribers_all
 ON ipat_ops.subscribers FOR ALL TO ipat_subscriber_diag_owner
 USING(true) WITH CHECK(true);
CREATE POLICY subscriber_diag_observations_all
 ON ipat_ops.diagnostic_observations FOR ALL TO ipat_subscriber_diag_owner
 USING(true) WITH CHECK(true);
CREATE POLICY subscriber_diag_pop_read ON ipat_ops.tenant_pops
 FOR SELECT TO ipat_subscriber_diag_owner USING(true);
CREATE POLICY subscriber_diag_site_read ON ipat_ops.tenant_sites
 FOR SELECT TO ipat_subscriber_diag_owner USING(true);
CREATE POLICY subscriber_diag_device_read ON ipat_ops.managed_devices
 FOR SELECT TO ipat_subscriber_diag_owner USING(true);

CREATE FUNCTION ipat_platform.upsert_subscriber360(
 p_issuer text,p_subject text,p_tenant uuid,p_subscriber text,p_name text,
 p_pppoe text,p_pop text,p_site text,p_distribution uuid,p_access uuid,p_ont text,
 p_expected_revision bigint)
 RETURNS bigint LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE
 current_revision bigint;
 current_pop text;
 current_site text;
 current_distribution uuid;
 current_access uuid;
 current_ont text;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
 OR p_subscriber IS NULL OR p_subscriber !~ '^[A-Za-z0-9_.-]{1,128}$'
 OR p_name IS NULL OR length(p_name) NOT BETWEEN 1 AND 160
 OR p_name<>btrim(p_name) OR p_name ~ '[[:cntrl:]]'
 OR p_pop IS NULL OR p_site IS NULL OR p_distribution IS NULL
 OR p_expected_revision IS NULL OR p_expected_revision<0
 OR p_pppoe IS NOT NULL AND (
    length(p_pppoe) NOT BETWEEN 1 AND 128 OR p_pppoe<>btrim(p_pppoe)
    OR p_pppoe ~ '[[:cntrl:][:space:]]')
 OR p_ont IS NOT NULL AND (
    length(p_ont) NOT BETWEEN 1 AND 128 OR p_ont !~ '^[A-Za-z0-9_.:/-]+$')
 OR NOT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
    p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.tenant_pops p
    WHERE p.tenant_id=p_tenant AND p.code=p_pop)
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.tenant_sites s
    WHERE s.tenant_id=p_tenant AND s.code=p_site AND s.parent_pop_code=p_pop)
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.managed_devices d
    WHERE d.tenant_id=p_tenant AND d.id=p_distribution
      AND d.device_kind='router' AND d.lifecycle_state='SAVED'
      AND d.archived_at IS NULL AND d.pop_id=p_site)
 OR (p_access IS NOT NULL AND NOT EXISTS(
    SELECT 1 FROM ipat_ops.managed_devices d
    WHERE d.tenant_id=p_tenant AND d.id=p_access
      AND d.device_kind IN('olt','ont') AND d.lifecycle_state='SAVED'
      AND d.archived_at IS NULL AND d.pop_id=p_site))
 THEN RETURN NULL; END IF;

 PERFORM pg_advisory_xact_lock(hashtextextended(
   p_tenant::text||'/subscriber/'||p_subscriber,0));
 SELECT revision,pop_id,site_code,distribution_device_id,access_device_id,ont_reference
 INTO current_revision,current_pop,current_site,current_distribution,current_access,current_ont
 FROM ipat_ops.subscribers
 WHERE tenant_id=p_tenant AND customer_ref=p_subscriber
 AND subscriber360_profile FOR UPDATE;

 IF NOT FOUND THEN
   IF p_expected_revision<>0 THEN RETURN NULL; END IF;
   INSERT INTO ipat_ops.subscribers(
    tenant_id,id,pop_id,customer_ref,device_id,display_name,pppoe_username,
    site_code,distribution_device_id,access_device_id,ont_reference,
    topology_state,topology_declared_at,subscriber360_profile,created_by_issuer,created_by_subject,
    updated_by_issuer,updated_by_subject,updated_at)
   VALUES(p_tenant,gen_random_uuid(),p_pop,p_subscriber,NULL,p_name,p_pppoe,
    p_site,p_distribution,p_access,p_ont,'declared',statement_timestamp(),true,p_issuer,p_subject,
    p_issuer,p_subject,statement_timestamp());
   RETURN 1;
 END IF;
 IF current_revision<>p_expected_revision THEN RETURN NULL; END IF;
 UPDATE ipat_ops.subscribers SET
   display_name=p_name,pppoe_username=p_pppoe,pop_id=p_pop,site_code=p_site,
   distribution_device_id=p_distribution,access_device_id=p_access,
   ont_reference=p_ont,subscriber360_profile=true,revision=revision+1,
   topology_state=CASE WHEN current_pop=p_pop AND current_site=p_site
     AND current_distribution=p_distribution
     AND current_access IS NOT DISTINCT FROM p_access
     AND current_ont IS NOT DISTINCT FROM p_ont
     THEN topology_state ELSE 'declared' END,
   topology_source_id=CASE WHEN current_pop=p_pop AND current_site=p_site
     AND current_distribution=p_distribution
     AND current_access IS NOT DISTINCT FROM p_access
     AND current_ont IS NOT DISTINCT FROM p_ont
     THEN topology_source_id ELSE NULL END,
   topology_verified_at=CASE WHEN current_pop=p_pop AND current_site=p_site
     AND current_distribution=p_distribution
     AND current_access IS NOT DISTINCT FROM p_access
     AND current_ont IS NOT DISTINCT FROM p_ont
     THEN topology_verified_at ELSE NULL END,
   topology_evidence_sha256=CASE WHEN current_pop=p_pop AND current_site=p_site
     AND current_distribution=p_distribution
     AND current_access IS NOT DISTINCT FROM p_access
     AND current_ont IS NOT DISTINCT FROM p_ont
     THEN topology_evidence_sha256 ELSE NULL END,
   topology_declared_at=CASE WHEN current_pop=p_pop AND current_site=p_site
     AND current_distribution=p_distribution
     AND current_access IS NOT DISTINCT FROM p_access
     AND current_ont IS NOT DISTINCT FROM p_ont
     THEN topology_declared_at ELSE statement_timestamp() END,
   updated_by_issuer=p_issuer,updated_by_subject=p_subject,
   updated_at=statement_timestamp()
 WHERE tenant_id=p_tenant AND customer_ref=p_subscriber;
 RETURN current_revision+1;
EXCEPTION WHEN unique_violation OR foreign_key_violation OR check_violation THEN
 RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.upsert_subscriber360(
 text,text,uuid,text,text,text,text,text,uuid,uuid,text,bigint)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.upsert_subscriber360(
 text,text,uuid,text,text,text,text,text,uuid,uuid,text,bigint) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.upsert_subscriber360(
 text,text,uuid,text,text,text,text,text,uuid,uuid,text,bigint)
 TO ipat_tenant_api_exec;


-- Single reusable scope decision. Browser/API callers cannot supply a role;
-- current database memberships and exact grants are re-evaluated every call.
CREATE FUNCTION ipat_platform.can_read_subscriber_scope(
 p_issuer text,p_subject text,p_tenant uuid,p_pop text,p_site text)
 RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT p_issuer IS NOT NULL AND p_subject IS NOT NULL
 AND p_tenant IS NOT NULL AND p_site IS NOT NULL
 AND (
   EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
   OR EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'helpdesk',p_site))
   OR EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'auditor',p_site))
   OR EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'noc_engineer',p_site))
   OR (p_pop IS NOT NULL AND EXISTS(
     SELECT 1 FROM ipat_platform.current_noc_real_pop_scopes(
       p_issuer,p_subject,p_tenant) n WHERE n.pop_code=p_pop))
 );
$body$;
ALTER FUNCTION ipat_platform.can_read_subscriber_scope(text,text,uuid,text,text)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.can_read_subscriber_scope(
 text,text,uuid,text,text) FROM PUBLIC,ipat_app_runtime,ipat_tenant_api_login;

CREATE FUNCTION ipat_platform.subscriber_diagnostic_ui_capabilities(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(can_read boolean,can_manage boolean)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 WITH admin AS (
   SELECT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
     p_issuer,p_subject,p_tenant,'tenant_admin',NULL)) AS allowed
 )
 SELECT
   admin.allowed OR EXISTS(
     SELECT 1 FROM ipat_ops.tenant_sites s
     WHERE s.tenant_id=p_tenant
       AND ipat_platform.can_read_subscriber_scope(
         p_issuer,p_subject,p_tenant,s.parent_pop_code,s.code)
   ),
   admin.allowed
 FROM admin;
$body$;
ALTER FUNCTION ipat_platform.subscriber_diagnostic_ui_capabilities(
 text,text,uuid) OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.subscriber_diagnostic_ui_capabilities(
 text,text,uuid) FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.subscriber_diagnostic_ui_capabilities(
 text,text,uuid) TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.list_subscriber360(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(
  subscriber_id text,display_name text,pppoe_username text,pop_code text,
  site_code text,distribution_device_id uuid,access_device_id uuid,
  ont_reference text,revision bigint,topology_state text,
  topology_source_id text,topology_verified_epoch bigint)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT s.customer_ref,s.display_name,s.pppoe_username,s.pop_id,s.site_code,
   s.distribution_device_id,s.access_device_id,s.ont_reference,s.revision,
   s.topology_state,s.topology_source_id,
   CASE WHEN s.topology_verified_at IS NULL THEN NULL
        ELSE extract(epoch from s.topology_verified_at)::bigint END
 FROM ipat_ops.subscribers s
 WHERE s.tenant_id=p_tenant AND s.subscriber360_profile
 AND ipat_platform.can_read_subscriber_scope(
   p_issuer,p_subject,p_tenant,s.pop_id,s.site_code)
 ORDER BY s.customer_ref LIMIT 500
$body$;
ALTER FUNCTION ipat_platform.list_subscriber360(text,text,uuid)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_subscriber360(text,text,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_subscriber360(text,text,uuid)
 TO ipat_tenant_api_exec;


-- Only a separately deployed trusted topology evidence worker may promote
-- DECLARED operator topology to VERIFIED. Tenant API/OIDC roles have no grant.
CREATE FUNCTION ipat_platform.record_subscriber_topology_verification(
 p_tenant uuid,p_subscriber text,p_pop text,p_site text,p_distribution uuid,
 p_access uuid,p_ont text,p_source text,p_observed_at timestamptz,
 p_evidence_sha256 text)
 RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE changed integer;
BEGIN
 IF p_tenant IS NULL OR p_subscriber IS NULL OR p_pop IS NULL OR p_site IS NULL
 OR p_distribution IS NULL OR p_source IS NULL OR p_observed_at IS NULL
 OR p_evidence_sha256 IS NULL
 OR p_source !~ '^[A-Za-z0-9_.:-]{1,128}$'
 OR p_evidence_sha256 !~ '^[0-9a-f]{64}$'
 OR p_observed_at>clock_timestamp()+interval '30 seconds'
 OR p_observed_at<clock_timestamp()-interval '24 hours'
 THEN RETURN false; END IF;

 UPDATE ipat_ops.subscribers s SET
   topology_state='verified',topology_source_id=p_source,
   topology_verified_at=p_observed_at,topology_evidence_sha256=p_evidence_sha256
 WHERE s.tenant_id=p_tenant AND s.customer_ref=p_subscriber
   AND s.subscriber360_profile AND s.pop_id=p_pop AND s.site_code=p_site
   AND s.distribution_device_id=p_distribution
   AND s.access_device_id IS NOT DISTINCT FROM p_access
   AND s.ont_reference IS NOT DISTINCT FROM p_ont
   AND s.topology_declared_at IS NOT NULL
   AND p_observed_at>=s.topology_declared_at;
 GET DIAGNOSTICS changed=ROW_COUNT;
 RETURN changed=1;
END $body$;
ALTER FUNCTION ipat_platform.record_subscriber_topology_verification(
 uuid,text,text,text,uuid,uuid,text,text,timestamptz,text)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_subscriber_topology_verification(
 uuid,text,text,text,uuid,uuid,text,text,timestamptz,text) FROM PUBLIC,
 ipat_app_runtime,ipat_tenant_api_login,ipat_oidc_session_issuer_login,
 ipat_diag_ingest_exec;
GRANT EXECUTE ON FUNCTION ipat_platform.record_subscriber_topology_verification(
 uuid,text,text,text,uuid,uuid,text,text,timestamptz,text)
 TO ipat_topology_verify_exec;

CREATE FUNCTION ipat_platform.record_diagnostic_observation(
 p_tenant uuid,p_distribution uuid,p_subscriber text,p_signal text,
 p_source text,p_observed_at timestamptz,p_evidence_sha256 text)
 RETURNS bigint LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE saved bigint;
BEGIN
 IF p_tenant IS NULL OR p_distribution IS NULL OR p_signal IS NULL
 OR p_source IS NULL OR p_observed_at IS NULL OR p_evidence_sha256 IS NULL
 OR p_observed_at>clock_timestamp()+interval '30 seconds'
 OR p_observed_at<clock_timestamp()-interval '24 hours'
 OR p_source !~ '^[A-Za-z0-9_.:-]{1,128}$'
 OR p_evidence_sha256 !~ '^[0-9a-f]{64}$'
 OR p_signal NOT IN(
    'distribution_uplink_down','distribution_uplink_healthy',
    'subscriber_unreachable','ont_optical_los','neighbor_ont_healthy',
    'ont_optical_normal','pppoe_authentication_rejected','cwmp_inform_missing')
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.managed_devices d
    WHERE d.tenant_id=p_tenant AND d.id=p_distribution
      AND d.device_kind='router' AND d.archived_at IS NULL)
 OR (p_subscriber IS NOT NULL AND NOT EXISTS(
    SELECT 1 FROM ipat_ops.subscribers s WHERE s.tenant_id=p_tenant
      AND s.customer_ref=p_subscriber AND s.subscriber360_profile
      AND s.distribution_device_id=p_distribution))
 OR (p_signal IN('subscriber_unreachable','ont_optical_los','neighbor_ont_healthy',
      'ont_optical_normal','pppoe_authentication_rejected','cwmp_inform_missing')
    AND p_subscriber IS NULL)
 OR (p_signal IN('distribution_uplink_down','distribution_uplink_healthy')
    AND p_subscriber IS NOT NULL)
 THEN RETURN NULL; END IF;
 INSERT INTO ipat_ops.diagnostic_observations(
  tenant_id,distribution_device_id,subscriber_id,signal,source_id,
  observed_at,evidence_sha256)
 VALUES(p_tenant,p_distribution,p_subscriber,p_signal,p_source,
  p_observed_at,p_evidence_sha256)
 RETURNING id INTO saved;
 RETURN saved;
END $body$;
ALTER FUNCTION ipat_platform.record_diagnostic_observation(
 uuid,uuid,text,text,text,timestamptz,text) OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_diagnostic_observation(
 uuid,uuid,text,text,text,timestamptz,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.record_diagnostic_observation(
 uuid,uuid,text,text,text,timestamptz,text) TO ipat_diag_ingest_exec;

CREATE FUNCTION ipat_platform.list_diagnostic_observations(
 p_issuer text,p_subject text,p_tenant uuid,p_distribution uuid)
 RETURNS TABLE(
  subscriber_id text,signal text,source_id text,
  observed_epoch bigint,evidence_sha256 text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT o.subscriber_id,o.signal,o.source_id,
   extract(epoch from o.observed_at)::bigint,o.evidence_sha256
 FROM ipat_ops.diagnostic_observations o
 WHERE o.tenant_id=p_tenant AND o.distribution_device_id=p_distribution
 AND o.observed_at>=clock_timestamp()-interval '1 hour'
 AND (
   (o.subscriber_id IS NULL AND EXISTS(
     SELECT 1 FROM ipat_ops.subscribers s
     WHERE s.tenant_id=p_tenant AND s.subscriber360_profile
       AND s.distribution_device_id=p_distribution
       AND ipat_platform.can_read_subscriber_scope(
         p_issuer,p_subject,p_tenant,s.pop_id,s.site_code)))
   OR
   (o.subscriber_id IS NOT NULL AND EXISTS(
     SELECT 1 FROM ipat_ops.subscribers s
     WHERE s.tenant_id=p_tenant AND s.subscriber360_profile
       AND s.customer_ref=o.subscriber_id
       AND s.distribution_device_id=p_distribution
       AND ipat_platform.can_read_subscriber_scope(
         p_issuer,p_subject,p_tenant,s.pop_id,s.site_code)))
 )
 ORDER BY o.observed_at DESC,o.id DESC LIMIT 1024
$body$;
ALTER FUNCTION ipat_platform.list_diagnostic_observations(text,text,uuid,uuid)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_diagnostic_observations(
 text,text,uuid,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_diagnostic_observations(
 text,text,uuid,uuid) TO ipat_tenant_api_exec;

COMMIT;
