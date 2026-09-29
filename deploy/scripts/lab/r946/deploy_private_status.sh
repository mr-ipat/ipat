#!/usr/bin/env bash
# R9.46 release: scoped non-root PRIVATE :3002 status indicator and persisted
# read-connector upgrade. No physical OLT commands or public :3000 changes.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -u)" != 0 && "$(id -un)" == openai ]] || exit 4

src=/home/openai/.cache/ipat/r946-stage/src
release=/home/openai/.cache/ipat/r946-release
preview=/home/openai/.config/systemd/user/ipat-r911-preview.service
collector=/home/openai/.config/systemd/user/ipat-r945-connector.service
old_binary=/home/openai/.cache/ipat/r945-release/control-api
new_binary="$src/target/debug/control-api"

expected_source=d0a605238b3cdcdb6e85a49e0e249a8724b1ab06
expected_preview=fdab0505685fb2424f35e59268132f0ab949ffed31ba872b628cbe7e3899ffe7
expected_collector=c5bf88f302a4eb04cf5091ace001f9848492b3cbfdeb034c2bb8daa40bc3be0e
expected_old_binary=7d8861a889a60f6f332d18dac74f0b76b1032ddbe267bea806bca4b08ef9729c
expected_new_binary=1ad39f677a9bf05acdb0443d7793d42c319ea2d7ac27965b20d292028aa6c3f4

[[ "$(git -C "$src" rev-parse HEAD)" == "$expected_source" ]] || exit 4
[[ "$(sha256sum "$preview"|cut -d' ' -f1)" == "$expected_preview" ]] || exit 4
[[ "$(sha256sum "$collector"|cut -d' ' -f1)" == "$expected_collector" ]] || exit 4
[[ "$(sha256sum "$old_binary"|cut -d' ' -f1)" == "$expected_old_binary" ]] || exit 4
[[ "$(sha256sum "$new_binary"|cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]] || exit 4
# Protect owner's current enrollment: this upgrade should not interrupt any
# active operator request or a newly enrolled OLT.
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler
o=build_opener(ProxyHandler({}))
with o.open('http://127.0.0.1:3002/lab/c320-owner-connection',timeout=3) as r:
    state=json.load(r)
assert state['target']=='DEV-01' and state['production_adopted'] is False
assert state['credentials_enrolled'] is False, 'owner enrolled; replan upgrade'
PY
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
if [[ "$mode" == --check ]];then
  echo R946_RESTRICTED_PREVIEW_UPGRADE_PREFLIGHT_PASS
  exit 0
fi
[[ "${IPAT_R946_PRIVATE_UPGRADE_APPROVED:-}" == YES ]] || exit 4

install -d -m0700 "$release"
install -m0700 "$new_binary" "$release/control-api"
install -m0600 "$preview" "$release/rollback-preview.service"
install -m0600 "$collector" "$release/rollback-connector.service"
python3 - "$preview" "$collector" "$release" <<'PY'
import sys
from pathlib import Path
preview,collector,destination=map(Path,sys.argv[1:])
a=preview.read_text()
b=collector.read_text()
x='WorkingDirectory=/home/openai/.cache/ipat/r945-stage/src'
y='ExecStart=/home/openai/.cache/ipat/r945-release/control-api'
z='/home/openai/.cache/ipat/r945-stage/src/deploy/scripts/lab/r945/persistent_c320_connector.py'
assert a.count(x)==1 and a.count(y)==1
assert b.count(z)==1
a=a.replace(x,'WorkingDirectory=/home/openai/.cache/ipat/r946-stage/src')
a=a.replace(y,'ExecStart=/home/openai/.cache/ipat/r946-release/control-api')
b=b.replace(z,'/home/openai/.cache/ipat/r946-stage/src/deploy/scripts/lab/r945/persistent_c320_connector.py')
for name,txt in [('next-preview.service',a),('next-connector.service',b)]:
    p=destination/name
    p.write_text(txt)
    p.chmod(0o600)
PY

rollback() {
  trap - ERR
  install -m0600 "$release/rollback-preview.service" "$preview"
  install -m0600 "$release/rollback-connector.service" "$collector"
  systemctl --user daemon-reload
  systemctl --user restart ipat-r945-connector.service || true
  systemctl --user restart ipat-r911-preview.service || true
  echo R946_PRIVATE_ROLLBACK_TO_PREVIOUS_RELEASE >&2
}
trap rollback ERR
install -m0600 "$release/next-preview.service" "$preview"
install -m0600 "$release/next-connector.service" "$collector"
systemctl --user daemon-reload
systemctl --user restart ipat-r945-connector.service
systemctl --user restart ipat-r911-preview.service
ready=no
for attempt in $(seq 1 24);do
  if curl --noproxy '*' -fsS --max-time 2 http://127.0.0.1:3002/lab/device-status-indicators.js >/dev/null 2>&1;then
    ready=yes
    break
  fi
  sleep 0.5
done
[[ "$ready" == yes ]]
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler,Request
from urllib.error import HTTPError
o=build_opener(ProxyHandler({}))
base='http://127.0.0.1:3002'
with o.open(base+'/lab/device-workbench',timeout=4) as r:
    html=r.read()
    assert b'data-ipat-device-status="DEV-01"' in html
    assert b'/lab/device-status-indicators.js' in html
with o.open(base+'/lab/device-status-indicators.js',timeout=4) as r:
    script=r.read()
    assert b'--connected' in script and b'--disconnected' in script
with o.open(base+'/lab/c320-owner-connection',timeout=4) as r:
    v=json.load(r)
    assert (v['device_status'],v['adoption_state'])==('PENDING','NOT_ENROLLED')
    assert v['connector_online'] and not v['credentials_enrolled']
    assert v['physical_writes_enabled'] is False
for action in ('PROVISION_ONTS','REBOOT_OLT','UPGRADE_OLT_FIRMWARE'):
    try:o.open(Request(base+'/lab/c320-actions/'+action,method='POST',data=b'{}'),timeout=3)
    except HTTPError as e:assert e.code==403,(action,e.code)
    else:raise AssertionError('hardware change unexpectedly available')
with o.open('http://127.0.0.1:3000/healthz',timeout=3) as r:
    assert r.status==200
print('R946_PRIVATE_DEVICE_STATUS_HTTP_PASS_PENDING_NOT_FALSE_GREEN_WRITES_DENIED')
PY
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]]
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]]
trap - ERR
echo R946_FOUR_COLOR_INDICATOR_DEPLOYED_PRIVATE_ONLY
