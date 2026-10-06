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

def validate_file(path:Path, expected_source:str, required_uid:int=0)->dict:
    if not path.is_absolute() or path.is_symlink():
        raise ValueError("attestation must be an absolute regular non-symlink file")
    st=path.stat()
    if not stat.S_ISREG(st.st_mode) or st.st_uid!=required_uid or stat.S_IMODE(st.st_mode)!=0o600 or st.st_nlink!=1:
        raise ValueError("attestation must be owner-only 0600 single-link")
    if st.st_size<80 or st.st_size>8192:
        raise ValueError("attestation size invalid")
    try:
        data=json.loads(path.read_text())
    except (OSError,json.JSONDecodeError) as e:
        raise ValueError("attestation JSON invalid") from e
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
