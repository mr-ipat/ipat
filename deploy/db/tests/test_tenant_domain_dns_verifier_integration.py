"""R9.56 restricted DNS verifier queue integration on disposable PostgreSQL.

The queue function does not resolve real DNS; the Rust verifier does.
"""
import os
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[3]
MIGRATION = ROOT / "deploy/db/migrations/0015_tenant_domain_dns_verifier.sql"

def query(statement, check=True):
    return subprocess.run(
        ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-At", "-c", statement],
        env=os.environ.copy(), text=True, capture_output=True, check=check
    )

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST") == "1", "disposable PostgreSQL only")
class DomainDnsVerifierIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.environ.get("PGDATABASE") == "ipat_synthetic"
        if query("SELECT to_regprocedure(\x27ipat_platform.record_tenant_domain_check(uuid,text,text,text)\x27) IS NOT NULL").stdout.strip() != "t":
            raise RuntimeError("R9.56 requires canonical migrations through 0014")
        if query("SELECT to_regprocedure(\x27ipat_platform.list_tenant_domain_ownership_checks(integer)\x27) IS NOT NULL").stdout.strip() != "t":
            subprocess.run(
                ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(MIGRATION)],
                env=os.environ.copy(), check=True, capture_output=True
            )
        query("""
          INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
          VALUES
            (\x2799999999-9999-4999-8999-999999999991\x27,\x27r956-active\x27,\x27active\x27),
            (\x2799999999-9999-4999-8999-999999999992\x27,\x27r956-suspended\x27,\x27suspended\x27)
          ON CONFLICT (id) DO UPDATE SET state=EXCLUDED.state;
        """)
        query("""
          INSERT INTO ipat_platform.tenant_domains(
              id,tenant_id,hostname,domain_type,verification_state,
              verification_method,routing_mode,verification_name,
              verification_value,requested_by_issuer,requested_by_subject,
              requested_at,activation_state
          ) VALUES
            (\x2799999999-9999-4999-8999-999999999901\x27,
             \x2799999999-9999-4999-8999-999999999991\x27,
             \x27pending-r956.example.net\x27,\x27custom_domain\x27,\x27pending\x27,\x27dns_txt\x27,
             \x27a_record\x27,\x27_ipat-verify.pending-r956.example.net\x27,
             \x27ipat-domain=99999999-9999-4999-8999-999999999901\x27,
             \x27ci\x27,\x27ci\x27,now(),\x27pending_dns\x27),
            (\x2799999999-9999-4999-8999-999999999902\x27,
             \x2799999999-9999-4999-8999-999999999992\x27,
             \x27suspended-r956.example.net\x27,\x27custom_domain\x27,\x27pending\x27,\x27dns_txt\x27,
             \x27a_record\x27,\x27_ipat-verify.suspended-r956.example.net\x27,
             \x27ipat-domain=99999999-9999-4999-8999-999999999902\x27,
             \x27ci\x27,\x27ci\x27,now(),\x27pending_dns\x27)
          ON CONFLICT (id) DO NOTHING;
        """)

    def test_only_pending_active_tenant_challenge_is_returned(self):
        result=query("""
          SET ROLE ipat_domain_verifier_login;
          SELECT id::text || \x27|\x27 || verification_name || \x27|\x27 || verification_value
          FROM ipat_platform.list_tenant_domain_ownership_checks(100)
          WHERE hostname LIKE \x27%r956.example.net\x27;
        """)
        self.assertIn(
            "99999999-9999-4999-8999-999999999901|_ipat-verify.pending-r956.example.net|"
            "ipat-domain=99999999-9999-4999-8999-999999999901",
            result.stdout
        )
        self.assertNotIn("99999999-9999-4999-8999-999999999902", result.stdout)

    def test_invalid_batch_limit_yields_no_challenges(self):
        for limit in (0,-1,101,1000):
            result=query(f"""
              SET ROLE ipat_domain_verifier_login;
              SELECT count(*) FROM ipat_platform.list_tenant_domain_ownership_checks({limit});
            """)
            self.assertEqual(result.stdout.strip().splitlines()[-1], "0")

    def test_verifier_cannot_select_registry_or_read_other_roles(self):
        forbidden=query("""
          SET ROLE ipat_domain_verifier_login;
          SELECT verification_value FROM ipat_platform.tenant_domains;
        """, check=False)
        self.assertNotEqual(forbidden.returncode,0)
        self.assertIn("permission denied",forbidden.stderr.lower())

    def test_successful_ownership_transition_removes_queue_item(self):
        digest="a"*64
        result=query(f"""
          SET ROLE ipat_domain_verifier_login;
          SELECT ipat_platform.record_tenant_domain_check(
             \x2799999999-9999-4999-8999-999999999901\x27,
             \x27ownership_verified\x27,\x27{digest}\x27,NULL
          );
        """)
        self.assertIn("ownership_verified", result.stdout)
        result=query("""
          SET ROLE ipat_domain_verifier_login;
          SELECT count(*) FROM ipat_platform.list_tenant_domain_ownership_checks(100)
          WHERE hostname = \x27pending-r956.example.net\x27;
        """)
        self.assertEqual(result.stdout.strip().splitlines()[-1],"0")

if __name__=="__main__":
    unittest.main()
