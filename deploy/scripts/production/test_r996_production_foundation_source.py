import json, os, tempfile, unittest, subprocess, shlex, sys
from datetime import datetime, timezone, timedelta
from pathlib import Path
from unittest import mock
import importlib.util

ROOT=Path(__file__).resolve().parents[3]
SCRIPT=ROOT/"deploy/scripts/production/r996_prepare_production_foundation.sh"
VAL=ROOT/"deploy/scripts/production/r996_validate_recovery_attestation.py"
spec=importlib.util.spec_from_file_location("r996att",VAL)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class R996Source(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.s=SCRIPT.read_text()

    def test_r987_explicit_child_optin_and_exact_root_are_propagated(self):
        # Prove the real Bash env-prefix executes, not just a static grep.
        first = self.s.index("IPAT_R987_BOOTSTRAP=CREATE_REVIEWED_LOCAL_POSTGRES18")
        last = self.s.index("\n\nfor unit in", first)
        child_call = self.s[first:last]
        self.assertIn('IPAT_R987_SOURCE_ROOT="$root"', child_call)
        self.assertIn('"$root/deploy/scripts/production/r987_bootstrap_postgres18.sh"', child_call)
        with tempfile.TemporaryDirectory() as td:
            child = Path(td) / "deploy/scripts/production/r987_bootstrap_postgres18.sh"
            child.parent.mkdir(parents=True)
            child.write_text(
                """#!/usr/bin/env bash
printf '%s\n%s\n' "$IPAT_R987_BOOTSTRAP" "$IPAT_R987_SOURCE_ROOT"
"""
            )
            child.chmod(0o700)
            result = subprocess.run(
                ["bash", "-c", f"root={shlex.quote(td)}\n{child_call}"],
                capture_output=True, text=True, timeout=10, check=True,
            )
            self.assertEqual(
                result.stdout.splitlines(),
                ["CREATE_REVIEWED_LOCAL_POSTGRES18", td],
            )

    def test_r1020_rollback_owned_trees_absent_before_mutation(self):
        # These complete directory trees are removed by R9.96 rollback.
        # An existing OIDC secret/state directory must never be removed on retry.
        paths = (
            "/opt/ipat",
            "/etc/ipat",
            "/var/lib/ipat",
            "/var/lib/ipat-platform-oidc",
            "/var/lib/ipat-tenant-oidc",
            "/var/lib/ipat-domain-ownership-verifier",
        )
        start = self.s.index('r1014_require_absent \\')
        finish = self.s.index('|| die "R996_ROLLBACK_OWNED_TREE_EXISTS', start)
        guarded = self.s[start:finish]
        for path in paths:
            self.assertIn(path, guarded)
        self.assertLess(finish, self.s.index("rollback_armed=YES"))
        self.assertLess(finish, self.s.index("IPAT_R985_PREPARE="))
        self.assertIn('. "$root/deploy/scripts/production/r1014_require_absent.sh"', self.s[:start])
        self.assertIn("rm -rf /var/lib/ipat-platform-oidc", self.s)
        self.assertIn("/var/lib/ipat-tenant-oidc /var/lib/ipat-domain-ownership-verifier", self.s)

    def test_r1020_existing_directory_file_and_dangling_symlink_refused_intact(self):
        helper = ROOT / "deploy/scripts/production/r1014_require_absent.sh"
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for kind in ("dir", "file", "symlink"):
                target = root / kind
                if kind == "dir":
                    target.mkdir()
                    (target / "tenant-secret.fixture").write_text("MUST_BE_PRESERVED")
                elif kind == "file":
                    target.write_text("MUST_BE_PRESERVED")
                else:
                    target.symlink_to(root / "does-not-exist")
                argv = ["bash", "-c",
                        'set -e; source "$1"; r1014_require_absent "$2"; echo SHOULD_NOT_PASS',
                        "r1020-test", str(helper), str(target)]
                p = subprocess.run(argv, text=True, capture_output=True, timeout=3)
                self.assertNotEqual(p.returncode, 0, kind)
                self.assertNotIn("SHOULD_NOT_PASS", p.stdout)
                self.assertIn("R1014_PREEXISTING_ARTIFACT_REFUSED", p.stderr)
                self.assertTrue(target.is_symlink() or target.exists())
                if kind == "dir":
                    self.assertEqual((target / "tenant-secret.fixture").read_text(), "MUST_BE_PRESERVED")
                elif kind == "file":
                    self.assertEqual(target.read_text(), "MUST_BE_PRESERVED")

    def test_r1021_root_owned_checkout_and_binary_gated_before_git_and_mutation(self):
        start = self.s.index("# R10.21: a root-run orchestrator")
        end = self.s.index("actual_source=$(git -C", start)
        guarded = self.s[start:end]
        for marker in (
            '[[ $root == /*',
            '[[ $binary == /*',
            'for trusted_path in "$root" "$(dirname -- "$binary")"',
            'stat -c \'%u %a\' "$trusted_path"',
            'group/world-writable source/binary ancestor',
            'binary requires root-owned single-link file',
            'unsafe_source=$(find "$root"',
            '-type l -o ! -uid 0 -o -perm /022',
            'untrusted checkout inode',
        ):
            self.assertIn(marker, guarded)
        self.assertLess(end, self.s.index('r996_validate_recovery_attestation.py'))
        self.assertLess(end, self.s.index('rollback_armed=YES'))
        self.assertNotIn('IPAT_R1021_ALLOW_UNSAFE', guarded)

    @unittest.skipUnless(sys.platform.startswith("linux"), "GNU find is required by Ubuntu26 installer")
    def test_r1021_gnu_find_refuses_untrusted_tree_without_modification(self):
        start = self.s.index('unsafe_source=$(find "$root"')
        end = self.s.index('\n[[ -z $unsafe_source ]]', start)
        snippet = self.s[start:end]
        with tempfile.TemporaryDirectory() as td:
            r = Path(td)
            clean = r / "fixture"
            clean.mkdir()
            os.chmod(clean, 0o700)
            (clean / "approved-file").write_text("PRESERVE")
            os.chmod(clean / "approved-file", 0o600)
            expected_uid = os.getuid()
            # Inspect actual GNU find selector, requiring both uid and mode.
            owner_filter = snippet.replace('! -uid 0', f'! -uid {expected_uid}')
            for kind in ("symlink", "writable", "directory"):
                target = clean / "untrusted"
                if kind == "symlink":
                    target.symlink_to(r / "not-present")
                elif kind == "writable":
                    target.write_text("PRESERVE")
                    os.chmod(target, 0o666)
                else:
                    target.mkdir()
                    os.chmod(target, 0o777)
                command = (
                    'root="$1"; die(){ exit 4; }; '
                    + owner_filter + '\n'
                    + 'printf "%s\\n" "$unsafe_source"'
                )
                p = subprocess.run(
                    ["bash", "-c", command, "r1021", str(clean)],
                    capture_output=True, text=True, timeout=4,
                )
                self.assertEqual(p.returncode, 0, p.stderr)
                self.assertIn(str(target), p.stdout)
                self.assertEqual((clean / "approved-file").read_text(), "PRESERVE")
                if target.is_symlink():
                    target.unlink()
                elif target.is_dir():
                    target.rmdir()
                else:
                    self.assertEqual(target.read_text(), "PRESERVE")
                    target.unlink()

    def test_r1023_git_checkout_requires_zero_tracked_untracked_or_ignored_files(self):
        # Execute the exact production Git status arguments against a real
        # disposable repository, not merely substring-assert expected flags.
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)/"source"
            root.mkdir()
            (root/".gitignore").write_text("shadowed.py\\n")
            tracked=root/"reviewed.py"
            tracked.write_text("print('reviewed')\\n")
            subprocess.run(["git","init","--quiet",str(root)],check=True,timeout=10)
            subprocess.run(["git","-C",str(root),"add",".gitignore","reviewed.py"],check=True,timeout=10)
            subprocess.run(
                ["git","-C",str(root),"-c","user.name=IPAT CI",
                 "-c","user.email=ci@example.invalid","commit","--quiet","-m","fixture"],
                check=True,timeout=10,
            )
            cmd=["git","-C",str(root),"status","--porcelain",
                 "--untracked-files=all","--ignored=matching"]
            clean=subprocess.run(cmd,text=True,capture_output=True,check=True,timeout=10)
            self.assertEqual(clean.stdout,"")
            for path,marker in ((root/"shadowed.py","!! "),
                                (root/"unknown.py","?? ")):
                path.write_text("raise RuntimeError('UNREVIEWED')\\n")
                status=subprocess.run(cmd,text=True,capture_output=True,check=True,timeout=10)
                self.assertIn(marker+path.name,status.stdout)
            tracked.write_text("raise RuntimeError('MODIFIED')\\n")
            status=subprocess.run(cmd,text=True,capture_output=True,check=True,timeout=10)
            self.assertIn(" M reviewed.py",status.stdout)
        guard=self.s.index("source_status=$(git -C")
        self.assertIn("--untracked-files=all --ignored=matching",self.s[guard:guard+200])
        self.assertIn('|| die "cannot verify Git checkout cleanliness"',self.s[guard:guard+300])
        self.assertLess(guard,self.s.index("r996_validate_recovery_attestation.py"))
        self.assertLess(guard,self.s.index("rollback_armed=YES"))

    def test_r1023_python_isolation_does_not_import_local_shadow(self):
        self.assertIn(
            'python3 -I "$root/deploy/scripts/production/r996_validate_recovery_attestation.py"',
            self.s,
        )
        with tempfile.TemporaryDirectory() as td:
            (Path(td)/"argparse.py").write_text("raise RuntimeError('SHADOWED')\\n")
            regular=subprocess.run([sys.executable,"-c","import argparse"],cwd=td,
                                   capture_output=True,text=True,timeout=10)
            self.assertNotEqual(regular.returncode,0)
            isolated=subprocess.run([sys.executable,"-I","-c","import argparse"],
                                    cwd=td,capture_output=True,text=True,timeout=10)
            self.assertEqual(isolated.returncode,0,isolated.stderr)

    def test_dependency_order_is_locked(self):
        marks=[
          "r985_prepare_platform_runtime.sh",
          "r992_prepare_tenant_runtime.sh",
          "r994_prepare_domain_ingress_verifier.sh",
          "r995_prepare_domain_ownership_verifier.sh",
          "r987_bootstrap_postgres18.sh",
        ]
        positions=[self.s.index(x) for x in marks]
        self.assertEqual(positions,sorted(positions))

    def test_no_public_ingress_firewall_or_device_action(self):
        for forbidden in ("r984_apply_ip_tls_ingress.sh","r993_apply_customer_ingress.sh",
                          "iptables","nft ","firewall-cmd","routeros","telnet ","snmp-set"):
            self.assertNotIn(forbidden,self.s)
        self.assertIn('"application_services":"DISABLED"',self.s)
        self.assertIn('"public_go":False',self.s)

    def test_first_mutation_follows_global_preflight(self):
        first=self.s.index("IPAT_R985_PREPARE=PREPARE_REVIEWED_PLATFORM_RUNTIME")
        for required in ("r996_validate_recovery_attestation.py","tracked, untracked and ignored source artifacts",
                         "existing IPAT runtime identity","managed/public/runtime port already has a listener"):
            self.assertLess(self.s.index(required),first)

    def test_rollback_drops_new_cluster_and_runtime_identities(self):
        self.assertIn("rollback_armed=YES",self.s)
        self.assertIn("declare -F rollback",self.s)
        self.assertIn("pg_dropcluster --stop 18 ipat",self.s)
        for u in ("ipatpgmigrate","ipatdnsverify","ipatdverify","ipattoidc","ipattapi","ipatpoidc","ipatpapi"):
            self.assertIn('userdel "$u"',self.s)
        self.assertIn("R996_ROLLBACK_SAFE_STATE_APPLICATIONS_DISABLED_NO_PUBLIC_EDGE",self.s)

    def test_attestation_is_recent_and_exact_source(self):
        now=datetime(2026,10,6,3,0,tzinfo=timezone.utc)
        src="a"*40
        data={
          "schema":"ipat.recovery-readiness.v1","source_commit":src,
          "provider_rescue_root_console_verified":True,
          "independent_full_host_restore_verified":True,
          "independent_backup_destination_verified":True,
          "reviewed_at_utc":(now-timedelta(hours=2)).isoformat(),
          "reviewer_id":"security-reviewer-1","evidence_id":"restore-run-20261006",
        }
        out=m.validate_document(data,src,now)
        self.assertFalse(out["public_go"])
        self.assertFalse(out["production_postgresql_pitr_verified"])
        bad=dict(data,source_commit="b"*40)
        with self.assertRaises(ValueError):m.validate_document(bad,src,now)
        stale=dict(data,reviewed_at_utc=(now-timedelta(days=15)).isoformat())
        with self.assertRaises(ValueError):m.validate_document(stale,src,now)

    def test_r1022_recovery_attestation_fd_refuses_duplicate_keys_and_nonfinite(self):
        src="a"*40
        now=datetime.now(timezone.utc)
        data={"schema":"ipat.recovery-readiness.v1","source_commit":src,
              "provider_rescue_root_console_verified":True,
              "independent_full_host_restore_verified":True,
              "independent_backup_destination_verified":True,
              "reviewed_at_utc":now.isoformat(),
              "reviewer_id":"security-reviewer-1","evidence_id":"offhost-restore-1"}
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/"evidence.json"
            ordinary=json.dumps(data)
            p.write_text(ordinary)
            os.chmod(p,0o600)
            self.assertFalse(m.validate_file(p,src,os.geteuid())["public_go"])
            duplicate=ordinary.replace('"source_commit":', '"source_commit":"'+("b"*40)+'", "source_commit":',1)
            p.write_text(duplicate)
            with self.assertRaisesRegex(ValueError,"duplicate JSON key"):
                m.validate_file(p,src,os.geteuid())
            p.write_text(ordinary[:-1]+', "unsupported_metric": NaN}')
            with self.assertRaisesRegex(ValueError,"non-JSON numeric constant"):
                m.validate_file(p,src,os.geteuid())
            p.write_bytes(bytes([255])+ordinary.encode("utf-8"))
            with self.assertRaisesRegex(ValueError,"UTF-8 JSON invalid"):
                m.validate_file(p,src,os.geteuid())

    def test_r1022_recovery_attestation_rejects_symlink_hardlink_and_fifo(self):
        src="a"*40
        doc={"schema":"ipat.recovery-readiness.v1","source_commit":src,
             "provider_rescue_root_console_verified":True,
             "independent_full_host_restore_verified":True,
             "independent_backup_destination_verified":True,
             "reviewed_at_utc":datetime.now(timezone.utc).isoformat(),
             "reviewer_id":"security-reviewer-1","evidence_id":"offhost-restore-1"}
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/"valid.json"
            p.write_text(json.dumps(doc))
            os.chmod(p,0o600)
            link=Path(td)/"symlink.json"
            link.symlink_to(p)
            with self.assertRaises(ValueError):
                m.validate_file(link,src,os.geteuid())
            hardlink=Path(td)/"hardlink.json"
            os.link(p,hardlink)
            with self.assertRaisesRegex(ValueError,"single-link"):
                m.validate_file(p,src,os.geteuid())
            hardlink.unlink()
            fifo=Path(td)/"fifo"
            os.mkfifo(fifo,0o600)
            with self.assertRaisesRegex(ValueError,"single-link"):
                m.validate_file(fifo,src,os.geteuid())
            p.write_bytes(b"X"*8193)
            with self.assertRaisesRegex(ValueError,"size invalid"):
                m.validate_file(p,src,os.geteuid())

    def test_r1022_attestation_path_swap_after_open_fails_closed(self):
        src="a"*40
        doc={"schema":"ipat.recovery-readiness.v1","source_commit":src,
             "provider_rescue_root_console_verified":True,
             "independent_full_host_restore_verified":True,
             "independent_backup_destination_verified":True,
             "reviewed_at_utc":datetime.now(timezone.utc).isoformat(),
             "reviewer_id":"security-reviewer-1","evidence_id":"offhost-restore-1"}
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/"evidence.json"
            p.write_text(json.dumps(doc))
            os.chmod(p,0o600)
            original_open=os.open
            observed_flags=[]
            def swap_after_open(path,flags,*args,**kwargs):
                fd=original_open(path,flags,*args,**kwargs)
                observed_flags.append(flags)
                p.rename(Path(td)/"previous.json")
                p.write_text(json.dumps(dict(doc,source_commit="b"*40)))
                os.chmod(p,0o600)
                return fd
            with mock.patch.object(m.os,"open",side_effect=swap_after_open):
                with self.assertRaisesRegex(ValueError,"path changed during validation"):
                    m.validate_file(p,src,os.geteuid())
            self.assertEqual(len(observed_flags),1)
            self.assertTrue(observed_flags[0]&os.O_NOFOLLOW)
            self.assertTrue(observed_flags[0]&os.O_NONBLOCK)
            self.assertTrue(observed_flags[0]&os.O_CLOEXEC)

    def test_attestation_file_mode_and_owner_are_enforced(self):
        src="a"*40
        now=datetime.now(timezone.utc)
        data={"schema":"ipat.recovery-readiness.v1","source_commit":src,
          "provider_rescue_root_console_verified":True,
          "independent_full_host_restore_verified":True,
          "independent_backup_destination_verified":True,
          "reviewed_at_utc":now.isoformat(),"reviewer_id":"reviewer-1","evidence_id":"evidence-1"}
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/"att.json";p.write_text(json.dumps(data));os.chmod(p,0o600)
            out=m.validate_file(p,src,os.geteuid());self.assertFalse(out["public_go"])
            os.chmod(p,0o644)
            with self.assertRaises(ValueError):m.validate_file(p,src,os.geteuid())

if __name__=="__main__":unittest.main()
