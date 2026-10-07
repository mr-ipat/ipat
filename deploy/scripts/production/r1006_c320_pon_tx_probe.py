#!/usr/bin/env python3
"""Bounded owner-private DEV-01 OLT-PON transmit optical read.

Exactly one fixed, read-only command is available:
  show pon power olt-tx gpon-olt_1/1/1

The command is documented in the ZXA10 C300/C320 command family as an OLT
transmit optical-power query. Actual DEV-01 behavior remains authoritative.
This probe never returns raw CLI, ONU IDs, serials, subscriber data, config,
credentials, or any write/control operation.
"""
from __future__ import annotations
import argparse
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import socket
import sys

ROOT=Path(__file__).resolve().parents[2]
R945=ROOT/"scripts/lab/r945/persistent_c320_connector.py"
STATUS_SOCKET=Path("/home/openai/.local/share/ipat/r940-live-agent/live.sock")
OPTIN="IPAT_R1006_OWNER_APPROVES_C320_PON_TX_READ"
MODE="OWNER_SUPERVISED_REAL_C320_READ_ONLY"
COMMAND="show pon power olt-tx gpon-olt_1/1/1"
PORT="1/1/1"
MAX_BYTES=4096
MAX_LINES=16

class Denied(RuntimeError):
    pass

def load_r945():
    spec=importlib.util.spec_from_file_location("ipat_r1006_r945",R945)
    if spec is None or spec.loader is None:
        raise Denied("canonical connector source unavailable")
    mod=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod

def status_idle(sock_path=STATUS_SOCKET):
    if not sock_path.exists() or sock_path.is_symlink():
        raise Denied("existing private connector socket unavailable")
    s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);s.settimeout(3)
    try:
        s.connect(str(sock_path));s.sendall(b"STATUS\n");data=b""
        while len(data)<=8192 and not data.endswith(b"\n"):
            part=s.recv(4096)
            if not part:break
            data+=part
    finally:s.close()
    if not 0<len(data)<=8192:raise Denied("connector status unavailable")
    try:value=json.loads(data)
    except Exception as exc:raise Denied("connector status invalid") from exc
    if value.get("mode")!=MODE or value.get("persistent_connector") is not True \
       or value.get("credentials_enrolled") is not True \
       or value.get("read_in_progress") is not False \
       or value.get("device_adopted") is not False \
       or value.get("device_writes") != 0:
        raise Denied("connector not in enrolled idle read-only pilot state")
    return True

def safe_bytes(raw: bytes):
    if not isinstance(raw,bytes) or len(raw)>MAX_BYTES:
        raise Denied("optical output outside byte bound")
    if b"\x00" in raw or b"\x1b" in raw or b"\x08" in raw:
        raise Denied("unsupported CLI control output")
    for byte in raw:
        if byte<32 and byte not in (9,10,13):
            raise Denied("unsupported CLI control byte")

def strip_exact_echo(raw: bytes):
    safe_bytes(raw)
    clean=raw.replace(b"\r",b"")
    lines=clean.splitlines()
    out=[];removed=False
    for line in lines:
        if not removed and line.strip()==COMMAND.encode("ascii"):
            removed=True;continue
        out.append(line)
    return out,removed

POWER=re.compile(rb"(?i)^tx\s+power\s*:\s*(n/a|[+-]?\d+(?:\.\d+)?)\s*\(dbm\)\s*$")
TABLE_HEADER=re.compile(rb"(?i)^channel\s+tx\s+power\s*$")
TABLE_VALUE=re.compile(rb"(?i)^1\(gpon\)\s+(?:(n/a)|([+-]?\d+(?:\.\d+)?)\s*\(dbm\))\s*$")
SEP=re.compile(rb"^[\s=_-]*$")

def safe_token_shape(token: bytes) -> str:
    low=token.lower()
    if low in (b"channel",b"tx",b"power",b"olt",b"pon"): return low.decode("ascii").upper()
    if re.fullmatch(rb"1\(GPON\)",token,re.I): return "GPON_CHANNEL_1"
    if re.fullmatch(rb"[+-]?\d+(?:\.\d+)?\(dbm\)",token,re.I): return "DBM_VALUE"
    if re.fullmatch(rb"[+-]?\d+(?:\.\d+)?",token): return "DECIMAL"
    if token.lower() in (b"n/a",b"na"): return "NA"
    if token==b":": return "COLON"
    if re.fullmatch(rb"[-=_]+",token): return "SEP"
    if re.fullmatch(rb"[A-Za-z]+",token): return "WORD_OTHER"
    if re.fullmatch(rb"\d+",token): return "NUMBER"
    if re.fullmatch(rb"[A-Za-z0-9_./:-]+",token): return "ATOM_OTHER"
    return "OTHER"

def sanitized_shape(lines: list[bytes]) -> str:
    out=[]
    for line in lines:
        stripped=line.strip()
        if not stripped: continue
        if SEP.fullmatch(stripped):
            out.append("SEP_LINE");continue
        out.append(",".join(safe_token_shape(t) for t in stripped.split()))
    return "|".join(out[:MAX_LINES])

