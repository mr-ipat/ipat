"""R10.07 disposable PostgreSQL: tenant-scoped PPPoE batch DRY-RUN only.

No RouterOS connection or execution exists in this test. The accepted plan is
intentionally non-executable even after independent maker/checker approval.
"""
import json
import os
import subprocess
import unittest
from uuid import UUID

ISS="https://identity.r1007.synthetic.invalid/realms/ipat"
T1="b7b7b7b7-b7b7-47b7-87b7-b7b7b7b7b7b1"
T2="b7b7b7b7-b7b7-47b7-87b7-b7b7b7b7b7b2"
ROUTER1="c7c7c7c7-c7c7-47c7-87c7-c7c7c7c7c7c1"
ROUTER2="c7c7c7c7-c7c7-47c7-87c7-c7c7c7c7c7c2"
PLAN="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7d1"
REQ="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7d2"
MAKER="r1007-maker"
CHECKER="r1007-checker"
OTHER="r1007-other"
DIGEST="a"*64

def sql(statement, role=None, check=True):
    prefix=f"SET ROLE {role};" if role else ""
    return subprocess.run(
        ["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",prefix+statement],
        text=True,capture_output=True,check=check)

def call_create(items, subject=MAKER, tenant=T1, plan=PLAN, req=REQ,
                router=ROUTER1, key="batch-001", digest=DIGEST):
    payload=json.dumps(items,separators=(",",":")).replace("'","''")
    return sql(
        "SELECT ipat_platform.create_pppoe_batch_dry_run("
        f"'{ISS}','{subject}','{tenant}','{plan}','{req}','{router}',"
        f"'{key}','{digest}','{payload}'::jsonb)",
        "ipat_tenant_api_login").stdout.strip()

