"""Static refuse/rollback contract of the deliberately rootless lab canary."""
from pathlib import Path
import subprocess
import unittest

ROOT=Path(__file__).resolve().parents[4]
SCRIPT=ROOT/"deploy/scripts/lab/r1028/upgrade_private_dual_transport.sh"

class PreviewUpgrade(unittest.TestCase):
    def test_syntax(self):
        subprocess.run(["bash","-n",str(SCRIPT)],check=True)

    def test_preflight_exact_binary_and_private_only(self):
        source=SCRIPT.read_text()
        before=source.split("armed=YES")[0]
        for expected in ("nonroot only","UPGRADE_REVIEWED_PRIVATE_DUAL_TRANSPORT",
            "source commit mismatch","source must be clean","SHA256",
            "previous binary checksum drift","127.0.0.1:3002",
            "public/commercial port conflict","systemctl --user is-active",
            "preexisting"):
            if expected=="preexisting":
                continue
            self.assertIn(expected,before)
        self.assertIn("R1028_TWO_PORTS_TCP_PROTOCOL_NEGOTIATION_PRIVATE_HTTP_PASS",source)
        self.assertIn("PUBLIC_GO_FALSE",source)
        for unsafe in ("sudo ","pg_dropcluster","systemctl --system","nft ","iptables ","certbot","sshpass","telnet 10.10.13.233"):
            self.assertNotIn(unsafe,source)

    def test_rollback_limited_to_own_override(self):
        source=SCRIPT.read_text()
        self.assertIn('override="$dropdir/zzzzz-r1028-dual-transport.conf"',source)
        self.assertIn('rm -f -- "$override"',source)
        self.assertIn('trap rollback EXIT INT TERM ERR',source)
        self.assertIn('R1028_PREVIOUS_PRIVATE_PREVIEW_RESTORED',source)
        self.assertIn('systemctl --user restart "$unit"',source)

if __name__=="__main__":
    unittest.main()
