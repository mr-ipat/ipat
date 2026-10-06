#!/usr/bin/env python3
"""Bounded one-shot DEV-01 PON/ONU count read with CLI pagination support.

Physical scope is intentionally narrow:
- exact owner-private DEV-01 target from sealed R945/R940 source
- ONLY three pre-existing qualified fixed read commands
- no serials, ONU IDs, raw CLI, config mode, writes, firmware/reboot
- pager continuation is a single SPACE only after an exact --More-- marker
- bounded pages, bytes and prompt timeout; anything ambiguous fails closed
"""
from __future__ import annotations
import argparse
import importlib.util
import json
import os
from pathlib import Path
import socket
import sys

ROOT=Path(__file__).resolve().parents[2]
R945=ROOT/"scripts/lab/r945/persistent_c320_connector.py"
STATUS_SOCKET=Path("/home/openai/.local/share/ipat/r940-live-agent/live.sock")
OPTIN="IPAT_R1002_OWNER_APPROVES_EXISTING_FIXED_C320_PAGED_READ"
MODE="OWNER_SUPERVISED_REAL_C320_READ_ONLY"
COMMANDS=(
    "show gpon onu uncfg",
    "show gpon onu state gpon-olt_1/1/1",
    "show run interface gpon-olt_1/1/1",
)
MAX_BYTES=32768
MAX_PAGES=12

class Denied(RuntimeError): pass

def load_r945():
    spec=importlib.util.spec_from_file_location("ipat_r1002_r945",R945)
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
    if value.get("mode")!=MODE or value.get("persistent_connector") is not True \
       or value.get("credentials_enrolled") is not True \
       or value.get("read_in_progress") is not False:
        raise Denied("connector not enrolled and idle")
    return True

def _safe_chunk(chunk: bytes):
    if not isinstance(chunk,bytes):
        raise Denied("non-bytes CLI chunk")
    if b"\x00" in chunk or b"\x1b" in chunk:
        raise Denied("unsafe CLI control sequence")
    for byte in chunk:
        if byte < 32 and byte not in (8,9,10,13):
            raise Denied("unsupported CLI control byte")

def paged_read(child,prompt,command,pexpect_module=None):
    if command not in COMMANDS:
        raise Denied("command outside fixed read allowlist")
    if pexpect_module is None:
        import pexpect as pexpect_module
    child.sendline(command.encode("ascii"))
    chunks=[]
    total=0
    pages=0
    while True:
        index=child.expect(
            [prompt,rb"--More--",pexpect_module.EOF,pexpect_module.TIMEOUT],
            timeout=15,
        )
        chunk=child.before or b""
        _safe_chunk(chunk)
        total+=len(chunk)
        if total>MAX_BYTES:
            raise Denied("fixed read exceeded byte bound")
        chunks.append(chunk)
        if index==0:
            return b"".join(chunks),pages
        if index==1:
            pages+=1
            if pages>MAX_PAGES:
                raise Denied("fixed read exceeded pager bound")
            # Session-local pager continuation only. No newline, command or
            # dynamic data can be sent here.
            child.send(b" ")
            continue
        if index==2:
            raise Denied("fixed read session closed")
        raise Denied("fixed read timed out")

def normalize_pager_artifacts(raw: bytes,pages: int) -> bytes:
    """Remove only line-prefix backspaces emitted while erasing a pager prompt."""
    _safe_chunk(raw)
    if not isinstance(pages,int) or not 0<=pages<=MAX_PAGES:
        raise Denied("invalid pager count")
    if b"\x08" not in raw:
        return raw
    if pages==0:
        raise Denied("backspace without observed pager")
    lines=raw.splitlines(keepends=True)
    touched=0
    out=[]
    for line in lines:
        if b"\x08" not in line:
            out.append(line);continue
        touched+=1
        if touched>pages or line.count(b"\x08")>32:
            raise Denied("pager erase artifact exceeds bound")
        # Backspace may exist only in leading terminal-erase whitespace.
        stripped=line.lstrip(b" \t\x08")
        prefix=line[:len(line)-len(stripped)]
        if b"\x08" in stripped or any(x not in b" \t\x08" for x in prefix):
            raise Denied("backspace inside CLI data")
        out.append(line.replace(b"\x08",b""))
    cleaned=b"".join(out)
    if len(cleaned)>MAX_BYTES or b"\x08" in cleaned:
        raise Denied("pager normalization failed")
    return cleaned

