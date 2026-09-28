"""Offline unit tests for route-table-only, zero-packet negative gate."""
import importlib.util
from pathlib import Path
import subprocess
import unittest

P=Path(__file__).with_name('verify_restricted_private_route.py')
sp=importlib.util.spec_from_file_location('route_gate',P)
M=importlib.util.module_from_spec(sp);sp.loader.exec_module(M)
IP='10.10.13.233'

class RouteGateTests(unittest.TestCase):
    def test_actual_vps_pattern_default_is_not_verified(self):
        result=M.parse(IP,[{'dst':IP,'gateway':'10.0.0.1','dev':'eth0'}],
                       [{'dst':'default','gateway':'10.0.0.1','dev':'eth0'}])
        self.assertEqual(result['route_candidate'],'DEFAULT_ROUTE_ONLY')
        self.assertFalse(result['isolated_management_route_verified'])
        self.assertEqual(result['network_packets_sent'],0)
    def test_other_routes_also_require_independent_proof(self):
        result=M.parse(IP,[{'dst':IP,'gateway':'10.100.2.1','dev':'wg-ipat'}],
                       [{'dst':'default','gateway':'10.0.0.1','dev':'eth0'}])
        self.assertEqual(result['route_candidate'],'NONDEFAULT_ROUTE_REQUIRES_PROOF')
        self.assertFalse(result['isolated_management_route_verified'])
    def test_rejects_wrong_ip_public_empty_missing_interface(self):
        for ip,route in [('27.121.113.1',[]),(IP,[]),(IP,[{'dst':'10.10.13.234'}])]:
            with self.assertRaises(ValueError): M.parse(ip,route,[])
        with self.assertRaises(ValueError):M.parse(IP,[{'dst':IP}],[])
    def test_exact_two_local_route_calls_without_packets(self):
        calls=[]
        def fake(cmd,**kwargs):
            calls.append(cmd)
            value=('[{"dst":"10.10.13.233","gateway":"10.0.0.1","dev":"eth0"}]'
                   if 'get' in cmd else
                   '[{"dst":"default","gateway":"10.0.0.1","dev":"eth0"}]')
            return subprocess.CompletedProcess(cmd,0,value,'')
        outcome=M.inspect(IP,fake)
        self.assertEqual(outcome['route_candidate'],'DEFAULT_ROUTE_ONLY')
        self.assertEqual(calls,[['ip','-j','-4','route','get',IP],
                                ['ip','-j','-4','route','show','default']])
        self.assertNotIn('ping',P.read_text())
        self.assertNotIn('ssh ',P.read_text())

if __name__=='__main__':unittest.main()
