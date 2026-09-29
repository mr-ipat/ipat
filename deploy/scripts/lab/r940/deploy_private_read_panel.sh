#!/usr/bin/env bash
# Owner-approved nonroot private LOOPBACK:3002 panel rollout only. ZERO OLT commands.
set -Eeuo pipefail
umask 077
MODE="${1:---check}"
[[ "$MODE" == --check || "$MODE" == --apply ]] || exit 4
[[ "$(id -un)" == openai && "$(id -u)" -ne 0 ]] || exit 4
SRC=/home/openai/.cache/ipat/r939-stage/src
STAGE=/home/openai/.cache/ipat/r940-stage
BASE=/home/openai/.cache/ipat/r940-release
UNIT=/home/openai/.config/systemd/user/ipat-r911-preview.service
SMOKE=/home/openai/.cache/ipat/r936-stage/r940-private-read-panel-smoke.py
EXPECTED_SOURCE=0b4e92b
EXPECTED_UNIT=0049e39c9435ad1efc911f2d21fbc8fbdb3c1a70745b883f190a182f8dece4a2
EXPECTED_BIN=55144b871d49e93482306dc7250e645d59ffa2a0cfeb9756ca6f6c6b5e1fd38c
[[ "$(git -C "$SRC" rev-parse --short=7 HEAD)" == "$EXPECTED_SOURCE" ]] || exit 4
[[ -z "$(git -C "$SRC" status --porcelain)" ]] || exit 4
[[ "$(sha256sum "$SRC/target/debug/control-api" | cut -d' ' -f1)" == "$EXPECTED_BIN" ]] || exit 4
[[ "$(sha256sum "$UNIT" | cut -d' ' -f1)" == "$EXPECTED_UNIT" ]] || exit 4
[[ "$(systemctl --user is-active ipat-r911-preview.service)" == active ]] || exit 4
[[ ! -e "$STAGE" && ! -L "$STAGE" && ! -e "$BASE" && ! -L "$BASE" ]] || exit 4
python3 -m py_compile "$SMOKE" "$SRC/deploy/scripts/lab/r940/owner_supervised_c320_read_agent.py"
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3002/lab/device-workbench >/dev/null
[[ ! -e /home/openai/.local/share/ipat/r940-live-agent/live.sock ]] || exit 4
if [[ "$MODE" == --check ]]; then echo R940_PRIVATE_PANEL_PREFLIGHT_PASS; exit 0; fi
[[ "${IPAT_R940_OWNER_APPROVES_READ_ONLY_PREVIEW:-}" == YES ]] || exit 4
install -d -m0700 "$STAGE" "$BASE"
git clone -q --shared "$SRC" "$STAGE/src"
test "$(git -C "$STAGE/src" rev-parse --short=7 HEAD)" = "$EXPECTED_SOURCE"
install -m 0700 "$SRC/target/debug/control-api" "$BASE/control-api"
install -m 0600 "$UNIT" "$BASE/rollback-r936.service"
python3 - "$UNIT" "$BASE/next.service" <<'PY'
from pathlib import Path
import sys
src=Path(sys.argv[1]).read_text()
old='WorkingDirectory=/home/openai/.cache/ipat/r936-stage/src'
assert src.count(old)==1
src=src.replace(old,'WorkingDirectory=/home/openai/.cache/ipat/r940-stage/src')
old='ExecStart=/home/openai/.cache/ipat/r936-stage/control-api'
assert src.count(old)==1
src=src.replace(old,'ExecStart=/home/openai/.cache/ipat/r940-release/control-api')
needle='Environment=IPAT_R911_PRIVATE_CANARY=YES'
assert src.count(needle)==1
src=src.replace(needle,needle+'\nEnvironment=IPAT_R940_PRIVATE_OWNER_READ=YES')
assert 'IPAT_R940_PRIVATE_OWNER_READ=YES' in src
with Path(sys.argv[2]).open('x') as file:file.write(src)
Path(sys.argv[2]).chmod(0o600)
PY
rollback(){
  trap - ERR
  install -m 0600 "$BASE/rollback-r936.service" "$UNIT"
  systemctl --user daemon-reload
  systemctl --user reset-failed ipat-r911-preview.service
  systemctl --user restart ipat-r911-preview.service
  echo R940_ROLLBACK_TO_PRIOR_R936_PRIVATE_PREVIEW >&2
}
trap rollback ERR
install -m 0600 "$BASE/next.service" "$UNIT"
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
ready=false
for _ in $(seq 1 20); do
 if curl --noproxy '*' -fsS --max-time 1 http://127.0.0.1:3002/lab/device-workbench >/dev/null 2>&1; then ready=true; break; fi
 sleep 0.5
done
[[ "$ready" == true ]] || { rollback; exit 7; }
IPAT_R940_PRIVATE_SMOKE=YES python3 "$SMOKE"
test "$(systemctl --user is-active ipat-r911-preview.service)" = active
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R940_OWNER_PRIVATE_LIVE_BUTTON_DEPLOYED_AGENT_INTERACTIVE_OPT_IN_REQUIRED
