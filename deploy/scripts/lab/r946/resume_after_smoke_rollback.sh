#!/usr/bin/env bash
# R9.46 narrow resume after original scoped rollback:
# original script's JS smoke expected static classes though JS generates them.
# ONLY reuses existing trusted staged binaries/units; no new bootstrap or CLI.
set -Eeuo pipefail
umask 077
[[ "$(id -u)" != 0 && "$(id -un)" == openai ]] || exit 4
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
release=/home/openai/.cache/ipat/r946-release
preview=/home/openai/.config/systemd/user/ipat-r911-preview.service
connector=/home/openai/.config/systemd/user/ipat-r945-connector.service
src=/home/openai/.cache/ipat/r946-stage/src
[[ "$(git -C "$src" rev-parse HEAD)" == d0a605238b3cdcdb6e85a49e0e249a8724b1ab06 ]] || exit 4
[[ "$(sha256sum "$preview"|cut -d' ' -f1)" == fdab0505685fb2424f35e59268132f0ab949ffed31ba872b628cbe7e3899ffe7 ]] || exit 4
[[ "$(sha256sum "$connector"|cut -d' ' -f1)" == c5bf88f302a4eb04cf5091ace001f9848492b3cbfdeb034c2bb8daa40bc3be0e ]] || exit 4
[[ "$(sha256sum "$release/control-api"|cut -d' ' -f1)" == 1ad39f677a9bf05acdb0443d7793d42c319ea2d7ac27965b20d292028aa6c3f4 ]] || exit 4
cmp "$preview" "$release/rollback-preview.service"
cmp "$connector" "$release/rollback-connector.service"
[[ "$(stat -c %a "$release")" == 700 ]] || exit 4
for p in "$release/next-preview.service" "$release/next-connector.service";do
  [[ "$(stat -c %a "$p")" == 600 && "$(stat -c %U "$p")" == openai ]] || exit 4
done
grep -Fxq 'ExecStart=/home/openai/.cache/ipat/r946-release/control-api' "$release/next-preview.service"
grep -Fq '/home/openai/.cache/ipat/r946-stage/src/deploy/scripts/lab/r945/persistent_c320_connector.py --serve' "$release/next-connector.service"
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]] || exit 4
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler
o=build_opener(ProxyHandler({}))
with o.open('http://127.0.0.1:3002/lab/c320-owner-connection',timeout=4) as r:
    v=json.load(r)
assert v['target']=='DEV-01' and v['credentials_enrolled'] is False
PY
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
if [[ "$mode" == --check ]];then
  echo R946_SAFE_RESUME_PREFLIGHT_PASS
  exit 0
fi
[[ "${IPAT_R946_PRIVATE_RESUME_APPROVED:-}" == YES ]] || exit 4

rollback(){
  trap - ERR
  install -m0600 "$release/rollback-preview.service" "$preview"
  install -m0600 "$release/rollback-connector.service" "$connector"
  systemctl --user daemon-reload
  systemctl --user restart ipat-r945-connector.service || true
  systemctl --user restart ipat-r911-preview.service || true
  echo R946_RESUME_AUTO_ROLLED_BACK >&2
}
trap rollback ERR
install -m0600 "$release/next-preview.service" "$preview"
install -m0600 "$release/next-connector.service" "$connector"
systemctl --user daemon-reload
systemctl --user restart ipat-r945-connector.service
systemctl --user restart ipat-r911-preview.service
ready=no
for i in $(seq 1 25);do
  if curl --noproxy '*' -fsS --max-time 2 http://127.0.0.1:3002/lab/device-status-indicators.js >/dev/null 2>&1;then
    ready=yes;break
  fi
  sleep 0.5
done
[[ "$ready" == yes ]]
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler,Request
from urllib.error import HTTPError
base='http://127.0.0.1:3002'
o=build_opener(ProxyHandler({}))
with o.open(base+'/lab/device-workbench',timeout=4) as r:
    page=r.read()
    assert b'data-ipat-device-status="DEV-01"' in page
    assert b'/lab/device-status-indicators.js' in page
with o.open(base+'/lab/device-status-indicators.js',timeout=4) as r:
    js=r.read()
    assert b'ipat-device-signal--' in js
    assert b'CONNECTED' in js and b'DISCONNECTED' in js
with o.open(base+'/lab/c320-owner-connection',timeout=4) as r:
    state=json.load(r)
assert state['device_status']=='PENDING' and state['adoption_state']=='NOT_ENROLLED'
assert state['connector_online'] and state['credentials_enrolled'] is False
assert state['production_adopted'] is False and state['physical_writes_enabled'] is False
for action in ('PROVISION_ONTS','REBOOT_OLT','UPGRADE_OLT_FIRMWARE'):
    try:o.open(Request(base+'/lab/c320-actions/'+action,method='POST',data=b'{}'),timeout=4)
    except HTTPError as e: assert e.code==403,(action,e.code)
    else:raise AssertionError('physical write incorrectly enabled')
with o.open('http://127.0.0.1:3000/healthz',timeout=4) as r:assert r.status==200
print('R946_STATUS_HTTP_SMOKE_PASS; PENDING_NOT_FALSE_GREEN; ALL_HARDWARE_WRITES_403')
PY
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]]
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]]
trap - ERR
echo R946_FOUR_COLOR_DEV01_PRIVATE_UI_ACTUALLY_DEPLOYED
