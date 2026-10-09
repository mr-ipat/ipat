"""R10.11 disposable PostgreSQL company activation maker/checker acceptance."""
import os, subprocess, unittest
ISS="https://r1011.synthetic.invalid/realm/platform"
OWNER1="owner-r1011-maker"
OWNER2="owner-r1011-checker"
TENANT="11111111-1010-4010-8010-111111111001"
RESERVATION="11111111-1010-4010-8010-111111111002"
REQUEST="11111111-1010-4010-8010-111111111003"
DOMAIN="11111111-1010-4010-8010-111111111004"
ADMIN_ISS="https://id.r1011.synthetic.invalid/realm/customer"
ADMIN_SUB="initial-admin-r1011"
HOST="portal.r1011-customer.net"
SHA="1"*64

def sql(q,role=None,check=True):
    prefix=f"SET ROLE {role};" if role else ""
    return subprocess.run(["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",prefix+q],
        text=True,capture_output=True,check=check)

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable only")
class Activation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        cls.admin_exp=sql("SELECT (clock_timestamp()+interval '30 days')::text").stdout.strip()
        sql(f"""INSERT INTO ipat_platform.platform_principals(
          issuer,subject,role,approved_by,expires_at) VALUES
          ('{ISS}','{OWNER1}','platform_owner','r1011-independent',now()+interval '1 day'),
          ('{ISS}','{OWNER2}','platform_owner','r1011-independent',now()+interval '1 day');
          INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
          ('{TENANT}','r1011-company','suspended');
          INSERT INTO ipat_platform.platform_tenant_reservations(
          request_id,tenant_id,tenant_slug,requested_by_issuer,requested_by_subject)
          VALUES('{RESERVATION}','{TENANT}','r1011-company','{ISS}','{OWNER1}');""")
    def test_00_owner_visibility_and_direct_table_shape_are_exact(self):
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.platform_principals WHERE issuer='{ISS}' AND subject='{OWNER1}'", "ipat_company_activation_owner").stdout.strip(),"1")
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.platform_tenant_reservations WHERE tenant_id='{TENANT}'", "ipat_company_activation_owner").stdout.strip(),"1")
        direct=sql(f"""BEGIN; SET LOCAL ROLE ipat_company_activation_owner;
          INSERT INTO ipat_platform.platform_tenant_activation_requests(
          request_id,tenant_id,domain_id,customer_hostname,routing_mode,admin_issuer,admin_subject,
          admin_expires_at,evidence_sha256,requested_by_issuer,requested_by_subject) VALUES(
          '11111111-1010-4010-8010-111111111099','{TENANT}',
          '11111111-1010-4010-8010-111111111098','debug.r1011-customer.net','a_record',
          '{ADMIN_ISS}','{ADMIN_SUB}','{self.admin_exp}','{SHA}','{ISS}','{OWNER1}'); ROLLBACK;""",check=False)
        self.assertEqual(direct.returncode,0,direct.stderr)

    def request(self,subject=OWNER1,request=REQUEST,host=HOST,sha=SHA):
        return sql(f"""SELECT ipat_platform.request_company_activation(
          '{ISS}','{subject}','{request}','{TENANT}','{DOMAIN}','{host}','a_record',
          '{ADMIN_ISS}','{ADMIN_SUB}','{self.admin_exp}','{sha}')""",
          "ipat_platform_session_owner").stdout.strip()
    def test_01_request_is_idempotent_and_wrong_owner_or_domain_denied(self):
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.platform_principals WHERE issuer=\'{ISS}\' AND subject=\'{OWNER1}\'", "ipat_company_activation_owner").stdout.strip(),"1")
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.platform_tenant_reservations WHERE tenant_id=\'{TENANT}\'", "ipat_company_activation_owner").stdout.strip(),"1")
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.platform_tenant_activation_requests WHERE request_id=\'{REQUEST}\'").stdout.strip(),"0")
        first=self.request()
        persisted=sql(f"SELECT count(*) FROM ipat_platform.platform_tenant_activation_requests WHERE request_id=\'{REQUEST}\'").stdout.strip()
        self.assertEqual(first,REQUEST,repr((first,persisted)))
        self.assertEqual(self.request(),REQUEST)
        self.assertEqual(self.request(subject="missing"),"")
        self.assertEqual(self.request(request=REQUEST,host="other.customer.net"),"")
        self.assertEqual(sql(f"SELECT state FROM ipat_platform.tenants WHERE id='{TENANT}'").stdout.strip(),"suspended")
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.identity_memberships WHERE tenant_id='{TENANT}'").stdout.strip(),"0")
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.tenant_domains WHERE tenant_id='{TENANT}'").stdout.strip(),"0")
    def test_02_maker_cannot_review_checker_can_approve_once(self):
        same=sql(f"SELECT ipat_platform.review_company_activation('{ISS}','{OWNER1}','{REQUEST}',true)",
          "ipat_platform_session_owner").stdout.strip()
        self.assertEqual(same,"f")
        ok=sql(f"SELECT ipat_platform.review_company_activation('{ISS}','{OWNER2}','{REQUEST}',true)",
          "ipat_platform_session_owner").stdout.strip()
        self.assertEqual(ok,"t")
        again=sql(f"SELECT ipat_platform.review_company_activation('{ISS}','{OWNER2}','{REQUEST}',true)",
          "ipat_platform_session_owner").stdout.strip()
        self.assertEqual(again,"f")
        self.assertEqual(sql(f"SELECT state FROM ipat_platform.tenants WHERE id='{TENANT}'").stdout.strip(),"active")
        membership=sql(f"""SELECT role||'|'||issuer||'|'||subject FROM ipat_platform.identity_memberships
          WHERE tenant_id='{TENANT}'""").stdout.strip()
        self.assertEqual(membership,f"tenant_admin|{ADMIN_ISS}|{ADMIN_SUB}")
        domain=sql(f"""SELECT hostname||'|'||verification_state||'|'||activation_state||'|'||verification_value
          FROM ipat_platform.tenant_domains WHERE tenant_id='{TENANT}'""").stdout.strip()
        self.assertEqual(domain,f"{HOST}|pending|pending_dns|ipat-domain={DOMAIN}")
    def test_03_no_runtime_raw_table_or_tenant_role_escalation(self):
        for role in ("ipat_tenant_api_login","ipat_oidc_session_issuer_login","ipat_app_runtime"):
            denied=sql("SELECT count(*) FROM ipat_platform.platform_tenant_activation_requests",role,False)
            self.assertNotEqual(denied.returncode,0)
            privilege=sql(f"""SELECT has_function_privilege('{role}',
             'ipat_platform.review_company_activation(text,text,uuid,boolean)','EXECUTE')::int""").stdout.strip()
            self.assertEqual(privilege,"0")
    def test_04_approved_domain_still_cannot_issue_tenant_session(self):
        # Domain is intentionally only pending_dns; no browser tenant session yet.
        self.assertEqual(sql(f"""SELECT count(*) FROM ipat_platform.tenant_domains
          WHERE tenant_id='{TENANT}' AND activation_state='active'""").stdout.strip(),"0")
        self.assertEqual(sql(f"""SELECT count(*) FROM ipat_platform.tenant_browser_sessions
          WHERE tenant_id='{TENANT}'""").stdout.strip(),"0")
if __name__=="__main__": unittest.main()
