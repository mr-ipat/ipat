import unittest
from unittest.mock import patch
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import r982_temporary_ip_tls_preflight as m

LIVE={
    "available":True,
    "PRIVATE_DOMAIN_IP":"202.162.204.121",
    "PORT443":"1",
}
class TemporaryIpv4Readiness(unittest.TestCase):
    def test_exact_ip_ca_san_positive_remains_no_go(self):
        out=m.report("202.162.204.121",remote=LIVE,tcp=True,tls=True)
        self.assertTrue(out["browser_ca_trusted_exact_ipv4_san_tls"])
        self.assertFalse(out["public_go"])
        self.assertEqual(out["production_status"],"BLOCKED")
    def test_no_port_or_bad_san_or_mismatched_owner_plan(self):
        self.assertFalse(m.report("202.162.204.121",remote=LIVE,tcp=True,tls=False)
                         ["browser_ca_trusted_exact_ipv4_san_tls"])
        self.assertFalse(m.report("202.162.204.122",remote=LIVE,tcp=True,tls=True)
                         ["target_matches_existing_private_owner_plan"])
        self.assertFalse(m.report("202.162.204.121",remote=LIVE,tcp=False,tls=True)
                         ["browser_ca_trusted_exact_ipv4_san_tls"])
    def test_disallow_unsafe_ip_and_never_probe(self):
        with patch.object(m.r963,"remote") as remote:
            for ip in ["127.0.0.1","10.10.13.233","203.0.113.22","198.51.100.7",
                       "192.168.1.1","100.64.0.1","192.0.2.3","202.162.204.121:443"]:
                with self.assertRaises(ValueError):
                    m.report(ip)
            remote.assert_not_called()
    def test_malformed_listener_count_never_means_active(self):
        for count in ("", "0", "unknown", "1;malicious", "-1", "9" * 5000, None):
            live = {**LIVE, "PORT443": count}
            out = m.report("202.162.204.121", remote=live, tcp=False, tls=False)
            self.assertFalse(out["vps_443_listener_observed"], count)

    def test_ssl_uses_ca_and_exact_ip_match(self):
        fake_socket=object()
        class TLS:
            def __enter__(self):return self
            def __exit__(self,*args):return False
            def getpeercert(self):return {"subjectAltName":[("IP Address","202.162.204.121")]}
        class Socket:
            def __enter__(self):return fake_socket
            def __exit__(self,*args):return False
        class Context:
            def __init__(self):self.check_hostname=None;self.verify_mode=None
            def wrap_socket(self,raw,server_hostname):
                assert raw is fake_socket
                assert server_hostname=="202.162.204.121"
                assert self.check_hostname is True
                assert self.verify_mode==m.ssl.CERT_REQUIRED
                return TLS()
        with patch.object(m.ssl,"create_default_context",return_value=Context()), \
             patch.object(m.socket,"create_connection",return_value=Socket()):
            self.assertTrue(m.trusted_ip_san_tls("202.162.204.121"))
