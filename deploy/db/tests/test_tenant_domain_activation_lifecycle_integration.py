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
M14 = ROOT / "deploy/db/migrations/0014_tenant_domain_activation_lifecycle.sql"

def run_psql(sql: str, *, check=True):
    return subprocess.run(
        ["psql","-X","-v","ON_ERROR_STOP=1","-At","-c",sql],
        env=os.environ.copy(), text=True, capture_output=True, check=check
    )

def run_file(path):
    return subprocess.run(
        ["psql","-X","-v","ON_ERROR_STOP=1","-f",str(path)],
        env=os.environ.copy(), text=True, capture_output=True, check=True
    )

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST") == "1", "ephemeral PostgreSQL only")
class DomainLifecycleIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if run_psql("SELECT to_regclass('ipat_platform.tenants') IS NOT NULL;").stdout.strip() != "t":
            run_file(M1)
        if run_psql("SELECT to_regclass('ipat_platform.identity_memberships') IS NOT NULL;").stdout.strip() != "t":
            run_file(M3)
        if run_psql("SELECT to_regprocedure('ipat_platform.lookup_active_membership(text,text,uuid,text,text)') IS NOT NULL;").stdout.strip() != "t":
            run_file(M4)
        if run_psql("SELECT to_regclass('ipat_platform.tenant_domains') IS NOT NULL;").stdout.strip() != "t":
            run_file(M12)
        if run_psql("SELECT EXISTS(SELECT 1 FROM information_schema.columns WHERE table_schema='ipat_platform' AND table_name='tenant_domains' AND column_name='routing_mode');").stdout.strip() != "t":
            run_file(M13)

        run_psql("""
          DROP FUNCTION IF EXISTS ipat_platform.record_tenant_domain_check(uuid,text,text,text);
          DROP ROLE IF EXISTS ipat_domain_verifier_login;
          DROP ROLE IF EXISTS ipat_domain_verifier;
        """)
        run_file(M14)
        run_psql("""
          INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
            ('66666666-6666-6666-6666-666666666666','lifecycle-a','active')
          ON CONFLICT (id) DO UPDATE SET state='active';
          INSERT INTO ipat_platform.identity_memberships(
            tenant_id,issuer,subject,role,approved_by,expires_at
          ) VALUES (
            '66666666-6666-6666-6666-666666666666',
            'https://id.example.invalid/realms/ipat','domain-admin',
            'tenant_admin','ci-security-admin','2099-01-01T00:00:00Z'
          )
          ON CONFLICT (tenant_id,issuer,subject,role) DO UPDATE
          SET revoked_at=NULL,expires_at='2099-01-01T00:00:00Z';

          INSERT INTO ipat_platform.tenant_domains(
            id,tenant_id,hostname,domain_type,verification_state,verification_method,
            routing_mode,verification_name,verification_value,
            requested_by_issuer,requested_by_subject,requested_at,
            activation_state
          ) VALUES (
            'cccccccc-cccc-cccc-cccc-ccccccccccc1',
            '66666666-6666-6666-6666-666666666666',
            'portal.lifecycle.example.net','custom_domain','pending','dns_txt',
            'a_record','_ipat-verify.portal.lifecycle.example.net',
            'ipat-domain=cccccccc-cccc-cccc-cccc-ccccccccccc1',
            'https://id.example.invalid/realms/ipat','domain-admin',clock_timestamp(),
            'pending_dns'
          )
          ON CONFLICT (id) DO UPDATE SET
            verification_state='pending',
            verified_at=NULL,
            disabled_at=NULL,
            activation_state='pending_dns',
            ownership_verified_at=NULL,
            routing_ready_at=NULL,
            tls_ready_at=NULL,
            activated_at=NULL,
            last_checked_at=NULL,
            last_error_code=NULL,
            last_evidence_sha256=NULL;
        """)

    def setUp(self):
        run_psql("""
          UPDATE ipat_platform.tenant_domains
          SET verification_state='pending',
              verified_at=NULL,
              disabled_at=NULL,
              activation_state='pending_dns',
              ownership_verified_at=NULL,
              routing_ready_at=NULL,
              tls_ready_at=NULL,
              activated_at=NULL,
              last_checked_at=NULL,
              last_error_code=NULL,
              last_evidence_sha256=NULL
          WHERE id='cccccccc-cccc-cccc-cccc-ccccccccccc1';
        """)

    def event(self, name, digest="a"*64, error="NULL"):
        value = f"'{error}'" if error != "NULL" else "NULL"
        out = run_psql(f"""
          SET ROLE ipat_domain_verifier_login;
          SELECT COALESCE(ipat_platform.record_tenant_domain_check(
            'cccccccc-cccc-cccc-cccc-ccccccccccc1','{name}','{digest}',{value}
          ),'NULL');
        """).stdout.splitlines()
        return [x for x in out if x and x != "SET"]

    def test_initial_request_is_persisted_as_pending_dns(self):
        out = run_psql("""
          SELECT activation_state||'|'||verification_state
          FROM ipat_platform.tenant_domains
          WHERE id='cccccccc-cccc-cccc-cccc-ccccccccccc1';
        """).stdout.strip()
        self.assertEqual(out, "pending_dns|pending")

    def test_verifier_cannot_skip_lifecycle_steps(self):
        self.assertEqual(self.event("tls_ready"), ["NULL"])
        self.assertEqual(self.event("activate"), ["NULL"])

    def test_full_verified_routing_tls_activation_sequence(self):
        self.assertEqual(self.event("ownership_verified"), ["ownership_verified"])
        self.assertEqual(self.event("routing_ready", "b"*64), ["routing_ready"])
        self.assertEqual(self.event("tls_ready", "c"*64), ["tls_ready"])
        self.assertEqual(self.event("activate", "d"*64), ["active"])
        out = run_psql("""
          SET ROLE ipat_domain_admin_login;
          SELECT activation_state||'|'||(ownership_verified_at IS NOT NULL)::text||'|'||
                 (routing_ready_at IS NOT NULL)::text||'|'||
                 (tls_ready_at IS NOT NULL)::text||'|'||(activated_at IS NOT NULL)::text
          FROM ipat_platform.list_tenant_domains_for_member(
            'https://id.example.invalid/realms/ipat','domain-admin',
            '66666666-6666-6666-6666-666666666666'
          )
          WHERE id='cccccccc-cccc-cccc-cccc-ccccccccccc1';
        """).stdout.splitlines()
        rows=[x for x in out if x and x!="SET"]
        self.assertEqual(rows, ["active|true|true|true|true"])

    def test_verifier_has_no_direct_registry_access(self):
        p=run_psql("""
          SET ROLE ipat_domain_verifier_login;
          SELECT hostname FROM ipat_platform.tenant_domains;
        """, check=False)
        self.assertNotEqual(p.returncode,0)
        self.assertIn("permission denied",p.stderr.lower())

    def test_verifier_login_is_restricted_and_passwordless(self):
        out=run_psql("""
          SELECT rolcanlogin::text||'|'||rolsuper::text||'|'||
                 rolbypassrls::text||'|'||(rolpassword IS NULL)::text
          FROM pg_authid WHERE rolname='ipat_domain_verifier_login';
        """).stdout.strip()
        self.assertEqual(out,"true|false|false|true")

if __name__=="__main__":
    unittest.main()
