#!/usr/bin/env python3
"""R9.45 actual private HTTP deployment smoke: ZERO device passwords/commands."""
import json
import os
from urllib.error import HTTPError
from urllib.request import Request, ProxyHandler, build_opener

assert os.geteuid()!=0 and os.getenv('IPAT_R945_PRIVATE_SMOKE')=='YES'
opener=build_opener(ProxyHandler({}))
base='http://127.0.0.1:3002'
def get(path):
    with opener.open(base+path,timeout=4) as r:
        assert r.status==200 and r.headers.get('Cache-Control')=='no-store'
        return r.read()
page=get('/lab/device-workbench')
assert b'id="ipat-c320-enroll-form"' in page
assert b'id="ipat-c320-console"' in page
assert b'<details class="ipat-lab-diagnostics"' in page
js=get('/lab/c320-connection-setup.js')
assert b'/lab/c320-owner-enroll' in js and b'localStorage' not in js
state=json.loads(get('/lab/c320-owner-connection'))
assert state['target']=='DEV-01'
assert state['connector_online'] is True
assert state['credentials_enrolled'] is False
assert state['production_adopted'] is False
assert state['physical_writes_enabled'] is False
assert state['adoption_state']=='NOT_ENROLLED'
request=Request(base+'/lab/c320-owner-enroll',data=b'{"device_profile":"zte_c320_lab"}',
    method='POST',headers={'Origin':'http://not-authorized.invalid',
                          'X-IPAT-Demo-Only':'1','Content-Type':'application/json'})
try:opener.open(request,timeout=3)
except HTTPError as error:assert error.code==403
else:raise AssertionError('cross-origin enrollment unexpectedly accepted')
request=Request(base+'/lab/c320-owner-enroll',
    data=b'{"device_profile":"other_olt","bootstrap_code":"synthetic_1234567890123456789012345678","password":"placeholder"}',
    method='POST',headers={'Origin':base,'X-IPAT-Demo-Only':'1',
                           'Content-Type':'application/json'})
try:opener.open(request,timeout=3)
except HTTPError as error:assert error.code==400
else:raise AssertionError('unapproved device profile accepted')
for action in ('PROVISION_ONTS','REBOOT_OLT','UPGRADE_OLT_FIRMWARE'):
    req=Request(base+'/lab/c320-actions/'+action,data=b'{}',method='POST')
    try:opener.open(req,timeout=3)
    except HTTPError as error:assert error.code==403
    else:raise AssertionError('physical writes enabled: '+action)
with opener.open('http://127.0.0.1:3000/healthz',timeout=3) as r:
    assert r.status==200
print('R945_PERSISTENT_DASHBOARD_OFFLINE_ENROLLMENT_READY_HTTP_PASS; NO_DEVICE_WRITES')
