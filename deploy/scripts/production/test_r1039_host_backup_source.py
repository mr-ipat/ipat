"""R10.39 encrypted off-VPS host backup stream: fail-closed contracts only."""
import importlib.util
from pathlib import Path
import os
import subprocess
import unittest
from unittest import mock

SCRIPT=Path(__file__).resolve().parent/"r1039/backup_host_to_mac_restic.py"
spec=importlib.util.spec_from_file_location("ipat_r1039_host_stream",SCRIPT)
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class R1039HostBackupSource(unittest.TestCase):
    def test_source_has_no_credentials_or_mutating_vps_actions(self):
        text=SCRIPT.read_text()
        for forbidden in ("F52q", "sshpass", "PASSWORD=", "PGPASSWORD=",
                          "apt-get install", "pg_dropcluster", "rm -rf",
                          "systemctl restart", "iptables ", "nft ",
                          "curl -X POST", "kubectl apply"):
            self.assertNotIn(forbidden,text)
        self.assertIn('"public_go": False',text)
        self.assertIn('"independent_boot_restore_verified": False',text)
        self.assertIn('"root_owned_recovery_attestation_created": False',text)

    def test_tar_is_archive_only_and_covers_boot_efi_independently(self):
        root=module.tar_args(True)
        efi=module.tar_args(False)
        self.assertEqual(root[:5],["sudo","-S","--","/usr/bin/tar","-c"])
        self.assertEqual(root[-3:],["-C","/","."])
        self.assertIn("--one-file-system",root)
        self.assertIn("--exclude=./home/openai/.cache",root)
        self.assertNotIn("--exclude=./etc",root)
        self.assertNotIn("--exclude=./root",root)
        self.assertNotIn("--exclude=./var/lib",root)
        self.assertEqual(efi[-3:],["-C","/boot/efi","."])
        self.assertNotIn("--one-file-system",efi)
        for args in (root,efi):
            self.assertNotIn("--extract",args)
            self.assertNotIn("--delete",args)

    def test_restic_context_rejects_remote_or_unknown_repo(self):
        with mock.patch.object(module.sys, "platform", "darwin"), mock.patch.object(
                module.os,"geteuid",return_value=501):
            with self.assertRaisesRegex(ValueError,"existing local"):
                module.restic_env(Path("/tmp/missing-ipat-r1039"))
            with mock.patch.dict(module.os.environ,{"PGPASSWORD":"not-a-real-secret"}):
                with self.assertRaisesRegex(ValueError,"security-sensitive"):
                    repo=Path.home()/"Backups/IPAT/R1039-host-recovery-current"
                    if (repo/"config").is_file():
                        module.restic_env(repo)

    def test_local_encrypted_repo_keychain_command_only(self):
        self.assertIn("/usr/bin/security find-generic-password",module.KEYCHAIN_PASSWORD_COMMAND)
        self.assertNotIn("RESTIC_PASSWORD=",SCRIPT.read_text())
        self.assertNotIn("RESTIC_PASSWORD_FILE=",SCRIPT.read_text())

    def test_ssh_transport_key_pinning_and_safe_capture(self):
        text=SCRIPT.read_text()
        self.assertIn('"StrictHostKeyChecking=yes"',text)
        self.assertIn('"BatchMode=yes"',text)
        self.assertIn('"ssh", "-T"',text)
        self.assertIn('sha256()',text)
        self.assertIn('if tar_status or restic.returncode or not count:',text)
        self.assertIn('raise ValueError(',text)
        self.assertNotIn('print(error_bytes',text)
        self.assertNotIn('print(sudo_password',text)


if __name__=="__main__":
    unittest.main()
