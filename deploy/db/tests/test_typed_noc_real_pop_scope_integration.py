"""Synthetic PG only: real POP grants never inherit old exact-Site rights."""
import os
import subprocess
import unittest

ISS="https://identity.r977.synthetic.invalid"
T1="77777777-7777-4777-8777-777777777761"
T2="77777777-7777-4777-8777-777777777762"

def sql(query,role=None,check=True):
    text=("SET ROLE "+role+";") if role else ""
    return subprocess.run(
      ["psql","-X","-q","-A","-t","-v","ON_ERROR_STOP=1","-c",text+query],
      text=True,capture_output=True,check=check)

def real(subject="noc",tenant=T1):
    return sql("SELECT pop_code FROM ipat_platform.current_noc_real_pop_scopes("
      f"'{ISS}','{subject}','{tenant}')",role="ipat_tenant_api_login").stdout.strip().splitlines()

def descendants(pop="POP-A",function="list_noc_real_pop_sites"):
    return sql(f"SELECT * FROM ipat_platform.{function}("
      f"'{ISS}','noc','{T1}','{pop}')",
      role="ipat_tenant_api_login").stdout.strip().splitlines()

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable PG only")
class TypedPopScope(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        assert os.getenv("PGHOST")=="127.0.0.1"
        assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD")=="local_ci_synthetic_only"
        sql(f"INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES "
            f"('{T1}','r977-one'),('{T2}','r977-two')")
        sql(f"""INSERT INTO ipat_platform.identity_memberships
           (tenant_id,issuer,subject,role,approved_by,created_at,expires_at) VALUES
           ('{T1}','{ISS}','noc','noc_engineer','independent-review',
              now()-interval '1 hour',now()+interval '1 day'),
           ('{T2}','{ISS}','noc','noc_engineer','independent-review',
              now()-interval '1 hour',now()+interval '1 day')""")
        sql(f"INSERT INTO ipat_platform.identity_pop_grants "
            f"(tenant_id,issuer,subject,role,pop_id) VALUES "
            f"('{T1}','{ISS}','noc','noc_engineer','SITE-A')")
        sql(f"""INSERT INTO ipat_ops.tenant_pops(
           tenant_id,code,display_name,created_by_issuer,created_by_subject,
           updated_by_issuer,updated_by_subject) VALUES
           ('{T1}','POP-A','POP A','{ISS}','fixture','{ISS}','fixture'),
           ('{T1}','POP-B','POP B','{ISS}','fixture','{ISS}','fixture'),
           ('{T2}','POP-A','Other tenant POP','{ISS}','fixture','{ISS}','fixture')""")
        sql(f"""INSERT INTO ipat_ops.tenant_sites(
          tenant_id,code,display_name,parent_pop_code,created_by_issuer,
          created_by_subject,updated_by_issuer,updated_by_subject) VALUES
          ('{T1}','SITE-A','Linked site','POP-A','{ISS}','fixture','{ISS}','fixture'),
          ('{T1}','SITE-B','Unlinked site',NULL,'{ISS}','fixture','{ISS}','fixture'),
          ('{T2}','SITE-A','Foreign site','POP-A','{ISS}','fixture','{ISS}','fixture')""")

    def test_01_no_implicit_legacy_grant_promotion(self):
        self.assertEqual(real(),[])
        self.assertEqual(descendants(),[])
        legacy=sql("SELECT code FROM ipat_platform.list_tenant_sites("
          f"'{ISS}','noc','{T1}','SITE-A',NULL)",
          role="ipat_tenant_api_login").stdout.strip()
        self.assertEqual(legacy,"SITE-A")

    def test_02_only_explicit_distinct_approved_current_real_pop(self):
        for tenant in [T1,T2]:
            sql(f"""INSERT INTO ipat_platform.noc_real_pop_grants(
            tenant_id,issuer,subject,role,pop_code,
            requested_by_issuer,requested_by_subject,approved_by_issuer,
            approved_by_subject,created_at,expires_at) VALUES
            ('{tenant}','{ISS}','noc','noc_engineer','POP-A',
            '{ISS}','requester','{ISS}','distinct-reviewer',
            now()-interval '1 minute',now()+interval '2 hours')""")
        self.assertEqual(real(),["POP-A"])
        self.assertEqual(real(tenant=T2),["POP-A"])
        self.assertEqual(real(subject="other"),[])
        self.assertEqual(len(descendants()),1)
        self.assertTrue(descendants()[0].startswith("SITE-A|"))
        self.assertEqual(descendants("POP-B"),[])

    def test_03_devices_are_redacted_and_exact_pop_scoped(self):
        first="77777777-7777-4777-8777-777777777781"
        other="77777777-7777-4777-8777-777777777782"
        for tenant,dev,site in [(T1,first,"SITE-A"),(T1,other,"SITE-B")]:
            sql(f"""INSERT INTO ipat_ops.managed_devices(
             tenant_id,id,request_id,pop_id,display_name,device_kind,vendor,
             management_transport,management_host,management_port,
             added_by_issuer,added_by_subject)
             VALUES('{tenant}','{dev}','{dev}','{site}','Synthetic record',
             'olt','ZTE','ssh','10.1.1.1',22,'{ISS}','fixture')""")
        rows=descendants(function="list_noc_real_pop_devices")
        self.assertEqual(len(rows),1)
        self.assertTrue(rows[0].startswith(first+"|SITE-A|"))
        self.assertNotIn("10.1.1.1",rows[0])
        self.assertEqual(descendants(pop="POP-B",
          function="list_noc_real_pop_devices"),[])
        function="ipat_platform.current_noc_real_pop_scopes(text,text,uuid)"
        self.assertEqual(sql("SELECT has_function_privilege("
          f"'ipat_oidc_session_issuer_login','{function}','EXECUTE')::int"
          ).stdout.strip(),"0")
        self.assertEqual(sql("SELECT has_table_privilege("
          "'ipat_tenant_api_login','ipat_platform.noc_real_pop_grants',"
          "'INSERT')::int").stdout.strip(),"0")
        self.assertNotEqual(sql("SELECT count(*) FROM ipat_platform.noc_real_pop_grants",
          role="ipat_tenant_api_login",check=False).returncode,0)

    def test_04_site_reassociation_removes_old_real_pop_visibility(self):
        sql(f"UPDATE ipat_ops.tenant_sites SET parent_pop_code='POP-B' "
            f"WHERE tenant_id='{T1}' AND code='SITE-A'")
        self.assertEqual(real(),["POP-A"])
        self.assertEqual(descendants(),[])
        self.assertEqual(descendants(function="list_noc_real_pop_devices"),[])
        # Legacy exact-Site NOC permission remains unchanged, not promoted.
        legacy=sql("SELECT code FROM ipat_platform.list_tenant_sites("
          f"'{ISS}','noc','{T1}','SITE-A',NULL)",
          role="ipat_tenant_api_login").stdout.strip()
        self.assertEqual(legacy,"SITE-A")
        sql(f"UPDATE ipat_ops.tenant_sites SET parent_pop_code='POP-A' "
            f"WHERE tenant_id='{T1}' AND code='SITE-A'")

    def test_05_checker_and_revocation_reject_ineligible_scope(self):
        same=sql(f"""INSERT INTO ipat_platform.noc_real_pop_grants(
          tenant_id,issuer,subject,pop_code,requested_by_issuer,
          requested_by_subject,approved_by_issuer,approved_by_subject,expires_at)
          VALUES('{T1}','{ISS}','noc','POP-B','{ISS}','operator',
          '{ISS}','operator',now()+interval '2 hours')""",check=False)
        self.assertNotEqual(same.returncode,0)
        sql(f"UPDATE ipat_platform.noc_real_pop_grants SET revoked_at=now() "
            f"WHERE tenant_id='{T1}' AND subject='noc'")
        self.assertEqual(real(),[])
        sql(f"UPDATE ipat_platform.noc_real_pop_grants SET revoked_at=NULL,"
            "created_at=now()-interval '3 days',"
            f"expires_at=now()-interval '2 days' WHERE tenant_id='{T1}'")
        self.assertEqual(real(),[])
        sql(f"UPDATE ipat_platform.noc_real_pop_grants SET "
            f"expires_at=now()+interval '2 hours' WHERE tenant_id='{T1}'")
        self.assertEqual(real(),["POP-A"])
        sql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=now() "
            f"WHERE tenant_id='{T1}' AND subject='noc'")
        self.assertEqual(real(),[])
        sql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL "
            f"WHERE tenant_id='{T1}' AND subject='noc'")
        sql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{T1}'")
        self.assertEqual(real(),[])

if __name__=="__main__":
    unittest.main()