def state_diagnostics(raw: bytes):
    """Return aggregate-only diagnostics; never expose ONU indices or raw rows."""
    _safe_chunk(raw)
    clean=raw.replace(b"\r",b"")
    import re
    raw_lines=clean.splitlines()
    matched=[line for line in raw_lines if re.match(rb"^\s*1/1/1:\d+\s+",line)]
    recovered=[line.replace(b"\x08",b"") for line in raw_lines
               if not re.match(rb"^\s*1/1/1:\d+\s+",line)
               and b"\x08" in line
               and re.match(rb"^\s*1/1/1:\d+\s+",line.replace(b"\x08",b""))]
    state_lines=[line.strip().split() for line in matched]
    unique_indices=len({row[0] for row in state_lines if row})
    lengths={}
    phase={}
    admin={}
    omcc={}
    channels={}
    for row in state_lines:
        lengths[len(row)]=lengths.get(len(row),0)+1
        if len(row)>=2:
            token=row[1].decode("ascii","replace")
            admin[token]=admin.get(token,0)+1
        if len(row)>=3:
            token=row[2].decode("ascii","replace")
            omcc[token]=omcc.get(token,0)+1
        if len(row)>=4:
            token=row[3].decode("ascii","replace")
            phase[token]=phase.get(token,0)+1
        if len(row)>=5:
            token=row[4].decode("ascii","replace")
            channels[token]=channels.get(token,0)+1
    import re
    summary=re.search(rb"ONU Number:\s*(\d+)\s*/\s*(\d+)",clean)
    return {
        "state_row_count":len(state_lines),
        "recoverable_rows_if_backspace_removed":len(recovered),
        "lines_with_backspace":sum(b"\x08" in line for line in raw_lines),
        "unique_state_row_count":unique_indices,
        "duplicate_state_rows":len(state_lines)-unique_indices,
        "token_length_histogram":{str(k):v for k,v in sorted(lengths.items())},
        "admin_state_histogram":dict(sorted(admin.items())),
        "omcc_state_histogram":dict(sorted(omcc.items())),
        "phase_state_histogram":dict(sorted(phase.items())),
        "channel_histogram":dict(sorted(channels.items())),
        "summary_online":int(summary[1]) if summary else None,
        "summary_total":int(summary[2]) if summary else None,
        "backspace_bytes":raw.count(b"\x08"),
        "pager_markers_remaining":raw.count(b"--More--"),
        "serials_returned":False,
        "onu_ids_returned":False,
        "raw_transcript_returned":False,
        "physical_writes":0,
    }

