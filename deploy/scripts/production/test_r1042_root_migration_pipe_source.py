"""R10.42 regression: root-owned release SQL stays private, migrator sees stdin."""
from pathlib import Path
import re
import subprocess
import unittest

ROOT=Path(__file__).resolve().parents[2]
SOURCE=(ROOT/"scripts/production/r987_bootstrap_postgres18.sh").read_text()
DOCKER=(ROOT/"db/tests/r987_ubuntu26_bootstrap_docker.sh").read_text()

class R1042RootSqlMigrationTests(unittest.TestCase):
    def test_root_reads_checksum_manifest_sql_migrator_only_stdin(self):
        self.assertIn("sha256sum -c deploy/db/production/r987_migrations.sha256",SOURCE)
        self.assertIn('cat -- "$root/$file" | runuser -u ipatpgmigrate -- psql',SOURCE)
        self.assertIn('-v ON_ERROR_STOP=1 -q -f -',SOURCE)
        self.assertNotIn('-f "$root/$file"',SOURCE)
        self.assertIn("set -Eeuo pipefail",SOURCE)
        self.assertLess(SOURCE.index("sha256sum -c deploy/db/production/r987_migrations.sha256"),
                        SOURCE.index('cat -- "$root/$file"'))

    def test_disposable_ubuntu_test_uses_actual_root_only_directory(self):
        self.assertIn("-v \"$root:/root/ipat-src:ro\"",DOCKER)
        self.assertIn("IPAT_R987_SOURCE_ROOT=/root/ipat-src",DOCKER)
        self.assertNotIn("IPAT_R987_SOURCE_ROOT=/src",DOCKER)
        self.assertIn("R987_UBUNTU26_DISPOSABLE_BOOTSTRAP_PASS",DOCKER)

    def test_stream_failure_does_not_become_false_sql_success(self):
        # The actual pipeline syntax is validated under bash pipefail.
        cmd='''set -Eeuo pipefail
cat /definitely/not/existing/ipat-r1042.sql | cat >/dev/null
echo INVALID_MIGRATION_SUCCESS
'''
        r=subprocess.run(["bash","-c",cmd],capture_output=True,text=True)
        self.assertNotEqual(r.returncode,0)
        self.assertNotIn("INVALID_MIGRATION_SUCCESS",r.stdout)

    def test_scripts_parse_and_prohibit_root_permission_downgrade(self):
        for p in (ROOT/"scripts/production/r987_bootstrap_postgres18.sh",
                  ROOT/"db/tests/r987_ubuntu26_bootstrap_docker.sh"):
            r=subprocess.run(["bash","-n",str(p)],capture_output=True,text=True)
            self.assertEqual(r.returncode,0,r.stderr)
        for forbidden in ("chmod -R", "chmod 777", "setfacl", "chown -R"):
            self.assertNotIn(forbidden,SOURCE)
