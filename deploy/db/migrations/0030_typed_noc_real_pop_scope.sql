-- R9.77: explicit typed real POP grants, SEPARATE from old exact-Site
-- identity_pop_grants. No implicit conversion or runtime grant write path.
BEGIN;
CREATE TABLE ipat_platform.noc_real_pop_grants(
 tenant_id uuid NOT NULL,
 issuer text NOT NULL,
 subject text NOT NULL,
 role text NOT NULL DEFAULT 'noc_engineer' CHECK(role='noc_engineer'),
 pop_code text NOT NULL CHECK(pop_code ~ '^[A-Za-z0-9_.-]{1,128}$'),
 requested_by_issuer text NOT NULL,
 requested_by_subject text NOT NULL,
 approved_by_issuer text NOT NULL,
 approved_by_subject text NOT NULL,
 created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 expires_at timestamptz NOT NULL,
 revoked_at timestamptz,
 PRIMARY KEY(tenant_id,issuer,subject,pop_code),
 CONSTRAINT typed_pop_exact_noc_identity FOREIGN KEY(
  tenant_id,issuer,subject,role)
  REFERENCES ipat_platform.identity_memberships(tenant_id,issuer,subject,role),
 CONSTRAINT typed_pop_real_tenant_parent FOREIGN KEY(tenant_id,pop_code)
  REFERENCES ipat_ops.tenant_pops(tenant_id,code),
 CONSTRAINT typed_pop_independent_approval CHECK (
  requested_by_issuer<>approved_by_issuer OR
  requested_by_subject<>approved_by_subject),
 CONSTRAINT typed_pop_future_expiry CHECK(expires_at>created_at),
 CONSTRAINT typed_pop_valid_revocation CHECK(
  revoked_at IS NULL OR revoked_at>=created_at)
);
ALTER TABLE ipat_platform.noc_real_pop_grants OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.noc_real_pop_grants ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.noc_real_pop_grants FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.noc_real_pop_grants FROM PUBLIC,
 ipat_app_runtime,ipat_tenant_api_login,ipat_oidc_session_issuer_login;
CREATE INDEX typed_pop_active_grants ON ipat_platform.noc_real_pop_grants(
 issuer,subject,tenant_id) WHERE revoked_at IS NULL;

CREATE ROLE ipat_noc_real_scope_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform,ipat_ops TO ipat_noc_real_scope_owner;
GRANT SELECT ON ipat_platform.tenants,ipat_platform.identity_memberships,
 ipat_platform.noc_real_pop_grants TO ipat_noc_real_scope_owner;
GRANT SELECT ON ipat_ops.tenant_pops,ipat_ops.tenant_sites,
 ipat_ops.managed_devices TO ipat_noc_real_scope_owner;
CREATE POLICY typed_pop_scope_grant_reader ON ipat_platform.noc_real_pop_grants
 FOR SELECT TO ipat_noc_real_scope_owner USING(true);
CREATE POLICY typed_pop_scope_member_reader ON ipat_platform.identity_memberships
 FOR SELECT TO ipat_noc_real_scope_owner USING(true);
CREATE POLICY typed_pop_scope_pop_reader ON ipat_ops.tenant_pops
 FOR SELECT TO ipat_noc_real_scope_owner USING(true);
CREATE POLICY typed_pop_scope_site_reader ON ipat_ops.tenant_sites
 FOR SELECT TO ipat_noc_real_scope_owner USING(true);
CREATE POLICY typed_pop_scope_device_reader ON ipat_ops.managed_devices
 FOR SELECT TO ipat_noc_real_scope_owner USING(true);

