#!/usr/bin/env bash
# Recover only the scoped R9.45 private deployment after the original
# cross-origin smoke sent structurally invalid JSON and returned safe 422.
# Previous private R9.44 unit remains backed up; do not regenerate secrets.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -un)" == openai && "$(id -u)" -ne 0 ]] || exit 4
release=/home/openai/.cache/ipat/r945-release
unit=/home/openai/.config/systemd/user/ipat-r911-preview.service
connector=/home/openai/.config/systemd/user/ipat-r945-connector.service
smoke=/home/openai/.cache/ipat/r944-release/r945-private-smoke.py
old=7d1bb967b59e61494f1fc44f437508106302f4b8f6aaaafaf6b631d1478156bb
new=7d8861a889a60f6f332d18dac74f0b76b1032ddbe267bea806bca4b08ef9729c
[[ "$(sha256sum "$unit" | cut -d' ' -f1)" == "$old" ]] || exit 4
[[ "$(sha256sum "$release/rollback-r944.service" | cut -d' ' -f1)" == "$old" ]] || exit 4
[[ "$(sha256sum "$release/control-api" | cut -d' ' -f1)" == "$new" ]] || exit 4
grep -Fq 'WorkingDirectory=/home/openai/.cache/ipat/r945-stage/src' "$release/next-preview.service"
grep -Fq 'ExecStart=/home/openai/.cache/ipat/r945-release/control-api' "$release/next-preview.service"
[[ -f "$connector" && -f "$smoke" ]] || exit 4
[[ "$(stat -c %a /home/openai/.local/share/ipat/r945-connection/bootstrap-token)" == 600 ]] || exit 4
[[ ! -e /home/openai/.local/share/ipat/r945-connection/device.fernet ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
python3 -m py_compile "$smoke"
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
if [[ "$mode" == --check ]];then echo R945_RESUME_AFTER_VALID_SCHEMA_SMOKE_PREFLIGHT_PASS;exit 0;fi
[[ "${IPAT_R945_RESUME_PRIVATE_DEPLOY:-}" == YES ]] || exit 4
rollback(){
    trap - ERR
    systemctl --user disable --now ipat-r945-connector.service >/dev/null 2>&1 || true
    install -m0600 "$release/rollback-r944.service" "$unit"
    systemctl --user daemon-reload
    systemctl --user reset-failed ipat-r911-preview.service
    systemctl --user restart ipat-r911-preview.service
    echo R945_SCOPED_RESUME_ROLLED_BACK_TO_R944_NO_SECRET_DISCLOSURE >&2
}
trap rollback ERR
systemctl --user reset-failed ipat-r945-connector.service
systemctl --user enable --now ipat-r945-connector.service
ready=no
for _ in $(seq 1 20);do
 if test -S /home/openai/.local/share/ipat/r940-live-agent/live.sock;then ready=yes;break;fi
 sleep 0.5
done
[[ "$ready" == yes ]]
install -m0600 "$release/next-preview.service" "$unit"
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
echo R945_PERSISTENT_DIRECT_C320_DASHBOARD_DEPLOYED_BOOTSTRAP_OWNER_ONLY
