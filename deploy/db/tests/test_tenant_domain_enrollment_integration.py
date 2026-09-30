import os
import pathlib
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[3]
M1 = ROOT / "deploy/db/migrations/0001_lab_tenant_rls.sql"
M3 = ROOT / "deploy/db/migrations/0003_lab_identity_memberships.sql"
M4 = ROOT / "deploy/db/migrations/0004_lab_scoped_identity_lookup.sql"
M12 = ROOT / "deploy/db/migrations/0012_tenant_domains.sql"
M13 = ROOT / "deploy/db/migrations/0013_tenant_domain_enrollment.sql"

def run_psql(sql: str, *, check=True):
    cmd = ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-At", "-c", sql]
    return subprocess.run(
        cmd, env=os.environ.copy(), text=True, capture_output=True, check=check
    )

def run_file(path: pathlib.Path):
    return subprocess.run(
        ["psql", "-X", "-v", "ON_ERROR_STOP=1", "-f", str(path)],
        env=os.environ.copy(), text=True, capture_output=True, check=True
    )

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST") == "1", "ephemeral PostgreSQL only")
class TenantDomainEnrollmentIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if run_psql("SELECT to_regclass('ipat_platform.tenants') IS NOT NULL;").stdout.strip() != "t":
            run_file(M1)
        if run_psql("SELECT to_regclass('ipat_platform.identity_memberships') IS NOT NULL;").stdout.strip() != "t":
            run_file(M3)
        if run_psql("""
          SELECT to_regprocedure(
            'ipat_platform.lookup_active_membership(text,text,uuid,text,text)'
          ) IS NOT NULL;
        """).stdout.strip() != "t":
            run_file(M4)
        if run_psql("SELECT to_regclass('ipat_platform.tenant_domains') IS NOT NULL;").stdout.strip() != "t":
            run_file(M12)

        run_psql("""
          DROP FUNCTION IF EXISTS ipat_platform.disable_tenant_custom_domain(text,text,uuid,uuid);
          DROP FUNCTION IF EXISTS ipat_platform.list_tenant_domains_for_member(text,text,uuid);
          DROP FUNCTION IF EXISTS ipat_platform.request_tenant_custom_domain(text,text,uuid,uuid,text,text,text);
          DROP ROLE IF EXISTS ipat_domain_admin_login;
          DROP ROLE IF EXISTS ipat_domain_admin;
        """)
        run_file(M13)

        run_psql("""
          INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
          ('44444444-4444-4444-4444-444444444444','domain-a','active'),
          ('55555555-5555-5555-5555-555555555555','domain-b','active')
          ON CONFLICT (id) DO UPDATE SET state='active';

          INSERT INTO ipat_platform.identity_memberships(
            tenant_id,issuer,subject,role,approved_by,expires_at
          ) VALUES
          ('44444444-4444-4444-4444-444444444444',
           'https://id.example.invalid/realms/ipat','tenant-a-admin',
           'tenant_admin','ci-security-admin','2099-01-01T00:00:00Z'),
          ('55555555-5555-5555-5555-555555555555',
           'https://id.example.invalid/realms/ipat','tenant-b-admin',
           'tenant_admin','ci-security-admin','2099-01-01T00:00:00Z')
          ON CONFLICT (tenant_id,issuer,subject,role) DO UPDATE
          SET revoked_at=NULL,expires_at='2099-01-01T00:00:00Z';
        """)

    def test_runtime_principal_is_restricted_and_passwordless(self):
        row = run_psql("""
          SELECT rolcanlogin::text||'|'||rolsuper::text||'|'||
                 rolbypassrls::text||'|'||(rolpassword IS NULL)::text
          FROM pg_authid WHERE rolname='ipat_domain_admin_login';
        """).stdout.strip()
        self.assertEqual(row, "true|false|false|true")

    def test_tenant_admin_can_request_and_list_own_pending_domain(self):
        out = run_psql("""
          SET ROLE ipat_domain_admin_login;
          SELECT ipat_platform.request_tenant_custom_domain(
            'https://id.example.invalid/realms/ipat','tenant-a-admin',
            '44444444-4444-4444-4444-444444444444',
            'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1',
            'portal.domain-a.example','a_record',
            'ipat-domain=bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1'
          );
        """).stdout.splitlines()
        rows = [x for x in out if x and x != "SET"]
        self.assertEqual(rows, ["bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1"])

        listed = run_psql("""
          SET ROLE ipat_domain_admin_login;
          SELECT hostname||'|'||verification_state||'|'||routing_mode||'|'||
                 verification_name||'|'||verification_value
          FROM ipat_platform.list_tenant_domains_for_member(
            'https://id.example.invalid/realms/ipat','tenant-a-admin',
            '44444444-4444-4444-4444-444444444444'
          )
          WHERE id='bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1';
        """).stdout.splitlines()
        rows = [x for x in listed if x and x != "SET"]
        self.assertEqual(rows, [
            "portal.domain-a.example|pending|a_record|"
            "_ipat-verify.portal.domain-a.example|"
            "ipat-domain=bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1"
        ])

    def test_duplicate_hostname_does_not_leak_other_tenant(self):
        out = run_psql("""
          SET ROLE ipat_domain_admin_login;
          SELECT COALESCE(ipat_platform.request_tenant_custom_domain(
            'https://id.example.invalid/realms/ipat','tenant-b-admin',
            '55555555-5555-5555-5555-555555555555',
            'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2',
            'portal.domain-a.example','cname',
            'ipat-domain=bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2'
          )::text,'NULL');
        """).stdout.splitlines()
        rows = [x for x in out if x and x != "SET"]
        self.assertEqual(rows, ["NULL"])

        listed = run_psql("""
          SET ROLE ipat_domain_admin_login;
          SELECT count(*)
          FROM ipat_platform.list_tenant_domains_for_member(
            'https://id.example.invalid/realms/ipat','tenant-b-admin',
            '55555555-5555-5555-5555-555555555555'
          )
          WHERE hostname='portal.domain-a.example';
        """).stdout.splitlines()
        rows = [x for x in listed if x and x != "SET"]
        self.assertEqual(rows, ["0"])

    def test_direct_table_read_is_denied(self):
        p = run_psql("""
          SET ROLE ipat_domain_admin_login;
          SELECT hostname FROM ipat_platform.tenant_domains;
        """, check=False)
        self.assertNotEqual(p.returncode, 0)
        self.assertIn("permission denied", p.stderr.lower())

    def test_wrong_subject_cannot_disable_domain(self):
        out = run_psql("""
          SET ROLE ipat_domain_admin_login;
          SELECT ipat_platform.disable_tenant_custom_domain(
            'https://id.example.invalid/realms/ipat','tenant-b-admin',
            '44444444-4444-4444-4444-444444444444',
            'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1'
          );
        """).stdout.splitlines()
        rows = [x for x in out if x and x != "SET"]
        self.assertEqual(rows, ["f"])

if __name__ == "__main__":
    unittest.main()
