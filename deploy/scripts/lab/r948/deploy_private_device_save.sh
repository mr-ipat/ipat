#!/usr/bin/env bash
# R9.48 bounded PRIVATE :3002 rollout: saved real device draft + connection
# diagnostics. No OLT/ONT writes, no new access paths, no public :3000 changes.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -un)" == openai && "$(id -u)" -ne 0 ]] || exit 4
source_dir=/home/openai/.cache/ipat/r948-stage/src
release=/home/openai/.cache/ipat/r948-release
preview=/home/openai/.config/systemd/user/ipat-r911-preview.service
collector=/home/openai/.config/systemd/user/ipat-r945-connector.service
old_binary=/home/openai/.cache/ipat/r947-release/control-api
new_binary="$source_dir/target/debug/control-api"
expected_source=fc881425affdc1130fdc61905b05d6fcf1c87195
expected_preview=47a981335a2a53b8ba8ebb1a40faf2755c0fa697589f67fb5b608688af2b40ab
expected_collector=0577ad145ff01b38f8423901f57ae6e515ddfcf63bfe80d4e10d0f74f61b55df
expected_old_binary=46717389d9104367bc89cc2c1bc373c137a1bba569fcadc0a651dcba8f1082d4
expected_new_binary=REPLACE_WITH_R948_VERIFIED_BUILD_SHA256

