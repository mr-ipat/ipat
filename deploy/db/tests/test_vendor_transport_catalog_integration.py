"""R9.75 platform-owned catalog + DB mandatory registration guard (synthetic)."""
import os
import subprocess
import unittest

ISSUER="https://catalog.r975.synthetic.invalid"
TENANT="75757575-7575-4757-8757-757575757571"

def q(s,check=True):
    return subprocess.run(["psql","-X","-q","-A","-t","-v","ON_ERROR_STOP=1","-c",s],
                          capture_output=True,text=True,check=check)

def listing(issuer=ISSUER,subject="catalog-admin",tenant=TENANT):
    return q("SET ROLE ipat_tenant_api_login;SELECT device_kind||':'||vendor||':'||"
             "management_transport||':'||qualification FROM "
             f"ipat_platform.list_tenant_vendor_catalog('{issuer}','{subject}','{tenant}')").stdout.strip().splitlines()

def insert(device,kind="olt",vendor="ZTE",transport="ssh"):
    return q(f"""INSERT INTO ipat_ops.managed_devices(
      tenant_id,id,request_id,pop_id,display_name,device_kind,vendor,
      management_transport,management_host,management_port,
      added_by_issuer,added_by_subject)
    VALUES('{TENANT}','{device}','{device}','POP-R975','Synthetic device',
    '{kind}','{vendor}','{transport}','10.1.2.3',22,'{ISSUER}','catalog-admin')""",
    check=False)

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable only")
class VendorCatalog(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        assert os.getenv("PGHOST")=="127.0.0.1"
        assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD")=="local_ci_synthetic_only"
        q(f"INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES('{TENANT}','r975')")
        q(f"""INSERT INTO ipat_platform.identity_memberships(
            tenant_id,issuer,subject,role,approved_by,created_at,expires_at)
        VALUES('{TENANT}','{ISSUER}','catalog-admin','tenant_admin','independent-fixture',
        now()-interval '1 minute',now()+interval '2 hours')""")
        q(f"""INSERT INTO ipat_ops.tenant_sites(
            tenant_id,code,display_name,created_by_issuer,created_by_subject,
            updated_by_issuer,updated_by_subject)
         VALUES('{TENANT}','POP-R975','Synthetic POP','{ISSUER}','catalog-admin',
         '{ISSUER}','catalog-admin')""")

    def test_01_server_list_only_for_current_admin(self):
        rows=listing()
        self.assertEqual(len(rows),11)
        self.assertIn("olt:ZTE:ssh:metadata_candidate",rows)
        self.assertIn("router:MikroTik:routeros_api_ssl:metadata_candidate",rows)
        self.assertEqual(listing(subject="fake"),[])
        self.assertEqual(listing(issuer="https://forged.invalid"),[])
        self.assertEqual(listing(tenant="aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),[])

    def test_02_roles_have_no_raw_table_or_cross_service_catalog_rights(self):
        for role in ("ipat_tenant_api_login","ipat_oidc_session_issuer_login",
                     "ipat_platform_onboard_exec","ipat_app_runtime"):
            self.assertEqual(q("SELECT has_table_privilege("
                f"'{role}','ipat_platform.vendor_transport_catalog','SELECT')::int").stdout.strip(),"0")
            self.assertNotEqual(q("SET ROLE "+role+
               ";SELECT count(*) FROM ipat_platform.vendor_transport_catalog",check=False).returncode,0)
        fn="ipat_platform.list_tenant_vendor_catalog(text,text,uuid)"
        self.assertEqual(q(f"SELECT has_function_privilege('ipat_tenant_api_login','{fn}','EXECUTE')::int").stdout.strip(),"1")
        self.assertEqual(q(f"SELECT has_function_privilege('ipat_oidc_session_issuer_login','{fn}','EXECUTE')::int").stdout.strip(),"0")

    def test_03_db_trigger_denies_unknown_vendor_and_protocol_even_as_migration_owner(self):
        self.assertEqual(insert("75757575-7575-4757-8757-757575757581").returncode,0)
        self.assertNotEqual(insert("75757575-7575-4757-8757-757575757582",vendor="FakeVendor").returncode,0)
        self.assertNotEqual(insert("75757575-7575-4757-8757-757575757583",kind="ont",vendor="VSOL",transport="ssh").returncode,0)
        self.assertEqual(q("SELECT count(*) FROM ipat_ops.managed_devices "
            f"WHERE tenant_id='{TENANT}'").stdout.strip(),"1")

    def test_04_disabled_selection_blocks_new_metadata_without_erasing_history(self):
        q("UPDATE ipat_platform.vendor_transport_catalog "
          "SET metadata_registration_enabled=false WHERE device_kind='olt' "
          "AND vendor='ZTE' AND management_transport='ssh'")
        self.assertNotIn("olt:ZTE:ssh:metadata_candidate",listing())
        self.assertNotEqual(insert("75757575-7575-4757-8757-757575757584").returncode,0)
        self.assertEqual(q("SELECT count(*) FROM ipat_ops.managed_devices "
            f"WHERE tenant_id='{TENANT}'").stdout.strip(),"1")
        q("UPDATE ipat_platform.vendor_transport_catalog "
          "SET metadata_registration_enabled=true WHERE device_kind='olt' "
          "AND vendor='ZTE' AND management_transport='ssh'")

    def test_05_revoked_admin_loses_catalog(self):
        q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=now() "
          f"WHERE tenant_id='{TENANT}' AND subject='catalog-admin'")
        self.assertEqual(listing(),[])
        q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL "
          f"WHERE tenant_id='{TENANT}' AND subject='catalog-admin'")

if __name__=="__main__":
    unittest.main()
