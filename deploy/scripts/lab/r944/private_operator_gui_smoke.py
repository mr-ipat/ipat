#!/usr/bin/env python3
"""R9.44 private, operator-first dashboard deployment verification.
NO actual OLT commands: this smoke must run only with owner agent absent.
"""
import json
import os
from urllib.error import HTTPError
from urllib.request import Request, ProxyHandler, build_opener

assert os.geteuid() != 0
assert os.environ.get("IPAT_R944_PRIVATE_SMOKE") == "YES"
base = "http://127.0.0.1:3002"
opener = build_opener(ProxyHandler({}))

def get(path):
    with opener.open(base + path, timeout=5) as result:
        assert result.status == 200, (path, result.status)
        assert result.headers.get("Cache-Control") == "no-store", path
        return result.read()

page = get("/lab/device-workbench")
script = get("/lab/c320-operator-console.js")
assert page.find(b'id="ipat-c320-console"') >= 0
assert page.find(b'id="ipat-c320-console"') < page.find(b'class="metric-row"')
for button in [b'id="ipat-c320-read-onus"', b'id="ipat-c320-read-cards"',
               b'id="ipat-c320-read-firmware"']:
    assert button in page, button
assert b'id="ipat-c320-onu-rows"' in page
assert b'/lab/c320-owner-manual-onu-snapshot' in script
assert b'/lab/c320-owner-live-refresh' in script

snapshot = json.loads(get("/lab/c320-owner-manual-onu-snapshot"))
ids = snapshot["manual_onu_ids"]
assert snapshot["snapshot_is_live"] is False
assert snapshot["manual_onu_rows_are_live"] is False
assert snapshot["real_olt_adopted"] is False
assert len(ids) == len(set(ids)) == 72
assert ids[0] == 2 and ids[-1] == 98
assert snapshot["registered_onu_status_rows"] == len(ids)
assert snapshot["onu_online"] == 0 and snapshot["onu_offline"] == len(ids)
body = json.dumps(snapshot)
assert "ZTEGC969" not in body

state = json.loads(get("/lab/c320-owner-agent-state"))
assert state["agent_ready"] is False
assert state["actual_olt_connectivity_verified"] is False
assert state["physical_writes_enabled"] is False
for path in ["/lab/c320-owner-live-refresh",
             "/lab/c320-owner-live-cards",
             "/lab/c320-owner-live-firmware"]:
    request = Request(base + path, method="POST", data=b"{}",
        headers={"Origin":base, "X-IPAT-Demo-Only":"1", "Content-Type":"application/json"})
    try:
        opener.open(request, timeout=6)
        raise AssertionError("unauthenticated physical read unexpectedly succeeded")
    except HTTPError as error:
        assert error.code == 503, (path, error.code)

for action in ["PROVISION_ONTS", "REBOOT_OLT", "UPGRADE_OLT_FIRMWARE"]:
    request = Request(base + "/lab/c320-actions/" + action,
                      method="POST", data=b"{}")
    try:
        opener.open(request, timeout=5)
        raise AssertionError("physical write unexpectedly enabled")
    except HTTPError as error:
        assert error.code == 403, (action, error.code)

with opener.open("http://127.0.0.1:3000/healthz", timeout=4) as result:
    assert result.status == 200
print("R944_PRIVATE_OPERATOR_GUI_DEPLOY_SMOKE_PASS: 72 OWNER-MANUAL IDs; "
      "READ_AGENT_OFFLINE; ALL_HW_WRITES_403; ORIGINAL_SERVICE_200")
