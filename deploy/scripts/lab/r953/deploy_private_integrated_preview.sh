#!/usr/bin/env bash
# Rootless private-only :3002 rollout with automatic rollback. NO public/DNS/device changes.
set -Eeuo pipefail
umask 077
mode=--preflight
if (( $# > 0 )); then mode="$1"; fi
[[ "$mode" == --preflight || "$mode" == --apply ]] || exit 2
[[ "$(id -u)" != 0 ]] || { echo R953_REFUSE_ROOT; exit 2; }
base="$(cd "$(dirname "$0")/../../../.." && pwd -P)"
[[ "$base" == /home/openai/.cache/ipat/r953-integration/src ]] || { echo R953_WRONG_SOURCE; exit 2; }
unit=ipat-r911-preview.service
drop="$HOME/.config/systemd/user/$unit.d/r953-integrated.conf"
bin="$HOME/.cache/ipat/r953-release/control-api"
built=/home/openai/workspaces/ipat/target/release/control-api
[[ ! -e "$drop" && -x "$built" ]] || { echo R953_EXISTING_OVERRIDE_OR_MISSING_BUILD; exit 2; }
systemctl --user is-active --quiet "$unit"
systemctl --user is-active --quiet ipat-r945-connector.service
existing="$(systemctl --user show "$unit" -p ExecStart --value)"
[[ "$existing" == *"/home/openai/.cache/ipat/r949-release/control-api"* ]] || { echo R953_UNKNOWN_OLD_BINARY; exit 2; }
env="$(systemctl --user show "$unit" -p Environment --value)"
for gate in IPAT_R911_PRIVATE_CANARY=YES IPAT_R940_PRIVATE_OWNER_READ=YES IPAT_CUSTOM_DOMAIN_INSTRUCTIONS=YES IPAT_CUSTOM_DOMAIN_ROUTING_READY=NO; do
  [[ "$env" == *"$gate"* ]] || { echo "R953_MISSING_GATE:$gate"; exit 2; }
done
[[ "$(ss -H -lnt '( sport = :3002 )' | awk '{print $4}')" == 127.0.0.1:3002 ]] || { echo R953_UNEXPECTED_BIND; exit 2; }
[[ "$(curl --silent -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/lab)" == 200 ]] || exit 2
old_hash="$(sha256sum /home/openai/.cache/ipat/r949-release/control-api | awk '{print $1}')"
new_hash="$(sha256sum "$built" | awk '{print $1}')"
[[ "$new_hash" == 619a8499e6e4b44b4de40aba4654b3fe3b2fbc0b464885f94644d38982b4aa9a ]] || { echo R953_BUILD_CHECKSUM_MISMATCH; exit 2; }
echo "R953_PREFLIGHT_OK previous=$old_hash integrated=$new_hash"
[[ "$mode" == --apply ]] || exit 0
[[ "$(printenv IPAT_R953_PRIVATE_DEPLOY 2>/dev/null || true)" == YES ]] || { echo R953_EXPLICIT_APPLY_REQUIRED; exit 2; }
mkdir -p "$(dirname "$drop")" "$(dirname "$bin")"
install -m 0700 "$built" "$bin"
systemctl --user cat "$unit" > "$HOME/.cache/ipat/r953-release/previous-user-unit.txt"
rollback=yes
restore() {
  status=$?
  if [[ "$rollback" == yes ]]; then
    echo R953_AUTOMATIC_ROLLBACK
    rm -f "$drop"
    systemctl --user daemon-reload || :
    systemctl --user restart "$unit" || :
    systemctl --user is-active --quiet "$unit" && echo R953_OLD_PREVIEW_RESTORED || echo R953_MANUAL_RECOVERY_REQUIRED
  fi
  exit "$status"
}
trap restore EXIT
cat > "$drop" <<EOF
[Service]
ExecStart=
ExecStart=$bin
WorkingDirectory=$base
EOF
systemctl --user daemon-reload
systemctl --user restart "$unit"
ready=no
for i in {1..15}; do
  if curl --silent --fail --max-time 2 http://127.0.0.1:3002/healthz | grep -qx ok; then ready=yes; break; fi
  sleep 1
done
[[ "$ready" == yes ]] || { echo R953_HEALTH_FAILED; exit 1; }
python3 - <<'PY'
import json,urllib.request,urllib.error
root='http://127.0.0.1:3002'
def read(path):
    with urllib.request.urlopen(root+path,timeout=4) as r:return r.status,r.read()
for path,text in (
    ('/lab/dashboard-preview',b'CONTROL-PLANE DOMAIN AKTIF'),
    ('/lab/device-workbench',b'Domains &amp; Branding'),
    ('/lab/domain-settings',b'Tampilkan instruksi DNS')):
    status,payload=read(path)
    assert status==200 and text in payload,path
c=json.loads(read('/lab/c320-owner-connection')[1])
assert c['target']=='DEV-01' and c['production_adopted'] is False
assert c['physical_writes_enabled'] is False
req=urllib.request.Request(root+'/v1/domains/instructions',
    data=json.dumps({'hostname':'portal.customer.co.id'}).encode(),
    headers={'Content-Type':'application/json'})
with urllib.request.urlopen(req,timeout=4) as r:d=json.load(r)
assert d['routing_mode']=='a_record' and d['routing_target_known'] is True
assert d['safe_to_point_now'] is False and d['authorization_granted'] is False
assert d['routing_records'] and d['selection_reason']
for path in ('/v1/tenant/domains','/v1/devices/DEV-01'):
    try:read(path)
    except urllib.error.HTTPError as e:assert e.code==401,(path,e.code)
    else:raise AssertionError(path+' wrongly authorized')
print('R953_REAL_HTTP_DOMAIN_C320_PRIVATE_GATES=PASS')
PY
[[ "$(ss -H -lnt '( sport = :3002 )' | awk '{print $4}')" == 127.0.0.1:3002 ]] || exit 1
[[ "$(curl --silent -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/lab)" == 200 ]] || exit 1
rollback=no
trap - EXIT
echo R953_PRIVATE_INTEGRATED_PREVIEW_ACTIVATED
