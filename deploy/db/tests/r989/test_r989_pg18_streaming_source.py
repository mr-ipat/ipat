from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[4]
SCRIPT=(ROOT/"deploy/db/tests/r989_pg18_streaming_failover_docker.sh").read_text()

class R989StreamingSource(unittest.TestCase):
    def test_pinned_postgres18_and_exact_migrations(self):
        self.assertIn("postgres:18-alpine@sha256:", SCRIPT)
        self.assertIn("r987_migrations.sha256", SCRIPT)
        self.assertIn("show server_version_num", SCRIPT)
        self.assertIn("IPAT_R989_PG18_STREAMING_DOCKER", SCRIPT)

    def test_physical_streaming_replication_and_controlled_promotion(self):
        for marker in ("REPLICATION PASSWORD", "host replication ipat_replica",
                       "pg_basebackup", "-Xs", "-R", "pg_stat_replication",
                       "state", "streaming", "pg_is_in_recovery()",
                       "docker stop -t 10", "pg_ctl -D /standby promote -w"):
            self.assertIn(marker, SCRIPT)

    def test_standby_write_denied_before_promotion_and_writable_after(self):
        self.assertIn("standby accepted write before promotion", SCRIPT)
        self.assertIn("WRITE-AFTER-PROMOTE", SCRIPT)
        self.assertIn("promoted node not writable after controlled promotion", SCRIPT)
        self.assertIn("primary still running before promotion", SCRIPT)

    def test_promoted_rls_is_rechecked(self):
        for marker in ("-U ipat_app_runtime", "SET LOCAL ipat.tenant_id",
                       "promoted runtime sees unscoped data",
                       "promoted tenant RLS count mismatch"):
            self.assertIn(marker, SCRIPT)

    def test_no_host_ports_and_no_network_device_mutation(self):
        self.assertIn('[[ -z $(docker port "$primary"', SCRIPT)
        self.assertIn('[[ -z $(docker port "$standby"', SCRIPT)
        lowered=SCRIPT.lower()
        for forbidden in ("-p 5432:", "--network host", "iptables ", "nft ", "ufw ",
                          "ssh ", "scp ", "10.10.13.233", "systemctl "):
            self.assertNotIn(forbidden, lowered)

    def test_scope_is_not_overclaimed(self):
        self.assertIn("NOT_AUTOMATIC_FAILOVER_NOT_FAILBACK_NOT_PRODUCTION_HA", SCRIPT)

if __name__=="__main__":
    unittest.main()
