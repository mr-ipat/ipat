#!/usr/bin/env python3
"""ONE owner-supervised, temporary LAB C320 READ-only agent for the PRIVATE dashboard.
Credential exists only in THIS terminal process memory, never env, disk or IPC.
Unix socket mode0600, owner UID, bounded 3 fixed commands, at most 5 refreshes/15m.
NOT a commercial worker, automatic adoption or write-capable device connection.
"""
import getpass
import importlib.util
import json
import os
import socket
import socketserver
import stat
import struct
import sys
import time
from pathlib import Path

ROOT=Path('/home/openai/.local/share/ipat/r940-live-agent')
SOCKET=ROOT/'live.sock'
R938=Path('/home/openai/.cache/ipat/r940-stage/src/deploy/scripts/lab/r938/owner_c320_onu_first_inventory.py')
MAX_REQUESTS=5
SESSION_SECONDS=900
PROMPT='START_ONE_OWNER_PRIVATE_READ_ONLY_C320_AGENT'

class Denied(RuntimeError):pass

def module():
    spec=importlib.util.spec_from_file_location('r938_strict_owner_reader',R938)
    if spec is None or spec.loader is None: raise Denied('verified R938 reader unavailable')
    m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    return m

def run_three_reads(m,password):
    import pexpect
    # No dynamic user-specified target, IP, port, command or host-key fallback.
    ssh=['ssh','-F','/dev/null','-tt','-p',str(m.PORT),
      '-o','HostKeyAlgorithms=ssh-rsa','-o','Ciphers=aes128-cbc',
      '-o','KexAlgorithms=diffie-hellman-group14-sha256',
      '-o','StrictHostKeyChecking=yes','-o',f'UserKnownHostsFile={m.PIN}',
      '-o','GlobalKnownHostsFile=/dev/null','-o','NumberOfPasswordPrompts=1',
      '-o','PreferredAuthentications=password','-o','PubkeyAuthentication=no',
      '-o','ProxyCommand=none','-o','ClearAllForwardings=yes',
      '-o','ConnectionAttempts=1','-o','ConnectTimeout=8',f'{m.USER}@{m.HOST}']
    c=None
    try:
        c=pexpect.spawn(ssh[0],ssh[1:],encoding=None,timeout=12,echo=False,maxread=4096)
        if c.expect([rb'(?i)password:\s*$',pexpect.EOF,pexpect.TIMEOUT])!=0:
            raise Denied('private SSH challenge missing')
        c.sendline(password.encode())
        if c.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=10)!=0:
            raise Denied('authenticated C320 prompt missing')
        results=[]
        for i,command in enumerate(m.COMMANDS):
            c.sendline(command.encode())
            if c.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=15)!=0:
                raise Denied('fixed read timed out or session closed')
            raw=m.bounded(c.before,command)
            results.append(m.classify(command,raw))
        if results[1]['rows']!=results[2]['rows']:
            raise Denied('state and config count discrepancy')
        return {'mode':'OWNER_SUPERVISED_REAL_C320_READ_ONLY',
          'snapshot_is_live':True,'port':'1/1/1','unconfigured':results[0]['rows'],
          'configured':results[1]['rows'],'online':results[1]['online'],
          'offline':results[1]['offline'],'configuration_rows':results[2]['rows'],
          'serials_returned':False,'device_adopted':False,'provisioning_enabled':False,
          'device_writes':0}
    finally:
        if c is not None:
            try:
                if c.isalive():c.sendline(b'exit');c.expect(pexpect.EOF,timeout=2)
            except Exception:c.close(force=True)

def main():
    if len(sys.argv)!=2 or sys.argv[1]!='--owner-terminal':
        raise Denied('owner terminal mode required')
    if os.geteuid()==0 or not sys.stdin.isatty() or os.getenv('IPAT_R940_OWNER_READ_AGENT')!='YES':
        raise Denied('nonroot human owner terminal and opt-in required')
    os.umask(0o077)
    m=module()
    m.private_regular(m.PIN)
    if not m.PIN.read_text('ascii').startswith(f'[{m.HOST}]:{m.PORT} ssh-rsa '):
        raise Denied('previous exact private network observation pin absent')
    if ROOT.exists():
        if ROOT.is_symlink() or ROOT.stat().st_uid!=os.getuid() or stat.S_IMODE(ROOT.stat().st_mode)!=0o700:
            raise Denied('owner-only socket directory invalid')
    else:
        ROOT.mkdir(mode=0o700)
    if SOCKET.exists() or SOCKET.is_symlink():
        raise Denied('old agent socket exists; stop safely before reuse')
    if input(f'Type {PROMPT}: ').strip()!=PROMPT: raise Denied('owner did not consent')
    password=getpass.getpass('Temporary LAB SSH password (memory only): ')
    if not 1<=len(password)<=128: raise Denied('password length invalid')
    started=time.monotonic()
    last=0.0
    count=0
    class Handler(socketserver.BaseRequestHandler):
        def handle(self):
            nonlocal last,count
            try:
                # Linux-only local IPC: the private control API runs under the SAME
                # unprivileged owner UID as this explicitly opted-in process.
                ucred=self.request.getsockopt(socket.SOL_SOCKET,socket.SO_PEERCRED,12)
                _,uid,_=struct.unpack('3i',ucred)
                if uid!=os.getuid():raise Denied('foreign process denied')
                self.request.settimeout(4)
                request=self.request.recv(64)
                if request!=b'REFRESH\n':raise Denied('unknown request denied')
                now=time.monotonic()
                if now-started>SESSION_SECONDS or count>=MAX_REQUESTS or now-last<10:
                    raise Denied('session expired, quota or rate limited')
                # Reserve BEFORE the physical connection to avoid free retries.
                count+=1;last=now
                payload=run_three_reads(m,password)
                from datetime import datetime, timezone
                payload['read_at_utc']=datetime.now(timezone.utc).isoformat()
                result=json.dumps(payload,separators=(',',':')).encode()
            except Exception:
                # Never expose exception details, CLI output, secrets or serials.
                result=b'{"error":"OWNER_READ_FAIL_CLOSED"}'
            self.request.sendall(result+b'\n')
    try:
        with socketserver.UnixStreamServer(str(SOCKET),Handler) as server:
            SOCKET.chmod(0o600)
            print('OWNER_PRIVATE_READ_AGENT_ACTIVE; five bounded read refreshes maximum; Ctrl-C to stop')
            server.timeout=1
            while time.monotonic()-started<SESSION_SECONDS and count<MAX_REQUESTS:
                server.handle_request()
    finally:
        password=''
        if SOCKET.exists() and SOCKET.is_socket():SOCKET.unlink()
        print('OWNER_PRIVATE_READ_AGENT_CLOSED')

if __name__=='__main__':
    try: main()
    except (Denied,OSError,ValueError):
        print('OWNER_PRIVATE_READ_AGENT_FAIL_CLOSED',file=sys.stderr)
        sys.exit(4)
