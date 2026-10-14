"""Fail-closed, non-mutating contract for the first commercial runtime inventory."""
from pathlib import Path
import subprocess
import unittest

SCRIPT = Path(__file__).with_name("r1024_readonly_runtime_inventory.sh")


class ReadOnlyInventory(unittest.TestCase):
    def test_bash_syntax(self):
        subprocess.run(["bash", "-n", str(SCRIPT)], check=True)

    def test_ss_failure_cannot_report_port_absent(self):
        import os
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            # Stubs ensure a failing ss command is distinguished from an empty success.
            for name, script in {
                "ss": "#!/bin/sh\nexit 12\n",
                "getent": "#!/bin/sh\nexit 2\n",
                "systemctl": "#!/bin/sh\nexit 3\n",
            }.items():
                path = Path(d) / name
                path.write_text(script)
                path.chmod(0o755)
            env = dict(os.environ, PATH=d + ":" + os.environ["PATH"])
            run = subprocess.run(["bash", str(SCRIPT)], env=env, check=True,
                                 capture_output=True, text=True)
            for port in (80, 443, 5432, 3003, 3005, 3006):
                self.assertIn(f"PORT_{port}=UNKNOWN_INSPECTION_FAILED", run.stdout)
                self.assertNotIn(f"PORT_{port}=NOT_OBSERVED", run.stdout)

    def test_no_mutation_or_false_public_go(self):
        body = SCRIPT.read_text()
        for forbidden in ("sudo ", "apt ", "rm -", "install ", "systemctl start", "systemctl enable", "PUBLIC_GO=YES", "curl "):
            self.assertNotIn(forbidden, body)
        self.assertIn("PUBLIC_GO=NOT_DETERMINED", body)
        self.assertIn("RECOVERY_RESTORE=NOT_ASSESSED", body)


if __name__ == "__main__":
    unittest.main()