def summarize(parser,raw_by_command,page_counts):
    if set(raw_by_command)!=set(COMMANDS) or set(page_counts)!=set(COMMANDS):
        raise Denied("incomplete fixed read set")
    shapes=[]
    for command in COMMANDS:
        bounded=parser.bounded(raw_by_command[command],command)
        shape=parser.classify(command,bounded)
        shapes.append(shape)
    unconfigured,state,config=shapes
    if state.get("shape")!="ONU_STATE_TABLE":
        raise Denied("state classification mismatch")
    if config.get("shape")!="PON_REGISTERED_REFERENCE":
        raise Denied("config classification mismatch")
    if unconfigured.get("shape") not in ("NO_UNCONFIGURED_REPORTED","ONU_UNCONFIGURED_TABLE"):
        raise Denied("unconfigured classification mismatch")
    if state.get("rows")!=config.get("rows"):
        raise Denied("state/config count discrepancy")
    configured=state.get("rows")
    online=state.get("online")
    offline=state.get("offline")
    if not all(isinstance(v,int) and 0<=v<=128 for v in
               (unconfigured.get("rows"),configured,online,offline)):
        raise Denied("count outside bounded range")
    if online+offline!=configured:
        raise Denied("state totals inconsistent")
    return {
        "schema":"ipat.c320.pon-onu-counts.v1",
        "target":"DEV-01",
        "vendor":"ZTE",
        "chassis_family":"C320",
        "evidence_type":"physical",
        "pon":"1/1/1",
        "unconfigured":unconfigured["rows"],
        "configured":configured,
        "online":online,
        "offline":offline,
        "configuration_rows":config["rows"],
        "pagination":{
            "uncfg_pages":page_counts[COMMANDS[0]],
            "state_pages":page_counts[COMMANDS[1]],
            "config_pages":page_counts[COMMANDS[2]],
        },
        "serials_returned":False,
        "onu_ids_returned":False,
        "subscriber_data_returned":False,
        "physical_writes":0,
        "device_adopted":False,
        "provisioning_enabled":False,
        "validation_status":"partial",
        "qualified_commands":["UNC_CFG_FIXED","ONU_STATE_FIXED_PON_1_1_1","RUN_INTERFACE_FIXED_PON_1_1_1"],
    }

def fixed_read(reader_module,password,diagnostic=False):
    import pexpect
    m=reader_module
    # Refuse a private parser command drift instead of silently broadening
    # the physical allowlist.
    if tuple(getattr(m,"COMMANDS",()))!=COMMANDS:
        raise Denied("sealed parser fixed command set changed")
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
        c=pexpect.spawn(ssh[0],ssh[1:],encoding=None,timeout=12,echo=False,maxread=65536)
        if c.expect([rb"(?i)password:\s*$",pexpect.EOF,pexpect.TIMEOUT])!=0:
            raise Denied("private SSH challenge unavailable")
        c.sendline(password.encode())
        if c.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=10)!=0:
            raise Denied("authenticated C320 prompt unavailable")
        raw={};pages={}
        for command in COMMANDS:
            raw[command],pages[command]=paged_read(c,m.PROMPT,command,pexpect)
        if diagnostic:
            # Still enforce the sealed parser's bounded raw transport guard.
            state_raw=m.bounded(raw[COMMANDS[1]],COMMANDS[1])
            return {
                "schema":"ipat.c320.pon-onu-state-diagnostic.v1",
                "target":"DEV-01","pon":"1/1/1",
                "pagination":{"state_pages":pages[COMMANDS[1]]},
                **state_diagnostics(state_raw),
            }
        normalized={command:normalize_pager_artifacts(raw[command],pages[command])
                    for command in COMMANDS}
        return summarize(m,normalized,pages)
    finally:
        if c is not None:
            try:
                if c.isalive():
                    c.sendline(b"exit");c.expect(pexpect.EOF,timeout=2)
            except Exception:
                c.close(force=True)

def collect(diagnostic=False):
    if os.geteuid()==0:
        raise Denied("must run as nonroot owner")
    if os.getenv(OPTIN)!="YES":
        raise Denied("explicit fixed-read approval missing")
    status_idle()
    r945=load_r945()
    _,reader_module=r945.module()
    password=r945.get_password()
    try:
        result=fixed_read(reader_module,password,diagnostic=diagnostic)
    finally:
        password=None
    return result

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--collect",action="store_true")
    p.add_argument("--diagnose-state",action="store_true",
                   help="aggregate-only state-shape diagnostics; no ONU IDs/raw")
    args=p.parse_args()
    if args.collect==args.diagnose_state:
        p.error("choose exactly one of --collect or --diagnose-state")
    try:
        print(json.dumps(collect(diagnostic=args.diagnose_state),sort_keys=True,indent=2))
        return 0
    except (Denied,OSError,ValueError,UnicodeError) as exc:
        print("R1002_C320_PAGED_READ_DENIED:"+str(exc),file=sys.stderr)
        return 4

if __name__=="__main__":
    raise SystemExit(main())
