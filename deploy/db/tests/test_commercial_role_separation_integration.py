"""R10.09 disposable PostgreSQL: commercial role separation for PPPoE dry-runs.

No RouterOS connection or execution exists. Tenant Admin prepares inventory,
Provisioning Officer creates the dry-run, Security Admin independently reviews.
"""
import json
import os
import subprocess
import unittest
from uuid import UUID

ISS="https://identity.r1009.synthetic.invalid/realms/ipat"
T1="b9090000-0000-4000-8000-000000000001"
T2="b9090000-0000-4000-8000-000000000002"
ROUTER="c9090000-0000-4000-8000-000000000001"
ADMIN="r1009-tenant-admin"
MAKER="r1009-provisioning"
CHECKER="r1009-security"
OTHER="r1009-other-admin"
PLAN="d9090000-0000-4000-8000-000000000001"
REQ="d9090000-0000-4000-8000-000000000002"
DIGEST="9"*64

def sql(statement, role=None, check=True):
    prefix=f"SET ROLE {role};" if role else ""
    return subprocess.run(
        ["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",prefix+statement],
        text=True,capture_output=True,check=check)

ITEMS=[
 {"subscriber_id":"SUB-R1009-A","action":"create","username":"r1009-a",
  "profile":"default","secret_ref":f"vault://tenant/{T1}/pppoe/SUB-R1009-A"},
 {"subscriber_id":"SUB-R1009-B","action":"update","username":"r1009-b-new",
  "profile":"premium","secret_ref":f"vault://tenant/{T1}/pppoe/SUB-R1009-B"},
]

