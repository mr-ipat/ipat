"""Static boundary assertions only; no USP protobuf or MTP tests here."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CORE = (ROOT / "crates/usp-core/src/lib.rs").read_text()
APP = (ROOT / "apps/usp-controller/src/main.rs").read_text()


class SyntheticUspBoundaryTests(unittest.TestCase):
    def test_transport_and_operator_proofs_have_no_public_constructors(self):
        for name in ("VerifiedAgent", "TrustedOperator"):
            self.assertIn("pub struct " + name, CORE)
            self.assertNotIn("impl " + name, CORE)
        self.assertIn("#[cfg(test)]", CORE)
        self.assertIn("fn peer(byte: u8) -> VerifiedAgent", CORE)
        self.assertIn("fn trusted_operator(label: &str) -> TrustedOperator", CORE)

    def test_synthetic_only_and_no_unauthorized_mutating_rpc(self):
        for label in ("NOT parse, serialize or implement TR-369 USP protobuf",
                      "pub fn plan_read(", "pub fn accept_reply(", "ReplayFull",
                      "WrongPeer", "WrongTenant", "MAX_REPLY"):
            self.assertIn(label, CORE)
        for forbidden in ("pub fn set_parameter", "pub fn reboot", "pub fn apply_config",
                          "TcpListener", "mqtt", "axum"):
            self.assertNotIn(forbidden, CORE)

    def test_loopback_explicit_optin_and_denied_untrusted_route(self):
        self.assertIn('IPAT_RUN_OFFLINE_USP_LAB', APP)
        self.assertIn('127.0.0.1:3100', APP)
        self.assertIn('StatusCode::SERVICE_UNAVAILABLE', APP)
        self.assertNotIn('"0.0.0.0:', APP)
        self.assertEqual(APP.count("#[tokio::test]"), 3)
        self.assertEqual(CORE.count("#[test]"), 12)


if __name__ == "__main__":
    unittest.main()
