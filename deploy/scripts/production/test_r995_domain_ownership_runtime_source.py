from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
PREP=(ROOT/"scripts/production/r995_prepare_domain_ownership_verifier.sh").read_text()
ENABLE=(ROOT/"scripts/production/r995_enable_domain_ownership_verifier.sh").read_text()
SERVICE=(ROOT.parent/"deploy/systemd/ipat-domain-ownership-verifier.service").read_text()
PG=(ROOT/"scripts/production/r987_bootstrap_postgres18.sh").read_text()
MAIN=(ROOT.parent/"apps/control-api/src/main.rs").read_text()
VERIFIER=(ROOT.parent/"apps/control-api/src/tenant_domain_verifier.rs").read_text()

class R995Source(unittest.TestCase):
    def test_dedicated_nonroot_ownership_identity_and_peer_role(self):
        self.assertIn("User=ipatdnsverify",SERVICE)
        self.assertIn("ipatdnsverify  ipat_domain_verifier_login",PG)
        self.assertIn("-U ipat_domain_verifier_login",ENABLE)
        self.assertIn("has_table_privilege",ENABLE)
        self.assertIn("record_tenant_domain_check(uuid,text,text,text)",ENABLE)
        self.assertIn("broad == 0",ENABLE)

    def test_runtime_config_is_service_owned_not_under_root_only_ipat_etc(self):
        self.assertIn("/var/lib/ipat-domain-ownership-verifier/postgres.conninfo",SERVICE)
        self.assertIn("install -d -o ipatdnsverify -g ipatdnsverify -m 0700",ENABLE)
        self.assertIn("chmod 0600",ENABLE)
        self.assertNotIn("/etc/ipat/domain-ownership-verifier",SERVICE+ENABLE)

    def test_prepare_and_enable_are_explicit_and_rollback(self):
        self.assertIn("PREPARE_REVIEWED_DOMAIN_OWNERSHIP_VERIFIER",PREP)
        self.assertIn("PREPARED_DISABLED",PREP)
        self.assertIn("ENABLE_REVIEWED_DOMAIN_OWNERSHIP_VERIFIER",ENABLE)
        self.assertIn("disable --now ipat-domain-ownership-verifier.service",ENABLE)
        self.assertIn("enable --now ipat-domain-ownership-verifier.service",ENABLE)
        self.assertIn("systemd-analyze verify",ENABLE)

    def test_service_is_background_only_and_hardened(self):
        self.assertIn("IPAT_TENANT_DOMAIN_VERIFIER=YES",SERVICE)
        self.assertIn("ExecStart=/opt/ipat/bin/control-api",SERVICE)
        for token in ("NoNewPrivileges=true","PrivateDevices=true","ProtectSystem=strict",
                      "ProtectHostname=true","MemoryDenyWriteExecute=true",
                      "RestrictNamespaces=true","CapabilityBoundingSet="):
            self.assertIn(token,SERVICE)
        self.assertNotIn("ListenStream",SERVICE)

    def test_verifier_mode_is_exclusive_from_http_lab_and_device_modes(self):
        verifier_gate=MAIN[MAIN.index('if std::env::var("IPAT_TENANT_DOMAIN_VERIFIER")'):
                           MAIN.index('// R9.83: dedicated Platform Owner')]
        for flag in ("IPAT_R969_COMMERCIAL_SERVICE","IPAT_R970_OIDC_ISSUER_SERVICE",
                     "IPAT_R981_PLATFORM_SERVICE","IPAT_R983_PLATFORM_OIDC_ISSUER_SERVICE",
                     "IPAT_LAB_WEB","IPAT_RUN_K3S_LAB","IPAT_R911_PRIVATE_CANARY",
                     "IPAT_R940_PRIVATE_OWNER_READ"):
            self.assertIn(flag,verifier_gate)

    def test_ownership_worker_calls_only_narrow_wrapper(self):
        self.assertIn("record_tenant_domain_ownership_check",VERIFIER)
        self.assertNotIn("SELECT ipat_platform.record_tenant_domain_check(",VERIFIER)

    def test_no_network_firewall_package_or_device_mutation(self):
        lowered=(PREP+ENABLE).lower()
        for forbidden in ("iptables ","nft ","ufw ","firewall-cmd","apt-get ",
                          "10.10.13.233","telnet ","ssh ","scp ","alter table","create table"):
            self.assertNotIn(forbidden,lowered)

if __name__=="__main__":
    unittest.main()
