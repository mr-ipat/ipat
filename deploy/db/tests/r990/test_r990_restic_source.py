from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[4]
SCRIPT=(ROOT/"deploy/db/tests/r990_pg18_restic_encrypted_pitr.sh").read_text()
class R990ResticSource(unittest.TestCase):
    def test_explicit_disposable_and_real_restic(self):
        for x in ("IPAT_R990_RESTIC_PITR_DOCKER","restic init","restic backup",
                  "restic check","restic restore latest","RESTIC_PASSWORD_FILE"):
            self.assertIn(x,SCRIPT)
    def test_real_pg18_base_wal_pitr(self):
        for x in ("pg_basebackup","pg_verifybackup","archive_mode=on","archive_command=",
                  "pg_create_restore_point('r990_before_bad_change')","recovery_target_name",
                  "recovery.signal","BAD-AFTER-RESTORE-POINT"):
            self.assertIn(x,SCRIPT)
    def test_plaintext_deleted_before_restore(self):
        self.assertLess(SCRIPT.index('rm -rf "$plain/base" "$plain/wal"'),
                        SCRIPT.index('restic restore latest'))
        self.assertIn("plaintext staging deletion failed",SCRIPT)
        self.assertIn("encrypted restore artifact hash mismatch",SCRIPT)
    def test_linux_bind_mount_not_read_by_host_before_normalization(self):
        self.assertIn('find /archive -maxdepth 1 -type f',SCRIPT)
        self.assertNotIn('find "$plain/wal" -maxdepth 1 -type f',SCRIPT)
        self.assertIn('chown -R $host_uid:$host_gid /cleanup',SCRIPT)
        self.assertIn('chown -R $host_uid:$host_gid /plain/base /plain/wal',SCRIPT)

    def test_restic_restore_integrity_check_is_readonly_root_then_runtime_reowned(self):
        self.assertIn('--user 0 -v "$restored/base:/verify:ro"',SCRIPT)
        self.assertIn('chown -R postgres:postgres /restore',SCRIPT)

    def test_rls_and_bad_change_absence(self):
        for x in ("-U ipat_app_runtime","SET LOCAL ipat.tenant_id",
                  '[[ "$good_count" == 1 && "$bad_count" == 0 ]]',
                  '[[ "$unscoped" == 0 && "$scoped" == 1 ]]'):
            self.assertIn(x,SCRIPT)
    def test_no_external_or_host_mutation_claim(self):
        lower=SCRIPT.lower()
        for x in ("ssh ","scp ","iptables ","nft ","ufw ","systemctl ","--network host",
                  "-p 5432:","10.10.13.233"):
            self.assertNotIn(x,lower)
        self.assertIn("LOCAL_RESTIC_REPOSITORY_ONLY_NOT_ACTUAL_OFFSITE_NOT_PRODUCTION_RPO_RTO",SCRIPT)
if __name__=="__main__":unittest.main()
