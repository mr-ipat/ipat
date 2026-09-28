import importlib.util
import subprocess
from pathlib import Path
import unittest

DIR=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('r917',DIR/'protocol_selection.py')
P=importlib.util.module_from_spec(spec);spec.loader.exec_module(P)
proto=importlib.util.spec_from_file_location('r917_tls',DIR/'private_tls_noauth_probe.py')
T=importlib.util.module_from_spec(proto);proto.loader.exec_module(T)

def opts(**kw):
    d=dict(device_profile='zte_c320',endpoint_ip='10.10.13.233',
           path_observed=True,management_segment_isolated=False,
           exact_protocol_on_firmware_verified=False,
           exact_device_identity_independently_verified=False,
           restricted_account_verified=False,actual_service_baseline_approved=False,
           candidate_protocol='ssh_pinned')
    d.update(kw)
    return P.Evidence(**d)

class DirectFirstTests(unittest.TestCase):
    def test_direct_private_c320_prioritized_and_no_assumed_api_ssl(self):
        outcome=P.evaluate(opts())
        self.assertEqual(outcome['preferred_transport'],'DIRECT_TO_DEVICE_OVER_EXISTING_NETWORK')
        self.assertFalse(outcome['wireguard_required'])
        self.assertFalse(outcome['device_authentication_allowed'])
        self.assertFalse(outcome['device_adopted'])
        self.assertIn('ssh_pinned',outcome['available_candidate_protocols'])
        self.assertNotIn('routeros_api_ssl',outcome['available_candidate_protocols'])
        with self.assertRaises(ValueError):P.evaluate(opts(candidate_protocol='routeros_api_ssl'))
    def test_routeros_direct_api_ssl_and_rest_https_candidates_only(self):
        r=P.evaluate(opts(device_profile='mikrotik_routeros7',candidate_protocol='routeros_api_ssl'))
        self.assertFalse(r['wireguard_required'])
        self.assertIn('routeros_rest_https',r['available_candidate_protocols'])
        self.assertIn('TLS_SERVER_CERTIFICATE_PINNING_EVIDENCE',r['missing_independent_proofs'])
    def test_even_all_self_declared_gates_not_operational_evidence(self):
        e=opts(management_segment_isolated=True,exact_protocol_on_firmware_verified=True,
          exact_device_identity_independently_verified=True,restricted_account_verified=True,
          actual_service_baseline_approved=True)
        r=P.evaluate(e)
        self.assertFalse(r['device_authentication_allowed'])
        self.assertFalse(r['actual_protocol_compatibility_proven'])
        self.assertFalse(r['device_adopted'])
        self.assertEqual(r['network_actions'],0)
    def test_optional_vpn_without_device_push(self):
        r=P.evaluate(opts(requested_tunnel='wireguard'))
        self.assertTrue(r['vpn_requested'])
        self.assertFalse(r['wireguard_required'])
        self.assertIn('SITE_GATEWAY_RECOVERY_PROOF',r['missing_independent_proofs'])
    def test_noauth_single_private_tls_probe_is_not_api_detection(self):
        called=[]
        class Raw:
            def __enter__(self):return self
            def __exit__(self,*_):return False
        class Tls(Raw):
            def version(self):return 'TLSv1.3'
        result=T.probe('10.10.13.233',connect=lambda a,timeout:(called.append((a,timeout)) or Raw()),
                       wrap=lambda raw,server_hostname:Tls())
        self.assertEqual(called,[(('10.10.13.233',443),3)])
        self.assertTrue(result['tls_identity_verified'])
        self.assertFalse(result['olt_api_supported'])
        self.assertFalse(result['device_adopted'])
        self.assertEqual(result['application_requests_sent'],0)
        self.assertFalse(result['credentials_sent'])
    def test_lab_ui_and_rust_endpoint_consistently_prioritize_existing_network(self):
        root=DIR.parents[3]
        html=(root/'web/lab/device-workbench.html').read_text()
        js=(root/'web/lab/device-workbench.js').read_text()
        rust=(root/'apps/control-api/src/device_workbench_lab.rs').read_text()
        for marker in ('Server Pusat IPAT', 'Gateway Lokasi',
                       'Jaringan Manajemen','id="direct-device-type"',
                       'id="direct-protocol-review"'):
            self.assertIn(marker,html)
        for marker in ('/lab/demo/direct-protocol-review',
                       'result.router_push_enabled!==false'):
            self.assertIn(marker,rust if marker=='/lab/demo/direct-protocol-review' else js)
        self.assertIn('wireguard_required!==false',js)
        self.assertIn('routeros_api_ssl',js)
        self.assertIn('snmpv3_authpriv',rust)
        self.assertNotIn('Site A',html)
        self.assertNotIn('Site B',html)

    def test_rejects_nonprivate_probes_and_unsupported_class(self):
        for address in ('0.0.0.0','127.0.0.1','8.8.8.8','169.254.0.1','224.0.0.1','240.0.0.1'):
            with self.assertRaises(ValueError):T.probe(address)
        with self.assertRaises(ValueError):P.evaluate(opts(device_profile='fictional_olt'))
        with self.assertRaises(ValueError):P.evaluate(opts(path_observed='yes'))

if __name__=='__main__':unittest.main()
