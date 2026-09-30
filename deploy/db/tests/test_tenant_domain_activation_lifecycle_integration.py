"""R9.54 ordered custom-domain activation lifecycle on the R9.52/R9.53 schema.
Disposable PostgreSQL only. No DNS query, certificate issuance, or live customer data.
"""
import os
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA, run, sql

ROOT = Path(__file__).resolve().parents[3]
MIGRATION = ROOT / "deploy/db/migrations/0014_tenant_domain_activation_lifecycle.sql"
ISSUER = "https://id.example.invalid/realms/ipat"
FQDN = "lifecycle.domain-a.co.id"

def role_query(role, query, expect=True):
    return sql(f"SET ROLE {role}; {query}", expect=expect)

def rows(result):
    return [line for line in result.stdout.strip().splitlines() if line and line != "SET"]

class TenantDomainLifecycleIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get("IPAT_PG_EPHEMERAL_TEST") != "1":
            raise unittest.SkipTest("Explicit ephemeral PostgreSQL test only")
        assert os.environ.get("PGDATABASE") == "ipat_synthetic"
        assert sql("""SELECT to_regprocedure(
          'ipat_platform.request_tenant_custom_domain(text,text,uuid,text,text,text)'
        ) IS NOT NULL""").stdout.strip() == "t"
        assert sql("""SELECT to_regprocedure(
          'ipat_platform.advance_tenant_domain_lifecycle(text,text,text)'
        ) IS NULL""").stdout.strip() == "t"
        run(["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(MIGRATION)])
        sql(f"""
          INSERT INTO ipat_platform.identity_memberships(
            tenant_id,issuer,subject,role,approved_by,expires_at
          ) VALUES (
            '{TA}','{ISSUER}','tenant-a-admin','tenant_admin',
            'ci-r954-security','2099-01-01T00:00:00Z'
          )
          ON CONFLICT (tenant_id,issuer,subject,role) DO UPDATE
          SET revoked_at=NULL,expires_at='2099-01-01T00:00:00Z'
        """)
        role_query(
            "ipat_domain_admin",
            f"""SELECT ipat_platform.request_tenant_custom_domain(
              '{ISSUER}','tenant-a-admin','{TA}','{FQDN}','a_record',
              'ipat-domain=95555555-2222-4333-8444-555555555555'
            )""",
        )

    def test_00_verifier_role_is_sealed(self):
        attrs = sql("""SELECT rolcanlogin::int||'|'||rolsuper::int||'|'||
                              rolbypassrls::int||'|'||rolinherit::int
                       FROM pg_roles WHERE rolname='ipat_domain_verifier'""").stdout.strip()
        self.assertEqual(attrs, "0|0|0|0")
        denied = role_query(
            "ipat_domain_verifier",
            "SELECT fqdn FROM ipat_platform.tenant_domains",
            expect=False,
        )
        self.assertNotEqual(denied.returncode, 0)
        for privilege in ("SELECT","INSERT","UPDATE","DELETE"):
            got = sql(
                "SELECT has_table_privilege("
                f"'ipat_domain_verifier','ipat_platform.tenant_domains','{privilege}')::int"
            ).stdout.strip()
            self.assertEqual(got, "0", privilege)

    def test_01_skip_to_routing_or_active_fails_closed(self):
        for target in ("routing_ready", "tls_ready", "active"):
            out = role_query(
                "ipat_domain_verifier",
                f"""SELECT ipat_platform.advance_tenant_domain_lifecycle(
                  '{FQDN}','{target}','R954.SKIP.DENIED'
                )""",
            )
            self.assertEqual(rows(out), ["f"], target)
        state = sql(f"""SELECT state||'|'||
                    (ownership_verified_at IS NOT NULL)::int||'|'||
                    routing_ready::int||'|'||tls_ready::int
                    FROM ipat_platform.tenant_domains WHERE fqdn='{FQDN}'""").stdout.strip()
        self.assertEqual(state, "pending|0|0|0")

    def test_02_ownership_then_routing_then_tls_then_active(self):
        sequence = [
            ("ownership_verified", "R954.DNS.TXT.MATCH"),
            ("routing_ready", "R954.INGRESS.ROUTE.READY"),
            ("tls_ready", "R954.TLS.CERT.READY"),
            ("active", "R954.ACTIVATION.APPROVED"),
        ]
        for target, evidence in sequence:
            out = role_query(
                "ipat_domain_verifier",
                f"""SELECT ipat_platform.advance_tenant_domain_lifecycle(
                  '{FQDN}','{target}','{evidence}'
                )""",
            )
            self.assertEqual(rows(out), ["t"], target)

        state = sql(f"""SELECT state||'|'||
                    (ownership_verified_at IS NOT NULL)::int||'|'||
                    routing_ready::int||'|'||tls_ready::int||'|'||
                    (verified_at IS NOT NULL)::int
                    FROM ipat_platform.tenant_domains WHERE fqdn='{FQDN}'""").stdout.strip()
        self.assertEqual(state, "verified|1|1|1|1")
        resolved = role_query(
            "ipat_domain_query",
            f"""SELECT count(*) FROM ipat_platform.resolve_verified_tenant_domain('{FQDN}')""",
        )
        self.assertEqual(rows(resolved), ["1"])

    def test_03_completed_domain_cannot_replay_transition(self):
        out = role_query(
            "ipat_domain_verifier",
            f"""SELECT ipat_platform.advance_tenant_domain_lifecycle(
              '{FQDN}','active','R954.REPLAY.DENIED'
            )""",
        )
        self.assertEqual(rows(out), ["f"])

    def test_04_error_recording_does_not_promote_state(self):
        other = "errorcase.domain-a.co.id"
        role_query(
            "ipat_domain_admin",
            f"""SELECT ipat_platform.request_tenant_custom_domain(
              '{ISSUER}','tenant-a-admin','{TA}','{other}','a_record',
              'ipat-domain=96666666-2222-4333-8444-555555555555'
            )""",
        )
        out = role_query(
            "ipat_domain_verifier",
            f"""SELECT ipat_platform.record_tenant_domain_lifecycle_error(
              '{other}','DNS.TXT.NOT_FOUND'
            )""",
        )
        self.assertEqual(rows(out), ["t"])
        state = sql(f"""SELECT state||'|'||
                    (ownership_verified_at IS NOT NULL)::int||'|'||
                    routing_ready::int||'|'||tls_ready::int||'|'||
                    lifecycle_last_error_code
                    FROM ipat_platform.tenant_domains WHERE fqdn='{other}'""").stdout.strip()
        self.assertEqual(state, "pending|0|0|0|DNS.TXT.NOT_FOUND")

if __name__ == "__main__":
    unittest.main()