[[ "$(git -C "$source_dir" rev-parse HEAD)" == "$expected_source" ]] || exit 4
[[ -z "$(git -C "$source_dir" status --porcelain)" ]] || exit 4
[[ "$(sha256sum "$preview"|cut -d' ' -f1)" == "$expected_preview" ]] || exit 4
[[ "$(sha256sum "$collector"|cut -d' ' -f1)" == "$expected_collector" ]] || exit 4
[[ "$(sha256sum "$old_binary"|cut -d' ' -f1)" == "$expected_old_binary" ]] || exit 4
[[ "$(sha256sum "$new_binary"|cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]] || exit 4
python3 - <<'PY'
import json, os, socket
from urllib.request import build_opener, ProxyHandler
o=build_opener(ProxyHandler({}))
with o.open('http://127.0.0.1:3002/lab/c320-owner-connection',timeout=4) as r:
    v=json.load(r)
assert v['target']=='DEV-01' and v['production_adopted'] is False
assert v['credentials_enrolled'] is False, 'Device became enrolled: stop and replan'
sock='/home/openai/.local/share/ipat/r940-live-agent/live.sock'
s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
s.settimeout(3)
s.connect(sock)
s.sendall(b'STATUS\n')
reply=s.recv(1024);s.close()
state=json.loads(reply)
assert state['read_in_progress'] is False
assert state['credentials_enrolled'] is False
PY
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
if [[ "$mode" == --check ]];then
 echo R948_PRIVATE_SAVE_AND_DIAGNOSTIC_PREFLIGHT_PASS
 exit 0
fi
[[ "${IPAT_R948_PRIVATE_DEPLOY_APPROVED:-}" == YES ]] || exit 4

install -d -m0700 "$release"
install -m0700 "$new_binary" "$release/control-api"
install -m0600 "$preview" "$release/rollback-r947-preview.service"
install -m0600 "$collector" "$release/rollback-r946-connector.service"
python3 - "$preview" "$collector" "$release" <<'PY'
from pathlib import Path
import sys
old_ui,old_agent,out=map(Path,sys.argv[1:])
u=old_ui.read_text();c=old_agent.read_text()
old_ui_work='WorkingDirectory=/home/openai/.cache/ipat/r947-stage/src'
old_ui_exec='ExecStart=/home/openai/.cache/ipat/r947-release/control-api'
old_daemon='/home/openai/.cache/ipat/r946-stage/src/deploy/scripts/lab/r945/persistent_c320_connector.py'
old_agent_work='WorkingDirectory=/home/openai/.cache/ipat/r945-stage/src'
assert u.count(old_ui_work)==u.count(old_ui_exec)==1
assert c.count(old_daemon)==c.count(old_agent_work)==1
u=u.replace(old_ui_work,'WorkingDirectory=/home/openai/.cache/ipat/r948-stage/src')
u=u.replace(old_ui_exec,'ExecStart=/home/openai/.cache/ipat/r948-release/control-api')
c=c.replace(old_daemon,'/home/openai/.cache/ipat/r948-stage/src/deploy/scripts/lab/r945/persistent_c320_connector.py')
c=c.replace(old_agent_work,'WorkingDirectory=/home/openai/.cache/ipat/r948-stage/src')
for name,data in [('next-preview.service',u),('next-connector.service',c)]:
    target=out/name;target.write_text(data);target.chmod(0o600)
PY
rollback(){
 trap - ERR
 install -m0600 "$release/rollback-r947-preview.service" "$preview"
 install -m0600 "$release/rollback-r946-connector.service" "$collector"
 systemctl --user daemon-reload
 systemctl --user reset-failed ipat-r911-preview.service
 systemctl --user reset-failed ipat-r945-connector.service
 systemctl --user restart ipat-r945-connector.service || true
 systemctl --user restart ipat-r911-preview.service || true
 echo R948_AUTO_ROLLBACK_TO_PREVIOUS_PRIVATE_SERVICES >&2
}
trap rollback ERR
install -m0600 "$release/next-preview.service" "$preview"
install -m0600 "$release/next-connector.service" "$collector"
systemctl --user daemon-reload
systemctl --user restart ipat-r945-connector.service
systemctl --user restart ipat-r911-preview.service

ready=no
for i in $(seq 1 24);do
 if curl --noproxy '*' -fsS --max-time 2 http://127.0.0.1:3002/lab/c320-owner-network-probe >/dev/null 2>&1;then
   ready=yes;break
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
with o.open(base+'/lab/device-workbench',timeout=5) as r:
    html=r.read()
    assert b'Add Device' in html and b'id="ipat-device-host"' in html
with o.open(base+'/lab/c320-connection-setup.js',timeout=5) as r:
    js=r.read()
    assert b'await saveDraft(draft)' in js and b'Pending owner verification' in js
with o.open(base+'/lab/device-workbench.js',timeout=5) as r:
    list_script=r.read()
    assert b'loadSavedPhysical' in list_script and b'savedPhysical' in list_script
with o.open(base+'/lab/c320-owner-connection',timeout=4) as r:
    before=json.load(r)
assert before['credentials_enrolled'] is False and before['physical_writes_enabled'] is False
body={'device_profile':'zte_c320_lab','device_type':'olt',
      'device_name':'ZTE C320 Lab','management_ip':'10.10.13.233',
      'ssh_port':321,'username':'zte'}
headers={'Content-Type':'application/json','Origin':base,'X-IPAT-Demo-Only':'1'}
request=Request(base+'/lab/c320-owner-save-draft',data=json.dumps(body).encode(),
                method='POST',headers=headers)
with o.open(request,timeout=5) as r:
    accepted=json.load(r)
assert accepted['saved'] is True and accepted['physical_writes_enabled'] is False
with o.open(base+'/lab/c320-owner-connection',timeout=4) as r:
    pending=json.load(r)
assert pending['draft_saved'] is True and pending['device_name']=='ZTE C320 Lab'
assert pending['adoption_state']=='DRAFT_SAVED_AWAITING_AUTH'
assert pending['device_status']=='PENDING' and not pending['credentials_enrolled']
with o.open(base+'/lab/c320-owner-network-probe',timeout=6) as r:
    probe=json.load(r)
assert probe['target']=='DEV-01' and probe['ssh_authentication_verified'] is False
assert probe['probe_stage'] in ('TCP_REACHABLE_AUTH_NOT_TESTED','SSH_PORT_REFUSED',
  'NETWORK_UNREACHABLE','NETWORK_OR_PORT_ERROR','TCP_TIMEOUT_NETWORK_OR_PORT')
invalid={**body,'management_ip':'192.0.2.30','ssh_port':2222,'username':'synthetic'}
bad=Request(base+'/lab/c320-owner-save-draft',data=json.dumps(invalid).encode(),
            method='POST',headers=headers)
try:o.open(bad,timeout=4)
except HTTPError as error:assert error.code==400,error.code
else:raise AssertionError('unverified management endpoint was accepted')
for action in ('PROVISION_ONTS','REBOOT_OLT','UPGRADE_OLT_FIRMWARE'):
    try:o.open(Request(base+'/lab/c320-actions/'+action,method='POST',data=b'{}'),timeout=4)
    except HTTPError as error:assert error.code==403,(action,error.code)
    else:raise AssertionError('physical configuration write was enabled')
with o.open('http://127.0.0.1:3000/healthz',timeout=4) as r:
    assert r.status==200
print('R948_ACTUAL_HTTP_DRAFT_PERSISTED_PENDING; DIAG='+probe['probe_stage']+
  '; BAD_TARGET_400; ALL_DEVICE_WRITES_403; ORIGINAL_3000_OK')
PY
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]]
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]]
trap - ERR
echo R948_PRIVATE_DEVICE_SAVE_DIAGNOSTIC_DEPLOYED
