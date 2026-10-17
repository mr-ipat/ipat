"""R10.41 actual first-install incident regression guards."""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
PROD=ROOT/"scripts/production"
R996=(PROD/"r996_prepare_production_foundation.sh").read_text()
R987=(PROD/"r987_bootstrap_postgres18.sh").read_text()
R997=(PROD/"r997_activate_platform_edge.sh").read_text()

class R1041RollbackFailureTests(unittest.TestCase):
    def test_failed_child_must_exit_nonzero_before_any_success_marker(self):
        for label,text,fn,marker in (
            ("R996",R996,"foundation_abort","R996_PRODUCTION_FOUNDATION_PREPARED_LOCAL_DB_ONLY_NO_PUBLIC_EDGE"),
            ("R987",R987,"bootstrap_abort","created=NO\ntrap - ERR INT TERM"),
            ("R997",R997,"edge_abort","R997_PLATFORM_EDGE_ACTIVE_PENDING_REAL_HUMAN_MFA_BROWSER_ACCEPTANCE")
        ):
            with self.subTest(script=label):
                self.assertIn("trap "+fn+" ERR INT TERM",text)
                match=re.search(r"(?m)^"+fn+r"\(\)\{(.*?)^\}",text,re.S|re.M)
                self.assertIsNotNone(match)
                body=match.group(1)
                self.assertIn("trap - ERR INT TERM",body)
                self.assertIn("rollback",body)
                self.assertRegex(body,r"(?m)^\s*exit 4\s*$")
                self.assertLess(text.index("trap "+fn+" ERR INT TERM"),text.index(marker))
                result=subprocess.run(["bash","-n",str(PROD/{"R996":"r996_prepare_production_foundation.sh",
                    "R987":"r987_bootstrap_postgres18.sh","R997":"r997_activate_platform_edge.sh"}[label])],
                    capture_output=True,text=True)
                self.assertEqual(result.returncode,0,result.stderr)

    def test_actual_abort_function_does_not_continue_after_failing_command(self):
        # Extract exact production handler body and run in an isolated shell:
        # no systemd, postgresql, or root mutations.
        for name,source in (("foundation_abort",R996),("bootstrap_abort",R987),("edge_abort",R997)):
            match=re.search(r"(?m)^"+name+r"\(\)\{.*?^\}",source,re.S|re.M)
            self.assertIsNotNone(match,name)
            code='''set -Eeuo pipefail
rollback(){ echo ONLY_SYNTHETIC_ROLLBACK; }
'''+match.group(0)+f'''
trap {name} ERR INT TERM
false
echo INVALID_FALSE_SUCCESS
'''
            r=subprocess.run(["bash","-c",code],capture_output=True,text=True)
            self.assertEqual(r.returncode,4,(name,r.stdout,r.stderr))
            self.assertIn("ONLY_SYNTHETIC_ROLLBACK",r.stdout)
            self.assertNotIn("INVALID_FALSE_SUCCESS",r.stdout)

    def test_pg18_locale_fixed_before_apt_and_pg_createcluster(self):
        self.assertIn("export LANG=C.UTF-8 LC_ALL=C.UTF-8",R987)
        self.assertLess(R987.index("export LANG=C.UTF-8 LC_ALL=C.UTF-8"),
                        R987.index("apt-get update -qq"))
        self.assertLess(R987.index("export LANG=C.UTF-8 LC_ALL=C.UTF-8"),
                        R987.index("pg_createcluster 18 ipat"))
        self.assertNotIn("locale-gen",R987)
        self.assertNotIn("update-locale",R987)

    def test_markers_require_actual_pg18_socket_users_and_unit_postconditions(self):
        for required in ('pg_lsclusters --no-header', '[[ -S /run/postgresql/.s.PGSQL.5432 ]]',
                         'getent passwd "$identity"', 'temporary migrator identity survived',
                         'required runtime unit missing after install'):
            self.assertIn(required,R996)
            self.assertLess(R996.index(required),R996.index('"schema":"ipat.production-foundation.v1"'))

    def test_root_only_db_and_public_blocker_still_intact(self):
        self.assertIn('r1040_validate_owner_risk_acceptance.py',R996)
        self.assertIn('"application_services":"DISABLED"',R996)
        self.assertIn('"public_http_https":"NOT_CONFIGURED"',R996)
        self.assertIn('"public_go":False',R996)
        self.assertIn("OWNER_ACCEPTED_UNTESTED_BOOT_RESTORE_LOCAL_ONLY",
                      (PROD/"r997_validate_pitr_acceptance.py").read_text())
        self.assertNotIn("R996_FOUNDATION_PREPARED_LOCAL_DB_ONLY_NO_PUBLIC_EDGE",
                         R987)
