#!/usr/bin/env bash
# R9.45 direct C320 one-time browser enrollment, persistent NONROOT PRIVATE
# connector. No OLT configuration, host firewall, K3s or public service writes.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -un)" == openai && "$(id -u)" -ne 0 ]] || exit 4
source_dir=/home/openai/.cache/ipat/r945-stage/src
release=/home/openai/.cache/ipat/r945-release
unit_dir=/home/openai/.config/systemd/user
old_unit="$unit_dir/ipat-r911-preview.service"
new_unit="$unit_dir/ipat-r945-connector.service"
smoke=/home/openai/.cache/ipat/r944-release/r945-private-smoke.py
previous=/home/openai/.cache/ipat/r944-release/control-api

expected_source=12f2298035ce6212918a99ffe1da366e60466fec
expected_old_unit=7d1bb967b59e61494f1fc44f437508106302f4b8f6aaaafaf6b631d1478156bb
expected_old_binary=94e3544e62ce3b0405b96f0367bee6e8bd2a0fdd5b41e89b9b68fa91e7fe9a49
expected_new_binary=7d8861a889a60f6f332d18dac74f0b76b1032ddbe267bea806bca4b08ef9729c

[[ "$(git -C "$source_dir" rev-parse HEAD)" == "$expected_source" ]] || exit 4
# cargo fmt may change the release source tree (not functional semantics),
# but NEVER accept any other uncommitted changes.
test "$(git -C "$source_dir" diff --name-only | tr '\n' ' ')" = "apps/control-api/src/c320_live_lab.rs " || exit 4
[[ "$(sha256sum "$old_unit" | cut -d' ' -f1)" == "$expected_old_unit" ]] || exit 4
[[ "$(sha256sum "$previous" | cut -d' ' -f1)" == "$expected_old_binary" ]] || exit 4
[[ "$(sha256sum "$source_dir/target/debug/control-api" | cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" && ! -e "$new_unit" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ ! -e /home/openai/.local/share/ipat/r940-live-agent/live.sock ]] || exit 4
[[ ! -e /home/openai/.local/share/ipat/r945-connection ]] || exit 4
/usr/bin/python3 -c 'import cryptography.fernet,pexpect,socket'
python3 -m py_compile "$smoke" "$source_dir/deploy/scripts/lab/r945/persistent_c320_connector.py"
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3002/lab/device-workbench >/dev/null
if [[ "$mode" == --check ]];then echo R945_PRIVATE_PERSISTENT_PREFLIGHT_PASS;exit 0;fi
[[ "${IPAT_R945_PRIVATE_PERSISTENT_APPROVED:-}" == YES ]] || exit 4

install -d -m0700 "$release"
install -m0700 "$source_dir/target/debug/control-api" "$release/control-api"
install -m0600 "$old_unit" "$release/rollback-r944.service"
/usr/bin/python3 "$source_dir/deploy/scripts/lab/r945/persistent_c320_connector.py" --init

cat > "$release/connector.service" <<'UNIT'
[Unit]
Description=IPAT private persistent C320 read-only management connector R9.45
After=network-online.target
[Service]
Type=simple
WorkingDirectory=/home/openai/.cache/ipat/r945-stage/src
ExecStart=/usr/bin/python3 /home/openai/.cache/ipat/r945-stage/src/deploy/scripts/lab/r945/persistent_c320_connector.py --serve
Restart=on-failure
RestartSec=5
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
Environment=PYTHONDONTWRITEBYTECODE=1
[Install]
WantedBy=default.target
UNIT
chmod 0600 "$release/connector.service"
/usr/bin/python3 - "$old_unit" "$release/next-preview.service" <<'PY'
from pathlib import Path
import sys
data=Path(sys.argv[1]).read_text()
a='WorkingDirectory=/home/openai/.cache/ipat/r944-stage/src'
b='ExecStart=/home/openai/.cache/ipat/r944-release/control-api'
assert data.count(a)==1 and data.count(b)==1
assert data.count('Environment=IPAT_R940_PRIVATE_OWNER_READ=YES')==1
data=data.replace(a,'WorkingDirectory=/home/openai/.cache/ipat/r945-stage/src')
data=data.replace(b,'ExecStart=/home/openai/.cache/ipat/r945-release/control-api')
p=Path(sys.argv[2])
p.write_text(data)
p.chmod(0o600)
PY
rollback(){
    trap - ERR
    systemctl --user stop ipat-r945-connector.service >/dev/null 2>&1 || true
    install -m0600 "$release/rollback-r944.service" "$old_unit"
    systemctl --user daemon-reload
    systemctl --user reset-failed ipat-r911-preview.service
    systemctl --user restart ipat-r911-preview.service
    echo R945_ROLLBACK_TO_R944_PRIVATE_PREVIEW_SECRETS_RETAINED_OWNER_ONLY >&2
}
trap rollback ERR
install -m0600 "$release/connector.service" "$new_unit"
systemctl --user daemon-reload
systemctl --user enable --now ipat-r945-connector.service
ready=no
for _ in $(seq 1 15);do
    if test -S /home/openai/.local/share/ipat/r940-live-agent/live.sock;then ready=yes;break;fi
    sleep 0.5
done
[[ "$ready" == yes ]]
install -m0600 "$release/next-preview.service" "$old_unit"
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
ready=no
for _ in $(seq 1 25);do
    if curl --noproxy '*' -fsS --max-time 1 http://127.0.0.1:3002/lab/c320-connection-setup.js >/dev/null 2>&1;then
      ready=yes;break
    fi
    sleep 0.4
done
[[ "$ready" == yes ]]
IPAT_R945_PRIVATE_SMOKE=YES python3 "$smoke"
test "$(systemctl --user is-active ipat-r945-connector.service)" = active
test "$(systemctl --user is-active ipat-r911-preview.service)" = active
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R945_PERSISTENT_DASHBOARD_INSTALLED_OWNER_BOOTSTRAP_READY_PRIVATE_FILE
