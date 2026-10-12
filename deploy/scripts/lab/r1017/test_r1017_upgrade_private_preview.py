import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SCRIPT = ROOT / "deploy/scripts/lab/r1017/upgrade_private_preview.sh"

class R1017Source(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = SCRIPT.read_text()

    def test_valid_bash(self):
        subprocess.run(["bash", "-n", str(SCRIPT)], check=True)

    def test_limited_to_rootless_private_preview(self):
        for required in ("-ne 0", "UPGRADE_PRIVATE_PREVIEW_ONLY",
                         "127.0.0.1:3002", "PUBLIC_GO_FALSE",
                         "sport = :443"):
            self.assertIn(required, self.source)
        for forbidden in ("sudo ", "pg_dropcluster", "iptables ", "nft ",
                          "certbot", "systemctl --system", "IPAT_R997_APPLY"):
            self.assertNotIn(forbidden, self.source)

    def test_integrity_and_preflight_prior_to_mutation(self):
        section = self.source[:self.source.index("armed=YES")]
        for required in ("SOURCE_COMMIT", "sha256sum --status -c SHA256SUMS",
                         "binary checksum failure", "ELF 64-bit.*x86-64",
                         "unreviewed previous executable", "connector status unsafe",
                         "preexisting override", "existing dashboard unhealthy"):
            self.assertIn(required, section)

    def test_rollback_will_restore_previous_service(self):
        for required in ('trap rollback EXIT INT TERM ERR', 'rm -f -- "$override"',
                         'systemctl --user daemon-reload',
                         'systemctl --user restart "$unit"',
                         'R1017_ROLLBACK_ORIGINAL_PREVIEW_RESTORED',
                         '[[ $resolved == "$binary" ]]'):
            self.assertIn(required, self.source)

    def test_read_only_endpoints_and_c320_connector(self):
        for endpoint in ("/healthz", "/lab/device-workbench",
                         "/lab/platform-admin/domains", "/lab/c320-owner-connection",
                         "/lab/owner/devices", "/lab/owner/platform-domains"):
            self.assertIn(endpoint, self.source)
        self.assertIn("is-active --quiet ipat-r945-connector.service", self.source)
        self.assertIn("grep -Fq '127.0.0.1:3002'", self.source)

if __name__ == "__main__":
    unittest.main()
