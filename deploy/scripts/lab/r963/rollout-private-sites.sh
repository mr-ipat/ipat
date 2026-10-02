#!/usr/bin/env bash
# One-time source/binary pinned R9.63 private-only owner-site module rollout.
set -Eeuo pipefail
umask 077
[[ $(id -u) != 0 ]] || { echo R963_REFUSE_ROOT;exit 2; }
base=/home/openai/workspaces/ipat
old_source=dcd2e6abc9e4a790e574229b63b23a3cb69932b2
binary=/home/openai/.cache/ipat/r963-release/control-api
binary_sha=c3d328029461abf283cd52afcea4ddfabe05063c7530d30b06a911ef10f06e8e
old_binary=/home/openai/.cache/ipat/r962-release/control-api
old_sha=2551b4d79c6735a8e3a71a646f98bef4f8c9da7933768bd2819794f201ffe461
unit=ipat-r911-preview.service
connector=ipat-r945-connector.service
drop="$HOME/.config/systemd/user/$unit.d/r963-standalone-sites.conf"
credential="$HOME/.local/share/ipat/r945-connection/device.fernet"
inventory="$HOME/.local/share/ipat/r961-device-inventory/devices.json"
domain="$HOME/.local/share/ipat/r962-domain-control/settings.json"
[[ $(git -C "$base" rev-parse HEAD) == "$old_source" && -z $(git -C "$base" status --porcelain) ]] || { echo R963_SOURCE_DRIFT;exit 2; }
[[ -x "$binary" && $(sha256sum "$binary"|cut -d' ' -f1) == "$binary_sha" ]] || { echo R963_BINARY_MISMATCH;exit 2; }
[[ -x "$old_binary" && $(sha256sum "$old_binary"|cut -d' ' -f1) == "$old_sha" ]] || { echo R963_ROLLBACK_BINARY_MISMATCH;exit 2; }
[[ ! -e "$drop" ]] || { echo R963_ALREADY_DEPLOYED;exit 2; }
for s in "$unit" "$connector";do [[ $(systemctl --user is-active "$s") == active ]] || { echo R963_SERVICE_MISSING;exit 2; };done
[[ $(systemctl --user show "$unit" -p ExecStart --value) == *'/home/openai/.cache/ipat/r962-release/control-api'* ]] || { echo R963_UNEXPECTED_OLD_UNIT;exit 2; }
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R963_NON_PRIVATE_BIND;exit 2; }
connect_pid=$(systemctl --user show "$connector" -p MainPID --value)
cred_before=$(sha256sum "$credential"|cut -d' ' -f1)
inv_before=$(sha256sum "$inventory"|cut -d' ' -f1)
domain_before=$(sha256sum "$domain"|cut -d' ' -f1)
python3 - <<'PY'
import json,urllib.request
base='http://127.0.0.1:3002';get=lambda p:json.load(urllib.request.urlopen(base+p,timeout=5))
d=get('/lab/owner/devices');assert d['persistent'] and any(x['id']=='DEV-01' and x['connection_state']=='CONNECTED' for x in d['devices'])
p=get('/lab/owner/pops');assert p['persistent'] is True
v=get('/lab/owner/platform-domains')['platform'];assert v['runtime_changes_applied'] is False and v['safe_to_point_now'] is False
print('R963_EXISTING_REAL_C320_SITES_DOMAIN_SAFE_BASELINE=PASS')
PY
rollback=1
restore(){
 if [[ "$rollback" == 1 ]];then
  echo R963_ROLLBACK_TO_EXISTING_R962
  rm -f -- "$drop"
  systemctl --user daemon-reload||:
  systemctl --user restart "$unit"||:
  systemctl --user is-active --quiet "$unit" && echo R963_ROLLBACK_RESTORED || echo R963_OWNER_INTERVENTION_NEEDED
 fi
}
trap restore EXIT
cat > "$drop" <<'UNIT'
[Service]
ExecStart=
ExecStart=/home/openai/.cache/ipat/r963-release/control-api
UNIT
chmod 0600 "$drop"
systemctl --user daemon-reload
systemctl --user restart "$unit"
ready=NO
for _ in {1..24};do
 [[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3002/healthz) == 200 ]] && { ready=YES; break; }
 sleep .4
done
[[ "$ready" == YES ]] || { echo R963_NEW_BINARY_HEALTH_FAILED; exit 3; }
python3 - <<'PY'
import json,urllib.request,urllib.error
base='http://127.0.0.1:3002'
def get(p):
 with urllib.request.urlopen(base+p,timeout=5) as response:
  assert response.status==200
  return response.read()
site=get('/lab/sites')
assert b'<html lang="en-US">' in site and b'Sites &amp; POPs' in site
assert b'id="ipat-sites-new"' in site and b'id="ipat-sites-form"' in site
assert b'createElement' in get('/lab/sites.js') and b'assigned_device_count' in get('/lab/sites.js')
assert b'.sidebar' in get('/lab/sites.css')
workbench=get('/lab/device-workbench')
assert b'href="/lab/sites"' in workbench and b'id="ipat-owner-pop"' in workbench
assert b'id="ipat-owner-pop-form"' not in workbench and b'id="ipat-owner-pop-rows"' not in workbench
assert b'/lab/sites?return=device' in get('/lab/owner-device-inventory.js')
assert b'/lab/sites' in get('/lab/platform-admin/domains')
pops=json.loads(get('/lab/owner/pops'));assert pops['persistent'] and isinstance(pops['pops'],list)
catalog=json.loads(get('/lab/owner/device-catalog'));assert len(catalog['vendors'])==4
active=json.loads(get('/lab/owner/devices'))
assert any(x['id']=='DEV-01' and x['connection_state']=='CONNECTED' for x in active['devices'])
domain=json.loads(get('/lab/owner/platform-domains'))['platform']
assert domain['public_https_ready'] is False and domain['safe_to_point_now'] is False
for route in ['/v1/tenant/domains','/v1/devices/DEV-01']:
 try:get(route)
 except urllib.error.HTTPError as exc:assert exc.code==401,(route,exc.code)
 else:raise AssertionError('Unauthenticated production API unexpectedly opened')
assert b'<html' in get('/lab/dashboard-preview')
print('R963_REAL_OWNER_SITE_MODULE_ROUTED_SEPARATELY=PASS',len(pops['pops']),'existing sites')
PY
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R963_NONPRIVATE_REGRESSION;exit 3; }
[[ $(systemctl --user show "$connector" -p MainPID --value) == "$connect_pid" ]] || { echo R963_CONNECTOR_RESTART_DETECTED;exit 3; }
[[ $(sha256sum "$credential"|cut -d' ' -f1) == "$cred_before" ]] || { echo R963_CREDENTIAL_CHANGED;exit 3; }
[[ $(sha256sum "$inventory"|cut -d' ' -f1) == "$inv_before" ]] || { echo R963_INVENTORY_CHANGED_ON_ROLLOUT;exit 3; }
[[ $(sha256sum "$domain"|cut -d' ' -f1) == "$domain_before" ]] || { echo R963_DOMAIN_PLAN_CHANGED_ON_ROLLOUT;exit 3; }
[[ $(curl -s --max-time 3 -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/lab) == 200 ]] || { echo R963_OLD_PREVIEW_UNAVAILABLE;exit 3; }
rollback=0
trap - EXIT
echo R963_PRIVATE_STANDALONE_SITE_AND_C320_SAFE_DEPLOYMENT=PASS
