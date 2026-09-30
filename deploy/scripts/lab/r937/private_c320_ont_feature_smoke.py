#!/usr/bin/env python3
"""Private LOCALHOST historical ONT feature contract; never touches hardware."""
import json
import os
from urllib.request import Request, build_opener, ProxyHandler
from urllib.error import HTTPError

assert os.geteuid() != 0 and os.environ.get('IPAT_R937_PRIVATE_SMOKE') == 'YES'
BASE='http://127.0.0.1:3002'
opener=build_opener(ProxyHandler({}))
with opener.open(BASE+'/lab/c320-ont-feature-readiness',timeout=4) as response:
    assert response.status==200 and response.headers['Cache-Control']=='no-store'
    data=json.load(response)
assert data['mode']=='PRIVATE_HISTORICAL_ONT_REVIEW_ONLY'
assert data['adoption_gate_report']['verified_gate_count']==2
assert data['adoption_gate_report']['required_gate_count']==10
assert data['real_hardware_adopted'] is False
assert data['actual_unconfigured_onu_discovery_verified'] is False
assert data['real_ont_model_firmware_verified'] is False
assert data['actual_vlan_tcont_gem_profiles_verified'] is False
assert len(data['available_offline_modules'])==3
assert data['physical_ont_register_enabled'] is False
assert data['physical_ont_config_enabled'] is False
assert data['network_actions']==0
for action in ('READ_CARD_INVENTORY','LIST_ONTS','PROVISION_ONTS','UPGRADE_OLT_FIRMWARE'):
    req=Request(BASE+'/lab/c320-actions/'+action,method='POST',data=b'{}',
        headers={'Origin':BASE,'Content-Type':'application/json','X-IPAT-Demo-Only':'1'})
    try:
        opener.open(req,timeout=4)
        raise AssertionError('physical write action mounted')
    except HTTPError as e:
        assert e.code==403
        assert json.load(e)['device_adopted'] is False
with opener.open(BASE+'/lab/c320-first-real-inventory.js',timeout=4) as r:
    source=r.read(20000)
    assert b'/lab/c320-ont-feature-readiness' in source
    assert b'Perintah nyata dan interoperabilitas firmware BELUM diuji' in source
with opener.open('http://127.0.0.1:3000/healthz',timeout=4) as r:
    assert r.status==200
print('R937_PRIVATE_ONT_FEATURE_HTTP_SAFE_PASS; REAL_ACTIONS_403')
