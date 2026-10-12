"""R10.18 root authentication handoff: read-only, fail-closed."""
import os
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "deploy/scripts/production/r1018_root_readiness.sh"

class R1018RootPreflightTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = SCRIPT.read_text()

    def test_syntax(self):
        subprocess.run(["bash", "-n", str(SCRIPT)], check=True, timeout=3)

    def test_refuses_untrusted_script_location(self):
        # Even a root process cannot execute the file from a user-writable checkout.
        r = subprocess.run(["bash", str(SCRIPT)], capture_output=True, text=True, timeout=3)
        self.assertEqual(r.returncode, 4)
        self.assertIn("ROOT_PREFLIGHT_REFUSED", r.stderr)

    def test_requires_interactive_root_but_never_issues_sudo(self):
        for term in ("EUID", "/root/ipat-r1018-root-readiness.sh",
                     "0:500:1", "ROOT_AUTHENTICATION=PASS",
                     "MUTATIONS_ATTEMPTED=false",
                     "PUBLIC_GO=false"):
            self.assertIn(term, self.source)
        for dangerous in ("sudo -n", "sudo -S", "NOPASSWD:",
                          "apt-get install", "pg_createcluster ", "pg_dropcluster ",
                          "systemctl restart", "systemctl enable", "systemctl start",
                          "certbot certonly", "iptables -", "nft add", "curl -k",
                          "useradd ", "rm -rf", "sed -i"):
            self.assertNotIn(dangerous, self.source)

    def test_fails_closed_on_missing_independent_attestation(self):
        for text in ("r996-recovery-attestation.json",
                     "ROOT_OWNED_RECOVERY_ATTESTATION_METADATA",
                     "0:600:1",
                     "PROVIDER_RESCUE_AND_INDEPENDENT_HOST_RESTORE=EXTERNAL_EVIDENCE_REQUIRED",
                     "ROOT_ATTESTATION_CONTENT_AND_OFFSITE_RESTORE=NOT_VALIDATED_BY_THIS_PREFLIGHT",
                     "R1018_ROOT_PREFLIGHT_BLOCKED_USE_INDEPENDENT_EVIDENCE_AND_REVIEW"):
            self.assertIn(text, self.source)

    def test_exact_sha_and_unoccupied_resources(self):
        for term in ("845eeac1f039168fd08af7f860ab50413f91d85a",
                     "1753d706b6d076fb67ba9509782e1e5c82a5aa673f901e9e809935bd98a66f20",
                     "POSTGRES_CLUSTER_UNOCCUPIED",
                     "PRODUCTION_PORTS_UNOCCUPIED",
                     "FIRST_INSTALL_RUNTIME_UNOCCUPIED",
                     "REAL_POSTGRES_PITR_HA_FENCING=NOT_YET_TESTED"):
            self.assertIn(term, self.source)

if __name__ == "__main__":
    unittest.main()
