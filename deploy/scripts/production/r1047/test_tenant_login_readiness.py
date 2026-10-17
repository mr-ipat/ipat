import importlib.util
import pathlib
import unittest
from unittest.mock import patch

path = pathlib.Path(__file__).with_name("tenant_login_readiness.py")
spec = importlib.util.spec_from_file_location("readiness", path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class ReadinessTests(unittest.TestCase):
    def test_fail_closed_when_tools_missing(self):
        with patch.object(module, "run", return_value=(127, "")):
            v = module.report()
        self.assertFalse(v["postgres18_ipat_online"])
        self.assertFalse(v["customer_login_verified"])
        self.assertFalse(v["public_production_go"])

    def test_active_processes_are_not_login_evidence(self):
        with patch.object(module, "run", side_effect=[
            (0, "active\nactive\nactive"),
            (0, "18 ipat 5432 online postgres /data /log"),
            (0, "LISTEN 0 128 127.0.0.1:3003 0.0.0.0:*\nLISTEN 0 128 0.0.0.0:443 0.0.0.0:*"),
        ]):
            v = module.report()
        self.assertTrue(v["tenant_api_active"])
        self.assertTrue(v["tenant_api_port_3003_listening"])
        self.assertFalse(v["customer_login_verified"])
        self.assertFalse(v["public_production_go"])

if __name__ == "__main__":
    unittest.main()
