#!/usr/bin/env bash
# R9.47 scoped Add Device UX rollout; only the nonroot private :3002 Axum
# service changes. Persistent C320 collector, live device, :3000 unchanged.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -u)" -ne 0 && "$(id -un)" == openai ]] || exit 4
stage=/home/openai/.cache/ipat/r947-stage/src
release=/home/openai/.cache/ipat/r947-release
unit=/home/openai/.config/systemd/user/ipat-r911-preview.service
collector=/home/openai/.config/systemd/user/ipat-r945-connector.service
new_binary="$stage/target/debug/control-api"
expected_source=1c341daf36f2235c0192cb81551dda5a4f7d8a13
expected_unit=f8168d55b93e5e48abb69adffe43ea1ec03baad07a767aa6df9664a3686bab4a
expected_collector=0577ad145ff01b38f8423901f57ae6e515ddfcf63bfe80d4e10d0f74f61b55df
expected_previous_binary=1ad39f677a9bf05acdb0443d7793d42c319ea2d7ac27965b20d292028aa6c3f4
expected_new_binary=REPLACE_WITH_BUILT_BINARY_SHA256

[[ "$(git -C "$stage" rev-parse HEAD)" == "$expected_source" ]] || exit 4
[[ "$(git -C "$stage" diff --name-only)" == apps/control-api/src/c320_live_lab.rs ]] || exit 4
[[ "$(sha256sum "$unit"|cut -d' ' -f1)" == "$expected_unit" ]] || exit 4
[[ "$(sha256sum "$collector"|cut -d' ' -f1)" == "$expected_collector" ]] || exit 4
[[ "$(sha256sum /home/openai/.cache/ipat/r946-release/control-api|cut -d' ' -f1)" == "$expected_previous_binary" ]] || exit 4
[[ "$(sha256sum "$new_binary"|cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]] || exit 4
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler
o=build_opener(ProxyHandler({}))
with o.open('http://127.0.0.1:3002/lab/c320-owner-connection',timeout=4) as r:s=json.load(r)
assert s['target']=='DEV-01' and s['production_adopted'] is False
assert not s['credentials_enrolled'], 'owner device enrolled: revise deployment plan'
PY
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
if [[ "$mode" == --check ]];then
 echo R947_PRIVATE_ADD_DEVICE_PREFLIGHT_PASS
 exit 0
fi
[[ "${IPAT_R947_PRIVATE_DEPLOY_APPROVED:-}" == YES ]] || exit 4
install -d -m0700 "$release"
install -m0700 "$new_binary" "$release/control-api"
install -m0600 "$unit" "$release/rollback-r946.service"
python3 - "$unit" "$release/next.service" <<'PY'
from pathlib import Path
import sys
a=Path(sys.argv[1]).read_text()
assert a.count('WorkingDirectory=/home/openai/.cache/ipat/r946-stage/src')==1
assert a.count('ExecStart=/home/openai/.cache/ipat/r946-release/control-api')==1
a=a.replace('WorkingDirectory=/home/openai/.cache/ipat/r946-stage/src',
            'WorkingDirectory=/home/openai/.cache/ipat/r947-stage/src')
a=a.replace('ExecStart=/home/openai/.cache/ipat/r946-release/control-api',
            'ExecStart=/home/openai/.cache/ipat/r947-release/control-api')
p=Path(sys.argv[2]);p.write_text(a);p.chmod(0o600)
PY
rollback(){
 trap - ERR
 install -m0600 "$release/rollback-r946.service" "$unit"
 systemctl --user daemon-reload
 systemctl --user reset-failed ipat-r911-preview.service
 systemctl --user restart ipat-r911-preview.service || true
 echo R947_PRIVATE_UI_AUTO_ROLLED_BACK_TO_R946 >&2
}
trap rollback ERR
install -m0600 "$release/next.service" "$unit"
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
ready=no
for i in $(seq 1 24);do
 if curl --noproxy '*' -fsS --max-time 2 http://127.0.0.1:3002/lab/device-workbench >/dev/null 2>&1;then
  ready=yes;break
 fi
 sleep 0.5
done
[[ "$ready" == yes ]]
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler,Request
from urllib.error import HTTPError
o=build_opener(ProxyHandler({}));base='http://127.0.0.1:3002'
with o.open(base+'/lab/device-workbench',timeout=4) as r:html=r.read()
for term in (b'Add Device',b'Device Type',b'Vendor / Model',b'Management IP',
             b'SSH Port',b'Username',b'Password',b'Advanced Security',
             b'id="ipat-device-host"',b'id="ipat-device-username"'):
 assert term in html,term
with o.open(base+'/lab/c320-connection-setup.js',timeout=4) as r:js=r.read()
assert b'Coming Soon' in js and b'TARGET_NOT_ALLOWLISTED' not in js
with o.open(base+'/lab/c320-owner-connection',timeout=4) as r:status=json.load(r)
assert status['device_status']=='PENDING' and status['adoption_state']=='NOT_ENROLLED'
assert status['physical_writes_enabled'] is False
bad=Request(base+'/lab/c320-owner-enroll',method='POST',
 data=json.dumps({'device_profile':'zte_c320_lab','device_type':'olt','device_name':'Sample',
  'management_ip':'192.0.2.99','ssh_port':2222,'username':'synthetic',
  'bootstrap_code':'synthetic_123456789012345678901234567890','password':'SYNTHETIC'}).encode(),
 headers={'Content-Type':'application/json','Origin':base,'X-IPAT-Demo-Only':'1'})
try:o.open(bad,timeout=4)
except HTTPError as e:assert e.code in (400,403),e.code
else:raise AssertionError('unsupported management target was accepted')
for action in ('PROVISION_ONTS','REBOOT_OLT','UPGRADE_OLT_FIRMWARE'):
 try:o.open(Request(base+'/lab/c320-actions/'+action,data=b'{}',method='POST'),timeout=4)
 except HTTPError as e:assert e.code==403,(action,e.code)
 else:raise AssertionError('physical changes should remain locked')
with o.open('http://127.0.0.1:3000/healthz',timeout=4) as r:assert r.status==200
print('R947_ADD_DEVICE_LIVE_HTTP_PASS; BAD_TARGET_DENIED; DEVICE_NOT_ENROLLED; HW_WRITES_DENIED')
PY
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]]
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]]
trap - ERR
echo R947_ADD_DEVICE_PRIVATE_GUI_DEPLOYED
