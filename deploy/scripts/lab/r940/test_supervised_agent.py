"""R9.40 synthetic ONLY; NEVER contact real C320 or use real credentials."""
import ast
import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
SCRIPT=Path(__file__).with_name('owner_supervised_c320_read_agent.py')
spec=importlib.util.spec_from_file_location('owner_supervised_c320_read_agent',SCRIPT)
mod=importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

class OwnerReadBoundaryTests(unittest.TestCase):
    def test_no_network_without_owner_tty_and_optin(self):
        p=subprocess.run([sys.executable,str(SCRIPT),'--owner-terminal'],
            input='',capture_output=True,text=True,timeout=3)
        self.assertEqual(p.returncode,4)
        self.assertIn('FAIL_CLOSED',p.stderr)
    def test_owner_limits_and_exact_fixed_command_source(self):
        self.assertEqual(mod.MAX_REQUESTS,5)
        self.assertEqual(mod.SESSION_SECONDS,900)
        self.assertTrue(str(mod.SOCKET).endswith('/r940-live-agent/live.sock'))
        code=SCRIPT.read_text()
        for bad in ('sshpass','StrictHostKeyChecking=no','conf t','onu 1 type','save configuration'):
            self.assertNotIn(bad,code)
        self.assertIn('m.COMMANDS',code)
        self.assertIn('socket.SO_PEERCRED',code)
        self.assertIn('password=getpass.getpass',code)
        self.assertNotIn('password=os.getenv',code)
    def test_exact_owner_reader_fixed_command_list(self):
        p=Path(__file__).parents[1]/'r938'/'owner_c320_onu_first_inventory.py'
        src=p.read_text()
        self.assertIn("'show gpon onu state gpon-olt_1/1/1'",src)
        self.assertIn("'%Code 62310",src)
        ast.parse(src)
if __name__=='__main__':unittest.main()
