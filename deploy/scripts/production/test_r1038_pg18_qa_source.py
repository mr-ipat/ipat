"""R10.38 tests verify fail-closed, local-only PG18 acceptance orchestration."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
from unittest import TestCase, mock

SCRIPT = Path(__file__).resolve().parent / "r1038/run_pg18_qa.py"
spec = importlib.util.spec_from_file_location("ipat_r1038_pg18_qa", SCRIPT)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class R1038Pg18QATests(TestCase):
    def test_environment_rejects_root_credentials_and_remote_docker(self):
        with self.assertRaisesRegex(ValueError, "root execution"):
            module.safe_environment({}, uid=0)
        for key in ("PGPASSWORD", "RESTIC_REPOSITORY", "DOCKER_HOST", "PGSERVICEFILE"):
            with self.subTest(key=key):
                with self.assertRaisesRegex(ValueError, "non-isolated environment keys"):
                    module.safe_environment({key: "not-reported"}, uid=1000)
        with self.assertRaisesRegex(ValueError, "preconfigure"):
            module.safe_environment({"IPAT_R988_PG18_PITR_DOCKER": "YES"}, uid=1000)
        module.safe_environment({"DOCKER_HOST": "", "PGDATABASE": ""}, uid=1000)

    def test_script_is_fixed_to_all_four_reviewed_existing_suites(self):
        self.assertEqual([s[0] for s in module.SUITES],
                         ["migration_rls", "basebackup_pitr", "streaming_ha", "encrypted_pitr"])
        for _, name, _, marker in module.SUITES:
            self.assertTrue((module.ROOT / "deploy/db/tests" / name).is_file(), name)
            self.assertTrue(marker.endswith("=PASS"))
        src = SCRIPT.read_text()
        for forbidden in ("sudo ", "apt-get install", "pg_dropcluster", "sshpass",
                          "R996_APPLY", "systemctl ", "kubectl apply", "chmod 777"):
            self.assertNotIn(forbidden, src)

    def test_local_docker_guard_rejects_remote_socket(self):
        local = subprocess.CompletedProcess([], 0, "unix:///var/run/docker.sock\n", "")
        linux = subprocess.CompletedProcess([], 0, "linux\n", "")
        restic = subprocess.CompletedProcess([], 0, "restic 0.18\n", "")
        remote = subprocess.CompletedProcess([], 0, "ssh://root@production\n", "")
        with mock.patch.object(module, "command", side_effect=[remote]):
            with self.assertRaisesRegex(ValueError, "local Unix socket"):
                module.ensure_local_docker()
        with mock.patch.object(module, "command", side_effect=[local, linux, restic]):
            module.ensure_local_docker()

    def test_runner_refuses_missing_or_symlinked_suite(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            folder = repo / "deploy/db/tests"
            folder.mkdir(parents=True)
            with mock.patch.object(module, "command", return_value=subprocess.CompletedProcess(
                [], 0, "a"*40+"\n", ""
            )):
                with self.assertRaisesRegex(ValueError, "missing or symlinked"):
                    module.validate_runner(repo)
                for _, name, _, _ in module.SUITES:
                    (folder / name).write_text("synthetic")
                (folder / module.SUITES[0][1]).unlink()
                (folder / module.SUITES[0][1]).symlink_to(folder / module.SUITES[1][1])
                with self.assertRaisesRegex(ValueError, "missing or symlinked"):
                    module.validate_runner(repo)

    def test_failure_cannot_emit_success_when_marker_missing(self):
        name, script, key, marker = module.SUITES[0]
        incomplete = subprocess.CompletedProcess([], 0, "unrelated output", "")
        with mock.patch.object(module.subprocess, "run", return_value=incomplete):
            with self.assertRaisesRegex(ValueError, "marker_verified=False"):
                module.run_suite(module.ROOT, name, script, key, marker, {})
        with mock.patch.object(module.subprocess, "run", return_value=subprocess.CompletedProcess(
            [], 0, marker+"\n", ""
        )):
            self.assertEqual(module.run_suite(module.ROOT, name, script, key, marker, {})["result"], "PASS")
