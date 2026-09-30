#!/usr/bin/env python3
"""Actual loopback HTTP smoke (agent must be absent), NEVER physical device commands."""
import os,json
from urllib.request import build_opener,ProxyHandler
assert os.geteuid()!=0 and os.getenv('IPAT_R942_PRIVATE_SMOKE')=='YES'
base='http://127.0.0.1:3002';o=build_opener(ProxyHandler({}))
with o.open(base+'/lab/device-workbench',timeout=4) as r:
    h=r.read();assert r.status==200 and b'id="c320-owner-agent-state"' in h
with o.open(base+'/lab/c320-first-real-inventory.js',timeout=4) as r:
    assert b'/lab/c320-owner-agent-state' in r.read()
with o.open(base+'/lab/c320-owner-agent-state',timeout=4) as r:
    assert r.headers['Cache-Control']=='no-store'
    d=json.load(r)
assert d=={'agent_ready':False,'seconds_left':0,'requests_left':0,
 'actual_olt_connectivity_verified':False,'device_adopted':False,'physical_writes_enabled':False}
with o.open('http://127.0.0.1:3000/healthz',timeout=3) as r:assert r.status==200
print('R942_PRIVATE_AGENT_STATUS_OFFLINE_VERIFIED_PANEL_HTTP_PASS')
