"""R9.53 disposable PostgreSQL custom-domain enrollment on R9.52 registry.
No public DNS, TLS issuance, customer account or live database is used here.
"""
import os
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA, TB, run, sql

ROOT = Path(__file__).resolve().parents[3]
MIGRATION = ROOT / "deploy/db/migrations/0013_tenant_domain_enrollment.sql"
ISSUER = "https://id.example.invalid/realms/ipat"

def role_query(role, query, expect=True):
    return sql(f"SET ROLE {role}; {query}", expect=expect)

def rows(result):
    return [line for line in result.stdout.strip().splitlines() if line and line != "SET"]

class TenantDomainEnrollmentIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get("IPAT_PG_EPHEMERAL_TEST") != "1":
            raise unittest.SkipTest("Explicit ephemeral PostgreSQL test only")
        assert os.environ.get("PGDATABASE") == "ipat_synthetic"
        assert os.environ.get("PGHOST") == "127.0.0.1"
        assert sql("SELECT to_regclass('ipat_platform.tenant_domains') IS NOT NULL").stdout.strip() == "t"
        assert sql("""SELECT to_regprocedure(
          'ipat_platform.resolve_verified_tenant_domain(text)'
        ) IS NOT NULL""").stdout.strip() == "t"
        assert sql("""SELECT to_regprocedure(
          'ipat_platform.request_tenant_custom_domain(text,text,uuid,text,text,text)'
        ) IS NULL""").stdout.strip() == "t"
        run(["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(MIGRATION)])

        sql(f"""
          INSERT INTO ipat_platform.identity_memberships(
            tenant_id,issuer,subject,role,approved_by,expires_at
          ) VALUES
          ('{TA}','{ISSUER}','tenant-a-admin','tenant_admin',
             'ci-domain-security','2099-01-01T00:00:00Z'),
          ('{TB}','{ISSUER}','tenant-b-admin','tenant_admin',
             'ci-domain-security','2099-01-01T00:00:00Z')
          ON CONFLICT (tenant_id,issuer,subject,role) DO UPDATE
          SET revoked_at=NULL,expires_at='2099-01-01T00:00:00Z'
        """)

    def test_00_roles_and_functions_are_sealed(self):
        roles = sql("""SELECT rolname,rolcanlogin::int,rolsuper::int,
                              rolbypassrls::int,rolinherit::int
                       FROM pg_roles WHERE rolname IN
                       ('ipat_domain_enrollment_owner','ipat_domain_admin')
                       ORDER BY rolname""").stdout.strip().splitlines()
        self.assertEqual(roles, [
            "ipat_domain_admin|0|0|0|0",
            "ipat_domain_enrollment_owner|0|0|0|0",
        ])
        for privilege in ("SELECT", "INSERT", "UPDATE", "DELETE", "TRUNCATE"):
            got = sql(
                "SELECT has_table_privilege("
                f"'ipat_domain_admin','ipat_platform.tenant_domains','{privilege}')::int"
            ).stdout.strip()
            self.assertEqual(got, "0", privilege)
        denied = role_query(
            "ipat_domain_admin",
            "SELECT fqdn FROM ipat_platform.tenant_domains",
            expect=False,
        )
        self.assertNotEqual(denied.returncode, 0)

        owner = sql("""SELECT r.rolname,p.prosecdef::int,p.proconfig::text
          FROM pg_proc p
          JOIN pg_namespace n ON n.oid=p.pronamespace
          JOIN pg_roles r ON r.oid=p.proowner
          WHERE n.nspname='ipat_platform'
            AND p.proname='request_tenant_custom_domain'""").stdout.strip()
        self.assertIn("ipat_domain_enrollment_owner|1|", owner)
        self.assertIn("search_path=pg_catalog, ipat_platform", owner)

    def test_01_pre_r953_verified_custom_domain_stays_resolvable(self):
        row = sql("""SELECT state,
                            (ownership_verified_at IS NOT NULL)::int,
                            routing_ready::int,tls_ready::int
                     FROM ipat_platform.tenant_domains
                     WHERE fqdn='ipat.fadly.id'""").stdout.strip()
        self.assertEqual(row, "verified|1|1|1")
        resolved = role_query(
            "ipat_domain_query",
            "SELECT tenant_slug FROM "
            "ipat_platform.resolve_verified_tenant_domain('ipat.fadly.id')",
        ).stdout.strip().splitlines()
        self.assertEqual(resolved[-1], "tenant-beta")

    def test_02_tenant_admin_requests_pending_domain_and_lists_own(self):
        result = role_query(
            "ipat_domain_admin",
            f"""SELECT COALESCE(ipat_platform.request_tenant_custom_domain(
              '{ISSUER}','tenant-a-admin','{TA}',
              'portal-a.domain-a.co.id','a_record',
              'ipat-domain=11111111-2222-4333-8444-555555555555'
            ),'NULL')""",
        )
        self.assertEqual(rows(result), ["portal-a.domain-a.co.id"])

        listed = role_query(
            "ipat_domain_admin",
            f"""SELECT fqdn||'|'||state||'|'||routing_mode||'|'||
                       verification_name||'|'||routing_ready::int||'|'||tls_ready::int
                FROM ipat_platform.list_tenant_domains_for_member(
                  '{ISSUER}','tenant-a-admin','{TA}'
                )
                WHERE fqdn='portal-a.domain-a.co.id'""",
        )
        self.assertEqual(rows(listed), [
            "portal-a.domain-a.co.id|pending|a_record|"
            "_ipat-verify.portal-a.domain-a.co.id|0|0"
        ])

    def test_03_duplicate_hostname_cross_tenant_returns_no_owner_metadata(self):
        seed = role_query(
            "ipat_domain_admin",
            f"""SELECT COALESCE(ipat_platform.request_tenant_custom_domain(
              '{ISSUER}','tenant-a-admin','{TA}',
              'duplicate.domain-a.co.id','cname',
              'ipat-domain=22222222-3333-4444-8555-666666666666'
            ),'NULL')""",
        )
        self.assertEqual(rows(seed), ["duplicate.domain-a.co.id"])
        other = role_query(
            "ipat_domain_admin",
            f"""SELECT COALESCE(ipat_platform.request_tenant_custom_domain(
              '{ISSUER}','tenant-b-admin','{TB}',
              'duplicate.domain-a.co.id','nameserver',
              'ipat-domain=33333333-4444-4555-8666-777777777777'
            ),'NULL')""",
        )
        self.assertEqual(rows(other), ["NULL"])
        hidden = role_query(
            "ipat_domain_admin",
            f"""SELECT count(*) FROM ipat_platform.list_tenant_domains_for_member(
                  '{ISSUER}','tenant-b-admin','{TB}'
                ) WHERE fqdn='duplicate.domain-a.co.id'""",
        )
        self.assertEqual(rows(hidden), ["0"])

    def test_04_wrong_identity_cannot_list_or_revoke_other_tenant(self):
        hidden = role_query(
            "ipat_domain_admin",
            f"""SELECT count(*) FROM ipat_platform.list_tenant_domains_for_member(
                  '{ISSUER}','tenant-b-admin','{TA}'
                )""",
        )
        self.assertEqual(rows(hidden), ["0"])
        revoked = role_query(
            "ipat_domain_admin",
            f"""SELECT ipat_platform.revoke_tenant_custom_domain(
                  '{ISSUER}','tenant-b-admin','{TA}','portal-a.domain-a.co.id'
                )""",
        )
        self.assertEqual(rows(revoked), ["f"])

    def test_05_owner_can_revoke_pending_domain_and_it_never_resolves(self):
        seed = role_query(
            "ipat_domain_admin",
            f"""SELECT COALESCE(ipat_platform.request_tenant_custom_domain(
              '{ISSUER}','tenant-a-admin','{TA}',
              'revoke.domain-a.co.id','a_record',
              'ipat-domain=44444444-5555-4666-8777-888888888888'
            ),'NULL')""",
        )
        self.assertEqual(rows(seed), ["revoke.domain-a.co.id"])
        revoked = role_query(
            "ipat_domain_admin",
            f"""SELECT ipat_platform.revoke_tenant_custom_domain(
                  '{ISSUER}','tenant-a-admin','{TA}','revoke.domain-a.co.id'
                )""",
        )
        self.assertEqual(rows(revoked), ["t"])
        state = sql("""SELECT state||'|'||routing_ready::int||'|'||tls_ready::int
                       FROM ipat_platform.tenant_domains
                       WHERE fqdn='revoke.domain-a.co.id'""").stdout.strip()
        self.assertEqual(state, "revoked|0|0")
        resolved = role_query(
            "ipat_domain_query",
            "SELECT count(*) FROM "
            "ipat_platform.resolve_verified_tenant_domain('revoke.domain-a.co.id')",
        )
        self.assertEqual(rows(resolved), ["0"])

if __name__ == "__main__":
    unittest.main()
