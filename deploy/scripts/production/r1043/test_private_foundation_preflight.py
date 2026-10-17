import importlib.util
import pathlib
import unittest
from unittest.mock import patch

FILE = pathlib.Path(__file__).with_name("private_foundation_preflight.py")
SPEC = importlib.util.spec_from_file_location("preflight", FILE)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class PreflightTest(unittest.TestCase):
    def test_missing_postgres_reports_false(self):
        with patch.object(MODULE, "command_output", side_effect=[
            (127, ""), (0, "")]), patch.object(
                MODULE.urllib.request, "urlopen", side_effect=OSError()):
            result = MODULE.probe()
        self.assertFalse(result["postgres18_ipat_online"])
        self.assertFalse(result["owner_private_health_http200"])
        self.assertFalse(result["public_go"])

    def test_happy_private_foundation_never_claims_public_go(self):
        cluster = "18 ipat 5432 online postgres /var/lib/postgresql/18/ipat logfile"
        class Response:
            status = 200
            def __enter__(self): return self
            def __exit__(self, *args): return False
        with patch.object(MODULE, "command_output", side_effect=[
            (0, cluster), (0, "LISTEN 0 128 127.0.0.1:3002 0.0.0.0:*")]), patch.object(
                MODULE.urllib.request, "urlopen", return_value=Response()):
            result = MODULE.probe()
        self.assertTrue(result["postgres18_ipat_online"])
        self.assertTrue(result["owner_private_health_http200"])
        self.assertFalse(result["public_go"])

    def test_public_port_is_rejected(self):
        cluster = "18 ipat 5432 online postgres /var/lib/postgresql/18/ipat logfile"
        with patch.object(MODULE, "command_output", side_effect=[
            (0, cluster), (0, "LISTEN 0 128 0.0.0.0:443 0.0.0.0:*")]), patch.object(
                MODULE.urllib.request, "urlopen", side_effect=OSError()):
            result = MODULE.probe()
        self.assertFalse(result["public_tcp443_absent"])


if __name__ == "__main__":
    unittest.main()
