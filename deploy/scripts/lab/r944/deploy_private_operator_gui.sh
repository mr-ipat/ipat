#!/usr/bin/env bash
# R9.44 controlled operator GUI deploy: ONLY existing nonroot localhost :3002.
# No OLT/ONT CLI, firewall, K3s, database, or public service changes.
set -Eeuo pipefail
umask 077
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ "$(id -un)" == openai && "$(id -u)" -ne 0 ]] || exit 4

src=/home/openai/.cache/ipat/r944-stage/src
release=/home/openai/.cache/ipat/r944-release
unit=/home/openai/.config/systemd/user/ipat-r911-preview.service
smoke=/home/openai/.cache/ipat/r943-release/r944-private-operator-gui-smoke.py
old_bin=/home/openai/.cache/ipat/r943-release/control-api

expected_source=9b4f5a1514fc6ed13ed842d6c0f496d98f48c396
expected_old_unit=f50f58eeb340391a4768ff2038fcfe9e2655eff5ce6759b219b0b4f104022c83
expected_old_binary=2f4e47d67bd063835dde8a69ca9149284fd121ddede91e97fe2285a0bccdb74c
expected_new_binary=REPLACE_WITH_PINNED_BUILD_SHA256

[[ "$(git -C "$src" rev-parse HEAD)" == "$expected_source" ]] || exit 4
[[ -z "$(git -C "$src" status --porcelain)" ]] || exit 4
[[ "$(sha256sum "$unit" | cut -d' ' -f1)" == "$expected_old_unit" ]] || exit 4
[[ "$(sha256sum "$old_bin" | cut -d' ' -f1)" == "$expected_old_binary" ]] || exit 4
[[ "$(sha256sum "$src/target/debug/control-api" | cut -d' ' -f1)" == "$expected_new_binary" ]] || exit 4
[[ ! -e "$release" && ! -L "$release" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ ! -e /home/openai/.local/share/ipat/r940-live-agent/live.sock ]] || exit 4
python3 -m py_compile "$smoke"
test -f "$src/web/lab/c320-operator-console.js"
grep -Fq 'id="ipat-c320-console"' "$src/web/lab/device-workbench.html"
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3000/healthz >/dev/null
curl --noproxy '*' -fsS --max-time 4 http://127.0.0.1:3002/lab/device-workbench >/dev/null

if [[ "$mode" == --check ]]; then
  echo R944_PRIVATE_DEPLOY_PREFLIGHT_PASS
  exit 0
fi
[[ "${IPAT_R944_PRIVATE_DEPLOY_APPROVED:-}" == YES ]] || exit 4
install -d -m 0700 "$release"
install -m 0700 "$src/target/debug/control-api" "$release/control-api"
install -m 0600 "$unit" "$release/rollback-r943.service"
python3 - "$unit" "$release/new.service" <<'PY'
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
a="WorkingDirectory=/home/openai/.cache/ipat/r943-stage/src"
b="ExecStart=/home/openai/.cache/ipat/r943-release/control-api"
assert s.count(a)==1 and s.count(b)==1
assert s.count("Environment=IPAT_R940_PRIVATE_OWNER_READ=YES")==1
s=s.replace(a,"WorkingDirectory=/home/openai/.cache/ipat/r944-stage/src")
s=s.replace(b,"ExecStart=/home/openai/.cache/ipat/r944-release/control-api")
p=Path(sys.argv[2])
p.write_text(s)
p.chmod(0o600)
PY

rollback() {
  trap - ERR
  install -m 0600 "$release/rollback-r943.service" "$unit"
  systemctl --user daemon-reload
  systemctl --user reset-failed ipat-r911-preview.service
  systemctl --user restart ipat-r911-preview.service
  echo R944_AUTO_ROLLBACK_TO_VERIFIED_R943 >&2
}
trap rollback ERR
install -m 0600 "$release/new.service" "$unit"
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service

ready=no
for _ in $(seq 1 30); do
  if curl --noproxy '*' -fsS --max-time 1 http://127.0.0.1:3002/lab/c320-operator-console.js >/dev/null 2>&1; then
    ready=yes
    break
  fi
  sleep 0.5
done
[[ "$ready" == yes ]]
IPAT_R944_PRIVATE_SMOKE=YES python3 "$smoke"
test "$(systemctl --user is-active ipat-r911-preview.service)" = active
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R944_PRIVATE_C320_OPERATOR_GUI_DEPLOYED; echo AGENT_REQUIRES_SEPARATE_OWNER_TTY
