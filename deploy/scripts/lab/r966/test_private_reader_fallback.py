"""Offline synthetic-only source selection; never imports the real C320 or reads its credential."""
import hashlib
import importlib.util
import os
import stat
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

AGENT_FILE=Path(__file__).resolve().parents[1]/'r945'/'persistent_c320_connector.py'
spec=importlib.util.spec_from_file_location('r966_candidate_fixed_reader',AGENT_FILE)
agent=importlib.util.module_from_spec(spec)
spec.loader.exec_module(agent)

class FixedOwnerReaderFallback(unittest.TestCase):
    def fixture(self,root):
        private=Path(root)/'owner'
        private.mkdir(mode=0o700)
        deploy=private/'r966-private-reader-recovery'
        for child in [deploy,deploy/'deploy',deploy/'deploy/scripts',deploy/'deploy/scripts/lab',
                      deploy/'deploy/scripts/lab/r938',deploy/'deploy/scripts/lab/r940']:
            child.mkdir(mode=0o700)
        lab=deploy/'deploy/scripts/lab'
        reader=lab/'r940/owner_supervised_c320_read_agent.py'
        parser=lab/'r938/owner_c320_onu_first_inventory.py'
        reader.write_bytes(b'# synthetic r940 source\n')
        parser.write_bytes(b'# synthetic r938 source\n')
        reader.chmod(0o600)
        parser.chmod(0o600)
        return private,deploy,reader,parser

    def test_00_adjacent_legacy_cache_never_overrides_sealed_private_reader(self):
        with tempfile.TemporaryDirectory() as t:
            private,recovery,reader,parser=self.fixture(t)
            legacy=Path(t)/'old-stage.py'
            legacy.write_text('# legacy synthetic cache not trusted\n')
            # Presence and even symlinks in a disposable stage must not
            # override the vetted private reader or bypass its checksum.
            with patch.object(agent,'SOURCE',legacy),patch.object(agent,'RECOVERY_ROOT',recovery),\
                 patch.object(agent,'RECOVERY_R940_SHA256',hashlib.sha256(reader.read_bytes()).hexdigest()),\
                 patch.object(agent,'RECOVERY_R938_SHA256',hashlib.sha256(parser.read_bytes()).hexdigest()):
                self.assertEqual(agent.fixed_reader_source(),reader)
                legacy.unlink()
                legacy.symlink_to('/tmp/invalid')
                self.assertEqual(agent.fixed_reader_source(),reader)
                reader.unlink()
                with self.assertRaisesRegex(FileNotFoundError,'owner_supervised'):
                    agent.fixed_reader_source()

    def test_01_no_adjacent_source_requires_exact_owner_private_checksum(self):
        with tempfile.TemporaryDirectory() as t:
            private,recovery,reader,parser=self.fixture(t)
            missing=Path(t)/'nonexistent.py'
            with patch.object(agent,'SOURCE',missing),patch.object(agent,'RECOVERY_ROOT',recovery),\
                 patch.object(agent,'RECOVERY_R940_SHA256',hashlib.sha256(reader.read_bytes()).hexdigest()),\
                 patch.object(agent,'RECOVERY_R938_SHA256',hashlib.sha256(parser.read_bytes()).hexdigest()):
                self.assertEqual(agent.fixed_reader_source(),reader)
                reader.write_bytes(b'# malicious modified source\n')
                with self.assertRaisesRegex(ValueError,'provenance mismatch'):
                    agent.fixed_reader_source()

    def test_02_parser_checksum_is_independent_and_fail_closed(self):
        with tempfile.TemporaryDirectory() as t:
            private,recovery,reader,parser=self.fixture(t)
            with patch.object(agent,'SOURCE',Path(t)/'missing'),patch.object(agent,'RECOVERY_ROOT',recovery),\
                 patch.object(agent,'RECOVERY_R940_SHA256',hashlib.sha256(reader.read_bytes()).hexdigest()),\
                 patch.object(agent,'RECOVERY_R938_SHA256','0'*64):
                with self.assertRaisesRegex(ValueError,'provenance mismatch'):
                    agent.fixed_reader_source()

    def test_03_rejects_public_permission_symlink_and_private_parent_drift(self):
        with tempfile.TemporaryDirectory() as t:
            private,recovery,reader,parser=self.fixture(t)
            expected_reader=hashlib.sha256(reader.read_bytes()).hexdigest()
            expected_parser=hashlib.sha256(parser.read_bytes()).hexdigest()
            with patch.object(agent,'SOURCE',Path(t)/'absent'),patch.object(agent,'RECOVERY_ROOT',recovery),\
                 patch.object(agent,'RECOVERY_R940_SHA256',expected_reader),\
                 patch.object(agent,'RECOVERY_R938_SHA256',expected_parser):
                reader.chmod(0o644)
                with self.assertRaisesRegex(ValueError,'invalid private owner file'):
                    agent.fixed_reader_source()
                reader.chmod(0o600)
                root=reader.parent
                reader.rename(root/'private-original')
                reader.symlink_to(root/'private-original')
                with self.assertRaisesRegex(ValueError,'invalid private owner file'):
                    agent.fixed_reader_source()
                reader.unlink();(root/'private-original').rename(reader)
                private.chmod(0o755)
                with self.assertRaisesRegex(ValueError,'invalid private owner directory'):
                    agent.fixed_reader_source()
                private.chmod(0o700)
                self.assertEqual(agent.fixed_reader_source(),reader)

    def test_04_does_not_fall_back_to_unknown_dynamic_source(self):
        with tempfile.TemporaryDirectory() as t:
            private,recovery,reader,parser=self.fixture(t)
            with patch.object(agent,'SOURCE',Path(t)/'absent'),patch.object(agent,'RECOVERY_ROOT',recovery):
                with self.assertRaisesRegex(ValueError,'provenance mismatch'):
                    agent.fixed_reader_source()

if __name__=='__main__':unittest.main()
