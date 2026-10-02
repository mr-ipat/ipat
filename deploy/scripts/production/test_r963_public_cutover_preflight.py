import importlib.util, unittest
from pathlib import Path
p=Path(__file__).resolve().parent/'r963_public_cutover_preflight.py'
spec=importlib.util.spec_from_file_location('r963_preflight',p)
mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
class TestPreflight(unittest.TestCase):
    def test_no_unverified_public_green_even_if_tcp_listens(self):
        fake={'available':True,'OS':'ubuntu:26.04','SERVICES':'active,active,',
          'PORT443':'1','PRIVILEGED_ADMIN':'VERIFIED_NOINPUT_SUDO'}
        domain={'platform':{'temporary_public_ipv4':'198.51.100.9',
          'public_https_ready':True,'safe_to_point_now':True}}
        devices={'devices':[{'id':'DEV-01','connection_state':'CONNECTED'}]}
        old=mod.tcp_connect;mod.tcp_connect=lambda ip,port:True
        try:r=mod.snapshot(fake,domain,devices)
        finally:mod.tcp_connect=old
        self.assertIs(r['public_go'],False)
        self.assertEqual(r['production_status'],'BLOCKED')
        self.assertEqual(r['gates']['actual_dns_zone_ownership_and_txt'],'UNVERIFIED')
        self.assertEqual(r['gates']['commercial_postgresql_runtime_and_pitr'],'UNVERIFIED')
    def test_host_without_sudo_and_443_returns_blocked(self):
        fake={'available':True,'PORT443':'0','PRIVILEGED_ADMIN':'NOT_AVAILABLE'}
        r=mod.snapshot(fake,{'platform':{}},{'devices':[]})
        self.assertFalse(r['gates']['real_privileged_deployment_identity'])
        self.assertFalse(r['gates']['local_port_443_listening'])
        self.assertFalse(r['gates']['private_c320_live_read_only'])
if __name__=='__main__':unittest.main()
