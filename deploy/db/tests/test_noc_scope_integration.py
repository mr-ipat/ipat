"""R9.72 restricted NOC POP capability; synthetic PostgreSQL only."""
import os
import subprocess
import unittest

T1 = "72727272-7272-4727-8727-727272727271"
T2 = "72727272-7272-4727-8727-727272727272"
I = "https://r972.synthetic.invalid/realm"


def query(sql, expect=True):
    return subprocess.run(
        ["psql", "-X", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1", "-c", sql],
        text=True, capture_output=True, check=expect,
    )


def granted(subject, tenant=T1, issuer=I):
    q = ("SET ROLE ipat_tenant_api_login;"
         f"SELECT pop_id FROM ipat_platform.current_noc_pop_scopes('{issuer}','{subject}','{tenant}') ORDER BY 1")
    return query(q).stdout.strip().splitlines()


@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST") == "1", "synthetic-only")
class NocScope(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE") == "ipat_synthetic"
        assert os.getenv("PGHOST") == "127.0.0.1"
        assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD") == "local_ci_synthetic_only"
        assert query("SELECT to_regprocedure('ipat_platform.current_noc_pop_scopes(text,text,uuid)') IS NOT NULL").stdout.strip() == "t"
        query(f"INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES"
              f"('{T1}','r972-a'),('{T2}','r972-b')")
        query(f"""INSERT INTO ipat_platform.identity_memberships
                (tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
                ('{T1}','{I}','noc','noc_engineer','independent-synthetic-review',now()+interval '2 hours'),
                ('{T2}','{I}','noc','noc_engineer','independent-synthetic-review',now()+interval '2 hours'),
                ('{T1}','{I}','helpdesk','helpdesk','independent-synthetic-review',now()+interval '2 hours'),
                ('{T1}','{I}','admin','tenant_admin','independent-synthetic-review',now()+interval '2 hours')""")
        query(f"""INSERT INTO ipat_platform.identity_pop_grants
                 (tenant_id,issuer,subject,role,pop_id) VALUES
                 ('{T1}','{I}','noc','noc_engineer','POP-A'),
                 ('{T2}','{I}','noc','noc_engineer','POP-B')""")
        query(f"""INSERT INTO ipat_ops.tenant_sites
                 (tenant_id,code,display_name,created_by_issuer,created_by_subject,updated_by_issuer,updated_by_subject) VALUES
                 ('{T1}','POP-A','Alpha Site','{I}','admin','{I}','admin'),
                 ('{T2}','POP-A','Other Tenant Same Code','{I}','noc','{I}','noc')""")

    def test_01_exact_current_grants_only(self):
        self.assertEqual(granted("noc"), ["POP-A"])
        self.assertEqual(granted("noc", T2), ["POP-B"])
        self.assertEqual(granted("helpdesk"), [])
        self.assertEqual(granted("admin"), [])
        self.assertEqual(granted("noc", issuer="https://forged.invalid"), [])

    def test_02_noc_can_read_only_explicit_site_scope(self):
        valid = query(
            "SET ROLE ipat_tenant_api_login;"
            f"SELECT code FROM ipat_platform.list_tenant_sites('{I}','noc','{T1}','POP-A',NULL)"
        ).stdout.strip().splitlines()
        other = query(
            "SET ROLE ipat_tenant_api_login;"
            f"SELECT code FROM ipat_platform.list_tenant_sites('{I}','noc','{T1}','POP-B',NULL)"
        ).stdout.strip().splitlines()
        self.assertEqual(valid, ["POP-A"])
        self.assertEqual(other, [])

    def test_03_no_raw_grants_and_issuer_has_no_capability(self):
        self.assertEqual(query(
            "SELECT has_function_privilege('ipat_tenant_api_login',"
            "'ipat_platform.current_noc_pop_scopes(text,text,uuid)','EXECUTE')::int"
        ).stdout.strip(), "1")
        self.assertEqual(query(
            "SELECT has_function_privilege('ipat_oidc_session_issuer_login',"
            "'ipat_platform.current_noc_pop_scopes(text,text,uuid)','EXECUTE')::int"
        ).stdout.strip(), "0")
        self.assertNotEqual(query(
            "SET ROLE ipat_tenant_api_login;SELECT count(*) FROM ipat_platform.identity_pop_grants",
            expect=False
        ).returncode, 0)

    def test_04_revocation_and_expiry_are_not_cached(self):
        query(f"UPDATE ipat_platform.identity_memberships SET revoked_at=now() WHERE tenant_id='{T1}' AND subject='noc'")
        self.assertEqual(granted("noc"), [])
        query(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL,expires_at=now()-interval '1 second' WHERE tenant_id='{T1}' AND subject='noc'")
        self.assertEqual(granted("noc"), [])
        query(f"UPDATE ipat_platform.identity_memberships SET expires_at=now()+interval '2 hours' WHERE tenant_id='{T1}' AND subject='noc'")

    def test_05_suspended_tenant_is_not_visible(self):
        query(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{T1}'")
        self.assertEqual(granted("noc"), [])
        query(f"UPDATE ipat_platform.tenants SET state='active' WHERE id='{T1}'")
        self.assertEqual(granted("noc"), ["POP-A"])


if __name__ == "__main__":
    unittest.main()
