"""R9.16 safety contracts; actual X25519 crypto exercised separately on VPS."""
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[4]
SCRIPT=(ROOT/'deploy/scripts/lab/r916/site_a_keypair.py').read_text()

class OfflineSiteAKeyPolicy(unittest.TestCase):
    def test_keys_generated_only_on_site_a_nonroot_not_downloaded(self):
        for text in ('X25519PrivateKey.generate()', 'PrivateFormat.Raw',
                     'site_a_public_key', 'private_key_exported\':False',
                     'O_NOFOLLOW','O_EXCL','mode=0o700','0o600',
                     'IPAT_R916_APPROVE_SITE_A_DEV_KEY_GENERATION'):
            self.assertIn(text,SCRIPT)
        for forbidden in ('paramiko','requests.post','httpx','routeros_api',
                          'wg-quick up','iptables','subprocess','ssh '):
            self.assertNotIn(forbidden,SCRIPT)
    def test_private_lab_dashboard_has_actual_local_site_a_public_readback_and_disabled_b_review(self):
        rust=(ROOT/'apps/control-api/src/site_a_pairing_lab.rs').read_text()
        wrapper=(ROOT/'apps/control-api/src/device_workbench_lab.rs').read_text()
        html=(ROOT/'web/lab/device-workbench.html').read_text()
        js=(ROOT/'web/lab/device-workbench.js').read_text()
        for marker in ('/lab/dev-site-a-public-key','/lab/demo/site-a-manual-pairing'):
            self.assertIn(marker,wrapper)
            self.assertIn(marker,js)
        for marker in ('id="site-a-public-key"','id="manual-site-b-key"',
                       'id="manual-site-b-form"','id="manual-pairing-output"'):
            self.assertIn(marker,html)
        for marker in ('O_NOFOLLOW','public.key','disabled=yes',
                       'real_peer_activation_authorized','return_route_independently_verified',
                       'site_a_private_key_exported','router_push_enabled'):
            self.assertIn(marker,rust)
        for marker in ('output.textContent','result.router_push_enabled!==false',
                       'result.config_applied!==false','result.site_a_listener_active!==false'):
            self.assertIn(marker,js)
        self.assertNotIn('localStorage',js)
        self.assertNotIn('innerHTML',js)

    def test_never_claims_active_peer_production_backup(self):
        for literal in ("'tunnel_active':False", "'network_actions':0",
                        "'production_vault_verified':False", "'backup_verified':False"):
            self.assertIn(literal,SCRIPT)

if __name__=='__main__':unittest.main()
