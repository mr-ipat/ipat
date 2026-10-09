from pathlib import Path
import hashlib
import unittest

ROOT=Path(__file__).resolve().parents[2]
SCRIPT=(ROOT/"scripts/production/r987_bootstrap_postgres18.sh").read_text()
MANIFEST=(ROOT/"db/production/r987_migrations.sha256").read_text().splitlines()

class R987BootstrapSource(unittest.TestCase):
    def test_manifest_is_pinned_and_complete(self):
        self.assertEqual(len(MANIFEST),32)
        for line in MANIFEST:
            digest,path=line.split("  ")
            self.assertEqual(len(digest),64)
            actual=hashlib.sha256((ROOT.parent/path).read_bytes()).hexdigest()
            self.assertEqual(actual,digest,path)

    def test_first_install_and_local_socket_only(self):
        self.assertIn("existing PostgreSQL cluster requires separate migration review",SCRIPT)
        self.assertIn("listen_addresses = ''",SCRIPT)
        self.assertIn("unix_socket_directories = '/run/postgresql'",SCRIPT)
        self.assertIn("TCP PostgreSQL listener forbidden",SCRIPT)
        self.assertNotIn("0.0.0.0",SCRIPT)

    def test_runtime_roles_are_peer_mapped_and_passwordless(self):
        self.assertIn("ipatpapi       ipat_platform_session_api_login",SCRIPT)
        self.assertIn("ipatpoidc      ipat_platform_session_issuer_login",SCRIPT)
        self.assertIn("ipattapi       ipat_tenant_api_login",SCRIPT)
        self.assertIn("ipattoidc      ipat_oidc_session_issuer_login",SCRIPT)
        self.assertIn("ipatdverify    ipat_domain_ingress_verifier_login",SCRIPT)
        self.assertIn("ipatdnsverify  ipat_domain_verifier_login",SCRIPT)
        self.assertIn("local   ipat_prod  ipat_domain_verifier_login                  peer map=ipat_runtime",SCRIPT)
        self.assertIn("local   all        all                                 reject",SCRIPT)
        self.assertNotIn("PGPASSWORD=",SCRIPT)

    def test_temporary_migrator_superuser_mapping_is_removed(self):
        self.assertIn("ipat_bootstrap  ipatpgmigrate  postgres",SCRIPT)
        self.assertIn("Replace bootstrap superuser mapping with final runtime-only auth",SCRIPT)
        self.assertIn("userdel ipatpgmigrate",SCRIPT)
        self.assertIn("temporary migration OS identity removal failed",SCRIPT)
        self.assertIn("Replace bootstrap superuser mapping with final runtime-only auth",SCRIPT)
        self.assertIn("local   all        postgres                            peer\nlocal   ipat_prod",SCRIPT)
        self.assertIn("ipat_bootstrap  postgres       postgres",SCRIPT)

    def test_package_install_suppresses_auto_default_cluster(self):
        self.assertIn("create_main_cluster = false",SCRIPT)
        self.assertIn("package install unexpectedly created a cluster",SCRIPT)
        self.assertIn("pg_createcluster 18 ipat --start-conf=auto",SCRIPT)

    def test_no_network_or_device_mutations(self):
        lowered=SCRIPT.lower()
        for forbidden in ("iptables ","nft ","ufw ","firewall-cmd","10.10.13.233","ssh ","scp "):
            self.assertNotIn(forbidden,lowered)

if __name__=="__main__":
    unittest.main()