CREATE FUNCTION ipat_platform.current_noc_real_pop_scopes(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(pop_code text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT g.pop_code
 FROM ipat_platform.noc_real_pop_grants g
 JOIN ipat_platform.identity_memberships m ON
   m.tenant_id=g.tenant_id AND m.issuer=g.issuer
   AND m.subject=g.subject AND m.role=g.role
 JOIN ipat_platform.tenants t ON t.id=g.tenant_id AND t.state='active'
 JOIN ipat_ops.tenant_pops p ON p.tenant_id=g.tenant_id
   AND p.code=g.pop_code
 WHERE p_issuer IS NOT NULL AND p_subject IS NOT NULL AND p_tenant IS NOT NULL
 AND g.tenant_id=p_tenant AND g.issuer=p_issuer AND g.subject=p_subject
 AND g.role='noc_engineer'
 AND g.revoked_at IS NULL AND g.created_at<=statement_timestamp()
 AND g.expires_at>statement_timestamp()
 AND m.revoked_at IS NULL AND m.created_at<=statement_timestamp()
 AND m.expires_at>statement_timestamp()
 AND length(btrim(m.approved_by))>0
 AND (g.requested_by_issuer<>g.approved_by_issuer OR
      g.requested_by_subject<>g.approved_by_subject)
 ORDER BY g.pop_code LIMIT 100
$body$;
ALTER FUNCTION ipat_platform.current_noc_real_pop_scopes(text,text,uuid)
 OWNER TO ipat_noc_real_scope_owner;
REVOKE ALL ON FUNCTION ipat_platform.current_noc_real_pop_scopes(text,text,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.current_noc_real_pop_scopes(text,text,uuid)
 TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.list_noc_real_pop_sites(
 p_issuer text,p_subject text,p_tenant uuid,p_pop text)
 RETURNS TABLE(code text,display_name text,revision bigint,assigned_devices bigint)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT s.code,s.display_name,s.revision,
 (SELECT count(*) FROM ipat_ops.managed_devices d
   WHERE d.tenant_id=s.tenant_id AND d.pop_id=s.code
    AND d.lifecycle_state='SAVED' AND d.archived_at IS NULL)::bigint
 FROM ipat_ops.tenant_sites s
 WHERE s.tenant_id=p_tenant AND s.parent_pop_code=p_pop
 AND EXISTS(SELECT 1 FROM ipat_platform.current_noc_real_pop_scopes(
  p_issuer,p_subject,p_tenant) x WHERE x.pop_code=p_pop)
 ORDER BY s.code LIMIT 500
$body$;
ALTER FUNCTION ipat_platform.list_noc_real_pop_sites(text,text,uuid,text)
 OWNER TO ipat_noc_real_scope_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_noc_real_pop_sites(text,text,uuid,text)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.list_noc_real_pop_sites(text,text,uuid,text)
 TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.list_noc_real_pop_devices(
 p_issuer text,p_subject text,p_tenant uuid,p_pop text)
 RETURNS TABLE(id uuid,site_code text,display_name text,
  device_kind text,vendor text,intended_model text,
  management_transport text,lifecycle_state text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT d.id,s.code,d.display_name,d.device_kind,d.vendor,
  d.intended_model,d.management_transport,d.lifecycle_state
 FROM ipat_ops.tenant_sites s
 JOIN ipat_ops.managed_devices d ON
   d.tenant_id=s.tenant_id AND d.pop_id=s.code
 WHERE s.tenant_id=p_tenant AND s.parent_pop_code=p_pop
 AND d.lifecycle_state='SAVED' AND d.archived_at IS NULL
 AND EXISTS(SELECT 1 FROM ipat_platform.current_noc_real_pop_scopes(
   p_issuer,p_subject,p_tenant) x WHERE x.pop_code=p_pop)
 ORDER BY d.display_name,d.id LIMIT 500
$body$;
ALTER FUNCTION ipat_platform.list_noc_real_pop_devices(text,text,uuid,text)
 OWNER TO ipat_noc_real_scope_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_noc_real_pop_devices(text,text,uuid,text)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.list_noc_real_pop_devices(text,text,uuid,text)
 TO ipat_tenant_api_exec;
COMMIT;
