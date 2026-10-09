-- R10.1: tenant-safe text branding with finite theme tokens.
-- No external logo URL, HTML, CSS or arbitrary asset is accepted here.
BEGIN;

CREATE TABLE ipat_platform.tenant_branding (
  tenant_id uuid PRIMARY KEY REFERENCES ipat_platform.tenants(id) ON DELETE CASCADE,
  display_name text NOT NULL CHECK (
    length(display_name) BETWEEN 1 AND 80
    AND display_name=btrim(display_name)
    AND display_name !~ '[[:cntrl:]]'
  ),
  mark_text text NOT NULL CHECK (
    length(mark_text) BETWEEN 1 AND 12
    AND mark_text=btrim(mark_text)
    AND mark_text !~ '[[:cntrl:]]'
  ),
  accent_token text NOT NULL CHECK (
    accent_token IN ('slate','blue','indigo','emerald','amber','rose')
  ),
  revision bigint NOT NULL DEFAULT 1 CHECK (revision >= 1),
  updated_by_issuer text NOT NULL CHECK (length(updated_by_issuer) BETWEEN 1 AND 512),
  updated_by_subject text NOT NULL CHECK (length(updated_by_subject) BETWEEN 1 AND 512),
  updated_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
ALTER TABLE ipat_platform.tenant_branding OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.tenant_branding ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.tenant_branding FORCE ROW LEVEL SECURITY;

CREATE TABLE ipat_platform.tenant_branding_events (
  request_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES ipat_platform.tenants(id) ON DELETE CASCADE,
  issuer text NOT NULL,
  subject text NOT NULL,
  expected_revision bigint NOT NULL CHECK (expected_revision >= 0),
  display_name text NOT NULL,
  mark_text text NOT NULL,
  accent_token text NOT NULL,
  result_revision bigint NOT NULL CHECK (result_revision >= 1),
  created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
ALTER TABLE ipat_platform.tenant_branding_events OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.tenant_branding_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.tenant_branding_events FORCE ROW LEVEL SECURITY;

CREATE ROLE ipat_tenant_branding_owner
  NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;

GRANT USAGE ON SCHEMA ipat_platform TO ipat_tenant_branding_owner;
GRANT SELECT ON ipat_platform.tenants,ipat_platform.identity_memberships
  TO ipat_tenant_branding_owner;
GRANT SELECT,INSERT,UPDATE ON ipat_platform.tenant_branding
  TO ipat_tenant_branding_owner;
GRANT SELECT,INSERT ON ipat_platform.tenant_branding_events
  TO ipat_tenant_branding_owner;

CREATE POLICY tenant_branding_owner_all
 ON ipat_platform.tenant_branding FOR ALL TO ipat_tenant_branding_owner
 USING (true) WITH CHECK (true);
CREATE POLICY tenant_branding_event_owner_select
 ON ipat_platform.tenant_branding_events FOR SELECT TO ipat_tenant_branding_owner
 USING (true);
CREATE POLICY tenant_branding_event_owner_insert
 ON ipat_platform.tenant_branding_events FOR INSERT TO ipat_tenant_branding_owner
 WITH CHECK (true);
-- This role is used only by SECURITY DEFINER branding functions. It gets a
-- dedicated current-membership SELECT policy; browser/API roles get no table rights.
CREATE POLICY tenant_branding_membership_select
 ON ipat_platform.identity_memberships FOR SELECT TO ipat_tenant_branding_owner
 USING (true);

CREATE FUNCTION ipat_platform.get_tenant_branding_for_member(
 p_issuer text,p_subject text,p_tenant uuid)
RETURNS TABLE(
 tenant_slug text,display_name text,mark_text text,accent_token text,revision bigint)
LANGUAGE plpgsql STABLE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL
    OR NOT EXISTS(
      SELECT 1
      FROM ipat_platform.identity_memberships m
      JOIN ipat_platform.tenants t ON t.id=m.tenant_id
      WHERE m.tenant_id=p_tenant AND m.issuer=p_issuer AND m.subject=p_subject
        AND m.role IN ('tenant_admin','system_admin','security_admin','noc_manager',
                       'noc_engineer','provisioning_officer','helpdesk',
                       'field_technician','auditor')
        AND m.revoked_at IS NULL
        AND m.created_at<=statement_timestamp()
        AND m.expires_at>statement_timestamp()
        AND t.state='active')
 THEN RETURN; END IF;

 RETURN QUERY
 SELECT t.tenant_slug,
        COALESCE(b.display_name,t.tenant_slug),
        COALESCE(b.mark_text,upper(substr(t.tenant_slug,1,2))),
        COALESCE(b.accent_token,'slate'),
        COALESCE(b.revision,0::bigint)
 FROM ipat_platform.tenants t
 LEFT JOIN ipat_platform.tenant_branding b ON b.tenant_id=t.id
 WHERE t.id=p_tenant AND t.state='active';
END $body$;
ALTER FUNCTION ipat_platform.get_tenant_branding_for_member(text,text,uuid)
 OWNER TO ipat_tenant_branding_owner;
REVOKE ALL ON FUNCTION ipat_platform.get_tenant_branding_for_member(text,text,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.get_tenant_branding_for_member(text,text,uuid)
 TO ipat_tenant_api_exec;

CREATE FUNCTION ipat_platform.set_tenant_branding(
 p_issuer text,p_subject text,p_tenant uuid,p_request uuid,p_expected_revision bigint,
 p_display_name text,p_mark_text text,p_accent_token text)
RETURNS bigint
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path=pg_catalog,ipat_platform AS $body$
DECLARE
 v_event ipat_platform.tenant_branding_events%ROWTYPE;
 v_result bigint;
BEGIN
 IF p_issuer IS NULL OR p_subject IS NULL OR p_tenant IS NULL OR p_request IS NULL
    OR p_expected_revision IS NULL OR p_expected_revision<0
    OR p_display_name IS NULL OR length(p_display_name) NOT BETWEEN 1 AND 80
    OR p_display_name<>btrim(p_display_name) OR p_display_name ~ '[[:cntrl:]]'
    OR p_mark_text IS NULL OR length(p_mark_text) NOT BETWEEN 1 AND 12
    OR p_mark_text<>btrim(p_mark_text) OR p_mark_text ~ '[[:cntrl:]]'
    OR p_accent_token NOT IN ('slate','blue','indigo','emerald','amber','rose')
    OR NOT EXISTS(
      SELECT 1
      FROM ipat_platform.identity_memberships m
      JOIN ipat_platform.tenants t ON t.id=m.tenant_id
      WHERE m.tenant_id=p_tenant AND m.issuer=p_issuer AND m.subject=p_subject
        AND m.role='tenant_admin' AND m.revoked_at IS NULL
        AND m.created_at<=clock_timestamp() AND m.expires_at>clock_timestamp()
        AND length(btrim(m.approved_by))>0 AND t.state='active')
 THEN RETURN NULL; END IF;

 PERFORM pg_advisory_xact_lock(hashtextextended(p_tenant::text||'/tenant-branding',0));

 SELECT * INTO v_event FROM ipat_platform.tenant_branding_events e
 WHERE e.request_id=p_request;
 IF FOUND THEN
   IF v_event.tenant_id=p_tenant AND v_event.issuer=p_issuer
      AND v_event.subject=p_subject
      AND v_event.expected_revision=p_expected_revision
      AND v_event.display_name=p_display_name
      AND v_event.mark_text=p_mark_text
      AND v_event.accent_token=p_accent_token
   THEN RETURN v_event.result_revision; END IF;
   RETURN NULL;
 END IF;

 IF p_expected_revision=0 THEN
   INSERT INTO ipat_platform.tenant_branding(
     tenant_id,display_name,mark_text,accent_token,revision,
     updated_by_issuer,updated_by_subject,updated_at)
   VALUES(p_tenant,p_display_name,p_mark_text,p_accent_token,1,
     p_issuer,p_subject,clock_timestamp())
   ON CONFLICT (tenant_id) DO NOTHING
   RETURNING revision INTO v_result;
 ELSE
   UPDATE ipat_platform.tenant_branding b
   SET display_name=p_display_name,mark_text=p_mark_text,accent_token=p_accent_token,
       revision=b.revision+1,updated_by_issuer=p_issuer,
       updated_by_subject=p_subject,updated_at=clock_timestamp()
   WHERE b.tenant_id=p_tenant AND b.revision=p_expected_revision
   RETURNING b.revision INTO v_result;
 END IF;
 IF v_result IS NULL THEN RETURN NULL; END IF;

 INSERT INTO ipat_platform.tenant_branding_events(
   request_id,tenant_id,issuer,subject,expected_revision,
   display_name,mark_text,accent_token,result_revision)
 VALUES(p_request,p_tenant,p_issuer,p_subject,p_expected_revision,
   p_display_name,p_mark_text,p_accent_token,v_result);
 RETURN v_result;
END $body$;
ALTER FUNCTION ipat_platform.set_tenant_branding(
 text,text,uuid,uuid,bigint,text,text,text)
 OWNER TO ipat_tenant_branding_owner;
REVOKE ALL ON FUNCTION ipat_platform.set_tenant_branding(
 text,text,uuid,uuid,bigint,text,text,text)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.set_tenant_branding(
 text,text,uuid,uuid,bigint,text,text,text)
 TO ipat_tenant_api_exec;

COMMIT;
