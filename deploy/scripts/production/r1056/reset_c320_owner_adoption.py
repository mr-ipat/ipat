#!/usr/bin/env python3
"""One-time owner VPS *lab* C320 read-credential reset with reversible archive.

This NEVER connects to or writes to the physical OLT. Only a verified
closed-loop service window allows resetting the single fixed DEV-01 record.
Old encrypted credential/draft/registry are preserved in owner-private 0700
archive; new one-time bootstrap code remains in a 0600 file, never logs.
"""
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import stat
import subprocess
import sys
import tempfile
import time

BASE = Path("/home/openai/.local/share/ipat")
REGISTRY = BASE / "r961-device-inventory" / "devices.json"
SECRETS = BASE / "r945-connection"
ARCHIVE_DIR = BASE / "r1056-c320-readoption"
UNITS = ("ipat-r911-preview.service", "ipat-r945-connector.service")
PROTOCOL = "R1056_RESET_DEV01_OWNER_LAB_ONLY"
FILES = ("device.fernet", "device-draft.json")
MAX_REGISTRY = 512 * 1024
MAX_AUDIT = 2048


def check_dir(p, uid):
    s = p.lstat()
    if not stat.S_ISDIR(s.st_mode) or s.st_uid != uid or stat.S_IMODE(s.st_mode) != 0o700:
        raise RuntimeError("unsafe owner-private directory")


def check_file(p, uid, maximum):
    s = p.lstat()
    if not stat.S_ISREG(s.st_mode) or s.st_uid != uid or s.st_nlink != 1 or stat.S_IMODE(s.st_mode) != 0o600 or not 0 < s.st_size <= maximum:
        raise RuntimeError("unsafe owner-private file")


def atomic(path, content):
    fd, tmp = tempfile.mkstemp(prefix=".r1056-", dir=path.parent)
    try:
        os.fchmod(fd, 0o600)
        with os.fdopen(fd, "wb") as f:
            fd = -1
            f.write(content)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, path)
        d = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(d)
        finally:
            os.close(d)
    finally:
        if fd >= 0:
            os.close(fd)
        if os.path.lexists(tmp):
            os.unlink(tmp)


def updated_registry(raw):
    data = json.loads(raw)
    if not isinstance(data, dict) or not isinstance(data.get("audit"), list):
        raise RuntimeError("invalid inventory schema")
    if len(data["audit"]) >= MAX_AUDIT:
        raise RuntimeError("audit capacity exhausted")
    if not isinstance(data.get("devices"), list) or data.get("linked_revision") not in (1, 2):
        raise RuntimeError("unexpected linked C320 state; manual review required")
    if data.get("linked_c320_hidden_until_reenrollment") is True:
        raise RuntimeError("already reset")
    revision = data["linked_revision"] + 1
    data["linked_c320_label"] = None
    data["linked_c320_pop"] = None
    data["linked_c320_hidden_until_reenrollment"] = True
    data["linked_revision"] = revision
    data["audit"].append({
        "sequence": len(data["audit"]) + 1,
        "operation": "RESET_LINKED_READ_ADOPTION",
        "device_id": "DEV-01", "revision": revision
    })
    return (json.dumps(data, sort_keys=True, separators=(",", ":")) + "\n").encode(), data


def run(*args):
    return subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                          check=True, timeout=20)


def systemd(action, name):
    run("systemctl", "--user", action, name)


