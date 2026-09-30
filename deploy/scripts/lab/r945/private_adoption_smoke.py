#!/usr/bin/env python3
"""R9.45 postdeploy private HTTP acceptance, NO physical OLT commands."""
import json
import os
from urllib.error import HTTPError
from urllib.request import Request, ProxyHandler, build_opener

assert os.geteuid() != 0 and os.environ.get('IPAT_R945_PRIVATE_SMOKE') == 'YES'
base='http://127.0.0.1:3002'
client=build_opener(ProxyHandler({}))
def get(path):
    with client.open(base+path,timeout=5) as response:
        assert response.status==200
        assert response.headers.get('Cache-Control')=='no-store'
        return response.read()

page=get('/lab/device-workbench')
assert b'id="ipat-c320-enroll-form"' in page
assert b'id="ipat-c320-console"' in page
assert b'<details class="ipat-lab-diagnostics"' in page
js=get('/lab/c320-connection-setup.js')
assert b'/lab/c320-owner-enroll' in js
status=json.loads(get('/lab/c320-owner-connection'))
assert status['target']=='DEV-01' and status['model']=='C320'
assert status['connector_online'] is True
assert status['credentials_enrolled'] is False
assert status['adoption_state']=='NOT_ENROLLED'
assert status['production_adopted'] is False
assert status['physical_writes_enabled'] is False
agent=json.loads(get('/lab/c320-owner-agent-state'))
assert agent['agent_ready'] is False
for action in ('PROVISION_ONTS','REBOOT_OLT','UPGRADE_OLT_FIRMWARE'):
    req=Request(base+'/lab/c320-actions/'+action,data=b'{}',method='POST')
    try:client.open(req,timeout=4)
    except HTTPError as error:assert error.code==403,(action,error.code)
    else:raise AssertionError('hardware write unexpectedly enabled: '+action)

bad=Request(base+'/lab/c320-owner-enroll',data=b'{}',method='POST',
    headers={'Origin':'http://untrusted.invalid','X-IPAT-Demo-Only':'1',
             'Content-Type':'application/json'})
try:client.open(bad,timeout=4)
except HTTPError as error:assert error.code==403,error.code
else:raise AssertionError('forged enrollment accepted')
with client.open('http://127.0.0.1:3000/healthz',timeout=4) as r:
    assert r.status==200
print('R945_PRIVATE_CONNECTOR_DEPLOY_PASS; NO_DEVICE_CREDENTIALS_OR_WRITES; ORIGINAL_3000_HEALTHY')
