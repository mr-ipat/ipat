from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[4]
SCRIPT=(ROOT/"deploy/db/tests/r988_pg18_wal_pitr_docker.sh").read_text()

class R988PitrSource(unittest.TestCase):
    def test_fail_closed_and_pinned_postgres18(self):
        self.assertIn("IPAT_R988_PG18_PITR_DOCKER",SCRIPT)
        self.assertIn("postgres:18-alpine@sha256:",SCRIPT)
        self.assertIn("r987_migrations.sha256",SCRIPT)
        self.assertGreaterEqual(SCRIPT.count("docker exec -i -u postgres"),2)
        self.assertIn("show server_version_num",SCRIPT)

    def test_real_basebackup_archive_and_named_pitr(self):
        for marker in ("pg_basebackup","pg_verifybackup","archive_mode=on","archive_command=",
                       "pg_create_restore_point('r988_before_bad_change')","pg_switch_wal()",
                       "restore_command =","recovery_target_name =",
                       "recovery_target_action =","recovery.signal","pg_is_in_recovery()"):
            self.assertIn(marker,SCRIPT)

    def test_proves_postpoint_change_absent_and_rls_still_denies(self):
        for marker in ("BAD-AFTER-PITR","post-restore-point synthetic bad row survived PITR",
                       "-U ipat_app_runtime","SET LOCAL ipat.tenant_id",
                       "restored tenant scoped runtime row count mismatch","sha256sum"):
            self.assertIn(marker,SCRIPT)

    def test_no_published_ports_or_host_network_mutations(self):
        self.assertIn('[[ -z $(docker port "$source_name"',SCRIPT)
        self.assertIn('[[ -z $(docker port "$restore_name"',SCRIPT)
        lowered=SCRIPT.lower()
        for forbidden in ("-p 5432:","--network host","iptables ","nft ","ufw ","ssh ","scp ",
                          "systemctl ","pg_ctlcluster","10.10.13.233"):
            self.assertNotIn(forbidden,lowered)

    def test_reports_scope_honestly(self):
        self.assertIn("NOT_OFFSITE_NOT_HA_NOT_PRODUCTION_RPO_RTO",SCRIPT)

if __name__=="__main__":
    unittest.main()
