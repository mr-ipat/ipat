#!/usr/bin/env python3
"""OFFLINE synthetic single-connection local IPC smoke, ZERO physical SSH/OLT calls."""
import json,os,socket,stat,threading
from pathlib import Path
from urllib.request import Request,build_opener,ProxyHandler

assert os.geteuid()!=0 and os.environ.get('IPAT_R940_SYNTHETIC_IPC_SMOKE')=='YES'
root=Path('/home/openai/.local/share/ipat/r940-live-agent')
socket_path=root/'live.sock'
os.umask(0o077)
if not root.exists():root.mkdir(mode=0o700)
assert not root.is_symlink() and root.stat().st_uid==os.getuid()
assert stat.S_IMODE(root.stat().st_mode)==0o700 and not socket_path.exists()
s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
try:
    s.bind(str(socket_path));socket_path.chmod(0o600);s.listen(1);s.settimeout(8)
    calls=[]
    def fake_owner_agent():
        with s.accept()[0] as c:
            calls.append(c.recv(32))
            payload={'mode':'OWNER_SUPERVISED_REAL_C320_READ_ONLY',
              'snapshot_is_live':True,'port':'1/1/1',
              'configured':72,'unconfigured':0,'online':0,'offline':72,
              'configuration_rows':72,'serials_returned':False,'device_adopted':False,
              'provisioning_enabled':False,'device_writes':0,
              'read_at_utc':'2026-09-29T11:00:00+00:00',
              'raw_serial':'SYNTHETIC_MUST_NOT_REACH_BROWSER'}
            c.sendall(json.dumps(payload).encode()+b'\n')
    worker=threading.Thread(target=fake_owner_agent,daemon=True)
    worker.start()
    base='http://127.0.0.1:3002'; opener=build_opener(ProxyHandler({}))
    req=Request(base+'/lab/c320-owner-live-refresh',method='POST',data=b'{}',
        headers={'Origin':base,'X-IPAT-Demo-Only':'1','Content-Type':'application/json'})
    with opener.open(req,timeout=8) as response:
        assert response.status==200
        result=response.read()
    worker.join(5)
    assert calls==[b'REFRESH\n']
    v=json.loads(result)
    assert (v['configured'],v['online'],v['offline'],v['unconfigured'])==(72,0,72,0)
    assert v['device_adopted'] is False and v['physical_writes_enabled'] is False
    assert b'SYNTHETIC_MUST_NOT_REACH_BROWSER' not in result
    print('R940_PRIVATE_POSITIVE_IPC_SYNTHETIC_SMOKE_PASS; ZERO_REAL_DEVICE_COMMANDS')
finally:
    s.close()
    if socket_path.exists() and socket_path.is_socket():socket_path.unlink()
