-- R9.70 menu capability derived from CURRENT PostgreSQL membership, NEVER
-- from a user-editable Host, JSON field or cached browser role claim.
-- Must be applied after 0018 (Site owner) and 0022 (restricted tenant API).
BEGIN;
CREATE FUNCTION ipat_platform.tenant_admin_ui_capability(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS boolean LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT p_issuer IS NOT NULL AND p_subject IS NOT NULL AND p_tenant IS NOT NULL
 AND EXISTS (SELECT 1 FROM ipat_platform.lookup_active_membership(
   p_issuer,p_subject,p_tenant,'tenant_admin',NULL));
$body$;
ALTER FUNCTION ipat_platform.tenant_admin_ui_capability(text,text,uuid)
 OWNER TO ipat_site_registry_owner;
REVOKE ALL ON FUNCTION ipat_platform.tenant_admin_ui_capability(text,text,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.tenant_admin_ui_capability(text,text,uuid)
 TO ipat_tenant_api_exec;
COMMIT;
