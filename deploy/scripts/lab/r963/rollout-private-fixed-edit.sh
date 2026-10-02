#!/usr/bin/env bash
# One-time private-only connected C320 Edit+Site validity hotfix.
set -Eeuo pipefail
umask 077
[[ $(id -u) != 0 ]] || { echo R963B_REFUSE_ROOT;exit 2; }
base=/home/openai/workspaces/ipat
expected_base=dcd2e6abc9e4a790e574229b63b23a3cb69932b2
prior=/home/openai/.cache/ipat/r963-release/control-api
prior_sha=c3d328029461abf283cd52afcea4ddfabe05063c7530d30b06a911ef10f06e8e
next=/home/openai/.cache/ipat/r963-fix-release/control-api
next_sha=b2cbc8f218c61ccf21c08e720b9f7154bf49f7d3b1d27828cec0e5d90bf38b92
unit=ipat-r911-preview.service
agent=ipat-r945-connector.service
old_drop="$HOME/.config/systemd/user/$unit.d/r963-standalone-sites.conf"
drop="$HOME/.config/systemd/user/$unit.d/r963b-edit-site-form-validity.conf"
cred="$HOME/.local/share/ipat/r945-connection/device.fernet"
inventory="$HOME/.local/share/ipat/r961-device-inventory/devices.json"
domains="$HOME/.local/share/ipat/r962-domain-control/settings.json"
[[ $(git -C "$base" rev-parse HEAD) == "$expected_base" ]] || { echo R963B_SOURCE_DRIFT;exit 2; }
[[ -z $(git -C "$base" status --porcelain) ]] || { echo R963B_SOURCE_DIRTY;exit 2; }
[[ -f "$old_drop" && ! -e "$drop" ]] || { echo R963B_OLD_OVERRIDE_MISSING;exit 2; }
[[ $(sha256sum "$prior" | cut -d' ' -f1) == "$prior_sha" ]] || { echo R963B_OLD_BINARY_UNKNOWN;exit 2; }
[[ $(sha256sum "$next" | cut -d' ' -f1) == "$next_sha" ]] || { echo R963B_NEW_BINARY_UNKNOWN;exit 2; }
[[ $(systemctl --user is-active "$unit") == active && $(systemctl --user is-active "$agent") == active ]] || { echo R963B_SERVICE_INACTIVE;exit 2; }
[[ $(systemctl --user show "$unit" -p ExecStart --value) == *'/home/openai/.cache/ipat/r963-release/control-api'* ]] || { echo R963B_WRONG_PRIOR_RELEASE;exit 2; }
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R963B_PUBLIC_BIND_FORBIDDEN;exit 2; }
pid=$(systemctl --user show "$agent" -p MainPID --value)
hash_cred=$(sha256sum "$cred"|cut -d' ' -f1)
hash_inv=$(sha256sum "$inventory"|cut -d' ' -f1)
hash_domains=$(sha256sum "$domains"|cut -d' ' -f1)
rollback=1
restore() { if [[ "$rollback" == 1 ]];then
  echo R963B_REVERT_TO_R963
  rm -f -- "$drop"
  systemctl --user daemon-reload||:
  systemctl --user restart "$unit"||:
fi; }
trap restore EXIT
cat > "$drop" <<'UNIT'
[Service]
ExecStart=
ExecStart=/home/openai/.cache/ipat/r963-fix-release/control-api
UNIT
chmod 0600 "$drop"
systemctl --user daemon-reload
systemctl --user restart "$unit"
ready=NO
for _ in {1..30};do
 if [[ $(curl -s --max-time 2 -o /dev/null -w '%{http_code}' http://127.0.0.1:3002/healthz) == 200 ]];then ready=YES;break;fi
 sleep .4
done
[[ "$ready" == YES ]] || { echo R963B_PRIVATE_SERVICE_FAILED;exit 3; }
python3 - <<'PY'
import json,urllib.request
base='http://127.0.0.1:3002';get=lambda p:urllib.request.urlopen(base+p,timeout=5).read()
script=get('/lab/owner-device-inventory.js')
assert b'function toggleExtraControls(linked)' in script
assert b'input.disabled=linked' in script
assert b"field('pop').disabled=false" in script
assert b'/lab/sites?return=device' in script
assert b'id="ipat-sites-form"' in get('/lab/sites')
assert b'id="ipat-owner-pop-form"' not in get('/lab/device-workbench')
assert json.loads(get('/lab/owner/platform-domains'))['platform']['safe_to_point_now'] is False
pops=json.loads(get('/lab/owner/pops'))
assert pops['persistent'] and isinstance(pops['pops'],list)
d=json.loads(get('/lab/owner/devices'))
assert any(item['id']=='DEV-01' and item['connection_state']=='CONNECTED' for item in d['devices'])
print('R963B_LIVE_FIXED_C320_EDIT_SITE_WITH_INDEPENDENT_SITES=PASS')
PY
[[ $(ss -H -lnt '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || { echo R963B_PUBLIC_BIND;exit 3; }
[[ $(systemctl --user show "$agent" -p MainPID --value) == "$pid" ]] || { echo R963B_CONNECTOR_CHANGED;exit 3; }
[[ $(sha256sum "$cred"|cut -d' ' -f1) == "$hash_cred" ]] || { echo R963B_CRED_CHANGED;exit 3; }
[[ $(sha256sum "$inventory"|cut -d' ' -f1) == "$hash_inv" ]] || { echo R963B_INVENTORY_CHANGED_ON_DEPLOY;exit 3; }
[[ $(sha256sum "$domains"|cut -d' ' -f1) == "$hash_domains" ]] || { echo R963B_DOMAIN_CHANGED;exit 3; }
rollback=0
trap - EXIT
echo R963B_EXACT_SOURCE_PRIVATE_FIXED_C320_EDIT_DEPLOYED=PASS
