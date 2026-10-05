from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
PREP=(ROOT/"scripts/production/r985_prepare_platform_runtime.sh").read_text()
APPLY=(ROOT/"scripts/production/r985_apply_platform_runtime.sh").read_text()
API=(ROOT/"systemd/ipat-platform-api.service").read_text()
OIDC=(ROOT/"systemd/ipat-platform-oidc.service").read_text()
API_ENV=(ROOT/"systemd/platform-api.env.example").read_text()
OIDC_ENV=(ROOT/"systemd/platform-oidc.env.example").read_text()

class R985RuntimeSource(unittest.TestCase):
    def test_units_are_distinct_nonroot_and_api_network_is_loopback_only(self):
        self.assertIn("User=ipatpapi",API)
        self.assertIn("User=ipatpoidc",OIDC)
        self.assertIn("IPAddressDeny=any",API)
        self.assertIn("IPAddressAllow=localhost",API)
        self.assertNotIn("IPAddressDeny=any",OIDC)
        for source in (API,OIDC):
            self.assertIn("NoNewPrivileges=true",source)
            self.assertIn("ProtectSystem=strict",source)
            self.assertIn("CapabilityBoundingSet=",source)
            self.assertIn("ExecStart=/opt/ipat/bin/control-api",source)

    def test_prepare_is_first_install_checksum_pinned_and_does_not_start_runtime(self):
        self.assertIn("IPAT_R985_BINARY_SHA256",PREP)
        self.assertIn("binary SHA256 mismatch",PREP)
        self.assertIn("existing runtime binary requires explicit upgrade review",PREP)
        conflict=PREP.index("existing runtime binary requires explicit upgrade review")
        install_binary=PREP.index('install -o root -g root -m 0755 "$src" /opt/ipat/bin/control-api')
        self.assertLess(conflict,install_binary)
        self.assertIn("systemctl disable --now ipat-platform-api.service ipat-platform-oidc.service",PREP)
        self.assertNotIn("enable --now ipat-platform-api",PREP)
        for forbidden in ("iptables ","nft ","ufw ","firewall-cmd","10.10.13.233"):
            self.assertNotIn(forbidden,PREP.lower())

    def test_env_parser_failure_is_checked_before_mapfile(self):
        self.assertIn('api_parsed=$(parse_env "$api_src" api) || die "invalid API env"',APPLY)
        self.assertIn('oidc_parsed=$(parse_env "$oidc_src" oidc) || die "invalid OIDC env"',APPLY)
        self.assertNotIn('mapfile -t api_lines < <(parse_env',APPLY)

    def test_apply_requires_exact_separate_database_roles_and_secure_oidc_files(self):
        self.assertIn("ipat_platform_session_api_login",APPLY)
        self.assertIn("ipat_platform_session_issuer_login",APPLY)
        self.assertIn("psql -X -w",APPLY)
        self.assertIn("ipatpoidc:600:1",APPLY)
        self.assertIn("ipatpoidc:700",APPLY)
        self.assertIn("/var/lib/ipat-platform-oidc/secrets/",APPLY)
        self.assertNotIn("PGPASSWORD=",APPLY)
        self.assertNotIn("ALTER ROLE",APPLY.upper())

    def test_apply_exact_host_loopback_and_timed_rollback(self):
        self.assertIn("IPAT_R985_PUBLIC_IPV4",APPLY)
        self.assertIn("IPAT_R981_EXACT_PLATFORM_HOST",APPLY)
        self.assertIn("IPAT_R983_HOST",APPLY)
        self.assertIn("127[.]0[.]0[.]1:3005",APPLY)
        self.assertIn("127[.]0[.]0[.]1:3006",APPLY)
        self.assertIn("wildcard listener forbidden",APPLY)
        armed=APPLY.index("systemd-run --quiet")
        install=APPLY.index('install -o root -g root -m 0600 "$api_src"')
        self.assertLess(armed,install)
        self.assertIn("disable --now ipat-platform-api.service ipat-platform-oidc.service",APPLY)
        self.assertIn("rm -f /etc/ipat/platform-api.env /etc/ipat/platform-oidc.env",APPLY)

    def test_no_production_secret_is_in_examples_or_scripts(self):
        self.assertIn("203.0.113.10",API_ENV)
        self.assertIn("replace-with-reviewed-key-id",OIDC_ENV)
        self.assertNotIn("client_secret=",OIDC_ENV.lower())
        combined=(PREP+APPLY+API_ENV+OIDC_ENV).lower()
        for forbidden in ("f52qgg","password:","bearer eyj"):
            self.assertNotIn(forbidden,combined)

    def test_no_firewall_dns_device_or_private_dashboard_mutation(self):
        combined=(PREP+APPLY+API+OIDC).lower()
        for forbidden in ("iptables ","nft ","ufw ","firewall-cmd","127.0.0.1:3002","10.10.13.233"):
            self.assertNotIn(forbidden,combined)

if __name__=="__main__":
    unittest.main()
