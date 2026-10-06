"""R9.45 persistent owner-C320 connector: no real network credentials or SSH."""
import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

FILE=Path(__file__).with_name('persistent_c320_connector.py')
spec=importlib.util.spec_from_file_location('persistent_c320_connector_test',FILE)
agent=importlib.util.module_from_spec(spec)
spec.loader.exec_module(agent)

class PersistentC320Tests(unittest.TestCase):
    def test_one_time_enrollment_encrypts_and_persists_without_plaintext(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'secret'
            parent=Path(t)/'sock'
            sock=parent/'live.sock'
            with patch.object(agent,'ROOT',root),patch.object(agent,'TOKEN',root/'bootstrap-token'),\
                 patch.object(agent,'TOKEN_HASH',root/'bootstrap-sha256'),\
                 patch.object(agent,'KEY',root/'envelope-key'),\
                 patch.object(agent,'CREDENTIAL',root/'device.fernet'),\
                 patch.object(agent,'PARENT',parent),patch.object(agent,'SOCKET',sock):
                agent.init()
                token=(root/'bootstrap-token').read_text().strip()
                fake=SimpleNamespace(run_three_reads=lambda m,p,a:{
                     'cards_in_service':3} if a==b'CARDS\n' else {})
                response=agent.verify_and_store(token,'TEST-ONLY-DEVICE-PASSWORD',fake,None)
                self.assertTrue(response['enrolled_for_read'])
                self.assertEqual(response['commercial_production_adopted'],False)
                self.assertFalse((root/'bootstrap-token').exists())
                self.assertFalse((root/'bootstrap-sha256').exists())
                self.assertNotIn(b'TEST-ONLY-DEVICE-PASSWORD',(root/'device.fernet').read_bytes())
                self.assertEqual(agent.get_password(),'TEST-ONLY-DEVICE-PASSWORD')
                self.assertTrue(agent.status()['persistent_connector'])
                self.assertTrue(agent.status()['agent_ready'])
                self.assertFalse(agent.status()['device_adopted'])
                self.assertEqual(oct((root/'device.fernet').stat().st_mode & 0o777),'0o600')
                with self.assertRaises(ValueError):
                    agent.verify_and_store(token,'NEW_PASSWORD',fake,None)
                agent.STATE['last_verified']=''
                self.assertTrue(agent.status()['agent_ready'])
                self.assertEqual(agent.status()['last_verified_at_utc'],'')
    def test_wrong_code_must_not_create_encrypted_credential(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'secret'
            parent=Path(t)/'sock'
            with patch.object(agent,'ROOT',root),patch.object(agent,'TOKEN',root/'bootstrap-token'),\
                 patch.object(agent,'TOKEN_HASH',root/'bootstrap-sha256'),\
                 patch.object(agent,'KEY',root/'envelope-key'),\
                 patch.object(agent,'CREDENTIAL',root/'device.fernet'),\
                 patch.object(agent,'PARENT',parent),patch.object(agent,'SOCKET',parent/'live.sock'):
                agent.init()
                with self.assertRaises(ValueError):
                    agent.verify_and_store('invalid_token_value_which_is_long_but_fake',
                        'SYNTHETIC',None,None)
                self.assertFalse(agent.enrolled())
    def test_stale_socket_cleanup_rejects_an_active_listener(self):
        import socket
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            path=root/'owner.sock'
            server=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
            server.bind(str(path))
            path.chmod(0o600)
            server.listen(1)
            with patch.object(agent,'SOCKET',path):
                with self.assertRaises(ValueError):
                    agent.ensure_socket_available()
                self.assertTrue(path.exists())
                server.close()
                agent.ensure_socket_available()
                self.assertFalse(path.exists())
                path.symlink_to(root/'nonexistent')
                with self.assertRaises(ValueError):
                    agent.ensure_socket_available()

    def test_pending_device_draft_is_persisted_without_password(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'private'
            parent=Path(t)/'socket'
            with patch.object(agent,'ROOT',root), patch.object(agent,'TOKEN',root/'bootstrap-token'), \
                 patch.object(agent,'TOKEN_HASH',root/'bootstrap-sha256'), \
                 patch.object(agent,'KEY',root/'envelope-key'), \
                 patch.object(agent,'CREDENTIAL',root/'device.fernet'), \
                 patch.object(agent,'DRAFT',root/'device-draft.json'), \
                 patch.object(agent,'PARENT',parent),patch.object(agent,'SOCKET',parent/'live.sock'):
                agent.init()
                saved=agent.save_draft('Core OLT Lab')
                self.assertTrue(saved['draft_saved'])
                self.assertEqual(saved['adoption_state'],'DRAFT_SAVED_AWAITING_AUTH')
                self.assertFalse(agent.enrolled())
                self.assertFalse((root/'device.fernet').exists())
                self.assertEqual(agent.load_draft()['device_name'],'Core OLT Lab')
                self.assertEqual(agent.status()['device_name'],'Core OLT Lab')
                self.assertTrue(agent.status()['draft_saved'])
                self.assertEqual(oct((root/'device-draft.json').stat().st_mode & 0o777),'0o600')
                with self.assertRaises(ValueError):
                    agent.save_draft('a'+chr(10)+'b')

    def test_paged_refresh_helpers_are_bounded_and_prefix_only(self):
        class PX:
            EOF=object(); TIMEOUT=object()
        class Child:
            def __init__(self,events):
                self.events=list(events);self.before=b'';self.sent=[];self.lines=[]
            def sendline(self,v):self.lines.append(v)
            def send(self,v):self.sent.append(v)
            def expect(self,patterns,timeout):
                idx,chunk=self.events.pop(0);self.before=chunk;return idx
        child=Child([(1,b'page1\n'),(0,b'\x08'*18+b' 1/1/1:2 enable disable OffLine 1(GPON)\n')])
        raw,pages=agent._paged_read(child,b'P',agent.PAGED_COMMANDS[1],PX)
        self.assertEqual(pages,1)
        self.assertEqual(child.sent,[b' '])
        clean=agent._normalize_pager_artifacts(raw,pages)
        self.assertNotIn(b'\x08',clean)
        self.assertIn(b'1/1/1:2',clean)
        with self.assertRaises(ValueError):
            agent._paged_read(Child([]),b'P','show users',PX)
        with self.assertRaises(ValueError):
            agent._normalize_pager_artifacts(b'1/1/1:\x082 bad\n',1)
        with self.assertRaises(ValueError):
            agent._normalize_pager_artifacts(b'\x08 bad\n',0)

    def test_command_allowlist_and_fixed_transport(self):
        src=FILE.read_text()
        self.assertIn("SAFE_ACTIONS = {b'REFRESH",src)
        self.assertIn("if uid != os.getuid()",src)
        self.assertIn("if enrolled():",src)
        self.assertNotIn("StrictHostKeyChecking=no",src)
        self.assertNotIn("sshpass",src)
        self.assertNotIn("os.getenv('PASSWORD'",src)

if __name__=='__main__':unittest.main()
