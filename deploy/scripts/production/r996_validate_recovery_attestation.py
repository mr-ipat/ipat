#!/usr/bin/env python3
"""Validate a nonsecret first-install recovery readiness attestation.

This is a deployment gate, not independent proof by itself. The referenced
evidence must already have been reviewed outside the production host.
"""
from __future__ import annotations
import argparse, json, os, re, stat
from datetime import datetime, timezone, timedelta
from pathlib import Path

SAFE=re.compile(r"^[A-Za-z0-9_.:@/-]{3,160}$")
SHA=re.compile(r"^[0-9a-f]{40}$")

def _time(value: str)->datetime:
    t=datetime.fromisoformat(value.replace("Z","+00:00"))
    if t.tzinfo is None:
        raise ValueError("reviewed_at_utc must include timezone")
    return t.astimezone(timezone.utc)

def validate_document(data:dict, expected_source:str, now:datetime|None=None)->dict:
    if data.get("schema")!="ipat.recovery-readiness.v1":
        raise ValueError("unexpected schema")
    if not SHA.fullmatch(expected_source or ""):
        raise ValueError("invalid expected source commit")
    if data.get("source_commit")!=expected_source:
        raise ValueError("attestation source commit mismatch")
    for key in (
        "provider_rescue_root_console_verified",
        "independent_full_host_restore_verified",
        "independent_backup_destination_verified",
    ):
        if data.get(key) is not True:
            raise ValueError(f"{key} is not verified")
    reviewer=str(data.get("reviewer_id",""))
    evidence=str(data.get("evidence_id",""))
    if not SAFE.fullmatch(reviewer) or not SAFE.fullmatch(evidence):
        raise ValueError("reviewer/evidence identifier invalid")
    current=(now or datetime.now(timezone.utc)).astimezone(timezone.utc)
    reviewed=_time(str(data.get("reviewed_at_utc","")))
    if reviewed>current+timedelta(minutes=5) or current-reviewed>timedelta(days=14):
        raise ValueError("recovery review is stale or future-dated")
    return {
        "schema":data["schema"],
        "source_commit":expected_source,
        "reviewed_at_utc":reviewed.isoformat(),
        "reviewer_id_sha256_required_for_public_report":True,
        "evidence_id":evidence,
        "provider_rescue_root_console_verified":True,
        "independent_full_host_restore_verified":True,
        "independent_backup_destination_verified":True,
        "production_postgresql_pitr_verified":False,
        "public_go":False,
    }

def _reject_duplicate_keys(pairs:list[tuple[str,object]])->dict:
    """Refuse ambiguous evidence accepted differently by independent reviewers."""
    out={}
    for key,value in pairs:
        if key in out:
            raise ValueError("attestation contains duplicate JSON key")
        out[key]=value
    return out

def _reject_nonfinite(value:str)->None:
    raise ValueError("attestation contains non-JSON numeric constant")

def validate_file(path:Path, expected_source:str, required_uid:int=0)->dict:
    """Inspect and read ONE already-opened inode, never stat then reopen its path.

    On Ubuntu O_NOFOLLOW excludes final-component symlink traversal, O_NONBLOCK
    prevents hanging on a malicious FIFO, and bounded fd reads avoid OOM.
    R9.96's root-only input custody/review is still an independent requirement.
    """
    if not path.is_absolute():
        raise ValueError("attestation must have an absolute path")
    if not all(hasattr(os,key) for key in ("O_NOFOLLOW","O_CLOEXEC","O_NONBLOCK")):
        raise ValueError("required Linux no-follow open flags unavailable")
    flags=os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC|os.O_NONBLOCK
    try:
        fd=os.open(path,flags)
    except OSError as e:
        raise ValueError("attestation cannot be opened without following symlinks") from e
    try:
        with os.fdopen(fd,"rb") as stream:
            st=os.fstat(stream.fileno())
            if (not stat.S_ISREG(st.st_mode) or st.st_uid!=required_uid
                    or stat.S_IMODE(st.st_mode)!=0o600 or st.st_nlink!=1):
                raise ValueError("attestation must be owner-only 0600 single-link")
            if not 80<=st.st_size<=8192:
                raise ValueError("attestation size invalid")
            raw=stream.read(st.st_size+1)
            after=os.fstat(stream.fileno())
    except OSError as e:
        raise ValueError("attestation file-descriptor read failed") from e
    if (len(raw)!=st.st_size or (st.st_dev,st.st_ino,st.st_size,st.st_mtime_ns,st.st_ctime_ns)
            !=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns)):
        raise ValueError("attestation changed during validation")
    try:
        named=os.stat(path,follow_symlinks=False)
    except OSError as e:
        raise ValueError("attestation path changed during validation") from e
    if (named.st_dev,named.st_ino)!=(st.st_dev,st.st_ino):
        raise ValueError("attestation path changed during validation")
    try:
        data=json.loads(raw.decode("utf-8"),object_pairs_hook=_reject_duplicate_keys,
                        parse_constant=_reject_nonfinite)
    except (UnicodeError,json.JSONDecodeError) as e:
        raise ValueError("attestation UTF-8 JSON invalid") from e
    if not isinstance(data,dict):
        raise ValueError("attestation root must be object")
    return validate_document(data,expected_source)

def main()->int:
    p=argparse.ArgumentParser()
    p.add_argument("--path",required=True,type=Path)
    p.add_argument("--expected-source",required=True)
    a=p.parse_args()
    try:
        out=validate_file(a.path,a.expected_source,0)
    except (OSError,ValueError) as e:
        print(json.dumps({"ok":False,"error":str(e),"public_go":False},sort_keys=True))
        return 4
    print(json.dumps({"ok":True,**out},sort_keys=True))
    return 0

if __name__=="__main__":
    raise SystemExit(main())
