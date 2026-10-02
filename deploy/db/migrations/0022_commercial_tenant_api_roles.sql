-- R9.68 composite execution identities for the commercial tenant BFF.
-- Requires 0013,0016,0018,0020,0021. Login roles start WITHOUT passwords;
-- deployment must provision approved peer/cert/SCRAM credentials out-of-band.
BEGIN;
CREATE ROLE ipat_tenant_api_exec NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT;
CREATE ROLE ipat_tenant_api_login LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT
 IN ROLE ipat_tenant_api_exec;
CREATE ROLE ipat_oidc_session_issuer_login LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT
 IN ROLE ipat_browser_session_issue_exec;

-- Tenant API may authenticate an existing durable session and invoke already
-- restricted tenant business functions. It CANNOT issue sessions.
GRANT ipat_browser_session_auth_exec,ipat_site_registry_exec,
      ipat_managed_registry_exec,ipat_domain_admin
 TO ipat_tenant_api_exec;
-- OIDC issuer may issue a session but receives no business CRUD or auth role.

REVOKE ALL ON SCHEMA ipat_ops FROM ipat_tenant_api_exec,ipat_tenant_api_login,ipat_oidc_session_issuer_login;
-- Schema USAGE needed only to resolve SECURITY DEFINER function names; no raw
-- table rights are granted. ipat_platform functions remain their own boundary.
GRANT USAGE ON SCHEMA ipat_platform TO ipat_tenant_api_exec,ipat_oidc_session_issuer_login;
COMMIT;
