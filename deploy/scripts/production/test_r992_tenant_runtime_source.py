from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
PREP=(ROOT/"scripts/production/r992_prepare_tenant_runtime.sh").read_text()
API=(ROOT/"scripts/production/r992_apply_tenant_api.sh").read_text()
OIDC=(ROOT/"scripts/production/r992_add_tenant_oidc_instance.sh").read_text()
PG=(ROOT/"scripts/production/r987_bootstrap_postgres18.sh").read_text()
MAIN=(ROOT.parent/"apps/control-api/src/main.rs").read_text()
API_UNIT=(ROOT.parent/"deploy/systemd/ipat-tenant-api.service").read_text()
OIDC_UNIT=(ROOT.parent/"deploy/systemd/ipat-tenant-oidc@.service").read_text()

class R992TenantRuntimeSource(unittest.TestCase):
    def test_distinct_nonroot_service_identities_and_units(self):
        self.assertIn("User=ipattapi",API_UNIT)
        self.assertIn("User=ipattoidc",OIDC_UNIT)
        for unit in (API_UNIT,OIDC_UNIT):
            self.assertIn("NoNewPrivileges=true",unit)
            self.assertIn("ProtectSystem=strict",unit)
            self.assertIn("CapabilityBoundingSet=",unit)
            self.assertIn("RestrictSUIDSGID=true",unit)
        self.assertIn("IPAddressDeny=any",API_UNIT)
        self.assertIn("IPAddressAllow=localhost",API_UNIT)

    def test_prepare_never_starts_or_mutates_db_network_device(self):
        self.assertIn("R992_TENANT_RUNTIME_PREPARED_SERVICES_DISABLED",PREP)
        lowered=PREP.lower()
        for forbidden in ("apt-get install","psql ","iptables ","nft ","ufw ","firewall-cmd","10.10.13.233","ssh ","scp "):
            self.assertNotIn(forbidden,lowered)

    def test_api_apply_requires_exact_role_and_loopback_only(self):
        self.assertIn("ipat_tenant_api_login",API)
        self.assertIn("127[.]0[.]0[.]1:3003",API)
        self.assertIn("wildcard listener forbidden",API)
        self.assertIn("systemd-run",API)
        self.assertIn("existing tenant API env requires upgrade review",API)

    def test_oidc_instances_are_exact_host_durable_and_bounded_ports(self):
        for expected in ("IPAT_R971_DURABLE_PENDING","IPAT_R970_VERIFIED_CONFIDENTIAL_IDP",
                         "IPAT_R992_TENANT_OIDC_PORT","31000 <= port <= 31999",
                         "ipat_oidc_session_issuer_login","127[.]0[.]0[.]1:$port"):
            self.assertIn(expected,OIDC)
        self.assertIn("IPAT_R970_SINGLE_INSTANCE_OIDC",OIDC)
        self.assertIn("instance already exists",OIDC)

    def test_postgres_bootstrap_maps_all_four_runtime_roles(self):
        for expected in ("ipattapi       ipat_tenant_api_login",
                         "ipattoidc      ipat_oidc_session_issuer_login",
                         "local   ipat_prod  ipat_tenant_api_login",
                         "local   ipat_prod  ipat_oidc_session_issuer_login"):
            self.assertIn(expected,PG)
        self.assertIn("restricted tenant API peer auth failed",PG)
        self.assertIn("restricted tenant OIDC peer auth failed",PG)

    def test_rust_oidc_listener_is_loopback_and_deployment_port_bounded(self):
        self.assertIn('IPAT_R992_TENANT_OIDC_PORT',MAIN)
        self.assertIn('(3004..=31999).contains(&oidc_port)',MAIN)
        self.assertIn('TcpListener::bind(("127.0.0.1", oidc_port))',MAIN)

    def test_no_secret_examples(self):
        examples=[
          (ROOT.parent/"deploy/systemd/tenant-api.env.example").read_text(),
          (ROOT.parent/"deploy/systemd/tenant-oidc.env.example").read_text(),
        ]
        self.assertNotIn("password=",("\n".join(examples)).lower())
        self.assertIn("replace-with-reviewed-client",examples[1])

if __name__=="__main__":
    unittest.main()
