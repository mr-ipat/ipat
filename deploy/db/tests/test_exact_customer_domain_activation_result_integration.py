"""R9.98 exact customer-domain activation result is ingress-verifier-only."""
import os, subprocess, unittest, uuid

def sql(statement, role=None, check=True):
    prefix=f"SET ROLE {role};" if role else ""
    return subprocess.run(
        ["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",prefix+statement],
        text=True,capture_output=True,check=check
    )

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable PostgreSQL only")
class R998ExactActivationResult(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        cls.tenant=str(uuid.uuid4())
        cls.active=str(uuid.uuid4())
        cls.owned=str(uuid.uuid4())
        cls.pending=str(uuid.uuid4())
        cls.suspended_tenant=str(uuid.uuid4())
        cls.suspended_domain=str(uuid.uuid4())
        sql(f"""
        INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
          VALUES('{cls.tenant}','r998-active','active'),
                ('{cls.suspended_tenant}','r998-suspended','suspended');
        INSERT INTO ipat_platform.tenant_domains(
          id,tenant_id,hostname,domain_type,verification_state,verification_method,
          verified_at,routing_mode,verification_name,verification_value,
          requested_by_issuer,requested_by_subject,requested_at,
          activation_state,ownership_verified_at,routing_ready_at,tls_ready_at,activated_at)
        VALUES
          ('{cls.active}','{cls.tenant}','active-r998.customer.example.net','custom_domain',
           'verified','dns_txt',clock_timestamp(),'a_record',
           '_ipat-verify.active-r998.customer.example.net','ipat-domain={cls.active}',
           'https://r998.invalid','admin',clock_timestamp(),'active',
           clock_timestamp(),clock_timestamp(),clock_timestamp(),clock_timestamp()),
          ('{cls.owned}','{cls.tenant}','owned-r998.customer.example.net','custom_domain',
           'verified','dns_txt',clock_timestamp(),'a_record',
           '_ipat-verify.owned-r998.customer.example.net','ipat-domain={cls.owned}',
           'https://r998.invalid','admin',clock_timestamp(),'ownership_verified',
           clock_timestamp(),NULL,NULL,NULL),
          ('{cls.pending}','{cls.tenant}','pending-r998.customer.example.net','custom_domain',
           'pending','dns_txt',NULL,'a_record',
           '_ipat-verify.pending-r998.customer.example.net','ipat-domain={cls.pending}',
           'https://r998.invalid','admin',clock_timestamp(),'pending_dns',
           NULL,NULL,NULL,NULL),
          ('{cls.suspended_domain}','{cls.suspended_tenant}','suspended-r998.customer.example.net','custom_domain',
           'verified','dns_txt',clock_timestamp(),'a_record',
           '_ipat-verify.suspended-r998.customer.example.net','ipat-domain={cls.suspended_domain}',
           'https://r998.invalid','admin',clock_timestamp(),'active',
           clock_timestamp(),clock_timestamp(),clock_timestamp(),clock_timestamp());
        """)

    def result(self,domain,role="ipat_domain_ingress_verifier_login"):
        return sql(
            f"SELECT id::text||'|'||hostname||'|'||activation_state||'|'||"
            f"COALESCE(last_error_code,'')||'|'||(activated_at IS NOT NULL)::int "
            f"FROM ipat_platform.get_tenant_domain_activation_result('{domain}'::uuid)",
            role, check=False)

    def test_exact_active_and_owned_are_visible(self):
        active=self.result(self.active)
        self.assertEqual(active.returncode,0)
        self.assertIn(f"{self.active}|active-r998.customer.example.net|active||1",active.stdout)
        owned=self.result(self.owned)
        self.assertEqual(owned.returncode,0)
        self.assertIn(f"{self.owned}|owned-r998.customer.example.net|ownership_verified||0",owned.stdout)

    def test_pending_suspended_and_unknown_are_absent(self):
        for domain in (self.pending,self.suspended_domain,str(uuid.uuid4())):
            out=self.result(domain)
            self.assertEqual(out.returncode,0)
            self.assertEqual(out.stdout.strip(),"")

    def test_tenant_and_ownership_roles_cannot_use_result(self):
        for role in ("ipat_tenant_api_login","ipat_domain_verifier_login",
                     "ipat_oidc_session_issuer_login","ipat_platform_session_api_login"):
            out=self.result(self.active,role)
            self.assertNotEqual(out.returncode,0,role)

    def test_ingress_role_still_has_no_raw_table_access(self):
        out=sql("SELECT activation_state FROM ipat_platform.tenant_domains LIMIT 1",
                "ipat_domain_ingress_verifier_login",check=False)
        self.assertNotEqual(out.returncode,0)

    def test_result_tracks_ordered_state_without_mutation_authority_change(self):
        h1="a"*64;h2="b"*64;h3="c"*64
        for event,digest,expected in (
            ("routing_ready",h1,"routing_ready"),
            ("tls_ready",h2,"tls_ready"),
            ("activate",h3,"active"),
        ):
            transitioned=sql(
                f"SELECT ipat_platform.record_tenant_domain_ingress_check("
                f"'{self.owned}'::uuid,'{event}','{digest}',NULL)",
                "ipat_domain_ingress_verifier_login").stdout.strip()
            self.assertEqual(transitioned,expected)
            current=self.result(self.owned)
            self.assertEqual(current.returncode,0)
            self.assertIn(f"|{expected}|",current.stdout)
        final=self.result(self.owned).stdout
        self.assertTrue(final.strip().endswith("|active||1"))

if __name__=="__main__":
    unittest.main()
