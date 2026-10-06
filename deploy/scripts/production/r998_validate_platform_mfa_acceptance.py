#!/usr/bin/env python3
"""Validate real Platform Owner browser/MFA acceptance before any customer activation."""
from __future__ import annotations
import argparse,json,ipaddress,re,stat
from datetime import datetime,timezone,timedelta
from pathlib import Path
SAFE=re.compile(r"^[A-Za-z0-9_.:@/-]{3,160}$")

def secure_json(path:Path,uid:int=0)->dict:
    if not path.is_absolute() or path.is_symlink():raise ValueError("evidence path must be absolute non-symlink")
    st=path.stat()
    if not stat.S_ISREG(st.st_mode) or st.st_uid!=uid or stat.S_IMODE(st.st_mode)!=0o600 or st.st_nlink!=1:
        raise ValueError("evidence file must be owner-only 0600 single-link")
    if not 80<=st.st_size<=16384:raise ValueError("evidence size invalid")
    try:data=json.loads(path.read_text())
    except (OSError,json.JSONDecodeError) as e:raise ValueError("invalid JSON") from e
    if not isinstance(data,dict):raise ValueError("evidence root must be object")
    return data

def validate(edge:dict,acceptance:dict,expected_ip:str,now:datetime|None=None)->dict:
    try:ip=str(ipaddress.IPv4Address(expected_ip))
    except ValueError as e:raise ValueError("invalid expected IPv4") from e
    if edge.get("schema")!="ipat.platform-edge.v1" or edge.get("public_ipv4")!=ip:
        raise ValueError("Platform edge marker mismatch")
    if edge.get("platform_https")!="ACTIVE_CA_TRUSTED_SHORTLIVED_IP_CERT" or edge.get("platform_runtime")!="ACTIVE":
        raise ValueError("Platform edge is not active")
    if acceptance.get("schema")!="ipat.platform-human-mfa-acceptance.v1" or acceptance.get("platform_public_ipv4")!=ip:
        raise ValueError("MFA acceptance marker mismatch")
    for key in ("real_human_mfa_browser_login_verified","logout_revocation_verified",
                "wrong_host_replay_denied","stale_session_denied"):
        if acceptance.get(key) is not True:raise ValueError(f"{key} not accepted")
    maker=str(acceptance.get("prepared_by",""));checker=str(acceptance.get("approved_by",""))
    evid=str(acceptance.get("evidence_id",""))
    if not SAFE.fullmatch(maker) or not SAFE.fullmatch(checker) or not SAFE.fullmatch(evid) or maker==checker:
        raise ValueError("invalid maker/checker evidence")
    t=datetime.fromisoformat(str(acceptance.get("reviewed_at_utc","")).replace("Z","+00:00"))
    if t.tzinfo is None:raise ValueError("review time missing timezone")
    current=(now or datetime.now(timezone.utc)).astimezone(timezone.utc);t=t.astimezone(timezone.utc)
    if t>current+timedelta(minutes=5) or current-t>timedelta(days=7):
        raise ValueError("Platform MFA acceptance stale or future-dated")
    return {"platform_public_ipv4":ip,"evidence_id":evid,
            "customer_activation_allowed":True,"customer_release_go":False}

def main()->int:
    p=argparse.ArgumentParser();p.add_argument("--edge",type=Path,required=True);p.add_argument("--acceptance",type=Path,required=True);p.add_argument("--ip",required=True);a=p.parse_args()
    try:out=validate(secure_json(a.edge),secure_json(a.acceptance),a.ip)
    except (OSError,ValueError) as e:
        print(json.dumps({"ok":False,"error":str(e),"customer_release_go":False},sort_keys=True));return 4
    print(json.dumps({"ok":True,**out},sort_keys=True));return 0
if __name__=="__main__":raise SystemExit(main())
