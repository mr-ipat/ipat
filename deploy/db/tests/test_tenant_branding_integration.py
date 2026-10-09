"""R10.1 tenant branding: current membership, admin-only CAS, no cross-tenant/raw access."""
import os
import subprocess
import unittest

T1="10101010-1010-4010-8010-101010101011"
T2="10101010-1010-4010-8010-101010101012"
ISS="https://id.r101.synthetic.invalid/realms/ipat"
ADMIN="r101-admin"
HELP="r101-helpdesk"
OTHER="r101-other-admin"
REQ1="10101010-1010-4010-8010-101010101021"
REQ2="10101010-1010-4010-8010-101010101022"

def sql(statement, role=None, check=True):
    q=(f"SET ROLE {role};" if role else "")+statement
    return subprocess.run(
        ["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",q],
        text=True,capture_output=True,check=check)

def get(subject, tenant=T1):
    return sql(
        "SELECT tenant_slug||'|'||display_name||'|'||mark_text||'|'||accent_token||'|'||revision "
        "FROM ipat_platform.get_tenant_branding_for_member("
        f"'{ISS}','{subject}','{tenant}'::uuid)",
        "ipat_tenant_api_login").stdout.strip()

def set_brand(subject, request, expected, name, mark, accent, tenant=T1):
    return sql(
        "SELECT ipat_platform.set_tenant_branding("
        f"'{ISS}','{subject}','{tenant}'::uuid,'{request}'::uuid,{expected},"
        f"'{name}','{mark}','{accent}')",
        "ipat_tenant_api_login").stdout.strip()

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable DB only")
class Branding(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        sql(f"""
        INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
          ('{T1}','r101-one','active'),('{T2}','r101-two','active')
        ON CONFLICT(id) DO UPDATE SET state='active';
        INSERT INTO ipat_platform.identity_memberships(
          tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
          ('{T1}','{ISS}','{ADMIN}','tenant_admin','r101-reviewer',clock_timestamp()+interval '1 day'),
          ('{T1}','{ISS}','{HELP}','helpdesk','r101-reviewer',clock_timestamp()+interval '1 day'),
          ('{T2}','{ISS}','{OTHER}','tenant_admin','other-reviewer',clock_timestamp()+interval '1 day')
        ON CONFLICT(tenant_id,issuer,subject,role) DO UPDATE
          SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day';
        """)

    def test_01_default_visible_only_to_current_member(self):
        self.assertEqual(get(ADMIN),"r101-one|r101-one|R1|slate|0")
        self.assertEqual(get(HELP),"r101-one|r101-one|R1|slate|0")
        self.assertEqual(get(OTHER),"")
        self.assertEqual(get("missing"),"")

    def test_02_admin_cas_idempotency_and_reader_visibility(self):
        self.assertEqual(set_brand(ADMIN,REQ1,0,"North Fiber","NF","indigo"),"1")
        self.assertEqual(set_brand(ADMIN,REQ1,0,"North Fiber","NF","indigo"),"1")
        self.assertEqual(set_brand(ADMIN,REQ1,0,"Forged","XX","rose"),"")
        self.assertEqual(get(HELP),"r101-one|North Fiber|NF|indigo|1")
        self.assertEqual(set_brand(ADMIN,REQ2,0,"Stale","ST","blue"),"")
        self.assertEqual(set_brand(ADMIN,REQ2,1,"North Fiber NOC","NF","emerald"),"2")
        self.assertEqual(get(ADMIN),"r101-one|North Fiber NOC|NF|emerald|2")
        self.assertEqual(sql(
            f"SELECT count(*) FROM ipat_platform.tenant_branding_events WHERE tenant_id='{T1}'"
        ).stdout.strip(),"2")

    def test_03_non_admin_and_cross_tenant_cannot_mutate(self):
        self.assertEqual(set_brand(HELP,
            "10101010-1010-4010-8010-101010101023",2,"Helpdesk Brand","HB","amber"),"")
        self.assertEqual(set_brand(OTHER,
            "10101010-1010-4010-8010-101010101024",2,"Cross Tenant","CT","rose"),"")
        self.assertEqual(get(ADMIN),"r101-one|North Fiber NOC|NF|emerald|2")

    def test_04_revocation_and_suspension_are_dynamic(self):
        sql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() "
            f"WHERE tenant_id='{T1}' AND issuer='{ISS}' AND subject='{HELP}' AND role='helpdesk'")
        self.assertEqual(get(HELP),"")
        sql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL "
            f"WHERE tenant_id='{T1}' AND issuer='{ISS}' AND subject='{HELP}' AND role='helpdesk'")
        sql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{T1}'")
        self.assertEqual(get(ADMIN),"")
        self.assertEqual(set_brand(ADMIN,
            "10101010-1010-4010-8010-101010101025",2,"Blocked","BL","slate"),"")
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id='{T1}'")

    def test_05_raw_tables_and_set_function_not_granted_to_issuer(self):
        for role in ("ipat_tenant_api_login","ipat_oidc_session_issuer_login","ipat_app_runtime"):
            denied=sql("SELECT count(*) FROM ipat_platform.tenant_branding",role,check=False)
            self.assertNotEqual(denied.returncode,0)
            denied=sql("SELECT count(*) FROM ipat_platform.tenant_branding_events",role,check=False)
            self.assertNotEqual(denied.returncode,0)
        self.assertEqual(sql(
            "SELECT has_function_privilege('ipat_oidc_session_issuer_login',"
            "'ipat_platform.set_tenant_branding(text,text,uuid,uuid,bigint,text,text,text)',"
            "'EXECUTE')::int").stdout.strip(),"0")
        self.assertEqual(sql(
            "SELECT has_function_privilege('ipat_tenant_api_login',"
            "'ipat_platform.set_tenant_branding(text,text,uuid,uuid,bigint,text,text,text)',"
            "'EXECUTE')::int").stdout.strip(),"1")

    def test_06_current_commercial_roles_can_read_but_not_write(self):
        expanded = sql(
            "SELECT pg_get_constraintdef(oid) LIKE '%provisioning_officer%' "
            "FROM pg_constraint WHERE conname='identity_memberships_role_check'"
        ).stdout.strip()
        if expanded != "t":
            if os.getenv("IPAT_R1015_FULL_ROLES_EXPECTED") == "1":
                self.fail("R10.09 nine-role schema missing from canonical integration")
            self.skipTest("Older isolated pre-0039 role vocabulary; current CI runs full migration")
        for i, role in enumerate(("system_admin", "security_admin", "noc_manager",
                                  "provisioning_officer", "field_technician")):
            subject = "r101-member-" + role
            sql(f"INSERT INTO ipat_platform.identity_memberships("
                f"tenant_id,issuer,subject,role,approved_by,expires_at) VALUES("
                f"'{T1}','{ISS}','{subject}','{role}','r101-reviewer',"
                f"clock_timestamp()+interval '1 day') "
                f"ON CONFLICT(tenant_id,issuer,subject,role) DO UPDATE "
                f"SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day'")
            self.assertEqual(get(subject), "r101-one|North Fiber NOC|NF|emerald|2", role)
            request = f"10101010-1010-4010-8010-10101010103{i}"
            self.assertEqual(set_brand(subject,request,2,"Unauthorized","NO","amber"), "", role)

if __name__=="__main__":
    unittest.main()
