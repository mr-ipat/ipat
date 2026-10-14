"""Fail-closed, non-mutating contract for the first commercial runtime inventory."""
from pathlib import Path
import subprocess
import unittest

SCRIPT = Path(__file__).with_name("r1024_readonly_runtime_inventory.sh")


class ReadOnlyInventory(unittest.TestCase):
    def test_bash_syntax(self):
        subprocess.run(["bash", "-n", str(SCRIPT)], check=True)

    def test_no_mutation_or_false_public_go(self):
        body = SCRIPT.read_text()
        for forbidden in ("sudo ", "apt ", "rm -", "install ", "systemctl start", "systemctl enable", "PUBLIC_GO=YES", "curl "):
            self.assertNotIn(forbidden, body)
        self.assertIn("PUBLIC_GO=NOT_DETERMINED", body)
        self.assertIn("RECOVERY_RESTORE=NOT_ASSESSED", body)


if __name__ == "__main__":
    unittest.main()
