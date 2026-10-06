from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
APPLY=(ROOT/"scripts/production/r993_apply_customer_ingress.sh").read_text()
BOOT=(ROOT/"nginx/r993-tenant-bootstrap.conf.template").read_text()
TLS=(ROOT/"nginx/r993-tenant-https.conf.template").read_text()

class R993CustomerIngress(unittest.TestCase):
    def test_http_bootstrap_never_serves_login_before_tls(self):
        self.assertIn("location / { return 503; }", BOOT)
        self.assertIn('if ($host != "__HOST__") { return 421; }', BOOT)
        self.assertNotIn("proxy_pass", BOOT)

    def test_tls_routes_exact_host_to_separate_tenant_services(self):
        self.assertIn("proxy_pass http://127.0.0.1:3003", TLS)
        self.assertIn("proxy_pass http://127.0.0.1:__OIDC_PORT__", TLS)
        self.assertIn("location ^~ /auth/oidc/", TLS)
        self.assertIn("proxy_set_header Host __HOST__", TLS)
        for forbidden in ("127.0.0.1:3002","127.0.0.1:3005","127.0.0.1:3006"):
            self.assertNotIn(forbidden, TLS)

    def test_apply_requires_loopback_backends_and_exact_public_a(self):
        self.assertIn("tenant API not exact loopback", APPLY)
        self.assertIn("tenant OIDC not exact loopback", APPLY)
        self.assertIn("wildcard tenant backend forbidden", APPLY)
        self.assertIn("exact persisted ownership token required", APPLY)
        self.assertIn('f"_ipat-verify.{host}"', APPLY)
        self.assertIn("public ownership TXT does not match persisted IPAT token", APPLY)
        self.assertIn("dig @1.1.1.1 +short A", APPLY)
        self.assertIn('[[ ${#resolved[@]} -eq 1 && ${resolved[0]} == "$ip" ]]', APPLY)

    def test_acme_staging_exact_san_and_timed_rollback(self):
        for token in ("--staging","-checkhost","systemd-run --quiet","rollback_now",
                      "--dry-run --run-deploy-hooks"):
            self.assertIn(token, APPLY)
        self.assertIn("R993_STAGING_CUSTOMER_ACME_VALIDATED_AND_ROLLED_BACK", APPLY)

    def test_no_firewall_database_or_device_mutation(self):
        combined=(APPLY+"\n"+BOOT+"\n"+TLS).lower()
        for forbidden in ("iptables ","nft ","ufw ","firewall-cmd","psql ","10.10.13.233",
                          "record_tenant_domain_check"):
            self.assertNotIn(forbidden, combined)

    def test_production_verifies_api_and_exact_oidc_start(self):
        self.assertIn('--resolve "$host:443:$ip"', APPLY)
        self.assertIn('"https://$host/"', APPLY)
        self.assertIn('"https://$host/auth/oidc/start"', APPLY)
        self.assertIn('[[ $oidc_status == 303 ]]', APPLY)

if __name__=="__main__":
    unittest.main()
