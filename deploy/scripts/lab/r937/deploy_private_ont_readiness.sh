#!/usr/bin/env bash
# R9.37 nonroot LOCALHOST-only historical UI rollout, no OLT commands.
set -Eeuo pipefail
umask 077
BASE=/home/openai/.cache/ipat/r937-release
SRC=/home/openai/.cache/ipat/r936-stage/src
UNIT=/home/openai/.config/systemd/user/ipat-r911-preview.service
EXPECTED_SOURCE=18389fc25b739d37fd70417f05555aa8d25e6f0b
EXPECTED_PREVIOUS_UNIT=0049e39c9435ad1efc911f2d21fbc8fbdb3c1a70745b883f190a182f8dece4a2
EXPECTED_PREVIOUS_BIN=a03a0fb3a445f45c60d032f5f3ca789abd70d8bba42793f73d68f37519f4ff1b
EXPECTED_NEW_BIN=79fcd282dc9e11e837e38a8524d6568dced6aacc2f02d42366ceb495638cf548
EXPECTED_SMOKE=4c7bf3fa1b886096584ea52156f676a359f5083a86b370b3de65c21cee0b6905
SMOKE=/home/openai/.cache/ipat/r936-stage/private_c320_ont_feature_smoke.py
MODE="${1:---check}"
[[ "$MODE" == --check || "$MODE" == --apply ]] || exit 4
[[ $(id -un) == openai && $(id -u) -ne 0 ]] || exit 4
[[ $(git -C "$SRC" rev-parse HEAD) == "$EXPECTED_SOURCE" ]] || exit 4
[[ -z $(git -C "$SRC" status --porcelain) ]] || exit 4
[[ $(sha256sum "$SRC/target/debug/control-api" | cut -d' ' -f1) == "$EXPECTED_NEW_BIN" ]] || exit 4
[[ $(sha256sum "$SMOKE" | cut -d' ' -f1) == "$EXPECTED_SMOKE" ]] || exit 4
[[ $(sha256sum "$UNIT" | cut -d' ' -f1) == "$EXPECTED_PREVIOUS_UNIT" ]] || exit 4
[[ $(sha256sum /home/openai/.cache/ipat/r936-stage/control-api | cut -d' ' -f1) == "$EXPECTED_PREVIOUS_BIN" ]] || exit 4
[[ $(systemctl --user is-active ipat-r911-preview.service) == active ]] || exit 4
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3002/lab/device-workbench >/dev/null
[[ ! -e "$BASE" && ! -L "$BASE" ]] || exit 4
if [[ "$MODE" == --check ]]; then echo R937_PRIVATE_READINESS_PREFLIGHT_PASS; exit 0; fi
[[ "${IPAT_R937_PRIVATE_PREVIEW_APPROVED:-}" == YES ]] || exit 4
install -m 0700 -d "$BASE"
install -m 0700 "$SRC/target/debug/control-api" "$BASE/control-api"
install -m 0600 "$SMOKE" "$BASE/private_c320_ont_feature_smoke.py"
install -m 0600 "$UNIT" "$BASE/rollback-r936.service"
python3 - "$UNIT" "$BASE/next.service" <<'PY'
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
old='/home/openai/.cache/ipat/r936-stage/control-api'
new='/home/openai/.cache/ipat/r937-release/control-api'
assert s.count(old)==1
with Path(sys.argv[2]).open('x') as f:f.write(s.replace(old,new))
Path(sys.argv[2]).chmod(0o600)
PY
rollback(){
  trap - ERR
  install -m 0600 "$BASE/rollback-r936.service" "$UNIT"
  systemctl --user daemon-reload
  systemctl --user reset-failed ipat-r911-preview.service
  systemctl --user restart ipat-r911-preview.service
  echo R937_ROLLBACK_TO_VERIFIED_R936 >&2
}
trap rollback ERR
install -m 0600 "$BASE/next.service" "$UNIT"
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
ready=false
for _ in $(seq 1 24); do
 if curl --noproxy '*' -fsS --max-time 1 http://127.0.0.1:3002/lab/device-workbench >/dev/null 2>&1; then ready=true; break; fi
 sleep 0.25
done
[[ "$ready" == true ]] || { rollback; exit 7; }
IPAT_R937_PRIVATE_SMOKE=YES python3 "$BASE/private_c320_ont_feature_smoke.py"
[[ $(systemctl --user is-active ipat-r911-preview.service) == active ]] || { rollback; exit 7; }
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R937_PRIVATE_LOOPBACK_ONT_READINESS_DEPLOY_PASS_NO_OLT_WRITES
