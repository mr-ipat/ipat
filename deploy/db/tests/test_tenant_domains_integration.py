"""R9.52 disposable PostgreSQL tenant-domain routing contract.
No real DNS, TLS certificate, customer account or live database is used here.
"""
import os
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA, TB, run, sql

ROOT = Path(__file__).resolve().parents[3]
MIGRATION = ROOT / "deploy/db/migrations/0012_tenant_domains.sql"
FN = "ipat_platform.resolve_verified_tenant_domain"
SIG = "(text)"

def role_query(role, query, expect=True):
    return sql(f"SET ROLE {role}; {query}", expect=expect)

def resolve(host):
    quoted = host.replace("'", "''")
    result = role_query(
        "ipat_domain_query",
        f"SELECT COALESCE((SELECT tenant_slug FROM {FN}('{quoted}') LIMIT 1),'DENIED');",
    )
    return result.stdout.strip().splitlines()[-1]

class TenantDomainsIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get("IPAT_PG_EPHEMERAL_TEST") != "1":
            raise unittest.SkipTest("Explicit ephemeral PostgreSQL test only")
        assert os.environ.get("PGDATABASE") == "ipat_synthetic"
        assert os.environ.get("PGHOST") == "127.0.0.1"
        assert os.environ.get("IPAT_PG_SYNTHETIC_PASSWORD") == "local_ci_synthetic_only"
        assert sql("SELECT to_regclass('ipat_platform.tenants') IS NOT NULL").stdout.strip() == "t"
        assert sql("SELECT to_regclass('ipat_platform.tenant_domains') IS NULL").stdout.strip() == "t"
        run(["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(MIGRATION)])
        assert sql("SELECT to_regrole('ipat_domain_reader') IS NULL").stdout.strip() == "t"
        sql("""CREATE ROLE ipat_domain_reader LOGIN INHERIT
          NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
          PASSWORD 'local_ci_synthetic_only'""")
        sql("GRANT ipat_domain_query TO ipat_domain_reader")
        sql(f"""INSERT INTO ipat_platform.tenant_domains
          (tenant_id,fqdn,domain_kind,verification_method,state,verified_at,tls_ready)
          VALUES
          ('{TA}','kangnet.ipat.id','platform_subdomain','platform_parent',
             'verified',statement_timestamp(),true),
          ('{TB}','ipat.fadly.id','custom_domain','dns_txt',
             'verified',statement_timestamp(),true)""")
    def setUp(self):
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
        sql("""UPDATE ipat_platform.tenant_domains
               SET state='verified',
                   verified_at=COALESCE(verified_at,statement_timestamp()),
                   tls_ready=true""")

    def test_00_only_sealed_query_role_can_execute(self):
        privileges = sql(f"""SELECT
          has_function_privilege('ipat_domain_query','{FN}{SIG}','EXECUTE')::int,
          has_function_privilege('ipat_app_runtime','{FN}{SIG}','EXECUTE')::int,
          has_function_privilege('ipat_domain_lookup_owner','{FN}{SIG}','EXECUTE')::int
        """).stdout.strip()
        self.assertEqual(privileges, "1|0|1")
        self.assertNotEqual(
            role_query(
                "ipat_app_runtime",
                f"SELECT * FROM {FN}('kangnet.ipat.id')",
                expect=False,
            ).returncode,
            0,
        )
        roles = sql("""SELECT rolname,rolcanlogin::int,rolsuper::int,rolbypassrls::int
                       FROM pg_roles WHERE rolname IN
                       ('ipat_domain_lookup_owner','ipat_domain_query')
                       ORDER BY rolname""").stdout.strip().splitlines()
        self.assertEqual(
            roles,
            ["ipat_domain_lookup_owner|0|0|0", "ipat_domain_query|0|0|0"],
        )

    def test_01_verified_tls_ready_domains_resolve_exactly(self):
        self.assertEqual(resolve("kangnet.ipat.id"), "kangnet")
        self.assertEqual(resolve("ipat.fadly.id"), "nengnet")
        self.assertEqual(resolve("IPAT.FADLY.ID"), "DENIED")
        self.assertEqual(resolve("other.ipat.id"), "DENIED")
    def test_02_pending_suspended_revoked_or_no_tls_fail_closed(self):
        sql("UPDATE ipat_platform.tenant_domains SET tls_ready=false WHERE fqdn='ipat.fadly.id'")
        self.assertEqual(resolve("ipat.fadly.id"), "DENIED")
        sql("""UPDATE ipat_platform.tenant_domains
               SET tls_ready=true,state='suspended' WHERE fqdn='ipat.fadly.id'""")
        self.assertEqual(resolve("ipat.fadly.id"), "DENIED")
        sql("UPDATE ipat_platform.tenant_domains SET state='revoked' WHERE fqdn='ipat.fadly.id'")
        self.assertEqual(resolve("ipat.fadly.id"), "DENIED")
        sql("""UPDATE ipat_platform.tenant_domains
               SET state='pending',verified_at=NULL WHERE fqdn='ipat.fadly.id'""")
        self.assertEqual(resolve("ipat.fadly.id"), "DENIED")

    def test_03_suspended_tenant_fails_without_affecting_other_tenant(self):
        sql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{TB}'")
        self.assertEqual(resolve("ipat.fadly.id"), "DENIED")
        self.assertEqual(resolve("kangnet.ipat.id"), "kangnet")

    def test_04_runtime_has_no_direct_domain_table_access(self):
        for role in ("ipat_domain_query", "ipat_app_runtime"):
            for privilege in ("SELECT", "INSERT", "UPDATE", "DELETE", "TRUNCATE"):
                got = sql(
                    f"SELECT has_table_privilege('{role}',"
                    f"'ipat_platform.tenant_domains','{privilege}')::int"
                ).stdout.strip()
                self.assertEqual(got, "0", (role, privilege))
        denied = role_query("ipat_domain_query", "SELECT * FROM ipat_platform.tenant_domains", expect=False)
        self.assertNotEqual(denied.returncode, 0)
    def test_05_malformed_or_duplicate_fqdn_is_rejected(self):
        for fqdn in (
            "Upper.Example",
            "bad..example",
            "-bad.example",
            "bad-.example",
            "singlelabel",
        ):
            q = f"""INSERT INTO ipat_platform.tenant_domains
              (tenant_id,fqdn,domain_kind,verification_method,state,tls_ready)
              VALUES ('{TA}','{fqdn}','custom_domain','dns_txt','pending',false)"""
            self.assertNotEqual(sql(q, expect=False).returncode, 0, fqdn)
        duplicate = f"""INSERT INTO ipat_platform.tenant_domains
          (tenant_id,fqdn,domain_kind,verification_method,state,verified_at,tls_ready)
          VALUES ('{TA}','ipat.fadly.id','custom_domain','dns_txt',
                  'verified',statement_timestamp(),true)"""
        self.assertNotEqual(sql(duplicate, expect=False).returncode, 0)

    def test_06_function_is_security_definer_with_locked_search_path(self):
        row = sql("""SELECT r.rolname,p.prosecdef::int,p.proconfig::text
                     FROM pg_proc p
                     JOIN pg_namespace n ON n.oid=p.pronamespace
                     JOIN pg_roles r ON r.oid=p.proowner
                     WHERE n.nspname='ipat_platform'
                     AND p.proname='resolve_verified_tenant_domain'""").stdout.strip()
        self.assertIn("ipat_domain_lookup_owner|1|", row)
        self.assertIn("search_path=pg_catalog, ipat_platform", row)
        policy = sql("""SELECT policyname FROM pg_policies
                        WHERE schemaname='ipat_platform'
                        AND tablename='tenant_domains'""").stdout.strip()
        self.assertEqual(policy, "tenant_domain_lookup_select")

if __name__ == "__main__":
    unittest.main()
