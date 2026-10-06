import json,os,tempfile,unittest,importlib.util
from datetime import datetime,timezone,timedelta
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
ACT=(ROOT/"deploy/scripts/production/r998_activate_single_customer.sh").read_text()
VERIFY=(ROOT/"deploy/scripts/production/r998_verify_exact_customer_ingress.sh").read_text()
MIG=(ROOT/"deploy/db/migrations/0034_exact_customer_domain_activation_result.sql").read_text()
VAL=ROOT/"deploy/scripts/production/r998_validate_platform_mfa_acceptance.py"
spec=importlib.util.spec_from_file_location("r998mfa",VAL)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class R998Source(unittest.TestCase):
    def test_exact_domain_authority_not_mass_timer(self):
        self.assertIn("get_tenant_domain_activation_result",ACT)
        self.assertIn("get_tenant_domain_activation_result",VERIFY)
        self.assertNotIn("list_tenant_domain_ingress_checks",VERIFY)
        self.assertNotIn("ipat-domain-ingress-verifier.timer",ACT)
        self.assertNotIn("r994_enable_domain_ingress_verifier.sh",ACT)
        self.assertIn("IPAT_R998_DOMAIN_ID",ACT)
        self.assertIn("IPAT_R998_DOMAIN_ID",VERIFY)

    def test_activation_order_and_global_rollback(self):
        order=[ACT.index(x) for x in (
          "r998_validate_platform_mfa_acceptance.py",
          "r992_add_tenant_oidc_instance.sh",
          "IPAT_R993_PHASE=staging",
          "IPAT_R993_PHASE=production",
          'runuser -u ipatdverify -- env',
        )]
        self.assertEqual(order,sorted(order))
        self.assertIn("customer_edge_rollback=",ACT)
        self.assertIn("IPAT_R993_ROLLBACK_EXPORT_FILE",ACT)
        self.assertNotIn("before_rollbacks=",ACT)
        self.assertNotIn("comm -13",ACT)
        self.assertIn('"$customer_edge_rollback"',ACT)
        self.assertIn('systemctl disable --now "ipat-tenant-oidc@$instance.service"',ACT)
        self.assertIn("NONACTIVE_DB_STATE_PRESERVED",ACT)
        self.assertIn("rollback_armed=NO",ACT)

    def test_active_boundary_keeps_customer_release_blocked(self):
        self.assertIn('"customer_release_go":False',ACT)
        self.assertIn('"tenant_human_mfa_browser_acceptance":"REQUIRED"',ACT)
        self.assertIn("R998_SINGLE_CUSTOMER_DOMAIN_ACTIVE_PENDING_TENANT_HUMAN_MFA_ACCEPTANCE",ACT)
        self.assertNotIn('"public_go":True',ACT)
        self.assertNotIn('"customer_release_go":True',ACT)

    def test_verifier_rechecks_txt_a_tls_and_ordered_transition(self):
        for required in ("dig @1.1.1.1 +short TXT","dig @1.1.1.1 +short A",
                         "openssl s_client","https://$host/auth/oidc/start",
                         "routing_ready","tls_ready","activate"):
            self.assertIn(required,VERIFY)
        self.assertIn("active DB state but ownership TXT no longer matches",VERIFY)
        self.assertIn('transition activate',VERIFY)
        tail=VERIFY.split('transition activate',1)[1]
        self.assertNotIn('result)',tail)

    def test_migration_is_read_only_and_ingress_only(self):
        self.assertIn("CREATE FUNCTION ipat_platform.get_tenant_domain_activation_result",MIG)
        self.assertIn("STABLE",MIG)
        self.assertIn("TO ipat_domain_ingress_verifier",MIG)
        self.assertIn("REVOKE ALL",MIG)
        self.assertNotIn("GRANT SELECT ON ipat_platform.tenant_domains",MIG)
        for mutation in ("INSERT INTO","UPDATE ipat_platform.tenant_domains","DELETE FROM"):
            self.assertNotIn(mutation,MIG)

    def good(self,now):
        edge={"schema":"ipat.platform-edge.v1","public_ipv4":"202.162.204.121",
              "platform_https":"ACTIVE_CA_TRUSTED_SHORTLIVED_IP_CERT","platform_runtime":"ACTIVE",
              "public_go":False}
        acc={"schema":"ipat.platform-human-mfa-acceptance.v1","platform_public_ipv4":"202.162.204.121",
             "real_human_mfa_browser_login_verified":True,"logout_revocation_verified":True,
             "wrong_host_replay_denied":True,"stale_session_denied":True,
             "prepared_by":"platform-operator","approved_by":"security-reviewer",
             "evidence_id":"platform-mfa-20261006","reviewed_at_utc":now.isoformat()}
        return edge,acc

    def test_platform_mfa_acceptance_is_current_maker_checker(self):
        now=datetime(2026,10,6,4,0,tzinfo=timezone.utc);edge,acc=self.good(now)
        out=m.validate(edge,acc,"202.162.204.121",now)
        self.assertTrue(out["customer_activation_allowed"])
        self.assertFalse(out["customer_release_go"])
        bad=dict(acc,approved_by=acc["prepared_by"])
        with self.assertRaises(ValueError):m.validate(edge,bad,"202.162.204.121",now)
        stale=dict(acc,reviewed_at_utc=(now-timedelta(days=8)).isoformat())
        with self.assertRaises(ValueError):m.validate(edge,stale,"202.162.204.121",now)
        replay=dict(acc,wrong_host_replay_denied=False)
        with self.assertRaises(ValueError):m.validate(edge,replay,"202.162.204.121",now)

    def test_secure_acceptance_files(self):
        now=datetime.now(timezone.utc);edge,acc=self.good(now)
        with tempfile.TemporaryDirectory() as td:
            a=Path(td)/"edge.json";b=Path(td)/"mfa.json"
            a.write_text(json.dumps(edge));b.write_text(json.dumps(acc))
            os.chmod(a,0o600);os.chmod(b,0o600)
            out=m.validate(m.secure_json(a,os.geteuid()),m.secure_json(b,os.geteuid()),"202.162.204.121",now)
            self.assertTrue(out["customer_activation_allowed"])
            os.chmod(b,0o644)
            with self.assertRaises(ValueError):m.secure_json(b,os.geteuid())

if __name__=="__main__":unittest.main()
