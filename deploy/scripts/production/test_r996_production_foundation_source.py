import json, os, tempfile, unittest
from datetime import datetime, timezone, timedelta
from pathlib import Path
import importlib.util

ROOT=Path(__file__).resolve().parents[3]
SCRIPT=ROOT/"deploy/scripts/production/r996_prepare_production_foundation.sh"
VAL=ROOT/"deploy/scripts/production/r996_validate_recovery_attestation.py"
spec=importlib.util.spec_from_file_location("r996att",VAL)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class R996Source(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.s=SCRIPT.read_text()

    def test_dependency_order_is_locked(self):
        marks=[
          "r985_prepare_platform_runtime.sh",
          "r992_prepare_tenant_runtime.sh",
          "r994_prepare_domain_ingress_verifier.sh",
          "r995_prepare_domain_ownership_verifier.sh",
          "r987_bootstrap_postgres18.sh",
        ]
        positions=[self.s.index(x) for x in marks]
        self.assertEqual(positions,sorted(positions))

    def test_no_public_ingress_firewall_or_device_action(self):
        for forbidden in ("r984_apply_ip_tls_ingress.sh","r993_apply_customer_ingress.sh",
                          "iptables","nft ","firewall-cmd","routeros","telnet ","snmp-set"):
            self.assertNotIn(forbidden,self.s)
        self.assertIn('"application_services":"DISABLED"',self.s)
        self.assertIn('"public_go":False',self.s)

    def test_first_mutation_follows_global_preflight(self):
        first=self.s.index("IPAT_R985_PREPARE=PREPARE_REVIEWED_PLATFORM_RUNTIME")
        for required in ("r996_validate_recovery_attestation.py","tracked working tree must be clean",
                         "existing IPAT runtime identity","managed/public/runtime port already has a listener"):
            self.assertLess(self.s.index(required),first)

    def test_rollback_drops_new_cluster_and_runtime_identities(self):
        self.assertIn("rollback_armed=YES",self.s)
        self.assertIn("declare -F rollback",self.s)
        self.assertIn("pg_dropcluster --stop 18 ipat",self.s)
        for u in ("ipatpgmigrate","ipatdnsverify","ipatdverify","ipattoidc","ipattapi","ipatpoidc","ipatpapi"):
            self.assertIn('userdel "$u"',self.s)
        self.assertIn("R996_ROLLBACK_SAFE_STATE_APPLICATIONS_DISABLED_NO_PUBLIC_EDGE",self.s)

    def test_attestation_is_recent_and_exact_source(self):
        now=datetime(2026,10,6,3,0,tzinfo=timezone.utc)
        src="a"*40
        data={
          "schema":"ipat.recovery-readiness.v1","source_commit":src,
          "provider_rescue_root_console_verified":True,
          "independent_full_host_restore_verified":True,
          "independent_backup_destination_verified":True,
          "reviewed_at_utc":(now-timedelta(hours=2)).isoformat(),
          "reviewer_id":"security-reviewer-1","evidence_id":"restore-run-20261006",
        }
        out=m.validate_document(data,src,now)
        self.assertFalse(out["public_go"])
        self.assertFalse(out["production_postgresql_pitr_verified"])
        bad=dict(data,source_commit="b"*40)
        with self.assertRaises(ValueError):m.validate_document(bad,src,now)
        stale=dict(data,reviewed_at_utc=(now-timedelta(days=15)).isoformat())
        with self.assertRaises(ValueError):m.validate_document(stale,src,now)

    def test_attestation_file_mode_and_owner_are_enforced(self):
        src="a"*40
        now=datetime.now(timezone.utc)
        data={"schema":"ipat.recovery-readiness.v1","source_commit":src,
          "provider_rescue_root_console_verified":True,
          "independent_full_host_restore_verified":True,
          "independent_backup_destination_verified":True,
          "reviewed_at_utc":now.isoformat(),"reviewer_id":"reviewer-1","evidence_id":"evidence-1"}
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/"att.json";p.write_text(json.dumps(data));os.chmod(p,0o600)
            out=m.validate_file(p,src,os.geteuid());self.assertFalse(out["public_go"])
            os.chmod(p,0o644)
            with self.assertRaises(ValueError):m.validate_file(p,src,os.geteuid())

if __name__=="__main__":unittest.main()
