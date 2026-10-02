#!/usr/bin/env bash
# R9.62 exact-source one-time rootless, private-loopback canary rollout.
set -Eeuo pipefail
umask 077
[[ $(id -u) != 0 ]] || { echo R962_REFUSE_ROOT; exit 2; }
base=/home/openai/workspaces/ipat
oldmain=4c2374aabb96cfdbef89086a49bbbd4cb7054bfa
bin=/home/openai/.cache/ipat/r962-release/control-api
expected=2551b4d79c6735a8e3a71a646f98bef4f8c9da7933768bd2819794f201ffe461
previous=56a6cb2fdbea7326ec21ac6b30e36a886109063b40321724ab99e19658c94a17
unit=ipat-r911-preview.service
connector=ipat-r945-connector.service
drop="$HOME/.config/systemd/user/$unit.d/r962-master-domain-admin.conf"
credential="$HOME/.local/share/ipat/r945-connection/device.fernet"
inventory="$HOME/.local/share/ipat/r961-device-inventory/devices.json"
[[ -x "$bin" && $(sha256sum "$bin" | cut -d ' ' -f1) == "$expected" ]] || { echo R962_BAD_BINARY; exit 2; }
[[ $(git -C "$base" rev-parse HEAD) == "$oldmain" ]] || { echo R962_BASE_SOURCE_DRIFT; exit 2; }
[[ ! -e "$drop" ]] || { echo R962_OVERRIDE_ALREADY_EXISTS; exit 2; }
[[ $(systemctl --user is-active "$unit") == active && $(systemctl --user is-active "$connector") == active ]] || { echo R962_PRIOR_SERVICE_DOWN; exit 2; }
[[ $(systemctl --user show "$unit" -p ExecStart --value) == *'/home/openai/.cache/ipat/r961-release/control-api'* ]] || { echo R962_PRIOR_BINARY_UNEXPECTED; exit 2; }
[[ $(sha256sum /home/openai/.cache/ipat/r961-release/control-api | cut -d ' ' -f1) == "$previous" ]] || { echo R962_ROLLBACK_BINARY_UNKNOWN; exit 2; }
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R962_NONPRIVATE_BIND; exit 2; }
vars=$(systemctl --user show "$unit" -p Environment --value)
for gate in IPAT_R911_PRIVATE_CANARY=YES IPAT_R940_PRIVATE_OWNER_READ=YES IPAT_CUSTOM_DOMAIN_INSTRUCTIONS=YES IPAT_CUSTOM_DOMAIN_ROUTING_READY=NO;do
 [[ "$vars" == *"$gate"* ]] || { echo R962_MISSING_OWNER_GATE; exit 2; }
done
connectorpid=$(systemctl --user show "$connector" -p MainPID --value)
credential_hash=$(sha256sum "$credential" | cut -d ' ' -f1)
inventory_hash=$(sha256sum "$inventory" | cut -d ' ' -f1)
python3 - <<'PY'
import json,urllib.request
base='http://127.0.0.1:3002'
old=json.load(urllib.request.urlopen(base+'/lab/owner/devices',timeout=5))
assert old['persistent'] is True and old['count']>=1
real=next(d for d in old['devices'] if d['id']=='DEV-01')
assert real['linked_live_connector'] is True and real['connection_state']=='CONNECTED'
assert real['physical_writes_enabled'] is False
print('R962_BASELINE_REAL_C320_AND_OLD_INVENTORY=PASS')
PY
rollback=1
restore(){
 if [[ "$rollback" == 1 ]];then
  echo R962_AUTO_RESTORE_R961
  rm -f -- "$drop"
  systemctl --user daemon-reload || :
  systemctl --user restart "$unit" || :
  systemctl --user is-active --quiet "$unit" && echo R962_PRIOR_UI_RESTORED || echo R962_OPERATOR_RECOVERY_NEEDED
 fi
}
trap restore EXIT
cat > "$drop" <<'UNIT'
[Service]
ExecStart=
ExecStart=/home/openai/.cache/ipat/r962-release/control-api
UNIT
chmod 0600 "$drop"
systemctl --user daemon-reload
systemctl --user restart "$unit"
ready=no
for i in {1..25};do
 if [[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3002/healthz) == 200 ]];then ready=yes;break;fi
 sleep .4
done
[[ "$ready" == yes ]] || { echo R962_SERVICE_HEALTH_FAILED; exit 3; }
python3 - <<'PY'
import json,urllib.request,urllib.error
base='http://127.0.0.1:3002'
def fetch(path):
 with urllib.request.urlopen(base+path,timeout=5) as r:
  assert r.status==200
  return r.read()
page=fetch('/lab/device-workbench')
assert b'<select id="ipat-owner-pop" required>' in page
assert b'<select id="ipat-owner-vendor" required>' in page
assert b'id="ipat-owner-pop-create"' in page
assert b'platform-admin/domains' in page
assert b'Capability &amp; Execution Center' in page
script=fetch('/lab/owner-device-inventory.js')
assert b'populateVendors' in script and b'loadMasters' in script
catalog=json.loads(fetch('/lab/owner/device-catalog'))
assert len(catalog['vendors'])==4 and catalog['new_device_connection_ready'] is False
assert catalog['arbitrary_vendor_allowed'] is False
pops=json.loads(fetch('/lab/owner/pops'))
assert pops['persistent'] is True and isinstance(pops['pops'],list)
managed=json.loads(fetch('/lab/owner/devices'))
assert managed['count']>=1 and any(d['id']=='DEV-01' and d['connection_state']=='CONNECTED' for d in managed['devices'])
admin=fetch('/lab/platform-admin/domains')
assert b'PRD PRODUCTION BLOCKER' in admin and b'id="ipat-domain-platform-host"' in admin
assert b'STAGE_DOMAIN_CONFIG' in fetch('/lab/platform-admin/domains.js')
settings=json.loads(fetch('/lab/owner/platform-domains'))
assert settings['owner_private_lab_only'] is True and settings['persisted'] is True
assert settings['platform']['runtime_changes_applied'] is False
assert settings['platform']['safe_to_point_now'] is False
assert settings['platform']['public_https_ready'] is False
for target in ['/v1/tenant/domains','/v1/devices/DEV-01']:
 try:fetch(target)
 except urllib.error.HTTPError as exc:assert exc.code==401,(target,exc.code)
 else:raise AssertionError('Public tenant API unexpectedly available')
assert b'<html' in fetch('/lab/dashboard-preview')
print('R962_REAL_PRIVATE_MASTER_POP_VENDOR_AND_DOMAIN_ADMIN_HTTP=PASS',len(catalog['vendors']),'vendors',len(pops['pops']),'pops')
PY
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R962_PUBLIC_EXPOSURE_REGRESSION; exit 3; }
[[ $(systemctl --user show "$connector" -p MainPID --value) == "$connectorpid" ]] || { echo R962_DEVICE_CONNECTOR_RESTARTED; exit 3; }
[[ $(sha256sum "$credential" | cut -d ' ' -f1) == "$credential_hash" ]] || { echo R962_SEALED_DEVICE_CREDENTIAL_CHANGED; exit 3; }
[[ $(sha256sum "$inventory" | cut -d ' ' -f1) == "$inventory_hash" ]] || { echo R962_EXISTING_INVENTORY_CHANGED; exit 3; }
[[ $(curl -s --max-time 3 -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/lab) == 200 ]] || { echo R962_ORIGINAL_LAB_BROKEN; exit 3; }
rollback=0
trap - EXIT
printf 'R962_PRIVATE_MASTER_AND_DOMAIN_ADMIN_DEPLOYED=PASS\n'
