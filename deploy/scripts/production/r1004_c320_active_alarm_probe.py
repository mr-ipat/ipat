#!/usr/bin/env python3
"""Bounded one-shot DEV-01 active-alarm shape probe.

Uses exactly ONE historically accepted firmware read-only command:
  show alarm crtv-active

Purpose: verify current command acceptance and output SHAPE only. This probe
NEVER prints or persists raw alarm records, source identifiers, ONU indices,
serials, subscriber data, configuration, credentials or device secrets.
No config mode, write, reboot, firmware, optical or provisioning command exists.
"""
from __future__ import annotations
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import socket
import sys

ROOT=Path(__file__).resolve().parents[2]
R945=ROOT/"scripts/lab/r945/persistent_c320_connector.py"
STATUS_SOCKET=Path("/home/openai/.local/share/ipat/r940-live-agent/live.sock")
OPTIN="IPAT_R1004_OWNER_APPROVES_EXISTING_C320_ACTIVE_ALARM_READ"
MODE="OWNER_SUPERVISED_REAL_C320_READ_ONLY"
COMMAND="show alarm crtv-active"
MAX_BYTES=24576
MAX_PAGES=8
MAX_LINES=512

class Denied(RuntimeError):
    pass

def load_r945():
    spec=importlib.util.spec_from_file_location("ipat_r1004_r945",R945)
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
        s.connect(str(sock_path));s.sendall(b"STATUS\n")
        data=b""
        while len(data)<=8192 and not data.endswith(b"\n"):
            part=s.recv(4096)
            if not part:break
            data+=part
    finally:
        s.close()
    if not 0<len(data)<=8192:
        raise Denied("connector status unavailable")
    try:value=json.loads(data)
    except Exception as exc:raise Denied("connector status invalid") from exc
    if value.get("mode")!=MODE \
       or value.get("persistent_connector") is not True \
       or value.get("credentials_enrolled") is not True \
       or value.get("read_in_progress") is not False:
        raise Denied("connector not enrolled and idle")
    return True

def safe_chunk(chunk: bytes):
    if not isinstance(chunk,bytes):
        raise Denied("non-bytes CLI output")
    if b"\x00" in chunk or b"\x1b" in chunk:
        raise Denied("unsafe CLI escape/control output")
    for byte in chunk:
        if byte<32 and byte not in (8,9,10,13):
            raise Denied("unsupported CLI control byte")

def paged_read(child,prompt,pexpect_module=None):
    if pexpect_module is None:
        import pexpect as pexpect_module
    child.sendline(COMMAND.encode("ascii"))
    chunks=[]
    total=0
    pages=0
    while True:
        index=child.expect(
            [prompt,rb"--More--",pexpect_module.EOF,pexpect_module.TIMEOUT],
            timeout=15,
        )
        chunk=child.before or b""
        safe_chunk(chunk)
        total+=len(chunk)
        if total>MAX_BYTES:
            raise Denied("alarm read exceeded byte bound")
        chunks.append(chunk)
        if index==0:
            return b"".join(chunks),pages
        if index==1:
            pages+=1
            if pages>MAX_PAGES:
                raise Denied("alarm read exceeded pager bound")
            child.send(b" ")
            continue
        if index==2:
            raise Denied("alarm read session closed")
        raise Denied("alarm read timed out")

def normalize_pager_prefix(raw: bytes,pages: int) -> bytes:
    safe_chunk(raw)
    if not 0<=pages<=MAX_PAGES:
        raise Denied("invalid pager count")
    if b"\x08" not in raw:
        return raw
    if pages==0:
        raise Denied("backspace without pager")
    out=[]
    touched=0
    for line in raw.splitlines(keepends=True):
        if b"\x08" not in line:
            out.append(line);continue
        touched+=1
        if touched>pages or line.count(b"\x08")>32:
            raise Denied("pager erase artifact exceeds bound")
        stripped=line.lstrip(b" \t\x08")
        prefix=line[:len(line)-len(stripped)]
        if b"\x08" in stripped or any(x not in b" \t\x08" for x in prefix):
            raise Denied("backspace inside alarm data")
        out.append(line.replace(b"\x08",b""))
    return b"".join(out)

def token_class(token: bytes) -> str:
    if re.fullmatch(rb"\d+",token): return "N"
    if re.fullmatch(rb"\d{4}[-/]\d{1,2}[-/]\d{1,2}",token): return "DATE"
    if re.fullmatch(rb"\d{1,2}:\d{2}(?::\d{2})?",token): return "TIME"
    if re.fullmatch(rb"[-=_]+",token): return "SEP"
    if re.fullmatch(rb"[A-Za-z][A-Za-z_-]*",token): return "WORD"
    if re.fullmatch(rb"[A-Za-z0-9_./:-]+",token): return "ATOM"
    return "OTHER"

