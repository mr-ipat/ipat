"""Offline-only R9.5 dashboard contract; NOT a provisioning acceptance test."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[4]
HTML = (ROOT/'web/lab/device-workbench.html').read_text()
JS = (ROOT/'web/lab/device-workbench.js').read_text()
RUST = (ROOT/'apps/control-api/src/device_workbench_lab.rs').read_text()

class ConnectionPreviewTests(unittest.TestCase):
    def test_multiple_methods_and_gateway_versions(self):
        for marker in ('id="connections"', 'id="connection-method"',
                       'id="connection-gateway"', 'id="connection-result"',
                       'value="direct_secure"', 'value="wireguard"',
                       'value="ipsec"', 'value="agent"',
                       'value="public_telnet"', 'value="routeros6"'):
            self.assertIn(marker, HTML)

    def test_unsafe_paths_fail_closed(self):
        self.assertIn('DITOLAK: Telnet melalui IP publik', JS)
        self.assertIn('method==="wireguard" && gateway==="routeros6"', JS)
        self.assertIn('tidak ada konfigurasi yang dikirim', JS)

    def test_no_secrets_or_network_mutation(self):
        self.assertNotIn('type="password"', HTML)
        self.assertNotIn('id="activate-tunnel"', HTML)
        for marker in ('localStorage', 'sessionStorage', 'innerHTML'):
            self.assertNotIn(marker, JS)
        self.assertIn('node("connection-result").textContent=', JS)
        self.assertIn('include_str!("../../../web/lab/device-workbench.html")', RUST)

if __name__ == '__main__':
    unittest.main()
