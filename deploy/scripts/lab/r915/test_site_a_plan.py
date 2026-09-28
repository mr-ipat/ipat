"""Lab-only no-device central Site A connection decision tests."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('site_a_plan',
    Path(__file__).with_name('site_a_plan.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
P = module

BASE = {'mode':'wireguard', 'site_a_endpoint':'198.51.100.9',
    'site_b_gateway':'routeros7','management_host':'10.10.13.233',
    'vpn_subnet':'10.253.77.0/30', 'site_a_networks':['10.99.0.0/24'],
    'site_b_networks':['10.10.13.0/24'],
    'private_path_verified':False, 'site_b_recovery_verified':False}

class HubSpokeTests(unittest.TestCase):
    def test_public_hub_is_spoke_self_configuration_only(self):
        result=P.review(BASE)
        self.assertEqual(result['site_a_endpoint_kind'],'PUBLIC')
        self.assertEqual(result['site_a_role'],'CENTRAL_HUB_LISTENER')
        self.assertEqual(result['site_b_role'],'SELF_CONFIGURED_SPOKE')
        self.assertFalse(result['router_push_enabled'])
        self.assertFalse(result['config_generated'])
        self.assertFalse(result['device_adopted'])
        self.assertEqual(result['network_actions'],0)
        self.assertEqual(result['target_host_route'],'10.10.13.233/32')
    def test_real_verified_same_private_network_no_tunnel_required(self):
        data=dict(BASE,mode='direct_private',site_a_endpoint='10.99.0.10',
                  private_path_verified=True)
        result=P.review(data)
        self.assertEqual(result['site_a_endpoint_kind'],'PRIVATE')
        self.assertEqual(result['link_mode'],'direct_private')
    def test_unverified_private_hub_cannot_be_external_endpoint(self):
        with self.assertRaises(ValueError):
            P.review(dict(BASE,site_a_endpoint='10.99.0.10'))
        with self.assertRaises(ValueError):
            P.review(dict(BASE,mode='direct_private'))
    def test_no_secrets_or_network_overlap_or_false_gateway(self):
        for data in (dict(BASE,password='forbidden'),
                     dict(BASE,vpn_subnet='10.10.13.0/30'),
                     dict(BASE,site_b_gateway='routeros6'),
                     dict(BASE,management_host='8.8.8.8'),
                     dict(BASE,private_path_verified='yes'),
                     dict(BASE,site_b_networks=['10.99.0.0/24','10.10.13.0/24'])):
            with self.subTest(data=data):
                with self.assertRaises((ValueError,TypeError)):
                    P.review(data)
    def test_ipsec_not_claimed_implemented(self):
        result=P.review(dict(BASE,mode='ipsec'))
        self.assertEqual(result['state'],'REVIEW_ONLY')
        self.assertFalse(result['router_push_enabled'])

class SiteADashboardContracts(unittest.TestCase):
    def test_hub_dashboard_and_backend_are_paired_and_no_push(self):
        root=Path(__file__).resolve().parents[4]
        html=(root/'web/lab/device-workbench.html').read_text()
        js=(root/'web/lab/device-workbench.js').read_text()
        rust=(root/'apps/control-api/src/device_workbench_lab.rs').read_text()
        for marker in ('site-a-hub-panel','hub-method','hub-address-scope',
                       'site-b-path','site-b-gateway','check-site-a-plan'):
            self.assertIn(marker,html)
        for marker in ('/lab/demo/site-a-plan','router_push_enabled!==false',
                       'site_path_independently_verified!==false',
                       'IPAT_CENTRAL_CONFIGURATION_AUTHORITY',
                       'SITE_OPERATOR_SELF_CONFIGURES_NO_PUSH'):
            self.assertIn(marker,js)
        self.assertIn('async fn preview_site_a_plan(',rust)
        self.assertIn('r915_hub_is_site_a_and_never_pushes_to_site_b',rust)
        self.assertIn('\"router_push_enabled\":false',rust)
        self.assertIn('\"network_actions\":0',rust)

if __name__=='__main__': unittest.main()