def do_reset():
    uid = os.getuid()
    if uid == 0 or os.geteuid() != uid:
        raise RuntimeError("root/elevated identity forbidden")
    if os.environ.get("IPAT_R1056_CONFIRM") != PROTOCOL:
        raise RuntimeError("owner confirmation missing")
    for p in (BASE, REGISTRY.parent, SECRETS):
        check_dir(p, uid)
    for p in (REGISTRY, SECRETS / "device.fernet",
              SECRETS / "device-draft.json", SECRETS / "envelope-key"):
        check_file(p, uid, MAX_REGISTRY)
    if any((SECRETS / n).exists() or (SECRETS / n).is_symlink()
           for n in ("bootstrap-token", "bootstrap-sha256")):
        raise RuntimeError("existing bootstrap token requires manual review")
    raw = REGISTRY.read_bytes()
    new_raw, data = updated_registry(raw)
    if len(new_raw) > MAX_REGISTRY:
        raise RuntimeError("inventory size limit reached")
    # Physical OLT has no provisioning channel in this operation.
    # Existing synthetic-only records are preserved, never wiped.
    if data.get("devices"):
        raise RuntimeError("other device metadata present; do not blanket-reset")
    if ARCHIVE_DIR.exists():
        check_dir(ARCHIVE_DIR, uid)
    else:
        ARCHIVE_DIR.mkdir(mode=0o700)
        check_dir(ARCHIVE_DIR, uid)
    tag = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime()) + "-" + secrets.token_hex(6)
    archive = ARCHIVE_DIR / tag
    archive.mkdir(mode=0o700)
    backup_names = ("devices.json", *FILES)
    try:
        for name, source in zip(backup_names, (REGISTRY, *(SECRETS / v for v in FILES))):
            dst = archive / name
            fd = os.open(dst, os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
            with os.fdopen(fd, "wb") as out:
                with source.open("rb") as reader:
                    shutil.copyfileobj(reader, out)
                out.flush()
                os.fsync(out.fileno())
            check_file(dst, uid, MAX_REGISTRY)
        check_dir(archive, uid)
    except Exception:
        shutil.rmtree(archive)
        raise
    # Only perform reversible mutation after both user services are stopped.
    stopped = False
    mutated = False
    try:
        stopped = True
        for service in UNITS:
            systemd("stop", service)
        # Reject a concurrent owner edit made after the initial snapshot.
        if REGISTRY.read_bytes() != raw:
            raise RuntimeError("concurrent registry change")
        token = secrets.token_urlsafe(32)
        atomic(REGISTRY, new_raw)
        mutated = True
        for name in FILES:
            (SECRETS / name).unlink()
        atomic(SECRETS / "bootstrap-token", (token + "\n").encode())
        atomic(SECRETS / "bootstrap-sha256", hashlib.sha256(token.encode()).digest())
        # Neither the token nor any old encrypted credential appears in stdout.
        for service in reversed(UNITS):
            systemd("start", service)
        print("R1056_C320_READ_ADOPTION_RESET_STAGED_REENROLL_REQUIRED")
        print("OLD_ENCRYPTED_CREDENTIAL_ARCHIVED_OWNER_ONLY")
        print("NO_DEVICE_COMMANDS_NO_PRODUCTION_TENANT_MUTATION")
        print("AUDIT_EVENT=RESET_LINKED_READ_ADOPTION")
        print("SECURE_OWNER_TOKEN_FILE=" + str(SECRETS / "bootstrap-token"))
        print("ARCHIVE_PATH=" + str(archive))
    except Exception:
        # Stop any partially restarted connector before restoring files.
        if stopped:
            for service in UNITS:
                try:
                    systemd("stop", service)
                except Exception:
                    pass
        if stopped and mutated:
            for name in ("bootstrap-token", "bootstrap-sha256"):
                (SECRETS / name).unlink(missing_ok=True)
            atomic(REGISTRY, (archive / "devices.json").read_bytes())
            for name in FILES:
                atomic(SECRETS / name, (archive / name).read_bytes())
        for service in reversed(UNITS):
            try:
                systemd("start", service)
            except Exception:
                pass
        raise


if __name__ == "__main__":
    try:
        do_reset()
    except Exception as exc:
        print("R1056_RESET_REFUSED_OR_ROLLED_BACK: " + type(exc).__name__, file=sys.stderr)
        sys.exit(4)
