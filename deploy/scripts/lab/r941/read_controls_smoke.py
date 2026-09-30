#!/usr/bin/env python3
"""Safe PRIVATE HTTP checks; never reaches physical OLT without owner agent."""
import os,json
from urllib.error import HTTPError
from urllib.request import Request,build_opener,ProxyHandler
assert os.geteuid()!=0 and os.getenv('IPAT_R941_PRIVATE_SMOKE')=='YES'
base='http://127.0.0.1:3002'
o=build_opener(ProxyHandler({}))
with o.open(base+'/lab/device-workbench',timeout=4) as r:
    html=r.read();assert r.status==200
    assert b'c320-read-cards' in html and b'c320-read-firmware' in html
for p in ('/lab/c320-owner-live-refresh','/lab/c320-owner-live-cards',
          '/lab/c320-owner-live-firmware'):
    req=Request(base+p,data=b'{}',method='POST',headers={
       'Origin':base,'X-IPAT-Demo-Only':'1','Content-Type':'application/json'})
    try:o.open(req,timeout=4)
    except HTTPError as e:assert e.code==503,(p,e.code)
    else:raise AssertionError('No owner agent: read unexpectedly successful')
for p in ('/lab/c320-owner-live-cards','/lab/c320-owner-live-firmware'):
    req=Request(base+p,data=b'{}',method='POST',headers={
       'Origin':'https://evil.invalid','X-IPAT-Demo-Only':'1'})
    try:o.open(req,timeout=4)
    except HTTPError as e:assert e.code==403,(p,e.code)
    else:raise AssertionError('Cross-origin read unexpectedly allowed')
with o.open(base+'/lab/c320-owner-manual-onu-snapshot',timeout=4) as r:
    d=json.loads(r.read());assert d['snapshot_is_live'] is False
    assert d['onu_online']==0 and d['registered_onu_status_rows']==72
with o.open('http://127.0.0.1:3000/healthz',timeout=4) as r:assert r.status==200
print('R941_PRIVATELY_DEPLOYED_READ_CONTROLS_PASS; ALL_READS_REQUIRE_OWNER_AGENT; WRITES_UNCHANGED')
