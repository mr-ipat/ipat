#!/usr/bin/env python3
"""Validate production PITR acceptance before any public-edge activation."""
from __future__ import annotations
import argparse, json, os, re, stat
from datetime import datetime, timezone, timedelta
from pathlib import Path

HEX64=re.compile(r"^[0-9a-f]{64}$")
SAFE=re.compile(r"^[A-Za-z0-9_.:@/-]{3,160}$")

def _load_secure(path:Path,uid:int=0)->dict:
    if not path.is_absolute() or path.is_symlink():
        raise ValueError("evidence path must be absolute regular non-symlink")
    st=path.stat()
    if not stat.S_ISREG(st.st_mode) or st.st_uid!=uid or stat.S_IMODE(st.st_mode)!=0o600 or st.st_nlink!=1:
        raise ValueError("evidence file must be owner-only 0600 single-link")
    if not 80 <= st.st_size <= 16384:
        raise ValueError("evidence size invalid")
    try:data=json.loads(path.read_text())
    except (OSError,json.JSONDecodeError) as e:raise ValueError("evidence JSON invalid") from e
    if not isinstance(data,dict):raise ValueError("evidence root must be object")
    return data

def validate_documents(foundation:dict,pitr:dict,now:datetime|None=None)->dict:
    if foundation.get("schema")!="ipat.production-foundation.v1":
        raise ValueError("foundation marker schema invalid")
    if foundation.get("application_services")!="DISABLED" or foundation.get("public_go") is not False:
        raise ValueError("foundation is not in pre-public disabled state")
    source=str(foundation.get("source_commit",""))
    if not re.fullmatch(r"[0-9a-f]{40}",source):
        raise ValueError("foundation source invalid")
    if pitr.get("schema")!="ipat.production-pitr-acceptance.v1":
        raise ValueError("PITR schema invalid")
    if pitr.get("foundation_source_commit")!=source:
        raise ValueError("PITR evidence does not match foundation source")
    for key in (
      "offsite_repository_verified","distinct_host_restore_verified","pg_verifybackup_passed",
      "named_pitr_passed","tenant_rls_validation_passed","streaming_standby_failover_verified",
      "fencing_verified",
    ):
        if pitr.get(key) is not True:raise ValueError(f"{key} is not verified")
    cluster=str(pitr.get("production_cluster_id_sha256",""))
    if not HEX64.fullmatch(cluster):raise ValueError("cluster fingerprint invalid")
    prepared=str(pitr.get("prepared_by",""));approved=str(pitr.get("approved_by",""));evidence=str(pitr.get("evidence_id",""))
    if not SAFE.fullmatch(prepared) or not SAFE.fullmatch(approved) or not SAFE.fullmatch(evidence) or prepared==approved:
        raise ValueError("maker/checker or evidence identifier invalid")
    for actual,target,name in (
      (pitr.get("rpo_seconds"),pitr.get("approved_rpo_seconds"),"RPO"),
      (pitr.get("rto_seconds"),pitr.get("approved_rto_seconds"),"RTO"),
    ):
        if not isinstance(actual,int) or isinstance(actual,bool) or actual<0:
            raise ValueError(f"{name} measurement invalid")
        if not isinstance(target,int) or isinstance(target,bool) or target<=0 or actual>target:
            raise ValueError(f"{name} exceeds approved target")
    t=datetime.fromisoformat(str(pitr.get("reviewed_at_utc","")).replace("Z","+00:00"))
    if t.tzinfo is None:raise ValueError("review time missing timezone")
    current=(now or datetime.now(timezone.utc)).astimezone(timezone.utc);t=t.astimezone(timezone.utc)
    if t>current+timedelta(minutes=5) or current-t>timedelta(days=7):
        raise ValueError("PITR acceptance stale or future-dated")
    return {
      "foundation_source_commit":source,
      "production_cluster_id_sha256":cluster,
      "rpo_seconds":pitr["rpo_seconds"],"rto_seconds":pitr["rto_seconds"],
      "evidence_id":evidence,"public_go":False,
      "human_mfa_browser_acceptance_required":True,
      "customer_domain_activation_required_separately":True,
    }

def validate_files(foundation:Path,pitr:Path,uid:int=0)->dict:
    return validate_documents(_load_secure(foundation,uid),_load_secure(pitr,uid))

def main()->int:
    p=argparse.ArgumentParser()
    p.add_argument("--foundation",type=Path,required=True)
    p.add_argument("--pitr",type=Path,required=True)
    a=p.parse_args()
    try:out=validate_files(a.foundation,a.pitr,0)
    except (OSError,ValueError) as e:
        print(json.dumps({"ok":False,"error":str(e),"public_go":False},sort_keys=True));return 4
    print(json.dumps({"ok":True,**out},sort_keys=True));return 0
if __name__=="__main__":raise SystemExit(main())