CREATE=[
 {"subscriber_id":"SUB-CREATE","action":"create","username":"new-create",
  "profile":"default","secret_ref":f"vault://tenant/{T1}/pppoe/SUB-CREATE"},
 {"subscriber_id":"SUB-UPDATE","action":"update","username":"new-update",
  "profile":"premium","secret_ref":f"vault://tenant/{T1}/pppoe/SUB-UPDATE"},
 {"subscriber_id":"SUB-DISABLE","action":"disable","username":None,
  "profile":None,"secret_ref":None},
]

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable DB only")
class PppoeDryRun(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        # Current, distinct humans. No new role vocabulary is invented here:
        # both maker/checker are tenant_admin until provisioning_officer /
        # security_admin receive a separately reviewed membership migration.
        sql(f"""
        INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
          ('{T1}','r1007-a','active'),('{T2}','r1007-b','active');
        INSERT INTO ipat_platform.identity_memberships(
          tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
          ('{T1}','{ISS}','{MAKER}','tenant_admin','independent-a',now()+interval '1 day'),
          ('{T1}','{ISS}','{CHECKER}','tenant_admin','independent-b',now()+interval '1 day'),
          ('{T2}','{ISS}','{OTHER}','tenant_admin','independent-c',now()+interval '1 day');
        """)
        for tenant,actor,pop,site in [
            (T1,MAKER,"POP-A","SITE-A"),(T2,OTHER,"POP-B","SITE-B")]:
            assert sql(
              f"SELECT ipat_platform.create_tenant_pop('{ISS}','{actor}','{tenant}',"
              f"'{pop}','{pop} synthetic')").stdout.strip()==pop
            assert sql(
              f"SELECT ipat_platform.create_tenant_site('{ISS}','{actor}','{tenant}',"
              f"'{site}','{site} synthetic')").stdout.strip()==site
            assert sql(
              f"SELECT ipat_platform.assign_tenant_site_to_pop('{ISS}','{actor}',"
              f"'{tenant}','{site}','{pop}',1)").stdout.strip()=="2"
        for tenant,actor,router,site in [
            (T1,MAKER,ROUTER1,"SITE-A"),(T2,OTHER,ROUTER2,"SITE-B")]:
            request=str(UUID(int=UUID(router).int+100))
            out=sql(
              "SELECT ipat_platform.register_managed_device("
              f"'{ISS}','{actor}','{tenant}','{router}','{request}','{site}',"
              f"'Synthetic MikroTik {site}','router','MikroTik','TEST-ONLY',"
              f"'routeros_api_ssl','router.invalid',8729,NULL)").stdout.strip()
            assert out==router
        # Three exact T1 subscribers are declared on ROUTER1.
        for sid,user in [
            ("SUB-CREATE",None),("SUB-UPDATE","old-update"),("SUB-DISABLE","old-disable")]:
            pppoe="NULL" if user is None else f"'{user}'"
            out=sql(
              "SELECT ipat_platform.upsert_subscriber360("
              f"'{ISS}','{MAKER}','{T1}','{sid}','{sid}',{pppoe},"
              f"'POP-A','SITE-A','{ROUTER1}',NULL,NULL,0)").stdout.strip()
            assert out=="1"

    def test_01_current_admin_exact_router_and_no_password_shape(self):
        self.assertEqual(call_create(CREATE),PLAN)
        self.assertEqual(call_create(CREATE),PLAN,"exact replay must be idempotent")
        self.assertEqual(call_create(CREATE,digest="b"*64),"","digest conflict denied")
        self.assertEqual(call_create(CREATE,subject=OTHER,tenant=T2,router=ROUTER1),"",
                         "cross-tenant router denied")
        password_item=[dict(CREATE[0],password="plaintext")]
        self.assertEqual(call_create(password_item,plan="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7e1",
                                     req="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7e2",
                                     key="bad-password"),"")

    def test_02_saved_scope_and_secret_never_returned(self):
        row=sql(
          "SELECT site_code||'|'||pop_code||'|'||state||'|'||"
          "execution_allowed::int||'|'||physical_readback_verified::int||'|'||"
          "rate_limit_per_minute FROM ipat_platform.list_pppoe_batch_dry_runs("
          f"'{ISS}','{MAKER}','{T1}') WHERE id='{PLAN}'",
          "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(row,"SITE-A|POP-A|awaiting_approval|0|0|20")
        items=sql(
          "SELECT ordinal||'|'||subscriber_id||'|'||action||'|'||"
          "coalesce(before_username,'-')||'|'||coalesce(desired_username,'-')||'|'||"
          "has_secret_ref::int FROM ipat_platform.list_pppoe_batch_items("
          f"'{ISS}','{MAKER}','{T1}','{PLAN}') ORDER BY ordinal",
          "ipat_tenant_api_login").stdout.strip().splitlines()
        self.assertEqual(items,[
          "1|SUB-CREATE|create|-|new-create|1",
          "2|SUB-UPDATE|update|old-update|new-update|1",
          "3|SUB-DISABLE|disable|old-disable|-|0",
        ])
        self.assertNotIn("vault://", "\n".join(items))

    def test_03_independent_checker_approval_is_still_non_executable(self):
        self.assertEqual(sql(
          "SELECT ipat_platform.review_pppoe_batch_dry_run("
          f"'{ISS}','{MAKER}','{T1}','{PLAN}',true)",
          "ipat_tenant_api_login").stdout.strip(),"f")
        self.assertEqual(sql(
          "SELECT ipat_platform.review_pppoe_batch_dry_run("
          f"'{ISS}','{CHECKER}','{T1}','{PLAN}',true)",
          "ipat_tenant_api_login").stdout.strip(),"t")
        row=sql(
          "SELECT state||'|'||execution_allowed::int||'|'||"
          "physical_readback_verified::int||'|'||"
          "(approval_expires_at>reviewed_at AND "
          "approval_expires_at<=reviewed_at+interval '30 minutes')::int "
          f"FROM ipat_ops.pppoe_batch_plans WHERE tenant_id='{T1}' AND id='{PLAN}'"
        ).stdout.strip()
        self.assertEqual(row,"approved|0|0|1")
        events=sql(
          "SELECT event FROM ipat_ops.pppoe_batch_audit "
          f"WHERE tenant_id='{T1}' AND plan_id='{PLAN}' ORDER BY sequence"
        ).stdout.strip().splitlines()
        self.assertEqual(events,["DRY_RUN_CREATED","APPROVED"])

    def test_04_no_execution_function_and_raw_tables_denied(self):
        for signature in [
          "ipat_platform.execute_pppoe_batch(uuid)",
          "ipat_platform.claim_pppoe_batch(uuid)",
          "ipat_platform.lease_pppoe_batch(uuid)",
        ]:
            self.assertEqual(sql(f"SELECT to_regprocedure('{signature}') IS NULL").stdout.strip(),"t")
        for role in ["ipat_tenant_api_login","ipat_oidc_session_issuer_login","ipat_app_runtime"]:
            for table in ["pppoe_batch_plans","pppoe_batch_items","pppoe_batch_audit"]:
                result=sql(f"SELECT count(*) FROM ipat_ops.{table}",role,check=False)
                self.assertNotEqual(result.returncode,0,(role,table))

    def test_05_wrong_secret_tenant_and_subscriber_scope_denied(self):
        wrong=[dict(CREATE[0],secret_ref=f"vault://tenant/{T2}/pppoe/SUB-CREATE")]
        self.assertEqual(call_create(wrong,plan="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7f1",
          req="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7f2",key="wrong-secret"),"")
        unknown=[dict(CREATE[0],subscriber_id="SUB-NOT-THERE")]
        self.assertEqual(call_create(unknown,plan="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7f3",
          req="d7d7d7d7-d7d7-47d7-87d7-d7d7d7d7d7f4",key="wrong-sub"),"")

if __name__=="__main__":
    unittest.main()
