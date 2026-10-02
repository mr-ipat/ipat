#!/usr/bin/env bash
# R9.59 owner UX private-only canary; preserves existing r953 service and C320 agent.
set -Eeuo pipefail
umask 077
[[ $(id -u) != 0 ]] || { echo REFUSE_ROOT; exit 2; }
unit=ipat-r911-preview.service
connector=ipat-r945-connector.service
base=/home/openai/workspaces/ipat
bin="$HOME/.cache/ipat/r959-release/control-api"
drop="$HOME/.config/systemd/user/$unit.d/r959-owner-ui.conf"
expected=e0e8fa860e8cc8de656ec3bbd767ad8344af45994a8f235f936e373d1e9bb467
[[ -x "$bin" && $(sha256sum "$bin" | cut -d' ' -f1) == "$expected" ]] || { echo BINARY_NOT_VERIFIED; exit 2; }
[[ $(git -C "$base" rev-parse HEAD) == aca82d3d5bbb518d4352598ed8b780a00b350028 ]] || { echo CANONICAL_SOURCE_CHANGED; exit 2; }
[[ ! -e "$drop" ]] || { echo UNKNOWN_PREEXISTING_OVERRIDE; exit 2; }
[[ $(systemctl --user is-active "$unit") == active && $(systemctl --user is-active "$connector") == active ]] || { echo PREEXISTING_SERVICE_NOT_HEALTHY; exit 2; }
old=$(systemctl --user show "$unit" -p ExecStart --value)
[[ "$old" == *'/home/openai/.cache/ipat/r953-release/control-api'* ]] || { echo UNEXPECTED_ACTIVE_BINARY; exit 2; }
env=$(systemctl --user show "$unit" -p Environment --value)
for gate in IPAT_R911_PRIVATE_CANARY=YES IPAT_R940_PRIVATE_OWNER_READ=YES IPAT_CUSTOM_DOMAIN_INSTRUCTIONS=YES IPAT_CUSTOM_DOMAIN_ROUTING_READY=NO; do
 [[ "$env" == *"$gate"* ]] || { echo MISSING_PRIVATE_GATE; exit 2; }
done
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo NONPRIVATE_BIND; exit 2; }
python3 - <<'PY'
import json,urllib.request
root='http://127.0.0.1:3002'
with urllib.request.urlopen(root+'/lab/c320-owner-connection',timeout=4) as r: status=json.load(r)
assert status['target']=='DEV-01' and status['connector_online'] is True
assert status['production_adopted'] is False and status['physical_writes_enabled'] is False
assert status['draft_saved'] is True and status['credentials_enrolled'] is False
print('R959_BASELINE_DRAFT_SAVED_NO_LIVE_AUTH=PASS')
PY
rollback=1
restore() {
 if [[ "$rollback" == 1 ]]; then
  printf 'R959_ROLLBACK_OLD_BINARY\n'
  rm -f -- "$drop"
  systemctl --user daemon-reload || :
  systemctl --user restart "$unit" || :
  systemctl --user is-active --quiet "$unit" && printf 'R959_PREVIOUS_SERVICE_RESTORED=YES\n' || printf 'R959_MANUAL_RESTORE_NEEDED=YES\n'
 fi
}
trap restore EXIT
cat > "$drop" <<'UNIT'
[Service]
ExecStart=
ExecStart=/home/openai/.cache/ipat/r959-release/control-api
UNIT
chmod 0600 "$drop"
systemctl --user daemon-reload
systemctl --user restart "$unit"
ok=NO
for i in {1..18}; do
 if [[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3002/healthz) == 200 ]]; then ok=YES; break; fi
 sleep .5
done
[[ "$ok" == YES ]] || { echo R959_HEALTH_FAIL; exit 3; }
python3 - <<'PY'
import json,urllib.request,urllib.error
root='http://127.0.0.1:3002'
def read(route):
 with urllib.request.urlopen(root+route,timeout=4) as r:
  assert r.status==200
  return r.read()
assert b'id="ipat-owner-step-status"' in read('/lab/device-workbench')
assert b'IPAT: KODE TERSALIN' in read('/lab/device-workbench')
assert b'Port Telnet laboratorium :323' in read('/lab/device-workbench')
assert b'OLT SUDAH TERSIMPAN' in read('/lab/c320-connection-setup.js')
assert b'looksLikeCommand' in read('/lab/c320-connection-setup.js')
assert b'CONTROL-PLANE DOMAIN AKTIF' in read('/lab/dashboard-preview')
assert b'Tampilkan instruksi DNS' in read('/lab/domain-settings')
assert read('/healthz')==b'ok'
s=json.loads(read('/lab/c320-owner-connection'))
assert s['connector_online'] is True and s['draft_saved'] is True
assert s['credentials_enrolled'] is False and s['production_adopted'] is False
assert s['physical_writes_enabled'] is False
for blocked in ['/v1/tenant/domains','/v1/devices/DEV-01']:
 try:read(blocked)
 except urllib.error.HTTPError as ex: assert ex.code==401,(blocked,ex.code)
 else:raise AssertionError('UNAUTHORIZED_API_EXPOSED')
print('R959_ACTUAL_PRIVATE_HTTP_UI_AND_REAL_PENDING_CONNECTOR=PASS')
PY
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo PUBLIC_BIND_REGRESSION; exit 3; }
[[ $(systemctl --user is-active "$connector") == active ]] || { echo CONNECTOR_DIED; exit 3; }
[[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/lab) == 200 ]] || { echo OLD_DEMO_REGRESSION; exit 3; }
rollback=0
trap - EXIT
printf 'R959_PRIVATE_OWNER_GUI_DEPLOYED=PASS\n'
