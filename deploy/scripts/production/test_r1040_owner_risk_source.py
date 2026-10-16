"""R10.40 explicit owner-risk exception: no false restoration, local-only."""
import importlib.util
from datetime import datetime, timedelta, timezone
from pathlib import Path
import json
import os
import subprocess
import tempfile
import unittest
import uuid

MODULE=Path(__file__).resolve().parent/"r1040_validate_owner_risk_acceptance.py"
R996=Path(__file__).resolve().parent/"r996_prepare_production_foundation.sh"
spec=importlib.util.spec_from_file_location("ipat_r1040_owner_risk", MODULE)
validator=importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)
COMMIT="a"*40

def sample(now=None):
    now=now or datetime.now(timezone.utc)
    return {
        "schema":"ipat.owner-risk-acceptance.v1",
        "source_commit":COMMIT,
        "decision":validator.DECISION,
        "owner_acknowledgement":validator.ACK,
        "independent_full_host_boot_restore_verified":False,
        "production_pitr_ha_verified":False,
        "public_edge_activation_authorized":False,
        "encrypted_restic_read_data_verified":True,
        "both_archives_tar_readable":True,
        "backup_snapshot_ids":{"rootfs":"2d767d09","efi":"af1a0129"},
        "approval_id":str(uuid.uuid4()),
        "approved_by_role":"product_owner",
        "approved_at_utc":now.isoformat(),
    }

class R1040OwnerRiskExceptionTests(unittest.TestCase):
    def test_explicit_owner_risk_acceptance_not_a_restore_pass(self):
        result=validator.validate_document(sample(),COMMIT)
        self.assertEqual(result["recovery_state"],"OWNER_RISK_ACCEPTED_NOT_RESTORED_ON_INDEPENDENT_VM")
        self.assertTrue(result["local_foundation_only"])
        self.assertFalse(result["public_go"])

    def test_reject_claim_of_successful_boot_restore_and_public_tls(self):
        for key, value in (("independent_full_host_boot_restore_verified",True),
                           ("production_pitr_ha_verified",True),
                           ("public_edge_activation_authorized",True),
                           ("encrypted_restic_read_data_verified",False),
                           ("both_archives_tar_readable",False)):
            with self.subTest(key=key):
                d=sample();d[key]=value
                with self.assertRaises(ValueError):validator.validate_document(d,COMMIT)

    def test_reject_missing_signed_intent_wrong_source_or_role(self):
        for key,value in (("owner_acknowledgement","YES"),
                          ("decision","ACTIVATE_PUBLIC"),
                          ("source_commit","b"*40),
                          ("approved_by_role","guest"),
                          ("backup_snapshot_ids",{"rootfs":"2d767d09"}),
                          ("approval_id","bad-id")):
            with self.subTest(key=key):
                d=sample();d[key]=value
                with self.assertRaises(ValueError):validator.validate_document(d,COMMIT)

    def test_expiration_future_or_unzoned_review(self):
        t=datetime.now(timezone.utc)
        for stamp in ((t-timedelta(hours=25)).isoformat(),(t+timedelta(minutes=6)).isoformat(),t.replace(tzinfo=None).isoformat()):
            d=sample();d["approved_at_utc"]=stamp
            with self.assertRaises(ValueError):validator.validate_document(d,COMMIT,now=t)

    def test_ownership_symlink_hardlink_and_duplicate_key(self):
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/"review.json"
            p.write_text(json.dumps(sample()))
            p.chmod(0o600)
            data=validator.read_secure(p,uid=os.geteuid(),trusted_parent=False)
            self.assertEqual(data["decision"],validator.DECISION)
            p.chmod(0o644)
            with self.assertRaises(ValueError):validator.read_secure(p,uid=os.geteuid(),trusted_parent=False)
            p.chmod(0o600)
            link=Path(td)/"link"
            link.symlink_to(p)
            with self.assertRaises(OSError):validator.read_secure(link,uid=os.geteuid(),trusted_parent=False)
            hard=Path(td)/"hard"
            os.link(p,hard)
            with self.assertRaises(ValueError):validator.read_secure(p,uid=os.geteuid(),trusted_parent=False)
            hard.unlink()
            p.write_text(p.read_text()[:-1]+',"schema":"duplicate"}')
            with self.assertRaises(ValueError):
                validator.read_secure(p,uid=os.geteuid(),trusted_parent=False)

    def test_original_restore_gate_preserved_and_risk_not_public(self):
        text=R996.read_text()
        self.assertIn('r996_validate_recovery_attestation.py',text)
        self.assertIn('r1040_validate_owner_risk_acceptance.py',text)
        self.assertIn('IPAT_R1040_ACCEPT_LOCAL_DB_RISK',text)
        self.assertIn('OWNER_ACCEPTED_UNTESTED_BOOT_RESTORE_LOCAL_ONLY',text)
        self.assertIn('"application_services":"DISABLED"',text)
        self.assertIn('"public_http_https":"NOT_CONFIGURED"',text)
        self.assertIn('"public_go":False',text)
        self.assertLess(text.index("r1040_validate_owner_risk_acceptance.py"),
                        text.index("IPAT_R985_PREPARE=PREPARE_REVIEWED_PLATFORM_RUNTIME"))
        result=subprocess.run(["bash","-n",str(R996)],capture_output=True,text=True)
        self.assertEqual(result.returncode,0,result.stderr)

    def test_r997_rejects_local_only_waiver_even_if_pitr_passes(self):
        import importlib.util
        source=MODULE.parent/"r997_validate_pitr_acceptance.py"
        spec=importlib.util.spec_from_file_location("ipat_r1040_platform_release",source)
        checker=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(checker)
        f={"schema":"ipat.production-foundation.v1","source_commit":COMMIT,
           "application_services":"DISABLED","public_go":False,
           "recovery_gate":"OWNER_ACCEPTED_UNTESTED_BOOT_RESTORE_LOCAL_ONLY",
           "independent_full_host_restore_verified":False}
        with self.assertRaisesRegex(ValueError,"cannot activate public edge"):
            checker.validate_documents(f, {}, datetime.now(timezone.utc))
        f["recovery_gate"]="INDEPENDENT_HOST_RECOVERY_VERIFIED"
        with self.assertRaisesRegex(ValueError,"lacks independent host restore"):
            checker.validate_documents(f, {}, datetime.now(timezone.utc))

    def test_nonroot_cannot_run_foundation(self):
        if os.geteuid()==0:self.skipTest("negative case needs nonroot CI")
        r=subprocess.run(["bash",str(R996)],capture_output=True,text=True,timeout=5)
        self.assertEqual(r.returncode,4)
        self.assertIn("root required",r.stderr)
