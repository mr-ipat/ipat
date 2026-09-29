"""Offline synthetic parser and noninteractive denial only; no hardware network."""
import importlib.util, os, subprocess, sys, unittest
from pathlib import Path
FILE=Path(__file__).with_name('owner_c320_onu_first_inventory.py')
spec=importlib.util.spec_from_file_location('r938_c320_owner_read',FILE)
M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)

class R938Safety(unittest.TestCase):
    def test_three_command_fixed_allowlist(self):
        self.assertEqual(M.COMMANDS,('show gpon onu uncfg','show gpon onu state',
            'show run interface gpon-olt_1/1/1'))
        source=FILE.read_text()
        for bad in ('sshpass','StrictHostKeyChecking=no','conf t','onu 1 type','save configuration'):
            self.assertNotIn(bad,source)
    def test_synthetic_unconfigured(self):
        raw=b'OnuIndex Sn State\n---\ngpon-onu_1/1/1:1 ZTEGABCDEF12 unknown\n'
        self.assertEqual(M.classify(M.COMMANDS[0],M.bounded(raw,M.COMMANDS[0]))['rows'],1)
        self.assertEqual(M.classify(M.COMMANDS[0],b'%Code 32310-GPONSRV : No related information to show.\n')['rows'],0)
    def test_rejects_unrecognized_or_secret_console(self):
        for raw in (b'',b'Invalid command',b'--More--',b'\x1b[0m',b'A'*32769):
            with self.assertRaises(M.Denied):M.bounded(raw,M.COMMANDS[0])
        with self.assertRaises(M.Denied):M.classify(M.COMMANDS[0],b'something unexpected')
    def test_state_and_registered_count(self):
        state=b'OnuIndex Admin State OMCC State O7 State Phase State\ngpon-onu_1/1/1:1 enable enable operation working\n'
        self.assertEqual(M.classify(M.COMMANDS[1],state)['rows'],1)
        reg=b'Building configuration...\ninterface gpon-olt_1/1/1\n onu 1 type TEST sn ZTEGABCDEF12\nend\n'
        self.assertEqual(M.classify(M.COMMANDS[2],reg)['rows'],1)
    def test_remote_noninteractive_blocked(self):
        env=os.environ.copy();env[M.APPROVAL]='YES'
        p=subprocess.run([sys.executable,str(FILE),'--owner-interactive-read'],
          input='',capture_output=True,text=True,env=env,timeout=3)
        self.assertEqual(p.returncode,4)
        self.assertIn('FAIL_CLOSED',p.stderr)
if __name__=='__main__':unittest.main()
