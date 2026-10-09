"""R10.14 real shell guard tests in isolated unprivileged directories."""
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
GUARD = ROOT / "deploy/scripts/production/r1014_require_absent.sh"
EDGE = (ROOT / "deploy/scripts/production/r997_activate_platform_edge.sh").read_text()
CUSTOMER = (ROOT / "deploy/scripts/production/r998_activate_single_customer.sh").read_text()


class R1014RollbackPreflight(unittest.TestCase):
    def guarded(self, *paths):
        return subprocess.run(
            ["bash", "-c", 'source "$1"; shift; r1014_require_absent "$@"',
             "r1014-test", str(GUARD), *(str(p) for p in paths)],
            capture_output=True, text=True, timeout=3, check=False,
        )

    def test_all_absent_allows_only_preflight(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td)
            self.assertEqual(self.guarded(p / "a", p / "space file").returncode, 0)

    def test_existing_sensitive_file_is_not_modified(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "tenant-oidc.env"
            p.write_bytes(b"EXISTING_SAFE_CONFIG")
            result = self.guarded(p)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(p.read_bytes(), b"EXISTING_SAFE_CONFIG")
            self.assertNotIn("EXISTING_SAFE_CONFIG", result.stderr)

    def test_existing_directory_is_not_removed(self):
        with tempfile.TemporaryDirectory() as td:
            d = Path(td) / "config"
            d.mkdir()
            (d / "secret").write_text("preserve")
            self.assertNotEqual(self.guarded(d).returncode, 0)
            self.assertEqual((d / "secret").read_text(), "preserve")

    def test_dangling_symlink_and_valid_symlink_are_denied(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td)
            dangling = p / "dangling"
            dangling.symlink_to(p / "nonexistent")
            destination = p / "real"
            destination.write_text("untouched")
            link = p / "link"
            link.symlink_to(destination)
            self.assertNotEqual(self.guarded(dangling).returncode, 0)
            self.assertNotEqual(self.guarded(link).returncode, 0)
            self.assertTrue(dangling.is_symlink())
            self.assertEqual(destination.read_text(), "untouched")

    def test_empty_arg_list_denied(self):
        self.assertNotEqual(self.guarded().returncode, 0)

    def test_platform_guard_before_arming_any_rollback(self):
        pos = EDGE.index("r1014_require_absent")
        self.assertLess(pos, EDGE.index("rollback_armed=YES"))
        self.assertIn('"/var/lib/ipat-domain-ownership-verifier"', EDGE)
        for path in ("/etc/ipat/platform-api.env",
                     "/etc/ipat/platform-oidc.env",
                     "/etc/ipat/tenant-api.env",
                     "/var/lib/ipat/r997-platform-edge.json"):
            self.assertIn(f'"{path}"', EDGE)
        self.assertIn('r1014_require_absent', EDGE)
        self.assertIn('R997_PREEXISTING_ACTIVATION_ARTIFACT', EDGE)

    def test_customer_guard_before_arming_and_existing_checks(self):
        pos = CUSTOMER.index("r1014_require_absent")
        self.assertLess(pos, CUSTOMER.index("rollback_armed=YES"))
        self.assertIn('"/etc/ipat/tenant-oidc/$instance.env"', CUSTOMER)
        self.assertIn('"/var/lib/ipat/customer-activations/$domain.json"', CUSTOMER)
        self.assertIn('R998_PREEXISTING_CUSTOMER_ARTIFACT', CUSTOMER)
        self.assertIn('R998_CUSTOMER_OIDC_ALREADY_ACTIVE', CUSTOMER)


if __name__ == "__main__":
    unittest.main()
