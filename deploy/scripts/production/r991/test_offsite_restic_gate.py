import argparse,json,os,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import offsite_restic_gate as m

def args(**kw):
    base=dict(repository_location="sftp:backup@dr.example.net:/srv/ipat",
      active_failure_domain="primary-vps-a",backup_failure_domain="offsite-storage-b",
      snapshot_tag="ipat-production-pg18",query=False)
    base.update(kw);return argparse.Namespace(**base)

class Gate(unittest.TestCase):
    def test_remote_rejects_local_credentials_and_private_destinations(self):
        good=["sftp:backup@dr.example.net:/srv/ipat","rest:https://backup.example.net/ipat",
              "s3:https://s3.example.net/ipat"]
        bad=["/mnt/backup","./repo","file:/tmp/repo","rest:http://localhost:8000/x",
             "rest:https://user:password@backup.example.net/x","sftp:user:pass@dr.example.net:/x",
             "sftp:root@127.0.0.1:/x",
             "sftp:backup@10.0.0.9:/x"]
        for x in good:self.assertTrue(m.remote_location(x),x)
        for x in bad:self.assertFalse(m.remote_location(x),x)
    def test_same_failure_domain_denied_before_query(self):
        with self.assertRaises(ValueError):
            m.report(args(backup_failure_domain="primary-vps-a"),{})
    def test_successful_readonly_query_still_no_go_and_no_secrets(self):
        with tempfile.TemporaryDirectory() as td:
            pf=Path(td)/"pw";pf.write_text("synthetic-only\n");pf.chmod(0o600)
            env={"RESTIC_REPOSITORY":"sftp:backup@dr.example.net:/srv/ipat",
                 "RESTIC_PASSWORD_FILE":str(pf)}
            payload=[{"id":"abcdef1234567890","time":"2026-10-05T10:00:00Z",
                      "tags":["ipat-production-pg18"]}]
            done=type("P",(),{"returncode":0,"stdout":json.dumps(payload),"stderr":""})()
            with patch.object(m.subprocess,"run",return_value=done) as run:
                out=m.report(args(query=True),env)
            command=run.call_args.args[0]
            self.assertEqual(command[:3],["restic","snapshots","--json"])
            self.assertIn("--no-lock",command)
            self.assertFalse(m.FORBIDDEN_RESTIC_WORDS.intersection(command))
            self.assertFalse(out["public_go"])
            self.assertFalse(out["repository_mutated"])
            self.assertNotIn("RESTIC_PASSWORD_FILE",json.dumps(out))
            self.assertNotIn(str(pf),json.dumps(out))
    def test_inline_secret_environment_denied(self):
        with tempfile.TemporaryDirectory() as td:
            pf=Path(td)/"pw";pf.write_text("x\n");pf.chmod(0o600)
            env={"RESTIC_REPOSITORY":"sftp:backup@dr.example.net:/srv/ipat",
                 "RESTIC_PASSWORD_FILE":str(pf),"RESTIC_PASSWORD":"forbidden"}
            ok,leaks=m.secret_environment_ok(env)
            self.assertTrue(ok)
            self.assertEqual(leaks,["RESTIC_PASSWORD"])
            with self.assertRaises(ValueError):m.run_restic_readonly("ipat-production-pg18",env)
    def test_repository_mismatch_denied(self):
        with self.assertRaises(ValueError):
            m.report(args(),{"RESTIC_REPOSITORY":"sftp:backup@other.example.net:/srv/ipat"})
if __name__=="__main__":unittest.main()
