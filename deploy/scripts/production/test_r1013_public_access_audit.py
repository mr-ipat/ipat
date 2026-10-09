import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import r1013_public_access_audit as a


class PublicAccessAuditTests(unittest.TestCase):
    def test_synthetic_healthy_still_does_not_approve_release(self):
        p = {
            "target_ip": "202.162.204.121",
            "private_vps_ssh_observed": True,
            "target_matches_existing_private_owner_plan": True,
            "vps_443_listener_observed": True,
            "internet_443_reachable": True,
            "browser_ca_trusted_exact_ipv4_san_tls": True,
        }
        o = {"ssh_unit_check_succeeded": True,
             "foundation_marker": "PRESENT", "edge_marker": "PRESENT",
             "services": {name: "active" for name in a.SERVICE_UNITS}}
        out = a.evaluate(p, o)
        self.assertEqual(out["missing_check_ids"], [])
        self.assertFalse(out["public_go"])
        self.assertEqual(out["production_status"], "NO_GO")
        self.assertIn("real_operator_oidc_mfa_and_revocation", out["independent_evidence_required"])

    def test_real_vps_missing_services_are_explicit_blockers(self):
        p = {
            "target_ip": "202.162.204.121",
            "private_vps_ssh_observed": True,
            "target_matches_existing_private_owner_plan": True,
            "vps_443_listener_observed": False,
            "internet_443_reachable": False,
            "browser_ca_trusted_exact_ipv4_san_tls": False,
        }
        o = {"ssh_unit_check_succeeded": True,
             "foundation_marker": "NOT_OBSERVED", "edge_marker": "NOT_OBSERVED",
             "services": {name: "inactive" for name in a.SERVICE_UNITS}}
        out = a.evaluate(p, o)
        self.assertIn("postgresql_running", out["missing_check_ids"])
        self.assertIn("external_https_reachable", out["missing_check_ids"])
        self.assertIn("foundation_marker_observed", out["missing_check_ids"])
        self.assertFalse(out["mutations_attempted"])

    def test_unknown_is_not_success_and_never_reflects_untrusted_stdout(self):
        with patch.object(a.subprocess, "run",
                          return_value=SimpleNamespace(returncode=0, stdout=(
                              "UNIT_nginx=active\nUNIT_postgresql=active\n"
                              "UNIT_platform_api=compromised\n"
                              "SECRET_VALUE=LEAK\nMARKER_edge=FAKE\n"))) as process:
            result = a.services_observation()
        command = process.call_args.args[0]
        self.assertIn("BatchMode=yes", command)
        self.assertIn("StrictHostKeyChecking=yes", command)
        self.assertEqual(result["services"]["nginx"], "active")
        self.assertEqual(result["services"]["platform_api"], "unknown")
        self.assertEqual(result["edge_marker"], "NOT_OBSERVED")
        self.assertNotIn("SECRET", str(result))

    def test_ssh_timeout_fail_closed(self):
        with patch.object(a.subprocess, "run", side_effect=subprocess_timeout()):
            result = a.services_observation()
        self.assertFalse(result["ssh_unit_check_succeeded"])
        self.assertTrue(all(v == "unknown" for v in result["services"].values()))
        self.assertFalse(a.evaluate({}, result)["checks"]["ssh_verified"])


def subprocess_timeout():
    import subprocess
    return subprocess.TimeoutExpired(cmd="ssh", timeout=13)


if __name__ == "__main__":
    unittest.main()
