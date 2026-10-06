import importlib.util, json, os, tempfile, unittest
from datetime import datetime, timezone, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
VAL = ROOT / "deploy/scripts/production/r999_validate_customer_browser_acceptance.py"
FINAL = (ROOT / "deploy/scripts/production/r999_finalize_customer_release.sh").read_text()
spec = importlib.util.spec_from_file_location("r999", VAL)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

D = "99999999-9999-4999-8999-999999999999"
T = "99999999-9999-4999-8999-999999999991"
C = "99999999-9999-4999-8999-999999999992"
H = "portal.customer.example.net"
CH = "portal.control.example.net"

class R999Source(unittest.TestCase):
    def good(self, now):
        activation = {
            "schema":"ipat.customer-domain-activation.v1",
            "domain_id":D,
            "hostname":H,
            "domain_activation_state":"active",
            "tenant_human_mfa_browser_acceptance":"REQUIRED",
            "customer_release_go":False,
        }
        acc = {
            "schema":"ipat.customer-human-mfa-acceptance.v1",
            "domain_id":D,
            "hostname":H,
            "tenant_id":T,
            "control_tenant_id":C,
            "control_hostname":CH,
            "real_human_mfa_browser_login_verified":True,
            "unauthorized_menu_hidden_verified":True,
            "unauthorized_direct_url_denied_verified":True,
            "unauthorized_api_denied_verified":True,
            "wrong_host_session_replay_denied":True,
            "cross_tenant_api_denied":True,
            "logout_revocation_verified":True,
            "stale_session_denied":True,
            "prepared_by":"tenant-operator",
            "approved_by":"security-checker",
            "evidence_id":"browser-run-r999",
            "artifact_sha256":"a"*64,
            "reviewed_at_utc":now.isoformat(),
        }
        return activation, acc

    def test_all_real_browser_denials_and_maker_checker_required(self):
        now = datetime(2026,10,6,4,tzinfo=timezone.utc)
        a,b = self.good(now)
        out = m.validate(a,b,D,H,now)
        self.assertTrue(out["customer_release_go"])
        for key in m.REQUIRED_BOOL:
            bad = dict(b)
            bad[key] = False
            with self.assertRaises(ValueError):
                m.validate(a,bad,D,H,now)
        with self.assertRaises(ValueError):
            m.validate(a,dict(b,approved_by=b["prepared_by"]),D,H,now)
        with self.assertRaises(ValueError):
            m.validate(a,dict(b,reviewed_at_utc=(now-timedelta(hours=73)).isoformat()),D,H,now)

    def test_exact_customer_and_distinct_cross_tenant_control(self):
        now = datetime.now(timezone.utc)
        a,b = self.good(now)
        for bad in (
            dict(b,domain_id=C),
            dict(b,hostname=CH),
            dict(b,control_tenant_id=T),
            dict(b,control_hostname=H),
        ):
            with self.assertRaises(ValueError):
                m.validate(a,bad,D,H,now)
        with self.assertRaises(ValueError):
            m.validate(dict(a,customer_release_go=True),b,D,H,now)

    def test_external_artifact_hash_mandatory(self):
        now = datetime.now(timezone.utc)
        a,b = self.good(now)
        for value in ("","0"*64,"xyz","A"*64):
            with self.assertRaises(ValueError):
                m.validate(a,dict(b,artifact_sha256=value),D,H,now)

    def test_finalizer_is_evidence_only_and_exact_active_db_read(self):
        self.assertIn("r999_validate_customer_browser_acceptance.py",FINAL)
        self.assertIn("get_tenant_domain_activation_result",FINAL)
        self.assertIn('"customer_release_go":True',FINAL)
        self.assertIn("runuser -u ipatdverify",FINAL)
        self.assertNotIn("record_tenant_domain_ingress_check",FINAL)
        for forbidden in ("nginx","certbot","ufw","iptables","nft ","routeros","telnet","snmp","systemctl"):
            self.assertNotIn(forbidden,FINAL.lower())

    def test_secure_evidence_file(self):
        now = datetime.now(timezone.utc)
        a,b = self.good(now)
        with tempfile.TemporaryDirectory() as td:
            p = Path(td)/"a.json"
            q = Path(td)/"b.json"
            p.write_text(json.dumps(a))
            q.write_text(json.dumps(b))
            os.chmod(p,0o600)
            os.chmod(q,0o600)
            self.assertEqual(m.secure_json(p,os.geteuid())["domain_id"],D)
            os.chmod(q,0o644)
            with self.assertRaises(ValueError):
                m.secure_json(q,os.geteuid())

if __name__=="__main__":
    unittest.main()
