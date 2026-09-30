#!/usr/bin/env python3
"""Owner-VPS loopback-only read panel HTTP smoke. NEVER logs real serials."""
import json,os
from urllib.request import Request,build_opener,ProxyHandler
from urllib.error import HTTPError
assert os.geteuid()!=0 and os.environ.get('IPAT_R940_PRIVATE_SMOKE')=='YES'
base='http://127.0.0.1:3002';o=build_opener(ProxyHandler({}))
def get(path):
    with o.open(base+path,timeout=4) as r:
        assert r.status==200 and r.headers['Cache-Control']=='no-store'
        return r.read()
page=get('/lab/device-workbench')
assert b'id="c320-read-live"' in page
js=get('/lab/c320-first-real-inventory.js')
assert b'/lab/c320-owner-live-refresh' in js
snapshot=json.loads(get('/lab/c320-owner-manual-onu-snapshot'))
assert snapshot['registered_onu_status_rows']==72 and snapshot['onu_online']==0
assert snapshot['snapshot_is_live'] is False
assert snapshot['real_olt_adopted'] is False
assert 'serial' not in json.dumps(snapshot).lower().replace('serial_numbers_disclosed','').replace('serials_returned','')
readiness=json.loads(get('/lab/c320-action-readiness'))
assert readiness['device_adopted'] is False and readiness['worker_enabled'] is False
for action in ('READ_CARD_INVENTORY','LIST_ONTS','PROVISION_ONTS','UPGRADE_OLT_FIRMWARE'):
    try: o.open(Request(base+'/lab/c320-actions/'+action,data=b'{}',method='POST'),timeout=4)
    except HTTPError as e: assert e.code==403
    else: raise AssertionError('hardware action unexpectedly available')
req=Request(base+'/lab/c320-owner-live-refresh',data=b'{}',method='POST',headers={
    'Origin':base,'X-IPAT-Demo-Only':'1','Content-Type':'application/json'})
try:o.open(req,timeout=5)
except HTTPError as e:
    assert e.code==503 # The separate owner-interactive agent is NOT started by deployment.
else:raise AssertionError('no-owner-agent live control unexpectedly succeeded')
with o.open('http://127.0.0.1:3000/healthz',timeout=3) as r:assert r.status==200
print('R940_PRIVATE_PANEL_DEPLOY_SMOKE_PASS; LIVE_BUTTON_BOUND_TO_REAL_AGENT_OFFLINE; WRITES_403')
