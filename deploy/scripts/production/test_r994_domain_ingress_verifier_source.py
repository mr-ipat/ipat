from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
VERIFY=(ROOT/"scripts/production/r994_verify_customer_ingress.sh").read_text()
PREP=(ROOT/"scripts/production/r994_prepare_domain_ingress_verifier.sh").read_text()
ENABLE=(ROOT/"scripts/production/r994_enable_domain_ingress_verifier.sh").read_text()
SERVICE=(ROOT.parent/"deploy/systemd/ipat-domain-ingress-verifier.service").read_text()
TIMER=(ROOT.parent/"deploy/systemd/ipat-domain-ingress-verifier.timer").read_text()
MIG=(ROOT.parent/"deploy/db/migrations/0033_tenant_domain_ingress_verifier.sql").read_text()
PG=(ROOT/"scripts/production/r987_bootstrap_postgres18.sh").read_text()
SYSTEMD_TEST=(ROOT/"scripts/production/r994_systemd_ubuntu26_verify.sh").read_text()

class R994Source(unittest.TestCase):
    def test_dedicated_nonroot_role_and_no_raw_table_grant(self):
        self.assertIn("dedicated ipatdverify OS identity required", VERIFY)
        self.assertIn("ipat_domain_ingress_verifier_login", VERIFY)
        self.assertIn("User=ipatdverify", SERVICE)
        self.assertIn("ipatdverify    ipat_domain_ingress_verifier_login", PG)
        self.assertNotIn("GRANT SELECT ON ipat_platform.tenant_domains", MIG)

    def test_ordered_checks_require_public_a_and_trusted_tls(self):
        self.assertIn("dig @1.1.1.1 +short A", VERIFY)
        self.assertIn("--resolve", VERIFY)
        self.assertIn("-verify_hostname", VERIFY)
        self.assertIn("-verify_return_error", VERIFY)
        self.assertIn("routing_ready", VERIFY)
        self.assertIn("tls_ready", VERIFY)
        self.assertIn("activate", VERIFY)
        self.assertIn("IPAT_R994_ALLOW_ACTIVATE", VERIFY)

    def test_no_privileged_network_database_schema_or_device_mutation(self):
        lowered=VERIFY.lower()
        for forbidden in ("iptables ","nft ","ufw ","firewall-cmd","apt-get ","create table",
                          "alter table","10.10.13.233","ssh ","scp "):
            self.assertNotIn(forbidden, lowered)

    def test_service_is_hardened_timer_and_prepare_leaves_it_disabled(self):
        for token in ("NoNewPrivileges=true","ProtectSystem=strict","PrivateDevices=true",
                      "CapabilityBoundingSet=","RestrictSUIDSGID=true","ProtectHostname=true",
                      "MemoryDenyWriteExecute=true","RestrictNamespaces=true"):
            self.assertIn(token, SERVICE)
        self.assertIn("OnUnitActiveSec=2min", TIMER)
        self.assertIn("RandomizedDelaySec=20s", TIMER)
        self.assertIn("PREPARED_DISABLED", PREP)
        self.assertIn("disable --now ipat-domain-ingress-verifier.timer", PREP)

    def test_migration_exposes_only_active_tenant_ordered_candidates(self):
        for token in ("t.state='active'","verification_state='verified'",
                      "ownership_verified_at IS NOT NULL",
                      "activation_state IN ('ownership_verified','routing_ready','tls_ready')",
                      "TO ipat_domain_ingress_verifier"):
            self.assertIn(token, MIG)
        self.assertIn("REVOKE ALL ON FUNCTION", MIG)

    def test_error_paths_do_not_advance_state(self):
        for code in ("DNS_TXT_OWNERSHIP_RECHECK_FAILED","DNS_A_TARGET_MISMATCH","TLS_HTTPS_VERIFY_FAILED",
                     "DNS_A_TARGET_CHANGED","TLS_HTTPS_RECHECK_FAILED"):
            self.assertIn(code, VERIFY)
        self.assertIn("'check_failed'", VERIFY)
        self.assertIn("record_tenant_domain_ingress_check", VERIFY)
        self.assertIn("record_tenant_domain_ownership_check", (ROOT.parent/"deploy/db/migrations/0033_tenant_domain_ingress_verifier.sql").read_text())

    def test_enable_is_peer_gated_atomic_and_rolls_back(self):
        self.assertIn("ENABLE_REVIEWED_DOMAIN_INGRESS_VERIFIER", ENABLE)
        self.assertIn("ipat_domain_ingress_verifier_login", ENABLE)
        self.assertIn("has_table_privilege", ENABLE)
        self.assertIn("systemd-analyze verify", ENABLE)
        self.assertIn("install -o root -g root -m 0600", ENABLE)
        self.assertIn("disable --now ipat-domain-ingress-verifier.timer", ENABLE)
        self.assertIn("enable --now ipat-domain-ingress-verifier.timer", ENABLE)
        self.assertIn("R994_UBUNTU26_SYSTEMD_VERIFY_PASS", SYSTEMD_TEST)
        for forbidden in ("iptables ","nft ","ufw ","apt-get ","10.10.13.233","ssh ","scp "):
            self.assertNotIn(forbidden, ENABLE.lower())

if __name__=="__main__":
    unittest.main()
