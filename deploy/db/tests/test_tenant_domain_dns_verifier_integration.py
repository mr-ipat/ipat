"""R9.55 DNS ownership verifier queue contract.
Disposable PostgreSQL only. No external DNS queries are performed here.
"""
import os
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA, run, sql

ROOT = Path(__file__).resolve().parents[3]
MIGRATION = ROOT / "deploy/db/migrations/0015_tenant_domain_dns_verifier.sql"
ISSUER = "https://id.example.invalid/realms/ipat"

def role_query(role, query, expect=True):
    return sql(f"SET ROLE {role}; {query}", expect=expect)

def rows(result):
    return [line for line in result.stdout.strip().splitlines() if line and line != "SET"]

class TenantDomainDnsVerifierIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get("IPAT_PG_EPHEMERAL_TEST") != "1":
            raise unittest.SkipTest("Explicit ephemeral PostgreSQL test only")
        assert os.environ.get("PGDATABASE") == "ipat_synthetic"
        assert sql("""SELECT to_regprocedure(
          'ipat_platform.advance_tenant_domain_lifecycle(text,text,text)'
        ) IS NOT NULL""").stdout.strip() == "t"
        assert sql("""SELECT to_regprocedure(
          'ipat_platform.list_tenant_domain_ownership_checks(integer)'
        ) IS NULL""").stdout.strip() == "t"
        run(["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(MIGRATION)])
        sql(f"""
          INSERT INTO ipat_platform.identity_memberships(
            tenant_id,issuer,subject,role,approved_by,expires_at
          ) VALUES (
            '{TA}','{ISSUER}','r955-admin','tenant_admin',
            'ci-r955-security','2099-01-01T00:00:00Z'
          )
          ON CONFLICT (tenant_id,issuer,subject,role) DO UPDATE
          SET revoked_at=NULL,expires_at='2099-01-01T00:00:00Z'
        """)
        for fqdn, token in [
            ("pending-r955.domain-a.co.id","ipat-domain=95555555-2222-4333-8444-555555555551"),
            ("verified-r955.domain-a.co.id","ipat-domain=95555555-2222-4333-8444-555555555552"),
        ]:
            role_query(
                "ipat_domain_admin",
                f"""SELECT ipat_platform.request_tenant_custom_domain(
                  '{ISSUER}','r955-admin','{TA}','{fqdn}','a_record','{token}'
                )""",
            )
        role_query(
            "ipat_domain_verifier",
            """SELECT ipat_platform.advance_tenant_domain_lifecycle(
              'verified-r955.domain-a.co.id','ownership_verified','R955.DNS.TXT.MATCH'
            )""",
        )

    def test_00_verifier_can_list_only_pending_ownership_challenges(self):
        result = role_query(
            "ipat_domain_verifier",
            """SELECT fqdn||'|'||verification_name||'|'||verification_value
               FROM ipat_platform.list_tenant_domain_ownership_checks(25)""",
        )
        got = rows(result)
        self.assertIn(
            "pending-r955.domain-a.co.id|_ipat-verify.pending-r955.domain-a.co.id|"
            "ipat-domain=95555555-2222-4333-8444-555555555551",
            got,
        )
        self.assertFalse(any("verified-r955.domain-a.co.id" in row for row in got))

    def test_01_invalid_limits_fail_closed(self):
        for limit in (0, -1, 101, 1000):
            result = role_query(
                "ipat_domain_verifier",
                f"SELECT count(*) FROM ipat_platform.list_tenant_domain_ownership_checks({limit})",
            )
            self.assertEqual(rows(result), ["0"], limit)

    def test_02_verifier_still_has_no_direct_registry_read(self):
        denied = role_query(
            "ipat_domain_verifier",
            "SELECT verification_value FROM ipat_platform.tenant_domains",
            expect=False,
        )
        self.assertNotEqual(denied.returncode, 0)
        self.assertIn("permission denied", denied.stderr.lower())

    def test_03_queue_does_not_leak_other_domain_states(self):
        result = role_query(
            "ipat_domain_verifier",
            """SELECT count(*) FROM ipat_platform.list_tenant_domain_ownership_checks(100)
               WHERE fqdn='verified-r955.domain-a.co.id'""",
        )
        self.assertEqual(rows(result), ["0"])

if __name__ == "__main__":
    unittest.main()
