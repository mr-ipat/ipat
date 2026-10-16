#!/usr/bin/env python3
"""Constrained owner risk exception for LOCAL-ONLY first install, NOT host restore proof."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone, timedelta
import json
import os
from pathlib import Path
import re
import stat
import uuid

SHA40 = re.compile(r"^[0-9a-f]{40}$")
SHA64 = re.compile(r"^[0-9a-f]{64}$")
HEXID = re.compile(r"^[0-9a-f]{8}$")
ACK = "ACCEPT_UNTESTED_INDEPENDENT_BOOT_RESTORE_RISK_FOR_LOCAL_DB_FOUNDATION_ONLY"
DECISION = "FIRST_INSTALL_LOCAL_SOCKET_POSTGRES_ONLY_NO_PUBLIC_INGRESS"

def _pairs(values):
    result = {}
    for key, value in values:
        if key in result:
            raise ValueError("duplicate risk acceptance JSON key")
        result[key] = value
    return result

def _constant(value):
    raise ValueError("invalid nonfinite risk acceptance value")

def validate_document(data: dict, expected_commit: str, now: datetime | None = None) -> dict:
    if not isinstance(data, dict) or data.get("schema") != "ipat.owner-risk-acceptance.v1":
        raise ValueError("wrong recovery risk document schema")
    if not SHA40.fullmatch(expected_commit) or data.get("source_commit") != expected_commit:
        raise ValueError("risk acceptance exact source mismatch")
    if data.get("decision") != DECISION or data.get("owner_acknowledgement") != ACK:
        raise ValueError("missing explicit owner risk acceptance")
    if data.get("independent_full_host_boot_restore_verified") is not False:
        raise ValueError("recovery test must be explicitly represented as unverified")
    if data.get("public_edge_activation_authorized") is not False:
        raise ValueError("recovery exception cannot authorize public ingress")
    if data.get("production_pitr_ha_verified") is not False:
        raise ValueError("exception cannot claim tested production PITR or HA")
    if data.get("encrypted_restic_read_data_verified") is not True or data.get("both_archives_tar_readable") is not True:
        raise ValueError("reviewed encrypted archive integrity proof required")
    backups = data.get("backup_snapshot_ids")
    if not isinstance(backups, dict) or set(backups) != {"rootfs", "efi"}:
        raise ValueError("rootfs and EFI backup snapshot references required")
    for name in ("rootfs", "efi"):
        if not isinstance(backups[name], str) or not HEXID.fullmatch(backups[name]):
            raise ValueError("invalid backup snapshot identifier")
    if backups["rootfs"] == backups["efi"]:
        raise ValueError("rootfs and EFI cannot have same snapshot")
    try:
        uuid_value = uuid.UUID(str(data.get("approval_id", "")))
    except (ValueError, AttributeError) as error:
        raise ValueError("unique owner approval id required") from error
    if str(uuid_value) != data["approval_id"]:
        raise ValueError("approval UUID must be canonical")
    if data.get("approved_by_role") != "product_owner":
        raise ValueError("explicit product owner role required")
    try:
        approved = datetime.fromisoformat(str(data.get("approved_at_utc", "")).replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError("invalid owner approval time") from error
    if approved.tzinfo is None:
        raise ValueError("approval timestamp must carry UTC offset")
    approved = approved.astimezone(timezone.utc)
    current = (now or datetime.now(timezone.utc)).astimezone(timezone.utc)
    if approved > current + timedelta(minutes=5) or current - approved > timedelta(hours=24):
        raise ValueError("owner risk acceptance expired or future dated")
    return {
        "schema": data["schema"],
        "source_commit": expected_commit,
        "risk_acceptance_id": str(uuid_value),
        "recovery_state": "OWNER_RISK_ACCEPTED_NOT_RESTORED_ON_INDEPENDENT_VM",
        "public_go": False,
        "local_foundation_only": True,
    }

def read_secure(path: Path, uid: int = 0, trusted_parent: bool = True) -> dict:
    if not path.is_absolute():
        raise ValueError("root-owned risk evidence must use absolute path")
    if trusted_parent:
        ancestor = path.parent
        while True:
            s = os.lstat(ancestor)
            if not stat.S_ISDIR(s.st_mode) or s.st_uid != uid or stat.S_IMODE(s.st_mode) & 0o022:
                raise ValueError("unsafe risk document ancestor")
            if ancestor == ancestor.parent:
                break
            ancestor = ancestor.parent
    flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    fd = os.open(path, flags)
    with os.fdopen(fd, "rb") as f:
        before = os.fstat(f.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_uid != uid or stat.S_IMODE(before.st_mode) != 0o600 or before.st_nlink != 1:
            raise ValueError("risk document requires root-only 0600 single-link regular file")
        if not 200 <= before.st_size <= 8192:
            raise ValueError("risk document size invalid")
        payload = f.read(before.st_size + 1)
        after = os.fstat(f.fileno())
    if len(payload) != before.st_size or (before.st_dev, before.st_ino, before.st_size,
      before.st_mtime_ns, before.st_ctime_ns) != (after.st_dev, after.st_ino, after.st_size,
      after.st_mtime_ns, after.st_ctime_ns):
        raise ValueError("risk document changed while read")
    named = os.stat(path, follow_symlinks=False)
    if (named.st_dev, named.st_ino) != (before.st_dev, before.st_ino):
        raise ValueError("risk document path substituted")
    data = json.loads(payload.decode("utf-8"), object_pairs_hook=_pairs, parse_constant=_constant)
    return data

def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--path", required=True, type=Path)
    p.add_argument("--expected-source", required=True)
    a = p.parse_args()
    try:
        accepted = validate_document(read_secure(a.path), a.expected_source)
    except (OSError, ValueError, UnicodeError, json.JSONDecodeError) as error:
        print(json.dumps({"ok": False, "reason": str(error), "public_go": False}))
        return 4
    print(json.dumps({"ok": True, **accepted}, sort_keys=True))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
