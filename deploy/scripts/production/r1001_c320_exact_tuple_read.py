#!/usr/bin/env python3
"""One-time sanitized exact DEV-01 C320 card/firmware tuple evidence.

Uses ONLY the already-qualified fixed owner-private commands:
  show card
  show version-running
No config mode, ONU/subscriber commands, firmware writes, reboot, serial output,
dynamic host/user/port/command input, or service restart. Secrets never print.
"""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[2]
R945=ROOT/"scripts/lab/r945/persistent_c320_connector.py"
STATUS_SOCKET=Path("/home/openai/.local/share/ipat/r940-live-agent/live.sock")
OPTIN="IPAT_R1001_OWNER_APPROVES_EXISTING_FIXED_C320_TUPLE_READ"

class Denied(RuntimeError): pass

def load_r945():
    spec=importlib.util.spec_from_file_location("ipat_r1001_r945",R945)
    if spec is None or spec.loader is None:
        raise Denied("canonical connector source unavailable")
    mod=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod

def status_idle(sock_path=STATUS_SOCKET):
    if not sock_path.exists() or sock_path.is_symlink():
        raise Denied("existing private connector socket unavailable")
    s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
    s.settimeout(3)
    try:
        s.connect(str(sock_path))
        s.sendall(b"STATUS\n")
        data=b""
        while len(data)<=8192 and not data.endswith(b"\n"):
            part=s.recv(4096)
            if not part: break
            data+=part
    finally:
        s.close()
    if not 0<len(data)<=8192:
        raise Denied("connector status unavailable")
    try:
        value=json.loads(data)
    except Exception as exc:
        raise Denied("connector status invalid") from exc
    if value.get("mode")!="OWNER_SUPERVISED_REAL_C320_READ_ONLY" \
       or value.get("persistent_connector") is not True \
       or value.get("credentials_enrolled") is not True \
       or value.get("read_in_progress") is not False:
        raise Denied("connector is not enrolled and idle")
    return True

def safe_ascii(token: bytes, maxlen=96):
    if not 1<=len(token)<=maxlen or not re.fullmatch(rb"[A-Za-z0-9._/+:-]+",token):
        raise Denied("unsafe tuple token")
    return token.decode("ascii")

def reject_controls(raw: bytes):
    if not raw or len(raw)>32768 or any(b in raw for b in (b"\x00",b"\x1b")):
        raise Denied("unsafe transcript payload")
    for byte in raw:
        if byte<32 and byte not in (9,10,13):
            raise Denied("unsafe transcript control character")

def parse_cards(raw: bytes):
    reject_controls(raw)
    if not all(x in raw for x in (b"Rack",b"Shelf",b"Slot",b"INSERVICE")):
        raise Denied("card table shape not recognized")
    rows=[]
    seen=set()
    for line in raw.replace(b"\r",b"").splitlines():
        parts=line.split()
        if parts[-1:] != [b"INSERVICE"]:
            continue
        if len(parts)<9 or any(not item.isdigit() for item in parts[:3]):
            raise Denied("card row shape rejected")
        rack,shelf,slot=map(lambda x:int(x),parts[:3])
        if not (1<=rack<=16 and 1<=shelf<=16 and 1<=slot<=64):
            raise Denied("card location out of bound")
        loc=f"{rack}/{shelf}/{slot}"
        if loc in seen: raise Denied("duplicate in-service card location")
        seen.add(loc)
        rows.append({
            "location":loc,
            "configured_type":safe_ascii(parts[3],32),
            "real_type":safe_ascii(parts[4],32),
            "ports":int(parts[5]) if parts[5].isdigit() else None,
            "hardware_version":safe_ascii(parts[6],32),
            "software_version":safe_ascii(parts[7],48),
            "status":"INSERVICE",
        })
    if not 1<=len(rows)<=22 or any(r["ports"] is None or not 0<=r["ports"]<=128 for r in rows):
        raise Denied("in-service card inventory rejected")
    return rows

def parse_versions(raw: bytes):
    reject_controls(raw)
    if not all(x in raw for x in (b"PhyLoc",b"FileType",b"VerType")):
        raise Denied("firmware table shape not recognized")
    rows=[]
    seen=set()
    for line in raw.replace(b"\r",b"").splitlines():
        if not re.match(rb"^\s*1/1/\d+\s+",line):
            continue
        p=line.split()
        if len(p)!=7 or p[2] not in (b"MVR",b"FW",b"BT"):
            raise Denied("firmware row shape rejected")
        loc=safe_ascii(p[0],24)
        if not re.fullmatch(r"\d+/\d+/\d+",loc):
            raise Denied("firmware location rejected")
        key=(loc,p[1],p[2],p[3])
        if key in seen: raise Denied("duplicate firmware row")
        seen.add(key)
        rows.append({
            "location":loc,
            "file_type":safe_ascii(p[1],32),
            "version_type":safe_ascii(p[2],8),
            "version_tag":safe_ascii(p[3],64),
            "build_date":safe_ascii(p[4],16),
            "build_time":safe_ascii(p[5],16),
            "version_length":int(p[6]) if p[6].isdigit() else None,
        })
    if not 1<=len(rows)<=66 or any(r["version_length"] is None or r["version_length"]<=0 for r in rows):
        raise Denied("firmware inventory rejected")
    return rows

def mapping_diagnostics(cards,versions):
    byloc={}
    for v in versions: byloc.setdefault(v["location"],[]).append(v)
    out=[]
    for c in cards:
        mvr=[v for v in byloc.get(c["location"],[]) if v["version_type"]=="MVR"]
        out.append({
            "location":c["location"],
            "configured_type":c["configured_type"],
            "real_type":c["real_type"],
            "hardware_version":c["hardware_version"],
            "software_version":c["software_version"],
            "mvr_file_types":[v["file_type"] for v in mvr],
            "mvr_version_tags":[v["version_tag"] for v in mvr],
            "mvr_builds":[v["build_date"]+"T"+v["build_time"] for v in mvr],
        })
    return out

