#!/usr/bin/env bash
# R9.45: one-time dashboard adoption + persistent fixed-C320 read collector,
# scoped ONLY to existing nonroot private :3002 laboratory environment.
# No physical CLI runs during deployment. NO network/firewall/K3s/:3000 changes.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -u)" -ne 0 && "$(id -un)" == openai ]] || exit 4
source_dir=/home/openai/.cache/ipat/r945-stage/src
release=/home/openai/.cache/ipat/r945-release
ui_unit=/home/openai/.config/systemd/user/ipat-r911-preview.service
daemon_unit=/home/openai/.config/systemd/user/ipat-r945-c320-connector.service
old_bin=/home/openai/.cache/ipat/r944-release/control-api
smoke=/home/openai/.cache/ipat/r944-release/r945-private-adoption-smoke.py
socket=/home/openai/.local/share/ipat/r940-live-agent/live.sock
state=/home/openai/.local/share/ipat/r945-connection

expected_source=847771f897d090f5f244f1250682ef9ee1685e14
expected_old_unit=7d1bb967b59e61494f1fc44f437508106302f4b8f6aaaafaf6b631d1478156bb
expected_old_binary=94e3544e62ce3b0405b96f0367bee6e8bd2a0fdd5b41e89b9b68fa91e7fe9a49
expected_new_binary=c529733f4cc1d822cc4670635dbcdc5fce596cc3faa7e69ab0244410bfed6408

[[ "$(git -C "$source_dir" rev-parse HEAD)" == "$expected_source" ]] || exit 4
[[ -z "$(git -C "$source_dir" status --porcelain)" ]] || exit 4
[[ "$(sha256sum "$ui_unit"|cut -d' ' -f1)" == "$expected_old_unit" ]] || exit 4
[[ "$(sha256sum "$old_bin"|cut -d' ' -f1)" == "$expected_old_binary" ]] || exit 4
[[ "$(sha256sum "$source_dir/target/debug/control-api"|cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" && ! -e "$daemon_unit" ]] || exit 4
[[ ! -e "$socket" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
python3 -m py_compile "$source_dir/deploy/scripts/lab/r945/persistent_c320_connector.py" "$smoke"
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3002/lab/device-workbench >/dev/null
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
if [[ "$mode" == --check ]];then echo R945_PERSISTENT_CONNECTOR_PREFLIGHT_PASS;exit 0;fi
[[ "${IPAT_R945_OWNER_APPROVES_PRIVATE_ROLLOUT:-}" == YES ]] || exit 4

install -d -m0700 "$release" /home/openai/.config/systemd/user
install -m0700 "$source_dir/target/debug/control-api" "$release/control-api"
install -m0600 "$ui_unit" "$release/rollback-r944.service"
python3 - "$ui_unit" "$release/next-preview.service" <<'PY'
import sys
from pathlib import Path
s=Path(sys.argv[1]).read_text()
old_work="WorkingDirectory=/home/openai/.cache/ipat/r944-stage/src"
old_bin="ExecStart=/home/openai/.cache/ipat/r944-release/control-api"
assert s.count(old_work)==1 and s.count(old_bin)==1
assert s.count("Environment=IPAT_R940_PRIVATE_OWNER_READ=YES")==1
s=s.replace(old_work,"WorkingDirectory=/home/openai/.cache/ipat/r945-stage/src")
s=s.replace(old_bin,"ExecStart=/home/openai/.cache/ipat/r945-release/control-api")
p=Path(sys.argv[2]);p.write_text(s);p.chmod(0o600)
PY
cat > "$release/next-connector.service" <<'EOF'
[Unit]
Description=IPAT R9.45 fixed-target private C320 READ-ONLY connector
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/home/openai/.cache/ipat/r945-stage/src
ExecStart=/usr/bin/python3 -B /home/openai/.cache/ipat/r945-stage/src/deploy/scripts/lab/r945/persistent_c320_connector.py --serve
Restart=on-failure
RestartSec=4
UMask=0077
NoNewPrivileges=true
Environment=PYTHONDONTWRITEBYTECODE=1
TimeoutStopSec=8

[Install]
WantedBy=default.target
EOF
chmod 0600 "$release/next-connector.service"
initialized=no
if [[ ! -e "$state" ]];then
 python3 "$source_dir/deploy/scripts/lab/r945/persistent_c320_connector.py" --init
 initialized=yes
fi
# Existing secrets require independent manual reconciliation; never regenerate
# encryption keys on rerun or ever echo bootstrap code in logs.
rollback(){
 trap - ERR
 install -m0600 "$release/rollback-r944.service" "$ui_unit"
 systemctl --user stop ipat-r945-c320-connector.service >/dev/null 2>&1 || true
 systemctl --user daemon-reload
 systemctl --user reset-failed ipat-r911-preview.service
 systemctl --user restart ipat-r911-preview.service
 echo R945_ROLLED_BACK_TO_R944_PRIVATE_UI >&2
}
trap rollback ERR
install -m0600 "$release/next-connector.service" "$daemon_unit"
install -m0600 "$release/next-preview.service" "$ui_unit"
systemctl --user daemon-reload
systemctl --user restart ipat-r945-c320-connector.service
systemctl --user restart ipat-r911-preview.service
ready=no
for _ in $(seq 1 30);do
 if curl --noproxy '*' -fsS --max-time 2 http://127.0.0.1:3002/lab/c320-owner-connection >/dev/null 2>&1;then
   ready=yes;break
 fi
 sleep 0.5
done
[[ "$ready" == yes ]]
IPAT_R945_PRIVATE_SMOKE=YES python3 "$smoke"
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]]
[[ "$(systemctl --user is-active ipat-r945-c320-connector.service)" == active ]]
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R945_PERSISTENT_DASHBOARD_CONNECTOR_DEPLOYED_NO_OLT_CREDENTIAL_YET
echo R945_BOOTSTRAP_ONLY_IN_OWNER0600_FILE_NO_PRINT
