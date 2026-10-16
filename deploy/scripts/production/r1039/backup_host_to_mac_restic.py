#!/usr/bin/env python3
"""Owner-authorized non-mutating Ubuntu host stream to a separate encrypted Mac.

Not a full-host restore certificate: recovery requires a separate booted VM/VPS.
The root sudo credential is read from a TTY once, never stored or logged.
"""
from __future__ import annotations
import argparse
import termios
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import threading

HOST = "ipat-lab"
KEYCHAIN_PASSWORD_COMMAND = (
    "/usr/bin/security find-generic-password -a ipat-lab-backup "
    "-s id.ipat.lab.restic.backup.v1 -w"
)
# Rebuildable/volatile data only; no persistent business records excluded.
ROOT_EXCLUDES = (
    "./dev", "./proc", "./sys", "./run", "./tmp", "./mnt", "./media",
    "./lost+found", "./swapfile", "./swap.img",
    "./home/openai/.cache", "./home/openai/ipat-build-*",
    "./var/cache", "./var/tmp", "./var/log/journal",
    "./home/openai/.local/share/Trash",
)
EXTRA_FILESYSTEM = ("boot/efi",)


def restic_env(repo: Path) -> dict[str, str]:
    if not repo.is_absolute() or repo.is_symlink() or not (repo / "config").is_file():
        raise ValueError("existing local encrypted Restic repository required")
    if not str(repo).startswith(str(Path.home() / "Backups/IPAT/")):
        raise ValueError("repository must be in dedicated Mac backup tree")
    if os.geteuid() == 0 or sys.platform != "darwin":
        raise ValueError("Mac nonroot only")
    for key in ("RESTIC_PASSWORD", "RESTIC_PASSWORD_FILE", "DOCKER_HOST", "PGPASSWORD"):
        if os.environ.get(key):
            raise ValueError(f"unexpected security-sensitive environment key: {key}")
    env = os.environ.copy()
    env["RESTIC_REPOSITORY"] = str(repo)
    env["RESTIC_PASSWORD_COMMAND"] = KEYCHAIN_PASSWORD_COMMAND
    return env


def tar_args(rootfs: bool) -> list[str]:
    args = ["sudo", "-S", "--", "/usr/bin/tar", "-c",
            "--acls", "--xattrs", "--numeric-owner", "-f", "-"]
    if rootfs:
        args.append("--one-file-system")
        args.extend("--exclude=" + path for path in ROOT_EXCLUDES)
        args += ["-C", "/", "."]
    else:
        args += ["-C", "/boot/efi", "."]
    return args


def stream_to_restic(passphrase: str, env: dict[str, str], *, rootfs: bool) -> dict:
    category = "rootfs" if rootfs else "efi"
    stdin_name = f"ipat-r1039-{category}.tar"
    ssh = subprocess.Popen(
        ["ssh", "-T", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=yes",
         "-o", "ConnectTimeout=12", HOST, *tar_args(rootfs)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    # Capture stderr in RAM only to avoid blocked pipes. Never print raw
    # paths, sensitive CLI transcripts, or credentials to the operator.
    error_bytes = bytearray()
    def drain_stderr():
        assert ssh.stderr is not None
        while chunk := ssh.stderr.read(8192):
            if len(error_bytes) < 32768:
                error_bytes.extend(chunk[:32768 - len(error_bytes)])
    thread = threading.Thread(target=drain_stderr, daemon=True)
    thread.start()
    restic = None
    digest = hashlib.sha256()
    count = 0
    try:
        assert ssh.stdin is not None and ssh.stdout is not None
        ssh.stdin.write(passphrase.encode("utf-8") + b"\n")
        ssh.stdin.flush()
        ssh.stdin.close()
        restic = subprocess.Popen(
            ["restic", "backup", "--stdin", "--stdin-filename", stdin_name,
             "--tag", "ipat-r1039-host-backup"],
            env=env, stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        assert restic.stdin is not None
        while chunk := ssh.stdout.read(1024 * 1024):
            digest.update(chunk)
            count += len(chunk)
            restic.stdin.write(chunk)
        restic.stdin.close()
        tar_status = ssh.wait(timeout=90)
        thread.join(timeout=5)
        # communicate() may flush closed stdin: clear it explicitly.
        restic.stdin = None
        stdout, stderr = restic.communicate(timeout=300)
        if tar_status or restic.returncode or not count:
            errors = error_bytes.decode("utf-8", errors="replace").lower()
            if "password" in errors or "authentication" in errors:
                reason = "REMOTE_SUDO_AUTHENTICATION"
            elif "terminal" in errors or "tty" in errors:
                reason = "REMOTE_TTY_REQUIREMENT"
            elif "tar:" in errors:
                reason = "REMOTE_TAR_ERROR"
            elif "sudo:" in errors:
                reason = "REMOTE_SUDO_ERROR"
            else:
                reason = "REMOTE_UNCLASSIFIED"
            raise ValueError(
                f"{category} backup incomplete: tar_exit={tar_status} restic_exit={restic.returncode} bytes={count} reason={reason}"
            )
        return {
            "component": category, "tar_stream_sha256": digest.hexdigest(),
            "tar_bytes": count, "restic_status": "SNAPSHOT_WRITTEN",
            # Snapshot IDs extracted separately from encrypted index. Never
            # mistake a successful upload for a bootable system recovery.
            "independent_boot_restore_verified": False,
        }
    finally:
        if ssh.poll() is None:
            ssh.kill()
        if restic is not None and restic.poll() is None:
            restic.kill()


def read_local_sudo_secret() -> str:
    """Read remote operator input through stdin, with TTY echo disabled.

    Do not use getpass() here: remote commander stdin may be separate from
    /dev/tty, so getpass can hang waiting on an unrelated terminal device.
    """
    fd = sys.stdin.fileno()
    prior = None
    try:
        if os.isatty(fd):
            prior = termios.tcgetattr(fd)
            hidden = termios.tcgetattr(fd)
            hidden[3] &= ~termios.ECHO
            termios.tcsetattr(fd, termios.TCSADRAIN, hidden)
        print("R1039_AWAITING_TEMPORARY_VPS_SUDO_INPUT", flush=True)
        secret = sys.stdin.readline().rstrip("\r\n")
        return secret
    finally:
        if prior is not None:
            termios.tcsetattr(fd, termios.TCSADRAIN, prior)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if not args.execute:
        parser.error("explicit --execute required")
    try:
        env = restic_env(args.repo)
        sudo_password = read_local_sudo_secret()
        if not sudo_password:
            raise ValueError("local sudo input required")
        results = []
        for rootfs in (True, False):
            results.append(stream_to_restic(sudo_password, env, rootfs=rootfs))
        sudo_password = ""
        print(json.dumps({
            "schema": "ipat.r1039.host-encrypted-stream.v1",
            "results": results, "live_production_db": "NOT_TOUCHED",
            "independent_boot_restore_verified": False,
            "root_owned_recovery_attestation_created": False,
            "public_go": False,
        }, sort_keys=True))
        return 0
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        detail = str(error) if str(error).startswith(
            ("rootfs backup incomplete:", "efi backup incomplete:")
        ) else "OTHER_REDACTED"
        print(json.dumps({"ok":False, "failure_class":type(error).__name__,
                          "safe_status":detail, "public_go":False}))
        return 4


if __name__ == "__main__":
    sys.exit(main())
