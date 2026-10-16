import importlib.util
from pathlib import Path
import hashlib
import os
import stat
import subprocess
from unittest import mock
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("r1034_readonly",HERE/"read_only_foundation_gate.py")
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class R1034Source(unittest.TestCase):
    def test_source_has_no_privileged_or_destructive_commands(self):
        source=(HERE/"read_only_foundation_gate.py").read_text()
        for disallowed in ("sudo ","apt-get install","apt-get update","pg_dropcluster","rm -rf",
                           "systemctl restart","iptables ","nft ","sshpass","password="):
            self.assertNotIn(disallowed,source)
        self.assertIn('"public_go":False',source)

    def test_candidate_manifest_exact_file_set(self):
        with tempfile.TemporaryDirectory() as temp:
            parent=Path(temp)
            commit="a"*40
            binary=parent/"control-api"
            binary.write_bytes(b"synthetic-no-executable")
            binary.chmod(0o500)
            source=parent/"SOURCE_COMMIT"
            source.write_text(commit+"\n")
            manifest=parent/"SHA256SUMS"
            manifest.write_text(
                hashlib.sha256(binary.read_bytes()).hexdigest()+"  control-api\n"+
                hashlib.sha256(source.read_bytes()).hexdigest()+"  SOURCE_COMMIT\n")
            digest=hashlib.sha256(binary.read_bytes()).hexdigest()
            module.candidate_check(parent,commit,digest)
            manifest.write_text(manifest.read_text()+"\n")
            with self.assertRaisesRegex(ValueError,"invalid or duplicate"):
                module.candidate_check(parent,commit,digest)

    def test_listening_ports_detected_and_inspection_failure_fails_closed(self):
        with mock.patch.object(module,"command",return_value=subprocess.CompletedProcess(
                ["ss"],0,"LISTEN 0 16 127.0.0.1:443 0.0.0.0:*\n","")) as called:
            self.assertEqual(module.inspected_port_state(443),"LISTENING")
            called.assert_called_once_with("ss","-H","-ltn","( sport = :443 )")
        with mock.patch.object(module,"command",return_value=subprocess.CompletedProcess(
                ["ss"],0,"","")):
            self.assertEqual(module.inspected_port_state(443),"NOT_LISTENING")
        with mock.patch.object(module,"command",return_value=subprocess.CompletedProcess(
                ["ss"],1,"","error")):
            with self.assertRaisesRegex(ValueError,"unable to inspect"):
                module.inspected_port_state(443)

    def test_symlink_and_wrong_binary_mode_denied(self):
        with tempfile.TemporaryDirectory() as temp:
            directory=Path(temp)
            binary=directory/"control-api"
            binary.write_bytes(b"synthetic")
            binary.chmod(0o600)
            (directory/"SOURCE_COMMIT").write_text("a"*40)
            (directory/"SHA256SUMS").write_text("")
            with self.assertRaisesRegex(ValueError,"permission drift"):
                module.candidate_check(directory,"a"*40,hashlib.sha256(b"synthetic").hexdigest())
            binary.unlink()
            binary.symlink_to(directory/"SOURCE_COMMIT")
            with self.assertRaisesRegex(ValueError,"unsafe candidate"):
                module.candidate_check(directory,"a"*40,hashlib.sha256(b"synthetic").hexdigest())


if __name__=="__main__":
    unittest.main()
