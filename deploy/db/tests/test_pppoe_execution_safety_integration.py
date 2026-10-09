"""R10.10 disposable PostgreSQL safety state machine; no network/device access."""
import json
import os
import subprocess
import unittest
from uuid import UUID

ISS="https://identity.r1010.synthetic.invalid/realms/ipat"
T="ba100000-0000-4000-8000-000000000001"
ROUTER="ca100000-0000-4000-8000-000000000001"
ADMIN="r1010-admin"
MAKER="r1010-provisioner"
CHECKER="r1010-security"
SYSTEM="r1010-system"
PLAN="da100000-0000-4000-8000-000000000001"
REQ="da100000-0000-4000-8000-000000000002"
SESSION="ea100000-0000-4000-8000-000000000001"
CHECKER_SESSION="ea100000-0000-4000-8000-000000000004"
CHECKER_COOKIE="d"*64
CHECKER_CSRF="e"*64
DOMAIN="fa100000-0000-4000-8000-000000000001"
HOST="tenant-r1010.example.net"
ATTEMPT="aa100000-0000-4000-8000-000000000001"
COOKIE="1"*64
CSRF="2"*64
STALE_SESSION="ea100000-0000-4000-8000-000000000002"
STALE_COOKIE="6"*64
STALE_CSRF="7"*64
MAKER_SESSION="ea100000-0000-4000-8000-000000000003"
MAKER_COOKIE="b"*64
MAKER_CSRF="c"*64
EVIDENCE="3"*64
RECOVERY="4"*64
DIGEST="5"*64

def sql(q, role=None, check=True):
    prefix=f"SET ROLE {role};" if role else ""
    result=subprocess.run(
        ["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",prefix+q],
        text=True,capture_output=True,check=False)
    if check and result.returncode:
        raise AssertionError(result.stderr.strip())
    return result

def seed():
    sql(f"""
    INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
      VALUES('{T}','r1010','active');
    INSERT INTO ipat_platform.identity_memberships(
      tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
      ('{T}','{ISS}','{ADMIN}','tenant_admin','admin-review',now()+interval '1 day'),
      ('{T}','{ISS}','{MAKER}','provisioning_officer','maker-review',now()+interval '1 day'),
      ('{T}','{ISS}','{CHECKER}','security_admin','security-review',now()+interval '1 day'),
      ('{T}','{ISS}','{SYSTEM}','system_admin','system-review',now()+interval '1 day');
    INSERT INTO ipat_platform.tenant_domains(
      id,tenant_id,hostname,domain_type,verification_state,verification_method,
      verified_at,routing_mode,verification_name,verification_value,
      requested_by_issuer,requested_by_subject,requested_at,activation_state,
      ownership_verified_at,routing_ready_at,tls_ready_at,activated_at)
    VALUES('{DOMAIN}','{T}','{HOST}','custom_domain','verified',
      'dns_txt',now(),'a_record','_ipat-verify.tenant-r1010.example.net',
      'ipat-domain={DOMAIN}','{ISS}','{ADMIN}',now(),'active',
      now(),now(),now(),now());
    """)
    assert sql(f"SELECT ipat_platform.create_tenant_pop('{ISS}','{ADMIN}','{T}','POP-R1010','POP R1010')").stdout.strip()=="POP-R1010"
    assert sql(f"SELECT ipat_platform.create_tenant_site('{ISS}','{ADMIN}','{T}','SITE-R1010','SITE R1010')").stdout.strip()=="SITE-R1010"
    assert sql(f"SELECT ipat_platform.assign_tenant_site_to_pop('{ISS}','{ADMIN}','{T}','SITE-R1010','POP-R1010',1)").stdout.strip()=="2"
    router_req=str(UUID(int=UUID(ROUTER).int+100))
    assert sql(
      "SELECT ipat_platform.register_managed_device("
      f"'{ISS}','{ADMIN}','{T}','{ROUTER}','{router_req}','SITE-R1010',"
      "'R1010 MikroTik','router','MikroTik','TEST-ONLY','routeros_api_ssl',"
      "'router.r1010.invalid',8729,NULL)").stdout.strip()==ROUTER
    for sid,user in [("SUB-R1010-A",None),("SUB-R1010-B","old-b")]:
        pppoe="NULL" if user is None else f"'{user}'"
        assert sql(
          "SELECT ipat_platform.upsert_subscriber360("
          f"'{ISS}','{ADMIN}','{T}','{sid}','{sid}',{pppoe},"
          f"'POP-R1010','SITE-R1010','{ROUTER}',NULL,NULL,0)"
        ).stdout.strip()=="1"
    items=[
      {"subscriber_id":"SUB-R1010-A","action":"create","username":"new-a",
       "profile":"default","secret_ref":f"vault://tenant/{T}/pppoe/SUB-R1010-A"},
      {"subscriber_id":"SUB-R1010-B","action":"update","username":"new-b",
       "profile":"premium","secret_ref":f"vault://tenant/{T}/pppoe/SUB-R1010-B"},
    ]
    payload=json.dumps(items,separators=(",",":")).replace("'","''")
    assert sql(
      "SELECT ipat_platform.create_pppoe_batch_dry_run("
      f"'{ISS}','{MAKER}','{T}','{PLAN}','{REQ}','{ROUTER}','r1010-plan',"
      f"'{DIGEST}','{payload}'::jsonb)", "ipat_tenant_api_login").stdout.strip()==PLAN
    assert sql(
      "SELECT ipat_platform.review_pppoe_batch_dry_run("
      f"'{ISS}','{CHECKER}','{T}','{PLAN}',true)",
      "ipat_tenant_api_login").stdout.strip()=="t"
    sql(
      "INSERT INTO ipat_platform.tenant_browser_sessions("
      "id,tenant_id,domain_id,issuer,subject,cookie_sha256,csrf_sha256,"
      "issued_at,last_seen_at,expires_at) VALUES("
      f"'{SESSION}','{T}','{DOMAIN}','{ISS}','{SYSTEM}','{COOKIE}','{CSRF}',"
      "clock_timestamp(),clock_timestamp(),clock_timestamp()+interval '10 minutes')")
    sql(
      "INSERT INTO ipat_platform.tenant_browser_sessions("
      "id,tenant_id,domain_id,issuer,subject,cookie_sha256,csrf_sha256,"
      "issued_at,last_seen_at,expires_at) VALUES("
      f"'{CHECKER_SESSION}','{T}','{DOMAIN}','{ISS}','{CHECKER}','{CHECKER_COOKIE}','{CHECKER_CSRF}',"
      "clock_timestamp(),clock_timestamp(),clock_timestamp()+interval '10 minutes')")


