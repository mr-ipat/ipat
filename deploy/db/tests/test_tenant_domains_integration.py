import os
import pathlib
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[3]
M1 = ROOT / "deploy/db/migrations/0001_lab_tenant_rls.sql"
M12 = ROOT / "deploy/db/migrations/0012_tenant_domains.sql"

def run_psql(sql: str, *, check=True):
    env = os.environ.copy()
    cmd = ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-At", "-c", sql]
    return subprocess.run(cmd, env=env, text=True, capture_output=True, check=check)

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST") == "1", "ephemeral PostgreSQL only")
class TenantDomainIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # CI migrations are intentionally cumulative. Never drop shared base
        # roles/schemas created and exercised by earlier integration tests.
        base = run_psql("SELECT to_regclass('ipat_platform.tenants') IS NOT NULL;").stdout.strip()
        if base != "t":
            subprocess.run(
                ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(M1)],
                env=os.environ.copy(), text=True, capture_output=True, check=True
            )
        run_psql("DROP FUNCTION IF EXISTS ipat_platform.resolve_active_tenant_domain(text); "
                  "DROP TABLE IF EXISTS ipat_platform.tenant_domains CASCADE; "
                  "DROP ROLE IF EXISTS ipat_domain_reader_login; "
                  "DROP ROLE IF EXISTS ipat_domain_reader;")
        subprocess.run(
            ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(M12)],
            env=os.environ.copy(), text=True, capture_output=True, check=True
        )
        run_psql("""
          INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
          ('11111111-1111-1111-1111-111111111111','fadly','active'),
          ('22222222-2222-2222-2222-222222222222','nengnet','active'),
          ('33333333-3333-3333-3333-333333333333','suspendedco','suspended');
          INSERT INTO ipat_platform.tenant_domains
            (id,tenant_id,hostname,domain_type,verification_state,verification_method,verified_at)
          VALUES
          ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa1','11111111-1111-1111-1111-111111111111',
           'ipat.fadly.id','custom_domain','verified','dns_txt',now()),
          ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa2','22222222-2222-2222-2222-222222222222',
           'nengnet.ipat.id','managed_subdomain','pending','platform_managed',NULL),
          ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa3','33333333-3333-3333-3333-333333333333',
           'suspended.ipat.id','managed_subdomain','verified','platform_managed',now());
        """)

    def test_verified_active_domain_resolves_exactly_one_tenant(self):
        out = run_psql("""
          SET ROLE ipat_domain_reader_login;
          SELECT tenant_slug||'|'||hostname||'|'||domain_type
          FROM ipat_platform.resolve_active_tenant_domain('ipat.fadly.id');
        """).stdout.splitlines()
        rows = [x for x in out if x and x != "SET"]
        self.assertEqual(rows, ["fadly|ipat.fadly.id|custom_domain"])

    def test_pending_unknown_and_suspended_domains_fail_closed(self):
        for host in ("nengnet.ipat.id", "unknown.ipat.id", "suspended.ipat.id"):
            out = run_psql(f"""
              SET ROLE ipat_domain_reader_login;
              SELECT count(*) FROM ipat_platform.resolve_active_tenant_domain('{host}');
            """).stdout.splitlines()
            rows = [x for x in out if x and x != "SET"]
            self.assertEqual(rows, ["0"], host)

    def test_reader_cannot_select_registry_table(self):
        p = run_psql("""
          SET ROLE ipat_domain_reader_login;
          SELECT hostname FROM ipat_platform.tenant_domains;
        """, check=False)
        self.assertNotEqual(p.returncode, 0)
        self.assertIn("permission denied", p.stderr.lower())

    def test_runtime_login_is_nonsuperuser_and_has_no_stored_password(self):
        out = run_psql("""
          SELECT rolcanlogin::text||'|'||rolsuper::text||'|'||
                 (rolpassword IS NULL)::text
          FROM pg_authid WHERE rolname='ipat_domain_reader_login';
        """).stdout.strip()
        self.assertEqual(out, "true|false|true")

    def test_noncanonical_host_is_not_silently_normalized_in_database(self):
        out = run_psql("""
          SET ROLE ipat_domain_reader_login;
          SELECT count(*) FROM ipat_platform.resolve_active_tenant_domain('IPAT.FADLY.ID');
        """).stdout.splitlines()
        rows = [x for x in out if x and x != "SET"]
        self.assertEqual(rows, ["0"])

if __name__ == "__main__":
    unittest.main()