def parse(raw: bytes):
    lines,echo_removed=strip_exact_echo(raw)
    if len(lines)>MAX_LINES:raise Denied("optical output outside line bound")
    nonblank=[x.strip() for x in lines if x.strip()]
    # Prevent accidental persistence/parsing of subscriber-specific output.
    lowered=b"\n".join(nonblank).lower()
    for forbidden in (b"gpon-onu_",b"epon-onu_",b"serial",b"subscriber",b"password",b"username"):
        if forbidden in lowered:raise Denied("subscriber or secret-shaped optical output")
    matches=[];table_header=0
    for line in nonblank:
        m=POWER.fullmatch(line)
        if m:
            matches.append(m.group(1));continue
        m=TABLE_VALUE.fullmatch(line)
        if m:
            matches.append(m.group(1) or m.group(2));continue
        if TABLE_HEADER.fullmatch(line):
            table_header+=1;continue
        if SEP.fullmatch(line):continue
        raise Denied("unexpected optical output shape:"+sanitized_shape(lines))
    if table_header not in (0,1):
        raise Denied("ambiguous optical table header")
    if table_header==1 and not any(TABLE_VALUE.fullmatch(line) for line in nonblank):
        raise Denied("optical table missing exact GPON channel")
    if table_header==0 and any(TABLE_VALUE.fullmatch(line) for line in nonblank):
        raise Denied("optical table value missing header")
    if len(matches)!=1:raise Denied("expected exactly one OLT transmit-power value")
    token=matches[0].lower()
    if token==b"n/a":
        value=None
    else:
        try:value=float(token.decode("ascii"))
        except ValueError as exc:raise Denied("invalid numeric optical value") from exc
        if not math.isfinite(value) or not -40.0<=value<=20.0:
            raise Denied("optical value outside conservative sanity bound")
        value=round(value,3)
    return {
      "schema":"ipat.c320.pon-olt-tx.v1","target":"DEV-01",
      "vendor":"ZTE","chassis_family":"C320","pon_port":PORT,
      "command_id":"PON_111_OLT_TX_FIXED","command_accepted":True,
      "exact_command_echo_removed":echo_removed,
      "tx_power_dbm":value,"measurement_available":value is not None,
      "raw_cli_returned":False,"onu_ids_returned":False,
      "serials_returned":False,"subscriber_data_returned":False,
      "physical_writes":0,"device_adopted":False,
      "diagnostic_health_inferred":False,
    }

def fixed_read(reader_module,password):
    import pexpect
    m=reader_module
    ssh=["ssh","-F","/dev/null","-tt","-p",str(m.PORT),
         "-o","HostKeyAlgorithms=ssh-rsa","-o","Ciphers=aes128-cbc",
         "-o","KexAlgorithms=diffie-hellman-group14-sha256",
         "-o","StrictHostKeyChecking=yes","-o",f"UserKnownHostsFile={m.PIN}",
         "-o","GlobalKnownHostsFile=/dev/null","-o","NumberOfPasswordPrompts=1",
         "-o","PreferredAuthentications=password","-o","PubkeyAuthentication=no",
         "-o","ProxyCommand=none","-o","ClearAllForwardings=yes",
         "-o","ConnectionAttempts=1","-o","ConnectTimeout=8",f"{m.USER}@{m.HOST}"]
    child=None
    try:
        child=pexpect.spawn(ssh[0],ssh[1:],encoding=None,timeout=12,echo=False,maxread=8192)
        if child.expect([rb"(?i)password:\s*$",pexpect.EOF,pexpect.TIMEOUT])!=0:
            raise Denied("private SSH challenge unavailable")
        child.sendline(password.encode())
        if child.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=10)!=0:
            raise Denied("authenticated C320 prompt unavailable")
        child.sendline(COMMAND.encode("ascii"))
        index=child.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=15)
        if index!=0:raise Denied("fixed optical read did not return prompt")
        return parse(child.before or b"")
    finally:
        if child is not None:
            try:
                if child.isalive():
                    child.sendline(b"exit");child.expect(pexpect.EOF,timeout=2)
            except Exception:child.close(force=True)

def collect():
    if os.geteuid()==0:raise Denied("must run as nonroot owner")
    if os.getenv(OPTIN)!="YES":raise Denied("explicit fixed optical-read approval missing")
    status_idle()
    r945=load_r945();_,reader_module=r945.module();password=r945.get_password()
    try:return fixed_read(reader_module,password)
    finally:password=None

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--collect",action="store_true")
    a=p.parse_args()
    if not a.collect:p.error("--collect required")
    try:
        print(json.dumps(collect(),sort_keys=True,indent=2));return 0
    except (Denied,OSError,ValueError,UnicodeError) as exc:
        print("R1006_C320_PON_TX_PROBE_DENIED:"+str(exc),file=sys.stderr);return 4
if __name__=="__main__":raise SystemExit(main())
