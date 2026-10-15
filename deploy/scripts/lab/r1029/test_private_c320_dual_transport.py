"""Synthetic negative contracts; real VPS acceptance is a separate test."""
from datetime import datetime, timezone, timedelta
import json
import unittest

from verify_private_c320_dual_transport import verify, live_fetch


NOW = datetime(2026, 10, 15, 3, 0, tzinfo=timezone.utc)


class DualTransportContract(unittest.TestCase):
    def setUp(self):
        self.probe = {
            "target": "DEV-01",
            "ssh": {"port": 321, "tcp_reachable": True,
                    "probe_stage": "TCP_REACHABLE_AUTH_NOT_TESTED",
                    "server_banner_observed": True},
            "telnet": {"port": 323, "tcp_reachable": True,
                       "probe_stage": "TCP_REACHABLE_AUTH_NOT_TESTED",
                       "negotiation_observed": True},
            "ssh_authentication_verified": False,
            "telnet_authentication_verified": False,
            "device_identity_verified_by_network_probe": False,
            "production_adopted": False,
            "physical_writes_enabled": False,
        }
        self.connection = {
            "target": "DEV-01", "device_status": "CONNECTED",
            "connector_online": True, "credentials_enrolled": True,
            "production_adopted": False, "physical_writes_enabled": False,
            "last_verified_at_utc": (NOW - timedelta(seconds=5)).isoformat(),
        }

    def fetch(self, path):
        if path == "/lab/device-workbench":
            return b'<button id="ipat-c320-check-network">Check</button>'
        return json.dumps(
            self.probe if path.endswith("network-probe") else self.connection
        ).encode()

    def test_all_required_evidence_passes(self):
        result = verify(self.fetch, NOW)
        self.assertEqual(len(result), 6)
        self.assertIn("R1029_PRODUCTION_ADOPTED=FALSE", result)

    def test_rejects_missing_telnet_proof(self):
        self.probe["telnet"]["negotiation_observed"] = False
        with self.assertRaisesRegex(ValueError, "telnet physical transport proof missing"):
            verify(self.fetch, NOW)

    def test_rejects_unsafe_claim_of_device_adoption(self):
        self.probe["production_adopted"] = True
        with self.assertRaisesRegex(ValueError, "unsafe network probe elevation claim"):
            verify(self.fetch, NOW)

    def test_rejects_stale_authenticated_read(self):
        self.connection["last_verified_at_utc"] = (
            NOW - timedelta(minutes=8)
        ).isoformat()
        with self.assertRaisesRegex(ValueError, "stale"):
            verify(self.fetch, NOW)

    def test_rejects_wrong_device(self):
        self.probe["target"] = "DEV-02"
        with self.assertRaisesRegex(ValueError, "unexpected device"):
            verify(self.fetch, NOW)

    def test_rejects_arbitrary_url_target(self):
        with self.assertRaisesRegex(ValueError, "unapproved diagnostic path"):
            live_fetch("/admin/secret")


if __name__ == "__main__":
    unittest.main()