def strip_exact_command_echo(clean: bytes):
    """Remove only one exact CLI command echo; never arbitrary output lines."""
    lines=clean.splitlines()
    removed=False
    out=[]
    for line in lines:
        if not removed and line.strip()==COMMAND.encode("ascii"):
            removed=True
            continue
        out.append(line)
    return b"\n".join(out),removed

def output_shape(raw: bytes,pages: int):
    safe_chunk(raw)
    if len(raw)>MAX_BYTES:
        raise Denied("alarm output oversized")
    clean=normalize_pager_prefix(raw,pages).replace(b"\r",b"")
    payload,echo_removed=strip_exact_command_echo(clean)
    lines=payload.splitlines()
    if len(lines)>MAX_LINES:
        raise Denied("alarm line count outside bound")
    lower=payload.lower()
    errors=(
        b"invalid input",b"unknown command",b"incomplete command",
        b"ambiguous command",b"syntax error",b"unrecognized command",
    )
    command_rejected=any(x in lower for x in errors)
    hist={}
    widths={}
    keyword_flags={}
    for line in lines:
        parts=line.split()
        widths[str(len(parts))]=widths.get(str(len(parts)),0)+1
        if not parts: continue
        signature=",".join(token_class(x) for x in parts)
        hist[signature]=hist.get(signature,0)+1
    for word in (b"alarm",b"severity",b"level",b"date",b"time",
                 b"source",b"status",b"total",b"no alarm"):
        keyword_flags[word.decode("ascii").replace(" ","_")]=word in lower
    return {
        "schema":"ipat.c320.active-alarm-shape.v1",
        "target":"DEV-01",
        "vendor":"ZTE",
        "chassis_family":"C320",
        "command_id":"ACTIVE_ALARM_FIXED",
        "historically_accepted_command":True,
        "current_command_rejected":command_rejected,
        "current_command_accepted":not command_rejected,
        "exact_command_echo_removed":echo_removed,
        "payload_empty_after_exact_echo":not any(x.strip() for x in lines),
        "line_count":len(lines),
        "nonblank_line_count":sum(bool(x.strip()) for x in lines),
        "byte_count":len(payload),
        "pages":pages,
        "token_count_histogram":dict(sorted(widths.items(),key=lambda x:int(x[0]))),
        "shape_signature_histogram":dict(sorted(hist.items())),
        "generic_header_keyword_flags":keyword_flags,
        "raw_alarm_records_returned":False,
        "alarm_source_values_returned":False,
        "onu_ids_returned":False,
        "serials_returned":False,
        "subscriber_data_returned":False,
        "raw_transcript_returned":False,
        "physical_writes":0,
        "alarm_health_semantically_validated":False,
        "device_adopted":False,
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
    c=None
    try:
        c=pexpect.spawn(ssh[0],ssh[1:],encoding=None,timeout=12,echo=False,maxread=32768)
        if c.expect([rb"(?i)password:\s*$",pexpect.EOF,pexpect.TIMEOUT])!=0:
            raise Denied("private SSH challenge unavailable")
        c.sendline(password.encode())
        if c.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=10)!=0:
            raise Denied("authenticated C320 prompt unavailable")
        raw,pages=paged_read(c,m.PROMPT,pexpect)
        return output_shape(raw,pages)
    finally:
        if c is not None:
            try:
                if c.isalive():
                    c.sendline(b"exit");c.expect(pexpect.EOF,timeout=2)
            except Exception:
                c.close(force=True)

def collect():
    if os.geteuid()==0:
        raise Denied("must run as nonroot owner")
    if os.getenv(OPTIN)!="YES":
        raise Denied("explicit fixed alarm-read approval missing")
    status_idle()
    r945=load_r945()
    _,reader_module=r945.module()
    password=r945.get_password()
    try:
        return fixed_read(reader_module,password)
    finally:
        password=None

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--collect",action="store_true")
    a=p.parse_args()
    if not a.collect:
        p.error("--collect required")
    try:
        print(json.dumps(collect(),sort_keys=True,indent=2))
        return 0
    except (Denied,OSError,ValueError,UnicodeError) as exc:
        print("R1004_C320_ACTIVE_ALARM_PROBE_DENIED:"+str(exc),file=sys.stderr)
        return 4

if __name__=="__main__":
    raise SystemExit(main())
