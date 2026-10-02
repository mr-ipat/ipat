-- R9.72: expose ONLY independently current exact POP grants to the
-- authenticated, Host-bound tenant API. No browser-supplied role selector.
BEGIN;
CREATE ROLE ipat_noc_scope_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_noc_scope_owner;
GRANT SELECT ON ipat_platform.tenants,ipat_platform.identity_memberships,
 ipat_platform.identity_pop_grants TO ipat_noc_scope_owner;
CREATE POLICY noc_scope_active_member ON ipat_platform.identity_memberships
 FOR SELECT TO ipat_noc_scope_owner USING (true);
CREATE POLICY noc_scope_pop_grant ON ipat_platform.identity_pop_grants
 FOR SELECT TO ipat_noc_scope_owner USING (true);
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
 text,text,uuid,text,text) TO ipat_noc_scope_owner;
CREATE FUNCTION ipat_platform.current_noc_pop_scopes(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(pop_id text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT g.pop_id
 FROM ipat_platform.identity_pop_grants g
 JOIN ipat_platform.tenants t ON t.id=g.tenant_id AND t.state='active'
 WHERE p_issuer IS NOT NULL AND p_subject IS NOT NULL AND p_tenant IS NOT NULL
 AND g.tenant_id=p_tenant AND g.issuer=p_issuer AND g.subject=p_subject
 AND g.role='noc_engineer'
 AND EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'noc_engineer',g.pop_id))
 ORDER BY g.pop_id LIMIT 100
$body$;
ALTER FUNCTION ipat_platform.current_noc_pop_scopes(text,text,uuid)
 OWNER TO ipat_noc_scope_owner;
REVOKE ALL ON FUNCTION ipat_platform.current_noc_pop_scopes(text,text,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.current_noc_pop_scopes(text,text,uuid)
 TO ipat_tenant_api_exec;
COMMIT;