def correlate(cards,versions):
    byloc={}
    for v in versions: byloc.setdefault(v["location"],[]).append(v)
    out=[]
    for c in cards:
        mvr=[v for v in byloc.get(c["location"],[]) if v["version_type"]=="MVR"]
        if len(mvr)!=1:
            raise Denied("exact card to MVR mapping incomplete")
        v=mvr[0]
        if v["file_type"] not in (c["configured_type"],c["real_type"]):
            raise Denied("card and MVR file type mismatch")
        out.append({
            **c,
            "running_file_type":v["file_type"],
            "running_version_tag":v["version_tag"],
            "running_build_date":v["build_date"],
            "running_build_time":v["build_time"],
        })
    return out

def fixed_read(reader_mod,password):
    import pexpect
    m=reader_mod
    ssh=["ssh","-F","/dev/null","-tt","-p",str(m.PORT),
         "-o","HostKeyAlgorithms=ssh-rsa","-o","Ciphers=aes128-cbc",
         "-o","KexAlgorithms=diffie-hellman-group14-sha256",
         "-o","StrictHostKeyChecking=yes","-o",f"UserKnownHostsFile={m.PIN}",
         "-o","GlobalKnownHostsFile=/dev/null","-o","NumberOfPasswordPrompts=1",
         "-o","PreferredAuthentications=password","-o","PubkeyAuthentication=no",
         "-o","ProxyCommand=none","-o","ClearAllForwardings=yes",
         "-o","ConnectionAttempts=1","-o","ConnectTimeout=8",f"{m.USER}@{m.HOST}"]
    c=None
    try:
        c=pexpect.spawn(ssh[0],ssh[1:],encoding=None,timeout=12,echo=False,maxread=4096)
        if c.expect([rb"(?i)password:\s*$",pexpect.EOF,pexpect.TIMEOUT])!=0:
            raise Denied("private SSH challenge unavailable")
        c.sendline(password.encode())
        if c.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=10)!=0:
            raise Denied("authenticated prompt unavailable")
        raw={}
        for command in ("show card","show version-running"):
            c.sendline(command.encode())
            if c.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=15)!=0:
                raise Denied("fixed read timed out")
            raw[command]=m.bounded(c.before,command)
        return raw
    finally:
        if c is not None:
            try:
                if c.isalive():
                    c.sendline(b"exit")
                    c.expect(pexpect.EOF,timeout=2)
            except Exception:
                c.close(force=True)

def collect():
    if os.geteuid()==0:
        raise Denied("must run as nonroot owner")
    if os.getenv(OPTIN)!="YES":
        raise Denied("explicit fixed-read approval missing")
    status_idle()
    r945=load_r945()
    reader,reader_module=r945.module()
    password=r945.get_password()
    try:
        raw=fixed_read(reader_module,password)
    finally:
        password=None
    cards=parse_cards(raw["show card"])
    versions=parse_versions(raw["show version-running"])
    observed=dt.datetime.now(dt.timezone.utc).isoformat()
    diagnostics=mapping_diagnostics(cards,versions)
    try:
        correlated=correlate(cards,versions)
    except Denied:
        return {
            "schema":"ipat.c320.exact-read-tuple.v1",
            "target":"DEV-01","vendor":"ZTE","chassis_family":"C320",
            "evidence_type":"physical","observed_at_utc":observed,
            "qualified_commands":["show card","show version-running"],
            "in_service_cards":len(cards),"firmware_rows":len(versions),
            "mapping_status":"BLOCKED_EXACT_ALIAS_REQUIRED",
            "card_mvr_candidates":diagnostics,
            "firmware_inventory":versions,
            "serials_returned":False,"subscriber_data_returned":False,
            "physical_writes":0,"firmware_write_enabled":False,
            "device_adopted":False,"validation_status":"partial",
            "limitations":["exact card-to-MVR alias unresolved; no wildcard accepted"],
        }
    canonical=json.dumps({"cards":cards,"versions":versions},sort_keys=True,separators=(",",":")).encode()
    return {
        "schema":"ipat.c320.exact-read-tuple.v1",
        "target":"DEV-01","vendor":"ZTE","chassis_family":"C320",
        "evidence_type":"physical","observed_at_utc":observed,
        "qualified_commands":["show card","show version-running"],
        "in_service_cards":len(cards),"firmware_rows":len(versions),
        "card_running_mvr_map":correlated,
        "firmware_inventory":versions,
        "sanitized_tuple_sha256":hashlib.sha256(canonical).hexdigest(),
        "serials_returned":False,"subscriber_data_returned":False,
        "physical_writes":0,"firmware_write_enabled":False,
        "device_adopted":False,
        "validation_status":"partial",
        "limitations":[
            "read-only fixed CLI evidence only",
            "SNMP/PON/ONU/alarm/native restore/write not qualified",
            "vendor-wide compatibility not implied",
        ],
    }

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--collect",action="store_true",help="run the two already-qualified fixed reads")
    a=p.parse_args()
    if not a.collect:
        p.error("--collect required")
    try:
        print(json.dumps(collect(),sort_keys=True,indent=2))
        return 0
    except (Denied,OSError,ValueError,UnicodeError,subprocess.SubprocessError) as exc:
        print("R1001_C320_TUPLE_READ_DENIED:"+str(exc),file=sys.stderr)
        return 4

if __name__=="__main__":
    raise SystemExit(main())
