#!/usr/bin/env python3
"""Nonroot localhost-only R9.36 private UI smoke. Never contacts C320."""
import json
import os
from urllib.error import HTTPError
from urllib.request import Request, build_opener, ProxyHandler

BASE='http://127.0.0.1:3002'
OPENER=build_opener(ProxyHandler({}))
assert os.geteuid()!=0 and os.environ.get('IPAT_R936_PRIVATE_SMOKE')=='YES'
with OPENER.open(BASE+'/lab/c320-action-readiness',timeout=4) as r:
    assert r.status==200 and r.headers.get('Cache-Control')=='no-store'
    payload=json.load(r)
gates=payload['adoption_gate_report']
assert gates['mode']=='HISTORICAL_LAB_DISPLAY_NOT_AUTHORIZATION'
assert gates['required_gate_count']==10 and gates['verified_gate_count']==2
assert gates['all_gates_verified'] is False
assert len(gates['gates'])==10
assert len([g for g in gates['gates'] if g['verified'] is False])==8
assert payload['firmware_inventory_fully_reconciled'] is False
assert payload['device_adopted'] is False and payload['worker_enabled'] is False
assert all(c['enabled'] is False for c in payload['capabilities'])
for action in ('READ_CARD_INVENTORY','LIST_ONTS','PROVISION_ONTS','UPGRADE_OLT_FIRMWARE'):
    req=Request(BASE+'/lab/c320-actions/'+action,data=b'{}',method='POST',
       headers={'Origin':BASE,'X-IPAT-Demo-Only':'1','Content-Type':'application/json'})
    try:
        OPENER.open(req,timeout=4)
        raise AssertionError('hardware action unexpectedly mounted')
    except HTTPError as e:
        assert e.code==403
        denial=json.load(e)
        assert denial['device_adopted'] is False
        assert denial['network_actions']==0
with OPENER.open(BASE+'/lab/c320-first-real-inventory.js',timeout=4) as r:
    js=r.read(16384)
    assert b'ADOPSI DITAHAN' in js and b'Registrasi ONT nyata belum diizinkan' in js
with OPENER.open('http://127.0.0.1:3000/healthz',timeout=4) as r:
    assert r.status==200
print('R936_PRIVATE_LOCALHOST_ONT_PRE_ADOPTION_SAFE_PASS')
