"""R10.36 root release custody: fail-closed source and nonroot refusal."""
from pathlib import Path
import re
import subprocess
import unittest

ROOT=Path(__file__).resolve().parents[2]
SCRIPT=ROOT/"scripts/production/r1036_root_release_handoff.sh"


class RootReleaseCustodyTests(unittest.TestCase):
    def test_shell_syntax(self):
        result=subprocess.run(["bash","-n",str(SCRIPT)],capture_output=True,text=True)
        self.assertEqual(result.returncode,0,result.stderr)

    def test_nonroot_call_refuses_before_mutation(self):
        if subprocess.run(["id","-u"],capture_output=True,text=True).stdout.strip()=="0":
            self.skipTest("CI must execute this refusal case as nonroot")
        result=subprocess.run(["bash",str(SCRIPT)],capture_output=True,text=True,timeout=4)
        self.assertEqual(result.returncode,4)
        self.assertEqual(result.stdout.strip(),"")

    def test_fixed_reviewed_commit_and_binary_digest(self):
        text=SCRIPT.read_text()
        self.assertRegex(text,r"commit=faf0b25e6f2456820c6a088719b26d0807fc0154")
        self.assertRegex(text,r"digest=7e041f843668e77d46c89c2fb7cc61972b794af12f3b674beb284cba3ecf91cd")
        self.assertRegex(text,r'git -C "\$source" fsck --full --no-reflogs')
        self.assertIn('git clone --quiet --no-local --no-checkout "$upstream" "$source"',text)
        self.assertIn('status --porcelain --untracked-files=all --ignored=matching',text)
        self.assertIn('test -z "$unsafe"',text) if 'test -z "$unsafe"' in text else self.assertIn('[[ -z $unsafe ]]',text)
        self.assertIn('install -o root -g root -m 0500 "$input" "$out/control-api"',text)
        self.assertIn('R1036_PUBLIC_GO=FALSE',text)

    def test_no_side_effects_outside_root_release_custody(self):
        text=SCRIPT.read_text()
        for forbidden in (
            "apt-get","pg_createcluster","pg_dropcluster","systemctl",
            "iptables", "nft ", "ufw ", "sshpass", "curl ", "wget ",
            "firmware", "show running", "R996_APPLY", "chmod 777",
        ):
            self.assertNotIn(forbidden,text)
        self.assertNotRegex(text,r'(?m)^\s*(?:rm|rmdir)\s')
        self.assertNotRegex(text,r'(?m)^\s*(?:cat|echo|printf).*password')

if __name__=="__main__":
    unittest.main()
