#!/usr/bin/env python3
"""Owner-interactive bounded actual C320 READ-ONLY discovery, NO auto-adoption.
Run ONLY in existing owner-VPS private account TTY; records owner-private output.
No automatic credential, configuration, firmware or ONT mutation.
"""
import getpass, hashlib, json, os, re, stat, sys, tempfile
from datetime import datetime, timezone
from pathlib import Path

HOST='10.10.13.233'; PORT=321; USER='zte'
PIN=Path('/home/openai/.local/share/ipat/r930-network-only-ssh-pin/network_observed_known_hosts')
PARENT=Path('/home/openai/.local/share/ipat')
APPROVAL='IPAT_R938_OWNER_ONE_TIME_DISCOVERY'
COMMANDS=('show gpon onu uncfg','show gpon onu state','show run interface gpon-olt_1/1/1')
PROMPT=re.compile(rb'(?m)^[-_.A-Za-z0-9]{1,32}#\s*$')
class Denied(Exception): pass

def private_regular(path,mode=0o600):
    s=path.lstat()
    if not stat.S_ISREG(s.st_mode) or s.st_uid!=os.getuid() or s.st_nlink!=1 \
            or stat.S_IMODE(s.st_mode)!=mode: raise Denied('private pinned file denied')

def bounded(raw,command):
    if not isinstance(raw,bytes) or not 1<=len(raw)<=32768 or b'\0' in raw \
       or b'\x1b' in raw or b'--More--' in raw or b'Invalid command' in raw \
       or b'%Error' in raw:
        raise Denied('bounded command response rejected')
    return raw
def classify(command,raw):
    if command==COMMANDS[0]:
        if b'No related information to show' in raw: return {'shape':'NO_UNCONFIGURED_REPORTED','rows':0}
        if not all(x in raw for x in (b'OnuIndex',b'Sn',b'State')):
            raise Denied('unconfigured ONU table unrecognized')
        lines=[x for x in raw.splitlines() if x.strip().startswith(b'gpon-onu_')]
        if not lines or len(lines)>128: raise Denied('unconfigured ONU rows not bounded')
        return {'shape':'ONU_UNCONFIGURED_TABLE','rows':len(lines)}
    if command==COMMANDS[1]:
        if not all(x in raw for x in (b'OnuIndex',b'Admin State',b'OMCC State')):
            raise Denied('state output unsupported on actual firmware')
        return {'shape':'ONU_STATE_TABLE','rows':sum(1 for x in raw.splitlines() if x.strip().startswith(b'gpon-onu_'))}
    if b'interface gpon-olt_1/1/1' not in raw or b'end' not in raw:
        raise Denied('bounded PON config response incomplete')
    return {'shape':'PON_REGISTERED_REFERENCE','rows':sum(1 for x in raw.splitlines() if re.match(rb'\s+onu\s+\d+\s+type\s+',x))}

def main():
    if len(sys.argv)!=2 or sys.argv[1]!='--owner-interactive-read':
        print('Read-only one-shot owner TTY, 3 fixed commands, never auto adoption.'); return 2
    os.umask(0o077)
    if os.geteuid()==0 or not sys.stdin.isatty() or os.getenv(APPROVAL)!='YES':
        raise Denied('owner-only nonroot interactive consent required')
    private_regular(PIN)
    if not PIN.read_text('ascii').startswith(f'[{HOST}]:{PORT} ssh-rsa '):
        raise Denied('exact previously observed network host key not pinned')
    if not PARENT.is_dir() or PARENT.is_symlink(): raise Denied('private parent absent')
    if input('Type ONE_OWNER_LAB_ONU_READ_NO_WRITES: ').strip()!='ONE_OWNER_LAB_ONU_READ_NO_WRITES':
        raise Denied('owner declined')
    secret=getpass.getpass('Temporary LAB-only credential (never stored): ')
    if not 1<=len(secret)<=128: raise Denied('credential length rejected')
    import pexpect
    cmd=['ssh','-F','/dev/null','-tt','-p',str(PORT),
        '-o','HostKeyAlgorithms=ssh-rsa','-o','Ciphers=aes128-cbc',
        '-o','KexAlgorithms=diffie-hellman-group14-sha256',
        '-o','StrictHostKeyChecking=yes','-o',f'UserKnownHostsFile={PIN}',
        '-o','GlobalKnownHostsFile=/dev/null','-o','NumberOfPasswordPrompts=1',
        '-o','PreferredAuthentications=password','-o','PubkeyAuthentication=no',
        '-o','ProxyCommand=none','-o','ClearAllForwardings=yes',
        '-o','ConnectionAttempts=1','-o','ConnectTimeout=8',f'{USER}@{HOST}']
    c=None; out=Path(tempfile.mkdtemp(prefix='r938-onu-read-',dir=PARENT));out.chmod(0o700)
    try:
        c=pexpect.spawn(cmd[0],cmd[1:],encoding=None,timeout=10,echo=False,maxread=4096)
        if c.expect([rb'(?i)password:\s*$',rb'(?i)host key verification failed',pexpect.EOF,pexpect.TIMEOUT])!=0:
            raise Denied('verified SSH password challenge missing')
        c.sendline(secret.encode()); secret=''
        if c.expect([PROMPT,rb'(?i)permission denied',pexpect.EOF,pexpect.TIMEOUT])!=0:
            raise Denied('authenticated CLI prompt not proven')
        results=[]
        for i,command in enumerate(COMMANDS):
            c.sendline(command.encode())
            if c.expect([PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=15)!=0:
                raise Denied('command failed to return exact prompt; stop')
            raw=bounded(c.before,command)
            shape=classify(command,raw)
            path=out/f'response-{i}.bin'
            fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
            with os.fdopen(fd,'wb') as f:
                f.write(raw);f.flush();os.fsync(f.fileno())
            results.append({'command_id':i,'sha256':hashlib.sha256(raw).hexdigest(),**shape})
        receipt={'observed_utc':datetime.now(timezone.utc).isoformat(),
            'mode':'OWNER_LAB_PHYSICAL_ONU_INVENTORY_READ_ONLY',
            'independent_oob_identity_verified':False,'credential_persisted':False,
            'auto_adopted':False,'device_configuration_writes':0,'results':results}
        fd=os.open(out/'summary.json',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        with os.fdopen(fd,'w') as f:
            json.dump(receipt,f,indent=2,sort_keys=True);f.write('\n');f.flush();os.fsync(f.fileno())
        print('R938_OWNER_LAB_ONU_THREE_READS_PASS; ZERO_DEVICE_WRITES')
        print('SANITIZED_COUNTS',[(r['command_id'],r['shape'],r['rows']) for r in results])
        print('OWNER_PRIVATE_RECEIPT_DIR',out)
    finally:
        secret=''
        if c is not None and c.isalive():
            try: c.sendline(b'exit');c.expect(pexpect.EOF,timeout=3)
            except Exception: c.close(force=True)

if __name__=='__main__':
    try: sys.exit(main())
    except (Denied,OSError,ValueError):
        print('R938_FAIL_CLOSED_NO_CREDENTIAL_OUTPUT',file=sys.stderr);sys.exit(4)