@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable DB only")
class PppoeExecutionSafety(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        seed()

    def test_01_raw_roles_cannot_forge_readiness_or_worker_claim(self):
        for role in ["ipat_tenant_api_login","ipat_oidc_session_issuer_login","ipat_app_runtime"]:
            q=sql(
              "SELECT ipat_platform.record_pppoe_router_readiness("
              f"'{T}','{ROUTER}','{EVIDENCE}','{RECOVERY}','7.99-test',"
              "now(),now(),now()+interval '10 minutes')", role, check=False)
            self.assertNotEqual(q.returncode,0)
        self.assertEqual(sql(
          "SELECT has_function_privilege('ipat_tenant_api_login',"
          "'ipat_platform.claim_next_pppoe_execution_item(uuid,uuid,text)','EXECUTE')::int"
        ).stdout.strip(),"0")
        self.assertEqual(sql(
          "SELECT has_function_privilege('ipat_pppoe_worker_exec',"
          "'ipat_platform.record_pppoe_router_readiness(uuid,uuid,text,text,text,timestamptz,timestamptz,timestamptz)','EXECUTE')::int"
        ).stdout.strip(),"0")
        self.assertEqual(sql(
          "SELECT (to_regprocedure("
          "'ipat_platform.arm_pppoe_batch_execution(text,text,uuid,uuid,uuid,uuid)'"
          ") IS NULL)::int"
        ).stdout.strip(),"1")
        self.assertEqual(sql(
          "SELECT (to_regprocedure("
          "'ipat_platform.arm_pppoe_batch_execution(text,text,text,uuid,uuid)'"
          ") IS NOT NULL)::int"
        ).stdout.strip(),"1")

    def test_02_arm_requires_current_readback_and_fresh_security_session(self):
        no_ready=sql(
          "SELECT ipat_platform.arm_pppoe_batch_execution("
          f"'{COOKIE}','{HOST}','{CSRF}','{PLAN}','{ATTEMPT}')",
          "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(no_ready,"")
        readiness=sql(
          "SELECT ipat_platform.record_pppoe_router_readiness("
          f"'{T}','{ROUTER}','{EVIDENCE}','{RECOVERY}','7.99-test',"
          "clock_timestamp(),clock_timestamp(),clock_timestamp()+interval '10 minutes')",
          "ipat_pppoe_readback_exec",check=False)
        self.assertEqual(readiness.returncode,0,readiness.stderr)
        self.assertEqual(readiness.stdout.strip(),"t")
        # Device metadata changing AFTER physical readback invalidates readiness.
        self.assertEqual(sql(
          "SELECT ipat_platform.edit_managed_device_metadata("
          f"'{ISS}','{ADMIN}','{T}','{ROUTER}',1,'R1010 MikroTik',"
          "'SITE-R1010','TEST-ONLY','router.r1010.invalid',8729)"
        ).stdout.strip(),"2")
        self.assertEqual(sql(
          "SELECT ipat_platform.arm_pppoe_batch_execution("
          f"'{COOKIE}','{HOST}','{CSRF}','{PLAN}',"
          "'aa100000-0000-4000-8000-000000000096')",
          "ipat_tenant_api_login").stdout.strip(),"")
        # Fresh physical readback binds to the new metadata revision.
        self.assertEqual(sql(
          "SELECT ipat_platform.record_pppoe_router_readiness("
          f"'{T}','{ROUTER}','{EVIDENCE}','{RECOVERY}','7.99-test',"
          "clock_timestamp(),clock_timestamp(),clock_timestamp()+interval '10 minutes')",
          "ipat_pppoe_readback_exec").stdout.strip(),"t")
        sql(
          "INSERT INTO ipat_platform.tenant_browser_sessions("
          "id,tenant_id,domain_id,issuer,subject,cookie_sha256,csrf_sha256,"
          "issued_at,last_seen_at,expires_at) VALUES("
          f"'{STALE_SESSION}','{T}','{DOMAIN}','{ISS}','{SYSTEM}','{STALE_COOKIE}','{STALE_CSRF}',"
          "clock_timestamp()-interval '6 minutes',clock_timestamp(),"
          "clock_timestamp()+interval '5 minutes'),("
          f"'{MAKER_SESSION}','{T}','{DOMAIN}','{ISS}','{MAKER}','{MAKER_COOKIE}','{MAKER_CSRF}',"
          "clock_timestamp(),clock_timestamp(),clock_timestamp()+interval '10 minutes')")
        self.assertEqual(sql(
          "SELECT ipat_platform.arm_pppoe_batch_execution("
          f"'{CHECKER_COOKIE}','{HOST}','{CHECKER_CSRF}','{PLAN}',"
          "'aa100000-0000-4000-8000-000000000095')",
          "ipat_tenant_api_login").stdout.strip(),"")
        self.assertEqual(sql(
          "SELECT ipat_platform.arm_pppoe_batch_execution("
          f"'{STALE_COOKIE}','{HOST}','{STALE_CSRF}','{PLAN}',"
          "'aa100000-0000-4000-8000-000000000099')",
          "ipat_tenant_api_login").stdout.strip(),"")
        self.assertEqual(sql(
          "SELECT ipat_platform.arm_pppoe_batch_execution("
          f"'{MAKER_COOKIE}','{HOST}','{MAKER_CSRF}','{PLAN}',"
          "'aa100000-0000-4000-8000-000000000098')",
          "ipat_tenant_api_login").stdout.strip(),"")
        self.assertEqual(sql(
          "SELECT ipat_platform.arm_pppoe_batch_execution("
          f"'{COOKIE}','{HOST}','{CSRF}','{PLAN}','{ATTEMPT}')",
          "ipat_tenant_api_login").stdout.strip(),ATTEMPT)
        state=sql(
          "SELECT execution_allowed::int||'|'||physical_readback_verified::int||'|'||basis "
          f"FROM ipat_ops.pppoe_batch_plans WHERE tenant_id='{T}' AND id='{PLAN}'").stdout.strip()
        self.assertEqual(state,"1|1|routeros_api_ssl_exact_readback")
        duplicate=sql(
          "SELECT ipat_platform.arm_pppoe_batch_execution("
          f"'{COOKIE}','{HOST}','{CSRF}','{PLAN}',"
          "'aa100000-0000-4000-8000-000000000097')",
          "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(duplicate,"")

    def test_03_worker_serializes_and_unknown_requires_reconcile(self):
        row=sql(
          "SELECT ordinal||'|'||subscriber_id||'|'||action||'|'||"
          "(secret_ref IS NOT NULL)::int "
          "FROM ipat_platform.claim_next_pppoe_execution_item("
          f"'{T}','{ATTEMPT}','worker-r1010')",
          "ipat_pppoe_worker_exec").stdout.strip()
        self.assertEqual(row,"1|SUB-R1010-A|create|1")
        # No second item while one result is ambiguous/in flight.
        self.assertEqual(sql(
          "SELECT count(*) FROM ipat_platform.claim_next_pppoe_execution_item("
          f"'{T}','{ATTEMPT}','worker-r1010')",
          "ipat_pppoe_worker_exec").stdout.strip(),"0")
        self.assertEqual(sql(
          "SELECT ipat_platform.record_pppoe_execution_item_result("
          f"'{T}','{ATTEMPT}',1,'worker-r1010','unknown_reconcile_required','{'8'*64}')",
          "ipat_pppoe_worker_exec").stdout.strip(),"t")
        self.assertEqual(sql(
          "SELECT count(*) FROM ipat_platform.claim_next_pppoe_execution_item("
          f"'{T}','{ATTEMPT}','worker-r1010')",
          "ipat_pppoe_worker_exec").stdout.strip(),"0")
        self.assertEqual(sql(
          "SELECT ipat_platform.reconcile_pppoe_execution_item("
          f"'{T}','{ATTEMPT}',1,'worker-r1010','readback_verified','{'9'*64}')",
          "ipat_pppoe_worker_exec").stdout.strip(),"t")
        # Move synthetic previous claim beyond configured rate interval.
        sql(f"UPDATE ipat_ops.pppoe_execution_items SET claimed_at=clock_timestamp()-interval '5 seconds' WHERE tenant_id='{T}' AND attempt_id='{ATTEMPT}' AND ordinal=1")
        second=sql(
          "SELECT ordinal FROM ipat_platform.claim_next_pppoe_execution_item("
          f"'{T}','{ATTEMPT}','worker-r1010')",
          "ipat_pppoe_worker_exec").stdout.strip()
        self.assertEqual(second,"2")

    def test_04_expired_claim_is_unknown_never_automatic_retry(self):
        sql(f"UPDATE ipat_ops.pppoe_execution_items SET claimed_at=clock_timestamp()-interval '60 seconds', claim_expires_at=clock_timestamp()-interval '1 second' WHERE tenant_id='{T}' AND attempt_id='{ATTEMPT}' AND ordinal=2")
        self.assertEqual(sql(
          "SELECT ipat_platform.expire_pppoe_execution_claims_to_unknown("
          f"'{T}','{ATTEMPT}','watchdog-r1010')",
          "ipat_pppoe_worker_exec").stdout.strip(),"1")
        row=sql(
          "SELECT state||'|'||COALESCE(outcome_sha256,'NULL') FROM ipat_ops.pppoe_execution_items "
          f"WHERE tenant_id='{T}' AND attempt_id='{ATTEMPT}' AND ordinal=2").stdout.strip()
        self.assertEqual(row,"unknown_reconcile_required|NULL")
        self.assertEqual(sql(
          "SELECT count(*) FROM ipat_platform.claim_next_pppoe_execution_item("
          f"'{T}','{ATTEMPT}','worker-r1010')",
          "ipat_pppoe_worker_exec").stdout.strip(),"0")
        self.assertEqual(sql(
          "SELECT ipat_platform.reconcile_pppoe_execution_item("
          f"'{T}','{ATTEMPT}',2,'worker-r1010','rolled_back','{'a'*64}')",
          "ipat_pppoe_worker_exec").stdout.strip(),"t")
        final=sql(
          "SELECT x.state||'|'||p.execution_allowed::int "
          "FROM ipat_ops.pppoe_execution_attempts x JOIN ipat_ops.pppoe_batch_plans p "
          "ON p.tenant_id=x.tenant_id AND p.id=x.plan_id "
          f"WHERE x.tenant_id='{T}' AND x.id='{ATTEMPT}'").stdout.strip()
        self.assertEqual(final,"rolled_back|0")

    def test_05_no_network_adapter_or_generic_execution_function(self):
        for signature in [
          "ipat_platform.execute_pppoe_batch(uuid)",
          "ipat_platform.routeros_command(text)",
          "ipat_platform.execute_router_command(text)"
        ]:
            self.assertEqual(sql(
              f"SELECT (to_regprocedure('{signature}') IS NULL)::int"
            ).stdout.strip(),"1")
        status=sql(
          "SELECT state||'|'||physical_execution_adapter_enabled::int "
          "FROM ipat_platform.get_pppoe_execution_status("
          f"'{ISS}','{CHECKER}','{T}','{PLAN}')",
          "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(status,"rolled_back|0")

if __name__=="__main__":
    unittest.main()
