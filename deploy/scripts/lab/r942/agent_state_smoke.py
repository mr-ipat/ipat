#!/usr/bin/env python3
"""Private owner-agent STATUS synthetic IPC smoke. Never queries actual OLT."""
import json,os,socket,stat,threading
from pathlib import Path
from urllib.request import build_opener,ProxyHandler
assert os.geteuid()!=0 and os.getenv('IPAT_R942_STATUS_SYNTHETIC')=='YES'
root=Path('/home/openai/.local/share/ipat/r940-live-agent');sock=root/'live.sock'
assert root.is_dir() and not root.is_symlink() and root.stat().st_uid==os.getuid()
assert stat.S_IMODE(root.stat().st_mode)==0o700 and not sock.exists()
url='http://127.0.0.1:3002/lab/c320-owner-agent-state'
o=build_opener(ProxyHandler({}))
def get():
    with o.open(url,timeout=4) as r:
        assert r.status==200 and r.headers['Cache-Control']=='no-store'
        return json.load(r)
off=get();assert off=={'agent_ready':False,'seconds_left':0,'requests_left':0,
  'actual_olt_connectivity_verified':False,'device_adopted':False,'physical_writes_enabled':False}
os.umask(0o077);s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
try:
    s.bind(str(sock));sock.chmod(0o600);s.listen(1);s.settimeout(5)
    got=[]
    def fake():
        with s.accept()[0] as c:
            got.append(c.recv(32))
            result={'mode':'OWNER_SUPERVISED_REAL_C320_READ_ONLY',
              'agent_ready':True,'seconds_left':600,'requests_left':4,
              'device_adopted':False,'device_writes':0,
              'raw_secret':'SYNTHETIC_NEVER_RELAY'}
            c.sendall(json.dumps(result).encode()+b'\n')
    thread=threading.Thread(target=fake,daemon=True);thread.start()
    v=get();thread.join(4)
    assert got==[b'STATUS\n']
    assert (v['agent_ready'],v['seconds_left'],v['requests_left'])==(True,600,4)
    assert v['actual_olt_connectivity_verified'] is False
    assert 'raw_secret' not in v
    print('R942_OFFLINE_AND_POSITIVE_STATUS_HTTP_PASS; NO_REAL_DEVICE_ACCESSED')
finally:
    s.close()
    if sock.exists() and sock.is_socket():sock.unlink()
