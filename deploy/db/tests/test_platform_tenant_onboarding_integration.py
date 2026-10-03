"""R9.73 synthetic-only platform-owner tenant reservation and role separation."""
import os
import subprocess
import unittest

ISSUER = "https://id.r973.synthetic.invalid/realms/ipat"
T = "73737373-7373-4737-8737-737373737371"
T2 = "73737373-7373-4737-8737-737373737372"
REQUEST = "73737373-7373-4737-8737-737373737381"
REQUEST2 = "73737373-7373-4737-8737-737373737382"

def sql(query, check=True):
    return subprocess.run(
        ["psql", "-X", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1", "-c", query],
        text=True, capture_output=True, check=check,
    )

def reserve(subject="owner", request=REQUEST, tenant=T, slug="synthetic-r973",
            issuer=ISSUER, role="ipat_platform_onboard_exec"):
    text = ("SET ROLE " + role + ";SELECT ipat_platform.reserve_suspended_tenant("
            f"'{issuer}','{subject}','{request}','{tenant}','{slug}')")
    return sql(text).stdout.strip()

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST") == "1", "disposable PG only")
class PlatformTenantReservation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE") == "ipat_synthetic"
        assert os.getenv("PGHOST") == "127.0.0.1"
        assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD") == "local_ci_synthetic_only"
        assert sql("SELECT to_regprocedure('ipat_platform.reserve_suspended_tenant(text,text,uuid,uuid,text)') IS NOT NULL").stdout.strip() == "t"
        sql(f"""INSERT INTO ipat_platform.platform_principals(
             issuer,subject,role,approved_by,created_at,expires_at) VALUES
             ('{ISSUER}','owner','platform_owner','independent-synthetic-reviewer',
              now()-interval '1 minute',now()+interval '1 hour'),
             ('{ISSUER}','revoked','platform_owner','independent-synthetic-reviewer',
              now()-interval '1 minute',now()+interval '1 hour'),
             ('{ISSUER}','expired','platform_owner','independent-synthetic-reviewer',
              now()-interval '1 hour',now()-interval '1 minute')""")
        sql(f"UPDATE ipat_platform.platform_principals SET revoked_at=now() WHERE subject='revoked'")

    def test_01_current_platform_owner_can_only_reserve_suspended(self):
        self.assertEqual(reserve(), T)
        self.assertEqual(sql(f"SELECT state FROM ipat_platform.tenants WHERE id='{T}'").stdout.strip(), "suspended")
        self.assertEqual(sql(
            f"SELECT count(*) FROM ipat_platform.identity_memberships WHERE tenant_id='{T}'"
        ).stdout.strip(), "0")
        self.assertEqual(sql(
            f"SELECT count(*) FROM ipat_platform.tenant_domains WHERE tenant_id='{T}'"
        ).stdout.strip(), "0")

    def test_02_exact_replay_is_idempotent_and_conflicts_fail(self):
        self.assertEqual(reserve(), T)
        self.assertEqual(reserve(tenant=T2), "")
        self.assertEqual(reserve(subject="revoked"), "")
        self.assertEqual(reserve(request=REQUEST2, tenant=T2, slug="synthetic-r973"), "")
        self.assertEqual(reserve(request=REQUEST2, tenant=T2, slug="../bad"), "")
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_platform.platform_tenant_reservations WHERE request_id='{REQUEST}'").stdout.strip(), "1")

    def test_03_missing_expired_revoked_wrong_issuer_fail(self):
        self.assertEqual(reserve(subject="expired", request=REQUEST2, tenant=T2, slug="other"), "")
        self.assertEqual(reserve(subject="revoked", request=REQUEST2, tenant=T2, slug="other"), "")
        self.assertEqual(reserve(subject="missing", request=REQUEST2, tenant=T2, slug="other"), "")
        self.assertEqual(reserve(request=REQUEST2, tenant=T2, slug="other",
                                 issuer="https://forged.invalid"), "")

    def test_04_tenant_api_and_issuer_cannot_list_or_provision(self):
        signature = "ipat_platform.reserve_suspended_tenant(text,text,uuid,uuid,text)"
        self.assertEqual(sql(
            f"SELECT has_function_privilege('ipat_tenant_api_login','{signature}','EXECUTE')::int"
        ).stdout.strip(), "0")
        self.assertEqual(sql(
            f"SELECT has_function_privilege('ipat_oidc_session_issuer_login','{signature}','EXECUTE')::int"
        ).stdout.strip(), "0")
        self.assertNotEqual(sql(
            "SET ROLE ipat_tenant_api_login; SELECT count(*) FROM ipat_platform.platform_principals",
            check=False,
        ).returncode, 0)
        self.assertNotEqual(sql(
            "SET ROLE ipat_platform_onboard_exec; SELECT count(*) FROM ipat_ops.managed_devices",
            check=False,
        ).returncode, 0)

    def test_05_owner_lists_reservations_without_tenant_data(self):
        out = sql(f"SET ROLE ipat_platform_onboard_exec;SELECT tenant_slug||':'||tenant_state "
                  f"FROM ipat_platform.list_reserved_tenants_for_platform_owner('{ISSUER}','owner')").stdout.splitlines()
        self.assertIn("synthetic-r973:suspended", out)
        self.assertEqual(sql("SET ROLE ipat_platform_onboard_exec;SELECT tenant_slug "
                             "FROM ipat_platform.list_reserved_tenants_for_platform_owner("
                             "'https://forged.invalid','owner')").stdout.strip(), "")
        sql(f"UPDATE ipat_platform.platform_principals SET revoked_at=now() WHERE subject='owner'")
        self.assertEqual(sql(f"SET ROLE ipat_platform_onboard_exec;SELECT tenant_slug "
                             f"FROM ipat_platform.list_reserved_tenants_for_platform_owner('{ISSUER}','owner')").stdout.strip(), "")

if __name__ == "__main__":
    unittest.main()
