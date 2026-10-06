import json, os, tempfile, unittest, importlib.util
from datetime import datetime, timezone, timedelta
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
SCRIPT=ROOT/"deploy/scripts/production/r997_activate_platform_edge.sh"
VAL=ROOT/"deploy/scripts/production/r997_validate_pitr_acceptance.py"
spec=importlib.util.spec_from_file_location("r997",VAL)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class R997Source(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.s=SCRIPT.read_text()

    def test_activation_order_and_scope(self):
        marks=["r985_apply_platform_runtime.sh","r992_apply_tenant_api.sh",
               "r995_enable_domain_ownership_verifier.sh","r984_prepare_ip_tls_prereqs.sh",
               "IPAT_R984_PHASE=staging","IPAT_R984_PHASE=production"]
        pos=[self.s.index(x) for x in marks]
        self.assertEqual(pos,sorted(pos))
        for forbidden in ("r992_add_tenant_oidc_instance.sh","r993_apply_customer_ingress.sh",
                          "r994_enable_domain_ingress_verifier.sh","iptables","nft ","firewall-cmd",
                          "telnet ","snmp-set","routeros"):
            self.assertNotIn(forbidden,self.s)

    def test_pitr_gate_precedes_activation(self):
        gate=self.s.index("r997_validate_pitr_acceptance.py")
        first=self.s.index("r985_apply_platform_runtime.sh")
        self.assertLess(gate,first)
        self.assertIn("R997_PLATFORM_EDGE_ACTIVE_PENDING_REAL_HUMAN_MFA_BROWSER_ACCEPTANCE",self.s)
        self.assertIn('"public_go":False',self.s)

    def test_global_rollback_disables_runtime_but_not_foundation(self):
        self.assertIn('edge_rollback=',self.s)
        self.assertIn('"$edge_rollback"',self.s)
        self.assertIn('before_edge_rollbacks=',self.s)
        self.assertIn('cannot identify exactly one new R9.84 rollback',self.s)
        for unit in ("ipat-platform-api.service","ipat-platform-oidc.service",
                     "ipat-tenant-api.service","ipat-domain-ownership-verifier.service"):
            self.assertIn(unit,self.s)
        self.assertNotIn("pg_dropcluster",self.s)
        self.assertNotIn("userdel",self.s)
        self.assertIn("PUBLIC_EDGE_SUBSCRIPT_HANDLES_OWN_TLS_ROLLBACK",self.s)

    def good(self,now):
        foundation={"schema":"ipat.production-foundation.v1","source_commit":"a"*40,
                    "application_services":"DISABLED","public_go":False}
        pitr={"schema":"ipat.production-pitr-acceptance.v1","foundation_source_commit":"a"*40,
          "offsite_repository_verified":True,"distinct_host_restore_verified":True,
          "pg_verifybackup_passed":True,"named_pitr_passed":True,"tenant_rls_validation_passed":True,
          "streaming_standby_failover_verified":True,"fencing_verified":True,
          "production_cluster_id_sha256":"b"*64,"prepared_by":"sre-operator","approved_by":"security-reviewer",
          "evidence_id":"pitr-20261006","rpo_seconds":30,"approved_rpo_seconds":60,
          "rto_seconds":120,"approved_rto_seconds":300,"reviewed_at_utc":now.isoformat()}
        return foundation,pitr

    def test_pitr_acceptance_requires_maker_checker_and_targets(self):
        now=datetime(2026,10,6,3,0,tzinfo=timezone.utc)
        f,p=self.good(now)
        out=m.validate_documents(f,p,now)
        self.assertFalse(out["public_go"])
        self.assertTrue(out["human_mfa_browser_acceptance_required"])
        bad=dict(p,approved_by=p["prepared_by"])
        with self.assertRaises(ValueError):m.validate_documents(f,bad,now)
        bad=dict(p,rpo_seconds=61)
        with self.assertRaises(ValueError):m.validate_documents(f,bad,now)
        bad=dict(p,fencing_verified=False)
        with self.assertRaises(ValueError):m.validate_documents(f,bad,now)

    def test_pitr_acceptance_stale_or_wrong_foundation_denied(self):
        now=datetime(2026,10,6,3,0,tzinfo=timezone.utc)
        f,p=self.good(now)
        stale=dict(p,reviewed_at_utc=(now-timedelta(days=8)).isoformat())
        with self.assertRaises(ValueError):m.validate_documents(f,stale,now)
        wrong=dict(p,foundation_source_commit="c"*40)
        with self.assertRaises(ValueError):m.validate_documents(f,wrong,now)

    def test_secure_files(self):
        now=datetime.now(timezone.utc);f,p=self.good(now)
        with tempfile.TemporaryDirectory() as td:
            a=Path(td)/"foundation.json";b=Path(td)/"pitr.json"
            a.write_text(json.dumps(f));b.write_text(json.dumps(p))
            os.chmod(a,0o600);os.chmod(b,0o600)
            out=m.validate_files(a,b,os.geteuid());self.assertFalse(out["public_go"])
            os.chmod(b,0o644)
            with self.assertRaises(ValueError):m.validate_files(a,b,os.geteuid())

if __name__=="__main__":unittest.main()