def create(subject=MAKER, plan=PLAN, req=REQ, key="r1009-plan"):
    payload=json.dumps(ITEMS,separators=(",",":")).replace("'","''")
    return sql(
        "SELECT ipat_platform.create_pppoe_batch_dry_run("
        f"'{ISS}','{subject}','{T1}','{plan}','{req}','{ROUTER}',"
        f"'{key}','{DIGEST}','{payload}'::jsonb)",
        "ipat_tenant_api_login").stdout.strip()

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable DB only")
class CommercialRoleSeparation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        sql(f"""
        INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
          ('{T1}','r1009-a','active'),('{T2}','r1009-b','active');
        INSERT INTO ipat_platform.identity_memberships(
          tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
          ('{T1}','{ISS}','{ADMIN}','tenant_admin','role-admin-review',now()+interval '1 day'),
          ('{T1}','{ISS}','{MAKER}','provisioning_officer','role-maker-review',now()+interval '1 day'),
          ('{T1}','{ISS}','{CHECKER}','security_admin','role-security-review',now()+interval '1 day'),
          ('{T2}','{ISS}','{OTHER}','tenant_admin','role-other-review',now()+interval '1 day');
        """)
        assert sql(
            f"SELECT ipat_platform.create_tenant_pop('{ISS}','{ADMIN}','{T1}',"
            "'POP-R1009','POP R1009')").stdout.strip()=="POP-R1009"
        assert sql(
            f"SELECT ipat_platform.create_tenant_site('{ISS}','{ADMIN}','{T1}',"
            "'SITE-R1009','SITE R1009')").stdout.strip()=="SITE-R1009"
        assert sql(
            f"SELECT ipat_platform.assign_tenant_site_to_pop('{ISS}','{ADMIN}','{T1}',"
            "'SITE-R1009','POP-R1009',1)").stdout.strip()=="2"
        router_req=str(UUID(int=UUID(ROUTER).int+100))
        assert sql(
            "SELECT ipat_platform.register_managed_device("
            f"'{ISS}','{ADMIN}','{T1}','{ROUTER}','{router_req}','SITE-R1009',"
            "'R1009 MikroTik','router','MikroTik','TEST-ONLY','routeros_api_ssl',"
            "'router.r1009.invalid',8729,NULL)").stdout.strip()==ROUTER
        for sid,user in [("SUB-R1009-A",None),("SUB-R1009-B","old-r1009-b")]:
            pppoe="NULL" if user is None else f"'{user}'"
            assert sql(
                "SELECT ipat_platform.upsert_subscriber360("
                f"'{ISS}','{ADMIN}','{T1}','{sid}','{sid}',{pppoe},"
                f"'POP-R1009','SITE-R1009','{ROUTER}',NULL,NULL,0)"
            ).stdout.strip()=="1"

    def test_01_role_vocabulary_and_scope_is_explicit(self):
        for role in ["system_admin","security_admin","provisioning_officer"]:
            subject="scope-"+role.replace("_","-")
            sql(
              "INSERT INTO ipat_platform.identity_memberships("
              "tenant_id,issuer,subject,role,approved_by,expires_at) VALUES("
              f"'{T1}','{ISS}','{subject}','{role}','scope-review',now()+interval '1 hour')")
            self.assertTrue(sql(
              "SELECT EXISTS(SELECT 1 FROM ipat_platform.lookup_active_membership("
              f"'{ISS}','{subject}','{T1}','{role}',NULL))").stdout.strip()=="t")
        bad=sql(
          "INSERT INTO ipat_platform.identity_memberships("
          "tenant_id,issuer,subject,role,approved_by,expires_at) VALUES("
          f"'{T1}','{ISS}','bad-role','superuser','x',now()+interval '1 hour')",
          check=False)
        self.assertNotEqual(bad.returncode,0)

    def test_02_only_provisioning_officer_can_create(self):
        self.assertEqual(create(),PLAN)
        self.assertEqual(create(),PLAN)
        self.assertEqual(create(subject=ADMIN,plan="d9090000-0000-4000-8000-000000000011",
                                req="d9090000-0000-4000-8000-000000000012",
                                key="admin-denied"),"")
        self.assertEqual(create(subject=CHECKER,plan="d9090000-0000-4000-8000-000000000013",
                                req="d9090000-0000-4000-8000-000000000014",
                                key="checker-denied"),"")
        caps=sql(
          "SELECT "
          f"ipat_platform.pppoe_batch_create_capability('{ISS}','{MAKER}','{T1}')::int||'|'||"
          f"ipat_platform.pppoe_batch_review_capability('{ISS}','{MAKER}','{T1}')::int||'|'||"
          f"ipat_platform.pppoe_batch_capability('{ISS}','{ADMIN}','{T1}')::int"
        ).stdout.strip()
        self.assertEqual(caps,"1|0|0")

    def test_03_only_security_admin_can_review_and_admin_cannot_even_list(self):
        self.assertEqual(sql(
          "SELECT count(*) FROM ipat_platform.list_pppoe_batch_dry_runs("
          f"'{ISS}','{ADMIN}','{T1}')",
          "ipat_tenant_api_login").stdout.strip(),"0")
        maker=sql(
          "SELECT can_review::int FROM ipat_platform.list_pppoe_batch_dry_runs("
          f"'{ISS}','{MAKER}','{T1}') WHERE id='{PLAN}'",
          "ipat_tenant_api_login").stdout.strip()
        checker=sql(
          "SELECT can_review::int FROM ipat_platform.list_pppoe_batch_dry_runs("
          f"'{ISS}','{CHECKER}','{T1}') WHERE id='{PLAN}'",
          "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(maker,"0")
        self.assertEqual(checker,"1")
        for subject in [ADMIN,MAKER,OTHER]:
            tenant=T2 if subject==OTHER else T1
            self.assertEqual(sql(
              "SELECT ipat_platform.review_pppoe_batch_dry_run("
              f"'{ISS}','{subject}','{tenant}','{PLAN}',true)",
              "ipat_tenant_api_login").stdout.strip(),"f")
        self.assertEqual(sql(
          "SELECT ipat_platform.review_pppoe_batch_dry_run("
          f"'{ISS}','{CHECKER}','{T1}','{PLAN}',true)",
          "ipat_tenant_api_login").stdout.strip(),"t")
        state=sql(
          "SELECT state||'|'||execution_allowed::int||'|'||"
          "physical_readback_verified::int FROM ipat_ops.pppoe_batch_plans "
          f"WHERE tenant_id='{T1}' AND id='{PLAN}'").stdout.strip()
        self.assertEqual(state,"approved|0|0")

    def test_04_revoked_security_admin_loses_review_immediately(self):
        plan="d9090000-0000-4000-8000-000000000021"
        req="d9090000-0000-4000-8000-000000000022"
        self.assertEqual(create(plan=plan,req=req,key="revoke-check"),plan)
        sql(
          "UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() "
          f"WHERE tenant_id='{T1}' AND issuer='{ISS}' AND subject='{CHECKER}' "
          "AND role='security_admin'")
        self.assertEqual(sql(
          "SELECT ipat_platform.review_pppoe_batch_dry_run("
          f"'{ISS}','{CHECKER}','{T1}','{plan}',true)",
          "ipat_tenant_api_login").stdout.strip(),"f")
        self.assertEqual(sql(
          "SELECT ipat_platform.pppoe_batch_capability("
          f"'{ISS}','{CHECKER}','{T1}')").stdout.strip(),"f")

if __name__=="__main__":
    unittest.main()
