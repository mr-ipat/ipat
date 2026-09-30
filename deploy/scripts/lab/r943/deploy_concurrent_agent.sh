#!/usr/bin/env bash
# Exact-checksum nonroot private :3002 read-only UI upgrade. NO OLT/ONT commands.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -un)" == openai && "$(id -u)" -ne 0 ]] || exit 4
source_dir=/home/openai/.cache/ipat/r943-stage/src
release=/home/openai/.cache/ipat/r943-release
unit=/home/openai/.config/systemd/user/ipat-r911-preview.service
smoke=/home/openai/.cache/ipat/r940-release/r943-owner-agent-smoke.py
expected_commit=fbda57a
expected_old_unit=fac4ef0174f28958c38478bd23a9e4ff677a5b0ad08aa9b18d7f0da5dcf7c950
expected_old_binary=b3fd6068b7f170e616b542b77cfd9a9dfe5c88f14f52c2426ebcf5788cf22ea1
expected_new_binary=2f4e47d67bd063835dde8a69ca9149284fd121ddede91e97fe2285a0bccdb74c
[[ "$(git -C "$source_dir" rev-parse --short=7 HEAD)" == "$expected_commit" ]] || exit 4
[[ -z "$(git -C "$source_dir" status --porcelain)" ]] || exit 4
[[ "$(sha256sum "$unit" | cut -d' ' -f1)" == "$expected_old_unit" ]] || exit 4
[[ "$(sha256sum /home/openai/.cache/ipat/r942-release/control-api | cut -d' ' -f1)" == "$expected_old_binary" ]] || exit 4
[[ "$(sha256sum "$source_dir/target/debug/control-api" | cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ ! -e /home/openai/.local/share/ipat/r940-live-agent/live.sock ]] || exit 4
python3 -m py_compile "$smoke" "$source_dir/deploy/scripts/lab/r940/owner_supervised_c320_read_agent.py"
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3002/lab/device-workbench >/dev/null
if [[ "$mode" == --check ]];then echo R943_PREVIEW_DEPLOY_PREFLIGHT_PASS;exit 0;fi
[[ "${IPAT_R943_READ_AGENT_FIX_APPROVED:-}" == YES ]] || exit 4
install -d -m0700 "$release"
install -m0700 "$source_dir/target/debug/control-api" "$release/control-api"
install -m0600 "$unit" "$release/rollback-r942.service"
python3 - "$unit" "$release/next.service" <<'PY'
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
a='WorkingDirectory=/home/openai/.cache/ipat/r942-stage/src'
b='ExecStart=/home/openai/.cache/ipat/r942-release/control-api'
assert s.count(a)==1 and s.count(b)==1
assert s.count('Environment=IPAT_R940_PRIVATE_OWNER_READ=YES')==1
s=s.replace(a,'WorkingDirectory=/home/openai/.cache/ipat/r943-stage/src')
s=s.replace(b,'ExecStart=/home/openai/.cache/ipat/r943-release/control-api')
p=Path(sys.argv[2]);p.write_text(s);p.chmod(0o600)
PY
rollback(){
 trap - ERR
 install -m0600 "$release/rollback-r942.service" "$unit"
 systemctl --user daemon-reload
 systemctl --user reset-failed ipat-r911-preview.service
 systemctl --user restart ipat-r911-preview.service
 echo R943_ROLLBACK_TO_R942_PREVIEW >&2
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
IPAT_R943_PRIVATE_SMOKE=YES python3 "$smoke"
test "$(systemctl --user is-active ipat-r911-preview.service)" = active
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R943_CONCURRENT_OWNER_AGENT_DASHBOARD_DEPLOYED
