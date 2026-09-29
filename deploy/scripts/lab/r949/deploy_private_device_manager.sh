#!/usr/bin/env bash
# Scoped private UI release only. No device commands, no firewall changes.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == "--check" || "$mode" == "--apply" ]] || exit 4
[[ "$(id -un)" == "openai" && "$(id -u)" -ne 0 ]] || exit 4
src=/home/openai/.cache/ipat/r949-stage/src
release=/home/openai/.cache/ipat/r949-release
unit=/home/openai/.config/systemd/user/ipat-r911-preview.service
prev=/home/openai/.cache/ipat/r948-release/control-api
binary="$src/target/debug/control-api"
old_unit_sha=50023a6fbf9450abe9be35ef67011dde6b9585d46c579d8798b992bf273f5ab2
old_bin_sha=3eeb6cb5fcffa8b6e3d10c4ad4af020eacd8c3f4c70fcc03be07b9cf6336c5c7
expected_source=f2b48a0da7f4c2f40cda7697c8dd569ab85d59e7
new_bin_sha=PIN_R949_BINARY_SHA256
[[ "$(git -C "$src" rev-parse HEAD)" == "$expected_source" ]] || exit 4
[[ -z "$(git -C "$src" status --porcelain)" ]] || exit 4
[[ "$(sha256sum "$unit"|cut -d' ' -f1)" == "$old_unit_sha" ]] || exit 4
[[ "$(sha256sum "$prev"|cut -d' ' -f1)" == "$old_bin_sha" ]] || exit 4
[[ "$(sha256sum "$binary"|cut -d' ' -f1)" == "$new_bin_sha" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]] || exit 4
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler
opener=build_opener(ProxyHandler({}))
with opener.open("http://127.0.0.1:3002/lab/c320-owner-connection",timeout=4) as r:
    v=json.load(r)
assert v["target"]=="DEV-01" and v["production_adopted"] is False
assert v["physical_writes_enabled"] is False
assert v["draft_saved"] is True
assert v["credentials_enrolled"] is False, "device now enrolled; require fresh change review"
PY
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
if [[ "$mode" == "--check" ]];then
  echo R949_PRIVATE_RELEASE_PREFLIGHT_PASS
  exit 0
fi
[[ "${IPAT_R949_PRIVATE_RELEASE_AUTHORIZED:-}" == YES ]] || exit 4
install -d -m0700 "$release"
install -m0700 "$binary" "$release/control-api"
install -m0600 "$unit" "$release/rollback-r948.service"
python3 - "$unit" "$release/new.service" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1])
s=p.read_text()
a="WorkingDirectory=/home/openai/.cache/ipat/r948-stage/src"
b="ExecStart=/home/openai/.cache/ipat/r948-release/control-api"
assert s.count(a)==1 and s.count(b)==1
s=s.replace(a,"WorkingDirectory=/home/openai/.cache/ipat/r949-stage/src")
s=s.replace(b,"ExecStart=/home/openai/.cache/ipat/r949-release/control-api")
out=Path(sys.argv[2]);out.write_text(s);out.chmod(0o600)
PY
rollback(){
 trap - ERR
 install -m0600 "$release/rollback-r948.service" "$unit"
 systemctl --user daemon-reload
 systemctl --user reset-failed ipat-r911-preview.service
 systemctl --user restart ipat-r911-preview.service || true
 echo R949_PRIVATE_ROLLED_BACK_TO_R948 >&2
}
trap rollback ERR
install -m0600 "$release/new.service" "$unit"
systemctl --user daemon-reload
systemctl --user restart ipat-r911-preview.service
ready=no
for i in $(seq 1 30);do
 if curl --noproxy '*' -fsS --max-time 2 http://127.0.0.1:3002/lab/c320-connection-setup.js >/dev/null 2>&1;then ready=yes;break;fi
 sleep 0.5
done
[[ "$ready" == yes ]]
python3 - <<'PY'
import json
from urllib.request import build_opener,ProxyHandler,Request
from urllib.error import HTTPError
opener=build_opener(ProxyHandler({}))
base="http://127.0.0.1:3002"
with opener.open(base+"/lab/device-workbench",timeout=5) as r:page=r.read()
for marker in [b'id="ipat-saved-physical-rows"',b'id="ipat-device-save-stage"',
               b'id="ipat-device-connect-stage"',b'id="ipat-device-diagnostic"']:
    assert marker in page,marker
assert page.index(b'id="ipat-saved-physical-rows"')<page.index(b'id="ipat-lab-diagnostics"')
with opener.open(base+"/lab/c320-connection-setup.js",timeout=5) as r:script=r.read()
assert b'resumeAfterVerification' in script
assert b'Saved to Device List' in script
with opener.open(base+"/lab/device-workbench.js",timeout=5) as r:script=r.read()
assert b"node('ipat-saved-physical-rows').replaceChildren(actual)" in script
with opener.open(base+"/lab/c320-owner-connection",timeout=5) as r:status=json.load(r)
assert status["draft_saved"] is True and status["device_status"]=="PENDING"
assert status["credentials_enrolled"] is False and status["physical_writes_enabled"] is False
for action in ["PROVISION_ONTS","REBOOT_OLT","UPGRADE_OLT_FIRMWARE"]:
    try:opener.open(Request(base+"/lab/c320-actions/"+action,method="POST",data=b"{}"),timeout=4)
    except HTTPError as error:assert error.code==403,(action,error.code)
    else:raise AssertionError("physical write unexpectedly enabled")
with opener.open("http://127.0.0.1:3000/healthz",timeout=4) as r:assert r.status==200
print("R949_VISIBLE_SAVED_DEVICE_AND_CONNECTION_DIAGNOSTICS_DEPLOY_HTTP_PASS")
PY
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]]
[[ "$(systemctl --user is-active ipat-r945-connector.service)" == active ]]
trap - ERR
echo R949_PRIVATE_DEVICE_MANAGER_DEPLOYED_NO_OLT_WRITES
