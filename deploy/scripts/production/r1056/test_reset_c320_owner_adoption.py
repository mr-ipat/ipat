import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SOURCE=Path(__file__).with_name("reset_c320_owner_adoption.py")
spec=importlib.util.spec_from_file_location("r1056",SOURCE)
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class R1056ResetTests(unittest.TestCase):
    def fixture(self,root):
        base=Path(root)/"ipat"
        base.mkdir(mode=0o700)
        registry=base/"r961-device-inventory"
        registry.mkdir(mode=0o700)
        secrets=base/"r945-connection"
        secrets.mkdir(mode=0o700)
        record={
          "next_id":0,
          "linked_c320_label":"ZTE C320 Lab",
          "linked_revision":2,
          "linked_c320_pop":None,
          "devices":[],
          "pops":[],
          "audit":[{"sequence":1,"operation":"CREATE","device_id":"DEV-01","revision":1}]
        }
        (registry/"devices.json").write_text(json.dumps(record)+"\n")
        (secrets/"device.fernet").write_bytes(b"ENCRYPTED_FIXTURE_ONLY")
        (secrets/"device-draft.json").write_text('{"device_name":"ZTE C320 Lab","device_profile":"zte_c320_lab"}')
        (secrets/"envelope-key").write_bytes(b"PLACEHOLDER_KEY_NOT_REAL")
        for f in registry.iterdir():f.chmod(0o600)
        for f in secrets.iterdir():f.chmod(0o600)
        return base,record

    def test_immutable_reset_rejects_double_reset_and_preserves_other_fields(self):
        with tempfile.TemporaryDirectory() as td:
            root,old=self.fixture(td)
            data=json.loads((root/"r961-device-inventory/devices.json").read_text())
            raw,updated=module.updated_registry(json.dumps(data).encode())
            self.assertIsNone(updated["linked_c320_label"])
            self.assertTrue(updated["linked_c320_hidden_until_reenrollment"])
            self.assertEqual(updated["linked_revision"],3)
            self.assertEqual(updated["audit"][-1]["operation"],"RESET_LINKED_READ_ADOPTION")
            self.assertEqual(updated["devices"],[])
            with self.assertRaises(RuntimeError):
                module.updated_registry(raw)

    def test_full_disposable_reset_and_private_reenrollment_token(self):
        with tempfile.TemporaryDirectory() as td:
            root,_=self.fixture(td)
            calls=[]
            with patch.object(module,"BASE",root), \
                 patch.object(module,"REGISTRY",root/"r961-device-inventory/devices.json"), \
                 patch.object(module,"SECRETS",root/"r945-connection"), \
                 patch.object(module,"ARCHIVE_DIR",root/"r1056-c320-readoption"), \
                 patch.object(module,"systemd",side_effect=lambda action,unit:calls.append((action,unit))), \
                 patch.object(module,"verify_reset"), \
                 patch.dict(os.environ,{"IPAT_R1056_CONFIRM":module.PROTOCOL}):
                module.do_reset()
            self.assertEqual(len(calls),4)
            self.assertFalse((root/"r945-connection/device.fernet").exists())
            self.assertFalse((root/"r945-connection/device-draft.json").exists())
            code=(root/"r945-connection/bootstrap-token").read_text().strip()
            self.assertGreaterEqual(len(code),40)
            self.assertEqual((root/"r945-connection/bootstrap-sha256").read_bytes(),
                             module.hashlib.sha256(code.encode()).digest())
            for f in ("bootstrap-token","bootstrap-sha256"):
                self.assertEqual((root/"r945-connection"/f).stat().st_mode & 0o777,0o600)
            archive=next((root/"r1056-c320-readoption").iterdir())
            self.assertEqual((archive/"device.fernet").read_bytes(),b"ENCRYPTED_FIXTURE_ONLY")
            d=json.loads((root/"r961-device-inventory/devices.json").read_text())
            self.assertTrue(d["linked_c320_hidden_until_reenrollment"])
            self.assertEqual(d["audit"][-1]["operation"],"RESET_LINKED_READ_ADOPTION")

    def test_postcondition_failure_rolls_back_after_both_services_restart(self):
        with tempfile.TemporaryDirectory() as td:
            root,original=self.fixture(td)
            calls=[]
            with patch.object(module,"BASE",root), \
                 patch.object(module,"REGISTRY",root/"r961-device-inventory/devices.json"), \
                 patch.object(module,"SECRETS",root/"r945-connection"), \
                 patch.object(module,"ARCHIVE_DIR",root/"r1056-c320-readoption"), \
                 patch.object(module,"systemd",side_effect=lambda action,unit:calls.append((action,unit))), \
                 patch.object(module,"verify_reset",side_effect=RuntimeError("synthetic postcondition refused")), \
                 patch.dict(os.environ,{"IPAT_R1056_CONFIRM":module.PROTOCOL}):
                with self.assertRaises(RuntimeError):
                    module.do_reset()
            self.assertEqual(json.loads((root/"r961-device-inventory/devices.json").read_text()),original)
            self.assertTrue((root/"r945-connection/device.fernet").exists())
            self.assertFalse((root/"r945-connection/bootstrap-token").exists())
            self.assertGreaterEqual(len(calls),8)

    def test_restart_failure_restores_original_and_no_bootstrap_left(self):
        with tempfile.TemporaryDirectory() as td:
            root,original=self.fixture(td)
            def failing(action,unit):
                if action=="start" and unit==module.UNITS[1]:
                    raise RuntimeError("synthetic startup failed")
            with patch.object(module,"BASE",root), \
                 patch.object(module,"REGISTRY",root/"r961-device-inventory/devices.json"), \
                 patch.object(module,"SECRETS",root/"r945-connection"), \
                 patch.object(module,"ARCHIVE_DIR",root/"r1056-c320-readoption"), \
                 patch.object(module,"systemd",side_effect=failing), \
                 patch.dict(os.environ,{"IPAT_R1056_CONFIRM":module.PROTOCOL}):
                with self.assertRaises(RuntimeError):
                    module.do_reset()
            self.assertEqual(json.loads((root/"r961-device-inventory/devices.json").read_text()),original)
            self.assertTrue((root/"r945-connection/device.fernet").exists())
            self.assertFalse((root/"r945-connection/bootstrap-token").exists())

if __name__=="__main__":unittest.main()
