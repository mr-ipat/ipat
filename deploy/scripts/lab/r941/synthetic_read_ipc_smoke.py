#!/usr/bin/env python3
"""Synthetic socket-only positive HTTP tests. NO OLT transport or passwords."""
import json,os,socket,stat,threading
from pathlib import Path
from urllib.request import Request,build_opener,ProxyHandler
assert os.geteuid()!=0 and os.getenv('IPAT_R941_IPC_SYNTHETIC_TEST')=='YES'
root=Path('/home/openai/.local/share/ipat/r940-live-agent');path=root/'live.sock'
assert root.is_dir() and not root.is_symlink() and root.stat().st_uid==os.getuid()
assert stat.S_IMODE(root.stat().st_mode)==0o700 and not path.exists()
os.umask(0o077)
s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
try:
    s.bind(str(path));path.chmod(0o600);s.listen(2);s.settimeout(8)
    requests=[]
    def fake():
        for kind,key,count in [('CARDS','cards_in_service',3),('FIRMWARE','firmware_rows',5)]:
            with s.accept()[0] as c:
                requests.append(c.recv(32))
                result={'mode':'OWNER_SUPERVISED_REAL_C320_READ_ONLY',
                  'read_kind':kind,key:count,'firmware_reconciled':False,
                  'snapshot_is_live':True,'serials_returned':False,
                  'device_adopted':False,'provisioning_enabled':False,
                  'device_writes':0,'read_at_utc':'2026-09-29T11:00:00+00:00',
                  'untrusted_serial':'SYNTHETIC_SECRET_MUST_NEVER_RELAY'}
                c.sendall(json.dumps(result).encode()+b'\n')
    t=threading.Thread(target=fake,daemon=True);t.start()
    base='http://127.0.0.1:3002';o=build_opener(ProxyHandler({}))
    for url,key,expected in [('/lab/c320-owner-live-cards','cards_in_service',3),
                             ('/lab/c320-owner-live-firmware','firmware_rows',5)]:
        req=Request(base+url,method='POST',data=b'{}',
          headers={'Origin':base,'X-IPAT-Demo-Only':'1','Content-Type':'application/json'})
        with o.open(req,timeout=8) as response:
            assert response.status==200
            raw=response.read();data=json.loads(raw)
        assert data[key]==expected and data['device_adopted'] is False
        assert data['physical_writes_enabled'] is False
        assert b'SYNTHETIC_SECRET_MUST_NEVER_RELAY' not in raw
    t.join(5)
    assert requests==[b'CARDS\n',b'FIRMWARE\n']
    print('R941_TWO_NEW_READ_CONTROLS_POSITIVE_SYNTHETIC_IPC_PASS; NO_REAL_DEVICE_ACCESSED')
finally:
    s.close()
    if path.exists() and path.is_socket():path.unlink()
