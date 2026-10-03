-- R9.75 canonical platform-operated vendor/transport metadata catalog.
-- A catalogue candidate is NEVER a proven model/firmware capability.
BEGIN;
CREATE TABLE ipat_platform.vendor_transport_catalog(
 device_kind text NOT NULL CHECK(device_kind IN('olt','ont','router')),
 vendor text NOT NULL CHECK(vendor ~ '^[A-Za-z0-9_.-]{1,64}$'),
 management_transport text NOT NULL CHECK(management_transport IN('ssh','snmp','routeros_api_ssl','cwmp','usp')),
 qualification text NOT NULL DEFAULT 'metadata_candidate' CHECK(qualification='metadata_candidate'),
 metadata_registration_enabled boolean NOT NULL DEFAULT true,
 description text NOT NULL CHECK(length(description) BETWEEN 1 AND 180),
 PRIMARY KEY(device_kind,vendor,management_transport),
 CONSTRAINT catalog_transport_category CHECK(
  management_transport<>'routeros_api_ssl' OR device_kind='router')
);
ALTER TABLE ipat_platform.vendor_transport_catalog OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.vendor_transport_catalog ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.vendor_transport_catalog FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.vendor_transport_catalog FROM PUBLIC,ipat_app_runtime,
 ipat_tenant_api_exec,ipat_oidc_session_issuer_login;
CREATE ROLE ipat_vendor_catalog_owner NOLOGIN NOSUPERUSER NOCREATEDB
 NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_vendor_catalog_owner;
GRANT SELECT ON ipat_platform.vendor_transport_catalog TO ipat_vendor_catalog_owner;
CREATE POLICY catalog_sealed_reader ON ipat_platform.vendor_transport_catalog
 FOR SELECT TO ipat_vendor_catalog_owner USING(true);
GRANT EXECUTE ON FUNCTION ipat_platform.lookup_active_membership(
 text,text,uuid,text,text) TO ipat_vendor_catalog_owner;

-- Platform-curated metadata-only protocol envelopes. No verified firmware
-- claims are seeded and disabled rows can be retained historically.
INSERT INTO ipat_platform.vendor_transport_catalog(
 device_kind,vendor,management_transport,description) VALUES
 ('olt','ZTE','ssh','Candidate; exact chassis, firmware and operation require a separate qualification'),
 ('olt','ZTE','snmp','Candidate; exact MIB, firmware and operation require a separate qualification'),
 ('olt','C-DATA','ssh','Candidate; exact chassis and firmware require a separate qualification'),
 ('olt','C-DATA','snmp','Candidate; exact MIB and firmware require a separate qualification'),
 ('ont','ZTE','cwmp','Candidate; exact ONT model and firmware not yet validated'),
 ('ont','ZTE','usp','Candidate; exact ONT model, firmware and USP MTP not yet validated'),
 ('ont','VSOL','cwmp','Candidate; exact ONT model and firmware not yet validated'),
 ('ont','VSOL','usp','Candidate; exact ONT model, firmware and USP MTP not yet validated'),
 ('router','MikroTik','ssh','Candidate; exact RouterOS/board operation not qualified'),
 ('router','MikroTik','snmp','Candidate; exact RouterOS/board MIB not qualified'),
 ('router','MikroTik','routeros_api_ssl','Candidate; exact RouterOS/board TLS operation not qualified');

CREATE FUNCTION ipat_platform.list_tenant_vendor_catalog(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(device_kind text,vendor text,management_transport text,
  qualification text,description text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
 SELECT c.device_kind,c.vendor,c.management_transport,c.qualification,c.description
 FROM ipat_platform.vendor_transport_catalog c
 WHERE c.metadata_registration_enabled AND EXISTS(
 SELECT 1 FROM ipat_platform.lookup_active_membership(
  p_issuer,p_subject,p_tenant,'tenant_admin',NULL))
 ORDER BY c.device_kind,c.vendor,c.management_transport
$body$;
ALTER FUNCTION ipat_platform.list_tenant_vendor_catalog(text,text,uuid)
 OWNER TO ipat_vendor_catalog_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_tenant_vendor_catalog(text,text,uuid)
 FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
GRANT EXECUTE ON FUNCTION ipat_platform.list_tenant_vendor_catalog(text,text,uuid)
 TO ipat_tenant_api_exec;

-- The DB is authoritative for saving metadata. API cannot bypass a platform
-- catalog row disabled after the dashboard was rendered.
CREATE FUNCTION ipat_platform.guard_managed_catalog_enrollment()
 RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
BEGIN
 IF NOT EXISTS (
 SELECT 1 FROM ipat_platform.vendor_transport_catalog c
 WHERE c.device_kind=NEW.device_kind AND c.vendor=NEW.vendor
 AND c.management_transport=NEW.management_transport
 AND c.metadata_registration_enabled
 ) THEN
 RAISE check_violation USING MESSAGE='Device metadata category is disabled';
 END IF;
 RETURN NEW;
END $body$;
ALTER FUNCTION ipat_platform.guard_managed_catalog_enrollment() OWNER TO ipat_vendor_catalog_owner;
REVOKE ALL ON FUNCTION ipat_platform.guard_managed_catalog_enrollment() FROM PUBLIC;
CREATE TRIGGER managed_catalog_enrollment_guard
 BEFORE INSERT ON ipat_ops.managed_devices FOR EACH ROW
 EXECUTE FUNCTION ipat_platform.guard_managed_catalog_enrollment();
COMMIT;
