"""Entirely offline mock tests; no SSH or customer equipment accessed."""
import importlib.util
from pathlib import Path
import subprocess
import unittest

P = Path(__file__).with_name("private_ssh_identity_probe.py")
spec = importlib.util.spec_from_file_location("r911_probe", P)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)

class NoAuthProbeTests(unittest.TestCase):
    def test_accepts_only_rfc1918_and_exact_port(self):
        self.assertEqual(probe.accepted_target("10.10.13.233", 321), "10.10.13.233")
        for host in ("27.121.113.1", "127.0.0.1", "localhost", "10.1.2.3;echo", "::1"):
            with self.assertRaises(ValueError, msg=host):
                probe.accepted_target(host, 321)
        with self.assertRaises(ValueError):
            probe.accepted_target("10.10.13.233", 0)

    def test_command_cannot_offer_credentials_or_execute_olt_commands(self):
        args=probe.argv("10.10.13.233",321)
        for token in ("PubkeyAuthentication=no", "PasswordAuthentication=no",
                      "KbdInteractiveAuthentication=no", "BatchMode=yes",
                      "StrictHostKeyChecking=yes", "HostKeyAlgorithms=ssh-rsa",
                      "Ciphers=aes128-cbc", "PreferredAuthentications=none",
                      "IdentityAgent=none", "ProxyCommand=none"):
            self.assertIn(token,args)
        self.assertNotIn("StrictHostKeyChecking=no",args)
        self.assertNotIn("show card",args)
        self.assertNotIn("show version-running",args)
        self.assertEqual(args[-1],"unprivileged-noauth-probe@10.10.13.233")

    def test_observed_key_is_never_treated_as_trusted(self):
        calls=[]
        def runner(argv, **kwargs):
            calls.append((argv,kwargs))
            return subprocess.CompletedProcess(argv,255,"",(
                "Remote protocol version 2.0, remote software version ZTE_SSH.1.0\n"
                "Server host key: ssh-rsa SHA256:4YxiisiqdRDJeys0+scIpKuHlEN1yPHynmQ8+NbBK9s\n"
                "Host key verification failed.\n"))
        result=probe.observe("10.10.13.233",321,runner)
        self.assertEqual(len(calls),1)
        self.assertEqual(calls[0][1]["timeout"],20)
        self.assertEqual(result["result"],"UNVERIFIED_PRIVATE_SSH_HOST_KEY")
        self.assertEqual(result["ssh_banner"],"ZTE_SSH.1.0")
        self.assertFalse(result["host_identity_verified"])
        self.assertFalse(result["device_adopted"])
        self.assertFalse(result["credentials_sent"])
        self.assertEqual(result["olt_commands_executed"],0)

    def test_timeout_and_unexpected_exit_fail_closed(self):
        def timeout(*args,**kwargs):
            raise subprocess.TimeoutExpired("ssh",9)
        denied=probe.observe("10.10.13.233",321,timeout)
        self.assertEqual(denied["result"],"TRANSPORT_UNAVAILABLE")
        def strange(argv,**kwargs):
            return subprocess.CompletedProcess(argv,0,"", "no verified key")
        denied=probe.observe("10.10.13.233",321,strange)
        self.assertEqual(denied["result"],"TRANSPORT_INCONCLUSIVE")
        self.assertFalse(denied["device_adopted"])

if __name__=="__main__": unittest.main()

class DashboardEvidenceContract(unittest.TestCase):
    def test_cred_free_real_observation_cannot_be_adoption(self):
        import json
        root=Path(__file__).resolve().parents[4]
        doc=json.loads((root/'web/lab/physical-intake-evidence.json').read_text())
        self.assertEqual(doc['target_slot'],'DEV-01')
        self.assertEqual(doc['mode'],'historical_credential_free_transport_observation')
        self.assertTrue(doc['private_ssh_transport_observed'])
        self.assertFalse(doc['device_adopted'])
        self.assertFalse(doc['credentials_sent'])
        self.assertEqual(doc['olt_commands_executed'],0)
        self.assertEqual(doc['physical_read_test'],'NOT_RUN')
        self.assertEqual(doc['health'],'NOT_MEASURED')
        for key in ('out_of_band_host_key_verified',
                    'management_segment_isolation_verified',
                    'dedicated_readonly_account_verified',
                    'firmware_exact_readonly_commands_verified',
                    'owner_approved_noimpact_baseline_verified',
                    'actual_worker_private_route_verified'):
            self.assertIs(doc[key],False)
        html=(root/'web/lab/device-workbench.html').read_text()
        script=(root/'web/lab/device-workbench.js').read_text()
        api=(root/'apps/control-api/src/device_workbench_lab.rs').read_text()
        self.assertIn('id="physical-evidence-gates"',html)
        self.assertIn('/lab/device-physical-evidence',script)
        self.assertIn('evidence.device_adopted!==false',script)
        self.assertIn('/lab/device-physical-evidence',api)
        self.assertNotIn('password',json.dumps(doc).lower())

class CanarySafetyContract(unittest.TestCase):
    def test_private_canary_runner_never_touches_current_service_or_devices(self):
        source=Path(__file__).with_name('private_loopback_canary_smoke.py').read_text()
        for needed in ('IPAT_R911_CANARY_SMOKE', '127.0.0.1:',
                       "BIN = Path('/home/openai/.cache/ipat/r911-canary/target/debug/control-api')",
                       'IPAT_R911_PRIVATE_CANARY', "get(3000, '/healthz')",
                       "get(3002, '/lab/device-physical-evidence')",
                       "get(3002, '/v1/devices/DEV-01')",
                       "get(3002, '/v1/tenant/overview')",
                       'proc.terminate()', 'sock.listen(1)'):
            self.assertIn(needed,source)
        for disallowed in ('sudo ', 'iptables ', 'nft ', 'ufw ', 'telnet ',
                           'ssh ', 'firewall-cmd', "sock.bind(('0.0.0.0', 3002))"):
            self.assertNotIn(disallowed,source)
