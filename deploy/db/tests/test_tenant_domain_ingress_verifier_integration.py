"""R9.94 disposable PostgreSQL ordered customer-domain ingress verifier authority."""
import os, subprocess, unittest, uuid

def sql(statement, role=None, check=True):
    prefix=f"SET ROLE {role};" if role else ""
    return subprocess.run(
        ["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",prefix+statement],
        text=True,capture_output=True,check=check
    )

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable PostgreSQL only")
class R994Pg(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        cls.tenant=str(uuid.uuid4())
        cls.domain=str(uuid.uuid4())
        cls.pending=str(uuid.uuid4())
        cls.inactive_tenant=str(uuid.uuid4())
        cls.inactive_domain=str(uuid.uuid4())
        sql(f"""
        INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
          VALUES('{cls.tenant}','r994-active','active'),
                ('{cls.inactive_tenant}','r994-inactive','suspended');
        INSERT INTO ipat_platform.tenant_domains(
          id,tenant_id,hostname,domain_type,verification_state,verification_method,
          verified_at,routing_mode,verification_name,verification_value,
          requested_by_issuer,requested_by_subject,requested_at,
          activation_state,ownership_verified_at)
        VALUES
          ('{cls.domain}','{cls.tenant}','r994.customer.invalid.example.net','custom_domain',
           'verified','dns_txt',clock_timestamp(),'a_record',
           '_ipat-verify.r994.customer.invalid.example.net','ipat-domain={cls.domain}',
           'https://r994.invalid','admin',clock_timestamp(),'ownership_verified',clock_timestamp()),
          ('{cls.pending}','{cls.tenant}','pending-r994.customer.invalid.example.net','custom_domain',
           'pending','dns_txt',NULL,'a_record',
           '_ipat-verify.pending-r994.customer.invalid.example.net','ipat-domain={cls.pending}',
           'https://r994.invalid','admin',clock_timestamp(),'pending_dns',NULL),
          ('{cls.inactive_domain}','{cls.inactive_tenant}','inactive-r994.customer.invalid.example.net','custom_domain',
           'verified','dns_txt',clock_timestamp(),'a_record',
           '_ipat-verify.inactive-r994.customer.invalid.example.net','ipat-domain={cls.inactive_domain}',
           'https://r994.invalid','admin',clock_timestamp(),'ownership_verified',clock_timestamp());
        """)

    def test_candidate_visibility_and_raw_table_denial(self):
        out=sql("SELECT id::text||'|'||routing_mode||'|'||verification_name||'|'||activation_state FROM ipat_platform.list_tenant_domain_ingress_checks(25)",
                "ipat_domain_ingress_verifier_login").stdout.strip().splitlines()
        self.assertIn(f"{self.domain}|a_record|_ipat-verify.r994.customer.invalid.example.net|ownership_verified",out)
        self.assertFalse(any(self.pending in x for x in out))
        self.assertFalse(any(self.inactive_domain in x for x in out))
        denied=sql("SELECT count(*) FROM ipat_platform.tenant_domains",
                   "ipat_domain_ingress_verifier_login",check=False)
        self.assertNotEqual(denied.returncode,0)
        tenant_role=sql("SELECT * FROM ipat_platform.list_tenant_domain_ingress_checks(1)",
                        "ipat_tenant_api_login",check=False)
        self.assertNotEqual(tenant_role.returncode,0)

    def test_dns_ownership_role_cannot_advance_ingress_and_ingress_cannot_claim_ownership(self):
        denied=sql(f"SELECT ipat_platform.record_tenant_domain_ingress_check('{self.domain}','routing_ready','{'9'*64}',NULL)",
                   "ipat_domain_verifier_login",check=False)
        self.assertNotEqual(denied.returncode,0)
        denied2=sql(f"SELECT ipat_platform.record_tenant_domain_ownership_check('{self.domain}','ownership_verified','{'8'*64}',NULL)",
                    "ipat_domain_ingress_verifier_login",check=False)
        self.assertNotEqual(denied2.returncode,0)
        broad_dns=sql(f"SELECT ipat_platform.record_tenant_domain_check('{self.domain}','routing_ready','{'7'*64}',NULL)",
                      "ipat_domain_verifier_login",check=False)
        self.assertNotEqual(broad_dns.returncode,0)

    def test_ingress_error_cannot_touch_pending_dns_scope(self):
        result=sql(f"SELECT ipat_platform.record_tenant_domain_ingress_check('{self.pending}','check_failed',NULL,'INGRESS_FAIL')",
                   "ipat_domain_ingress_verifier_login").stdout.strip()
        self.assertEqual(result,"")
        last=sql(f"SELECT COALESCE(last_error_code,'') FROM ipat_platform.tenant_domains WHERE id='{self.pending}'").stdout.strip()
        self.assertEqual(last,"")

    def test_order_cannot_be_skipped_and_exact_sequence_works(self):
        h1="1"*64;h2="2"*64;h3="3"*64
        early=sql(f"SELECT ipat_platform.record_tenant_domain_ingress_check('{self.domain}','tls_ready','{h2}',NULL)",
                  "ipat_domain_ingress_verifier_login").stdout.strip()
        self.assertEqual(early,"")
        routing=sql(f"SELECT ipat_platform.record_tenant_domain_ingress_check('{self.domain}','routing_ready','{h1}',NULL)",
                    "ipat_domain_ingress_verifier_login").stdout.strip()
        self.assertEqual(routing,"routing_ready")
        early_active=sql(f"SELECT ipat_platform.record_tenant_domain_ingress_check('{self.domain}','activate','{h3}',NULL)",
                         "ipat_domain_ingress_verifier_login").stdout.strip()
        self.assertEqual(early_active,"")
        tls=sql(f"SELECT ipat_platform.record_tenant_domain_ingress_check('{self.domain}','tls_ready','{h2}',NULL)",
                "ipat_domain_ingress_verifier_login").stdout.strip()
        self.assertEqual(tls,"tls_ready")
        active=sql(f"SELECT ipat_platform.record_tenant_domain_ingress_check('{self.domain}','activate','{h3}',NULL)",
                   "ipat_domain_ingress_verifier_login").stdout.strip()
        self.assertEqual(active,"active")
        row=sql(f"SELECT activation_state||'|'||(routing_ready_at IS NOT NULL)::int||'|'||(tls_ready_at IS NOT NULL)::int||'|'||(activated_at IS NOT NULL)::int FROM ipat_platform.tenant_domains WHERE id='{self.domain}'").stdout.strip()
        self.assertEqual(row,"active|1|1|1")

if __name__=="__main__":
    unittest.main()
