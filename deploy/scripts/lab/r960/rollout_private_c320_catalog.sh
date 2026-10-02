#!/usr/bin/env bash
# One-time exact binary pin, rootless owner-private C320 catalog rollout.
set -Eeuo pipefail
umask 077
[[ $(id -u) != 0 ]] || { echo R960_REFUSE_ROOT; exit 2; }
unit=ipat-r911-preview.service
connector=ipat-r945-connector.service
base=/home/openai/workspaces/ipat
bin="$HOME/.cache/ipat/r960-release/control-api"
drop="$HOME/.config/systemd/user/$unit.d/r960-c320-catalog.conf"
expected=715c68b8006276a3255c3a9f0738c088715e0aafaebb58ac63b20bf1d1824591
[[ -x "$bin" && $(sha256sum "$bin" | cut -d' ' -f1) == "$expected" ]] || { echo R960_UNKNOWN_BINARY; exit 2; }
[[ $(git -C "$base" rev-parse HEAD) == bc7e8f522f0c67a1cc6fb0554cac8e14b3f7af9d ]] || { echo R960_SOURCE_DRIFT; exit 2; }
[[ ! -e "$drop" ]] || { echo R960_OVERRIDE_EXISTS; exit 2; }
[[ $(systemctl --user is-active "$unit") == active && $(systemctl --user is-active "$connector") == active ]] || { echo R960_SERVICE_NOT_ACTIVE; exit 2; }
[[ $(systemctl --user show "$unit" -p ExecStart --value) == *'/home/openai/.cache/ipat/r959-release/control-api'* ]] || { echo R960_OLD_BINARY_UNEXPECTED; exit 2; }
env=$(systemctl --user show "$unit" -p Environment --value)
for gate in IPAT_R911_PRIVATE_CANARY=YES IPAT_R940_PRIVATE_OWNER_READ=YES IPAT_CUSTOM_DOMAIN_INSTRUCTIONS=YES IPAT_CUSTOM_DOMAIN_ROUTING_READY=NO; do
 [[ "$env" == *"$gate"* ]] || { echo R960_MISSING_PRIVATE_GATE; exit 2; }
done
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R960_NONPRIVATE_BIND; exit 2; }
original_connector_pid=$(systemctl --user show "$connector" -p MainPID --value)
python3 - <<'PY'
import json,urllib.request
with urllib.request.urlopen('http://127.0.0.1:3002/lab/c320-owner-connection',timeout=4) as r:s=json.load(r)
assert s['connector_online'] is True and s['credentials_enrolled'] is True
assert s['production_adopted'] is False and s['physical_writes_enabled'] is False
print('R960_BASELINE_LIVE_C320_CONNECTED_NO_PHYSICAL_WRITES=PASS')
PY
rollback=1
restore() {
 if [[ "$rollback" == 1 ]]; then
   echo R960_AUTO_RESTORE_PREVIOUS_R959
   rm -f -- "$drop"
   systemctl --user daemon-reload || :
   systemctl --user restart "$unit" || :
   systemctl --user is-active --quiet "$unit" && echo R960_OLD_PRIVATE_UI_RESTORED || echo R960_MANUAL_RESTORE_NEEDED
 fi
}
trap restore EXIT
cat > "$drop" <<'UNIT'
[Service]
ExecStart=
ExecStart=/home/openai/.cache/ipat/r960-release/control-api
UNIT
chmod 0600 "$drop"
systemctl --user daemon-reload
systemctl --user restart "$unit"
ready=NO
for i in {1..18}; do
 if [[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3002/healthz) == 200 ]]; then ready=YES; break; fi
 sleep .5
done
[[ "$ready" == YES ]] || { echo R960_HEALTH_FAIL; exit 3; }
python3 - <<'PY'
import json,urllib.request,urllib.error
root='http://127.0.0.1:3002'
def read(path):
 with urllib.request.urlopen(root+path,timeout=5) as r:
  assert r.status==200
  return r.read()
html=read('/lab/device-workbench')
assert b'id="c320-capabilities"' in html and b'Capability &amp; Execution Center' in html
assert b'IPAT: KODE TERSALIN' in html # R9.59 retained
js=read('/lab/c320-action-catalog.js')
assert b'Object.freeze' in js and b'AVAILABLE_READ_ONLY' in js
catalog=json.loads(read('/lab/c320-owner-action-catalog'))
assert catalog['connected'] is True and catalog['mode']=='OWNER_PRIVATE_LAB_CAPABILITIES'
assert catalog['production_adopted'] is False and catalog['physical_writes_enabled'] is False
entries=catalog['entries']
assert len(entries)==16,len(entries)
ready=[e for e in entries if e['state']=='AVAILABLE_READ_ONLY']
assert [(e['id'],e['endpoint']) for e in ready] == [
 ('cards','/lab/c320-owner-live-cards'),
 ('firmware','/lab/c320-owner-live-firmware')]
assert next(e for e in entries if e['id']=='onu_counts')['state']=='DEGRADED'
for e in entries:
 if e['operation']!='READ':
  assert e['endpoint'] is None and e['state']!='AVAILABLE_READ_ONLY',e['id']
assert b'CONTROL-PLANE DOMAIN AKTIF' in read('/lab/dashboard-preview')
assert b'Tampilkan instruksi DNS' in read('/lab/domain-settings')
assert read('/healthz')==b'ok'
for path in ('/v1/tenant/domains','/v1/devices/DEV-01'):
 try:read(path)
 except urllib.error.HTTPError as exc: assert exc.code==401,(path,exc.code)
 else:raise AssertionError('unauthorized API opened '+path)
print('R960_LIVE_PRIVATE_HTTP_CAPABILITY_CENTER=PASS_ENTRIES_16_EXECUTABLE_READS_2_NO_WRITES')
PY
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R960_PUBLIC_BIND_REGRESSION; exit 3; }
[[ $(systemctl --user show "$connector" -p MainPID --value) == "$original_connector_pid" ]] || { echo R960_CONNECTOR_RESTARTED_UNEXPECTED; exit 3; }
[[ $(systemctl --user is-active "$connector") == active ]] || { echo R960_CONNECTOR_DIED; exit 3; }
[[ $(curl -s --max-time 3 -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/lab) == 200 ]] || { echo R960_OLDER_LAB_REGRESSION; exit 3; }
rollback=0
trap - EXIT
printf 'R960_PRIVATE_C320_OPERATIONS_GUI_DEPLOYED=PASS\n'
