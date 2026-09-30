"""Offline synthetic parser and noninteractive denial only; no hardware network."""
import importlib.util, os, subprocess, sys, unittest
from pathlib import Path
FILE=Path(__file__).with_name('owner_c320_onu_first_inventory.py')
spec=importlib.util.spec_from_file_location('r938_c320_owner_read',FILE)
M=importlib.util.module_from_spec(spec);spec.loader.exec_module(M)

class R938Safety(unittest.TestCase):
    def test_three_command_fixed_allowlist(self):
        self.assertEqual(M.COMMANDS,('show gpon onu uncfg','show gpon onu state gpon-olt_1/1/1',
            'show run interface gpon-olt_1/1/1'))
        source=FILE.read_text()
        for bad in ('sshpass','StrictHostKeyChecking=no','conf t','onu 1 type','save configuration'):
            self.assertNotIn(bad,source)
    def test_synthetic_unconfigured(self):
        raw=b'OnuIndex Sn State\n---\ngpon-onu_1/1/1:1 ZTEGABCDEF12 unknown\n'
        self.assertEqual(M.classify(M.COMMANDS[0],M.bounded(raw,M.COMMANDS[0]))['rows'],1)
        self.assertEqual(M.classify(M.COMMANDS[0],b'%Code 62310-GPONSRV : No related information to show.\n')['rows'],0)
    def test_rejects_unrecognized_or_secret_console(self):
        for raw in (b'',b'Invalid command',b'--More--',b'\x1b[0m',b'A'*32769):
            with self.assertRaises(M.Denied):M.bounded(raw,M.COMMANDS[0])
        with self.assertRaises(M.Denied):M.classify(M.COMMANDS[0],b'something unexpected')
    def test_state_and_registered_count(self):
        state=b'OnuIndex Admin State OMCC State Phase State Channel\n1/1/1:2 enable disable OffLine 1(GPON)\nONU Number: 0/1\n'
        self.assertEqual(M.classify(M.COMMANDS[1],state),{'shape':'ONU_STATE_TABLE','rows':1,'online':0,'offline':1})
        reg=b'Building configuration...\ninterface gpon-olt_1/1/1\n onu 1 type TEST sn ZTEGABCDEF12\nend\n'
        self.assertEqual(M.classify(M.COMMANDS[2],reg)['rows'],1)
    def test_owner_firmware_seventy_two_row_state_synthetic_shape(self):
        raw=b'OnuIndex   Admin State OMCC State Phase State Channel\n'
        raw+=b'--------------------------------------------------\n'
        raw+=b''.join(f'1/1/1:{i} enable disable OffLine 1(GPON)\n'.encode() for i in range(1,73))
        raw+=b'ONU Number: 0/72\n'
        actual_shape=M.classify(M.COMMANDS[1],M.bounded(raw,M.COMMANDS[1]))
        self.assertEqual(actual_shape,{'shape':'ONU_STATE_TABLE','rows':72,'online':0,'offline':72})
        self.assertNotIn('serial',str(actual_shape).lower())
    def test_remote_noninteractive_blocked(self):
        env=os.environ.copy();env[M.APPROVAL]='YES'
        p=subprocess.run([sys.executable,str(FILE),'--owner-interactive-read'],
          input='',capture_output=True,text=True,env=env,timeout=3)
        self.assertEqual(p.returncode,4)
        self.assertIn('FAIL_CLOSED',p.stderr)
if __name__=='__main__':unittest.main()
