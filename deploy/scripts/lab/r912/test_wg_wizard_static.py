"""R9.12 LAB-only WireGuard wizard contract; not real tunnel proof."""
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[4]
HTML=(ROOT/'web/lab/device-workbench.html').read_text()
JS=(ROOT/'web/lab/device-workbench.js').read_text()
API=(ROOT/'apps/control-api/src/device_workbench_lab.rs').read_text()

class R912StaticContracts(unittest.TestCase):
    def test_dashboard_synthetic_review_inputs_no_credential_fields(self):
        for value in ('id="wg-gateway"', 'id="wg-segmentation"',
                      'id="wg-recovery"', 'id="wg-baseline"',
                      'id="check-wg-review"', 'id="wg-review-status"'):
            self.assertIn(value,HTML)
        self.assertIn('TANPA KONFIGURASI AKTIF',HTML)
        self.assertNotIn('type="password"',HTML)
        self.assertNotIn('id="wg-private-key"',HTML)
    def test_backend_is_no_credential_no_deploy(self):
        self.assertIn('async fn preview_tunnel_review(', API)
        self.assertIn('serde(deny_unknown_fields)',API)
        self.assertIn('/lab/demo/tunnel-review',API)
        self.assertIn('"network_actions":0',API)
        self.assertIn('"tunnel_created":false',API)
        self.assertIn('"secrets_accepted":false',API)
        self.assertIn('"BLOCKED_PENDING_REAL_REVIEW"',API)
        self.assertIn('"INDEPENDENT_APPROVAL_REQUIRED"',API)
        self.assertIn('"DEVICE_HOST_KEY_NOT_PINNED"',API)
        self.assertIn('synthetic_wireguard_wizard_never_accepts_secrets_or_activates',API)
    def test_browser_rejects_unsafe_server_success(self):
        self.assertIn('result.tunnel_created!==false',JS)
        self.assertIn('result.network_actions!==0',JS)
        self.assertIn('result.service_impact_measured!==false',JS)
        self.assertIn('result.secrets_accepted!==false',JS)
        self.assertNotIn('localStorage',JS)
        self.assertNotIn('innerHTML',JS)

if __name__=='__main__': unittest.main()
