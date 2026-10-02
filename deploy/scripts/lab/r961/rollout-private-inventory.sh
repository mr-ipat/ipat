#!/usr/bin/env bash
# One-time exact-source rootless private owner inventory with rollback.
set -Eeuo pipefail
umask 077
[[ $(id -u) != 0 ]] || { echo R961_REFUSE_ROOT; exit 2; }
base=/home/openai/workspaces/ipat
old=ff47849aebb08dd794f33f26bbb7de788c2c518d
bin="$HOME/.cache/ipat/r961-release/control-api"
expected=56a6cb2fdbea7326ec21ac6b30e36a886109063b40321724ab99e19658c94a17
unit=ipat-r911-preview.service
connector=ipat-r945-connector.service
drop="$HOME/.config/systemd/user/$unit.d/r961-owner-device-crud.conf"
[[ -x "$bin" && $(sha256sum "$bin" | cut -d ' ' -f1) == "$expected" ]] || { echo R961_UNVERIFIED_BINARY; exit 2; }
[[ $(git -C "$base" rev-parse HEAD) == "$old" ]] || { echo R961_CANONICAL_DRIFT; exit 2; }
[[ ! -e "$drop" ]] || { echo R961_DUPLICATE_OVERRIDE; exit 2; }
[[ $(systemctl --user is-active "$unit") == active && $(systemctl --user is-active "$connector") == active ]] || { echo R961_SERVICES_NOT_ACTIVE; exit 2; }
[[ $(systemctl --user show "$unit" -p ExecStart --value) == *'/home/openai/.cache/ipat/r960-release/control-api'* ]] || { echo R961_UNEXPECTED_PREVIOUS_RELEASE; exit 2; }
vars=$(systemctl --user show "$unit" -p Environment --value)
for gate in IPAT_R911_PRIVATE_CANARY=YES IPAT_R940_PRIVATE_OWNER_READ=YES IPAT_CUSTOM_DOMAIN_INSTRUCTIONS=YES IPAT_CUSTOM_DOMAIN_ROUTING_READY=NO; do
 [[ "$vars" == *"$gate"* ]] || { echo R961_MISSING_PRIVATE_GATE; exit 2; }
done
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R961_NONPRIVATE_BIND; exit 2; }
connector_pid=$(systemctl --user show "$connector" -p MainPID --value)
credential=/home/openai/.local/share/ipat/r945-connection/device.fernet
test -f "$credential"
original_credential_sha=$(sha256sum "$credential" | cut -d ' ' -f1)
python3 - <<'PY'
import json,urllib.request
base='http://127.0.0.1:3002'
s=json.load(urllib.request.urlopen(base+'/lab/c320-owner-connection',timeout=5))
assert s['connector_online'] is True and s['credentials_enrolled'] is True
assert s['device_status']=='CONNECTED' and s['physical_writes_enabled'] is False
print('R961_BASELINE_REAL_C320_CONNECTED_SEALED=PASS')
PY
rollback=1
recover() {
 if [[ "$rollback" == 1 ]]; then
   echo R961_ROLLBACK_PREVIOUS_R960_UI
   rm -f -- "$drop"
   systemctl --user daemon-reload || :
   systemctl --user restart "$unit" || :
   systemctl --user is-active --quiet "$unit" && echo R961_PREVIOUS_GUI_RESTORED || echo R961_MANUAL_RESTORE_REQUIRED
 fi
}
trap recover EXIT
cat > "$drop" <<'UNIT'
[Service]
ExecStart=
ExecStart=/home/openai/.cache/ipat/r961-release/control-api
UNIT
chmod 0600 "$drop"
systemctl --user daemon-reload
systemctl --user restart "$unit"
ready=NO
for i in {1..24}; do
 if [[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3002/healthz) == 200 ]];then ready=YES;break;fi
 sleep .4
done
[[ "$ready" == YES ]] || { echo R961_SERVER_HEALTH_FAILED; exit 3; }
python3 - <<'PY'
import json,urllib.request,urllib.error
base='http://127.0.0.1:3002'
def get(path,headers=None):
 with urllib.request.urlopen(urllib.request.Request(base+path,headers=headers or {}),timeout=5) as r:return r.read()
page=get('/lab/device-workbench')
assert b'id="managed-devices"' in page and b'owner-device-inventory.js' in page
assert b'Capability &amp; Execution Center' in page
assert b'IPAT: KODE TERSALIN' in page
script=get('/lab/owner-device-inventory.js')
assert b'SAVED \xc2\xb7 NOT CONNECTED' in script
assert b'expected_revision' in script and b'method:\'DELETE\'' in script
registry=json.loads(get('/lab/owner/devices'))
assert registry['persistent'] is True and registry['production_tenant_registry'] is False
assert registry['owner_private_lab_only'] is True
linked=[d for d in registry['devices'] if d['id']=='DEV-01']
assert len(linked)==1 and linked[0]['linked_live_connector'] is True
assert linked[0]['connection_state']=='CONNECTED'
assert linked[0]['physical_writes_enabled'] is False
for path in ['/v1/tenant/domains','/v1/devices/DEV-01']:
 try:get(path)
 except urllib.error.HTTPError as e:assert e.code==401,(path,e.code)
 else:raise AssertionError('PRODUCTION_API_EXPOSED')
print('R961_LIVE_PRIVATE_MANAGED_DEVICES_UI_AND_LINKED_C320=PASS')
PY
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R961_PUBLIC_BIND_REGRESSION; exit 3; }
[[ $(systemctl --user show "$connector" -p MainPID --value) == "$connector_pid" ]] || { echo R961_CONNECTOR_RESTARTED; exit 3; }
[[ $(sha256sum "$credential" | cut -d ' ' -f1) == "$original_credential_sha" ]] || { echo R961_SEALED_CREDENTIAL_CHANGED; exit 3; }
[[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/lab) == 200 ]] || { echo R961_OLD_LAB_BROKEN; exit 3; }
rollback=0
trap - EXIT
printf 'R961_OWNER_PRIVATE_PERSISTENT_CRUD_DEPLOYED=PASS\n'
