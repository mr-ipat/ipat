"""R9.79 production-shaped security_admin role without importing lab R8.4."""
import os
import subprocess
import unittest

ISS="https://identity.r979.synthetic.invalid"
TENANT="79797979-7979-4797-8797-797979797971"

def q(sql, role=None, check=True):
    prefix=("SET ROLE "+role+";") if role else ""
    return subprocess.run(
        ["psql","-X","-q","-A","-t","-v","ON_ERROR_STOP=1","-c",prefix+sql],
        capture_output=True,text=True,check=check)

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","synthetic only")
class ProductionSecurityAdminRole(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        assert os.getenv("PGHOST")=="127.0.0.1"
        assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD")=="local_ci_synthetic_only"
        q(f"INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES('{TENANT}','r979')")
        q(f"""INSERT INTO ipat_platform.identity_memberships(
          tenant_id,issuer,subject,role,approved_by,created_at,expires_at) VALUES
          ('{TENANT}','{ISS}','reviewer','security_admin','independent-r979',
           now()-interval '1 minute',now()+interval '1 day'),
          ('{TENANT}','{ISS}','admin','tenant_admin','independent-r979',
           now()-interval '1 minute',now()+interval '1 day')""")

    def test_01_security_admin_is_valid_but_unknown_roles_remain_denied(self):
        definition=q("""SELECT pg_get_constraintdef(oid) FROM pg_constraint
          WHERE conrelid='ipat_platform.identity_memberships'::regclass
          AND conname='identity_memberships_role_check'""").stdout.strip()
        self.assertIn("security_admin",definition)
        bad=q(f"""INSERT INTO ipat_platform.identity_memberships(
          tenant_id,issuer,subject,role,approved_by,expires_at)
          VALUES('{TENANT}','{ISS}','bad','super_admin','x',now()+interval '1 day')""",
          check=False)
        self.assertNotEqual(bad.returncode,0)

    def test_02_firmware_reviewer_sees_security_admin_but_normal_lookup_does_not(self):
        reviewer=q("SELECT ipat_platform.firmware_current_reviewer("
          f"'{ISS}','reviewer','{TENANT}')").stdout.strip()
        self.assertEqual(reviewer,"t")
        ordinary=q("SELECT count(*) FROM ipat_platform.lookup_active_membership("
          f"'{ISS}','reviewer','{TENANT}','security_admin',NULL)",
          role="ipat_identity_query").stdout.strip()
        self.assertEqual(ordinary,"0")

    def test_03_security_admin_does_not_gain_tenant_admin_menu(self):
        visible=q("SELECT ipat_platform.tenant_admin_ui_capability("
          f"'{ISS}','reviewer','{TENANT}')",role="ipat_tenant_api_login").stdout.strip()
        self.assertEqual(visible,"f")
        self.assertEqual(q("SELECT has_function_privilege('ipat_tenant_api_login',"
          "'ipat_platform.firmware_current_reviewer(text,text,uuid)','EXECUTE')::int"
          ).stdout.strip(),"0")
        self.assertEqual(q("SELECT has_function_privilege('ipat_oidc_session_issuer_login',"
          "'ipat_platform.firmware_current_reviewer(text,text,uuid)','EXECUTE')::int"
          ).stdout.strip(),"0")

    def test_04_revoke_expiry_and_tenant_suspension_fail_closed(self):
        q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=now() "
          f"WHERE tenant_id='{TENANT}' AND subject='reviewer'")
        self.assertEqual(q("SELECT ipat_platform.firmware_current_reviewer("
          f"'{ISS}','reviewer','{TENANT}')").stdout.strip(),"f")
        q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL,"
          "created_at=now()-interval '2 hours',expires_at=now()-interval '1 hour' "
          f"WHERE tenant_id='{TENANT}' AND subject='reviewer'")
        self.assertEqual(q("SELECT ipat_platform.firmware_current_reviewer("
          f"'{ISS}','reviewer','{TENANT}')").stdout.strip(),"f")
        q(f"UPDATE ipat_platform.identity_memberships SET "
          "expires_at=now()+interval '1 day' "
          f"WHERE tenant_id='{TENANT}' AND subject='reviewer'")
        q(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{TENANT}'")
        self.assertEqual(q("SELECT ipat_platform.firmware_current_reviewer("
          f"'{ISS}','reviewer','{TENANT}')").stdout.strip(),"f")

if __name__=="__main__":
    unittest.main()
