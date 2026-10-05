#!/usr/bin/env python3
"""R9.91 read-only offsite Restic evidence gate.

Never accepts repository credentials on argv. It reads RESTIC_* only from the
process environment, runs no backup/forget/prune/unlock/restore, and NEVER
returns production GO. A separate independent restore is mandatory.
"""
from __future__ import annotations
import argparse, hashlib, ipaddress, json, os, socket, subprocess, sys
from datetime import datetime, timezone
from urllib.parse import urlsplit

ALLOWED_SCHEMES={"sftp","rest","s3"}
FORBIDDEN_RESTIC_WORDS={"backup","restore","forget","prune","unlock","init","migrate","rewrite","repair"}

def clean_text(v:str,max_len=160)->bool:
    return bool(v) and len(v)<=max_len and all(31<ord(c)<127 for c in v)

def remote_location(value:str)->bool:
    if not clean_text(value,300) or any(x in value for x in ("\n","\r","\t")):
        return False
    if value.startswith(("/",".","~","file:")):
        return False
    # SFTP Restic syntax may be sftp:user@host:/path. Reject password-like
    # userinfo, localhost and loopback/private literal destinations.
    if value.startswith("sftp:") and "://" not in value:
        tail=value[5:]
        if ":" not in tail:return False
        authority=tail.split(":",1)[0]
        if authority.count("@")>1:return False
        if "@" in authority and ":" in authority.rsplit("@",1)[0]:return False
        host=authority.rsplit("@",1)[-1].strip("[]")
    elif value.startswith("rest:http://") or value.startswith("rest:https://"):
        try:u=urlsplit(value[5:])
        except ValueError:return False
        if u.scheme not in {"http","https"} or u.username is not None or u.password is not None:return False
        host=u.hostname or ""
    elif value.startswith("s3:http://") or value.startswith("s3:https://"):
        try:u=urlsplit(value[3:])
        except ValueError:return False
        if u.scheme not in {"http","https"} or u.username is not None or u.password is not None:return False
        host=u.hostname or ""
    else:
        return False
    host=host.lower().rstrip(".")
    if host in {"","localhost","localhost.localdomain"}:return False
    try:
        ip=ipaddress.ip_address(host)
        if ip.is_loopback or ip.is_unspecified or ip.is_link_local or ip.is_private:
            return False
    except ValueError:
        if "." not in host:return False
    return True

def secret_environment_ok(env:dict[str,str])->tuple[bool,list[str]]:
    leaks=[]
    for key in ("RESTIC_PASSWORD","AWS_SECRET_ACCESS_KEY","AZURE_ACCOUNT_KEY","B2_ACCOUNT_KEY"):
        if env.get(key): leaks.append(key)
    password_file=env.get("RESTIC_PASSWORD_FILE","")
    repository=env.get("RESTIC_REPOSITORY","")
    ok=bool(repository and remote_location(repository) and password_file and os.path.isabs(password_file))
    if ok:
        try:
            st=os.stat(password_file)
            ok=st.st_uid==os.geteuid() and (st.st_mode & 0o077)==0 and st.st_size>0
        except OSError:ok=False
    return ok,leaks

def run_restic_readonly(tag:str,env:dict[str,str])->dict:
    if not clean_text(tag,80) or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.:" for c in tag):
        raise ValueError("invalid snapshot tag")
    ok,leaks=secret_environment_ok(env)
    if not ok or leaks:
        raise ValueError("external RESTIC_REPOSITORY/password-file boundary not satisfied")
    cmd=["restic","snapshots","--json","--no-lock","--tag",tag]
    assert not FORBIDDEN_RESTIC_WORDS.intersection(cmd)
    proc=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=30)
    if proc.returncode!=0:
        raise RuntimeError("restic readonly snapshot query failed")
    try:data=json.loads(proc.stdout)
    except json.JSONDecodeError as e:raise RuntimeError("restic returned invalid snapshot JSON") from e
    if not isinstance(data,list) or not data:raise RuntimeError("no matching production snapshot")
    candidates=[]
    for x in data:
        if not isinstance(x,dict):continue
        sid=str(x.get("id",""))
        when=str(x.get("time",""))
        tags=x.get("tags",[])
        if len(sid)<8 or tag not in tags:continue
        try:t=datetime.fromisoformat(when.replace("Z","+00:00")).astimezone(timezone.utc)
        except ValueError:continue
        candidates.append((t,sid))
    if not candidates:raise RuntimeError("no valid tagged production snapshot")
    t,sid=max(candidates)
    age=max(0,int((datetime.now(timezone.utc)-t).total_seconds()))
    return {
      "latest_snapshot_id_sha256":hashlib.sha256(sid.encode()).hexdigest(),
      "latest_snapshot_time_utc":t.isoformat(),
      "snapshot_age_seconds":age,
      "snapshot_count_matching":len(candidates),
    }

def report(args,env=os.environ)->dict:
    if args.active_failure_domain==args.backup_failure_domain:
        raise ValueError("backup failure domain must differ from active database failure domain")
    if not clean_text(args.active_failure_domain,80) or not clean_text(args.backup_failure_domain,80):
        raise ValueError("invalid failure-domain identifier")
    if not remote_location(args.repository_location):
        raise ValueError("repository location is not an eligible remote-only nonsecret identifier")
    repo_env=env.get("RESTIC_REPOSITORY","")
    if repo_env and repo_env!=args.repository_location:
        raise ValueError("reported repository does not match RESTIC_REPOSITORY")
    observed=run_restic_readonly(args.snapshot_tag,dict(env)) if args.query else None
    return {
      "schema":"ipat.r991.offsite-restic-evidence.v1",
      "observed_at_utc":datetime.now(timezone.utc).isoformat(),
      "active_failure_domain_sha256":hashlib.sha256(args.active_failure_domain.encode()).hexdigest(),
      "backup_failure_domain_sha256":hashlib.sha256(args.backup_failure_domain.encode()).hexdigest(),
      "repository_location_sha256":hashlib.sha256(args.repository_location.encode()).hexdigest(),
      "readonly_snapshot_observation":observed,
      "actual_independent_restore_required":True,
      "actual_pg_verifybackup_required":True,
      "actual_named_pitr_required":True,
      "measured_rpo_rto_required":True,
      "real_production_data_shape_required":True,
      "secrets_emitted":False,
      "repository_mutated":False,
      "public_go":False,
      "production_status":"BLOCKED",
    }

def main()->int:
    p=argparse.ArgumentParser()
    p.add_argument("--repository-location",required=True,help="Nonsecret remote Restic repository identifier; credentials forbidden")
    p.add_argument("--active-failure-domain",required=True)
    p.add_argument("--backup-failure-domain",required=True)
    p.add_argument("--snapshot-tag",default="ipat-production-pg18")
    p.add_argument("--query",action="store_true",help="Run only restic snapshots --json --no-lock using external env secrets")
    a=p.parse_args()
    try:r=report(a)
    except (ValueError,RuntimeError) as e:
        print(json.dumps({"schema":"ipat.r991.offsite-restic-evidence.v1","production_status":"BLOCKED","public_go":False,"error":str(e)},sort_keys=True))
        return 3
    print(json.dumps(r,indent=2,sort_keys=True))
    return 3
if __name__=="__main__":raise SystemExit(main())
