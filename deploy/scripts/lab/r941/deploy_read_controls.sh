#!/usr/bin/env bash
# Exact-checksum nonroot private :3002 read-only UI upgrade. NO OLT/ONT commands.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -un)" == openai && "$(id -u)" -ne 0 ]] || exit 4
source_dir=/home/openai/.cache/ipat/r941-stage/src
release=/home/openai/.cache/ipat/r941-release
unit=/home/openai/.config/systemd/user/ipat-r911-preview.service
smoke=/home/openai/.cache/ipat/r940-release/r941-read-controls-smoke.py
expected_commit=b6d6a73
expected_old_unit=4a2404537ec27c1a97aa22c34fb7281a9c910db8fa81648d00f95aa9e0bc3189
expected_old_binary=55144b871d49e93482306dc7250e645d59ffa2a0cfeb9756ca6f6c6b5e1fd38c
expected_new_binary=6e4084becdacfa57f525639e1cffbcf1e7dc50c2e3dc48e507730f60d7de2ee5
[[ "$(git -C "$source_dir" rev-parse --short=7 HEAD)" == "$expected_commit" ]] || exit 4
[[ -z "$(git -C "$source_dir" status --porcelain)" ]] || exit 4
[[ "$(sha256sum "$unit" | cut -d' ' -f1)" == "$expected_old_unit" ]] || exit 4
[[ "$(sha256sum /home/openai/.cache/ipat/r940-release/control-api | cut -d' ' -f1)" == "$expected_old_binary" ]] || exit 4
[[ "$(sha256sum "$source_dir/target/debug/control-api" | cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ ! -e /home/openai/.local/share/ipat/r940-live-agent/live.sock ]] || exit 4
python3 -m py_compile "$smoke" "$source_dir/deploy/scripts/lab/r940/owner_supervised_c320_read_agent.py"
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3002/lab/device-workbench >/dev/null
if [[ "$mode" == --check ]];then echo R941_EXACT_PRIVATE_READ_UI_PREFLIGHT_PASS;exit 0;fi
[[ "${IPAT_R941_PRIVATE_READ_UPGRADE_APPROVED:-}" == YES ]] || exit 4
install -d -m0700 "$release"
install -m0700 "$source_dir/target/debug/control-api" "$release/control-api"
install -m0600 "$unit" "$release/rollback-r940.service"
python3 - "$unit" "$release/next.service" <<'PY'
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
a='WorkingDirectory=/home/openai/.cache/ipat/r940-stage/src'
b='ExecStart=/home/openai/.cache/ipat/r940-release/control-api'
assert s.count(a)==1 and s.count(b)==1
assert s.count('Environment=IPAT_R940_PRIVATE_OWNER_READ=YES')==1
s=s.replace(a,'WorkingDirectory=/home/openai/.cache/ipat/r941-stage/src')
s=s.replace(b,'ExecStart=/home/openai/.cache/ipat/r941-release/control-api')
p=Path(sys.argv[2]);p.write_text(s);p.chmod(0o600)
PY
rollback(){
 trap - ERR
 install -m0600 "$release/rollback-r940.service" "$unit"
 systemctl --user daemon-reload
 systemctl --user reset-failed ipat-r911-preview.service
 systemctl --user restart ipat-r911-preview.service
 echo R941_ROLLBACK_TO_ACTUAL_R940_PREVIEW >&2
}
trap rollback ERR
install -m0600 "$release/next.service" "$unit"
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
ready=no
for _ in $(seq 1 25);do
 if curl --noproxy '*' -fsS --max-time 1 http://127.0.0.1:3002/lab/device-workbench >/dev/null 2>&1;then ready=yes;break;fi
 sleep 0.4
done
[[ "$ready" == yes ]]
IPAT_R941_PRIVATE_SMOKE=YES python3 "$smoke"
test "$(systemctl --user is-active ipat-r911-preview.service)" = active
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R941_PRIVATE_THREE_READ_BUTTONS_DEPLOYED_OWNER_AGENT_STILL_INTERACTIVE
