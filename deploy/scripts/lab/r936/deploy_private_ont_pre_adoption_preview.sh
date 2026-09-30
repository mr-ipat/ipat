#!/usr/bin/env bash
# Private NONROOT loopback-only C320 pre-adoption UI. ZERO physical device ops.
set -Eeuo pipefail
umask 077
BASE=/home/openai/.cache/ipat/r936-stage
SRC=$BASE/src
UNIT=/home/openai/.config/systemd/user/ipat-r911-preview.service
SOURCE_SHA=9292e9057aba0f422a25372dbae1d92a43641013
OLD_UNIT_SHA=10d45985ffaf9d2040cb02216752133230c9e497d14bf55db0f7ba0ff71a63de
EXPECTED_BINARY_SHA=a03a0fb3a445f45c60d032f5f3ca789abd70d8bba42793f73d68f37519f4ff1b
NEW_UNIT=$BASE/ipat-r936-preview.service
ROLLBACK=$BASE/r934-verified-rollback.service
mode="${1:---check}"
[[ "$mode" == --check || "$mode" == --apply ]] || exit 4
[[ $(id -un) == openai && $(id -u) -ne 0 ]] || exit 4
[[ $(git -C "$SRC" rev-parse HEAD) == "$SOURCE_SHA" ]] || exit 4
[[ -z $(git -C "$SRC" status --porcelain) ]] || exit 4
[[ $(sha256sum "$SRC/target/debug/control-api" | cut -d' ' -f1) == "$EXPECTED_BINARY_SHA" ]] || exit 4
[[ $(sha256sum "$UNIT" | cut -d' ' -f1) == "$OLD_UNIT_SHA" ]] || exit 4
[[ $(systemctl --user is-active ipat-r911-preview.service) == active ]] || exit 4
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3002/lab/device-workbench >/dev/null
if [[ "$mode" == --check ]]; then echo R936_PRIVATE_DEPLOY_PREFLIGHT_PASS; exit 0; fi
[[ "${IPAT_R936_APPROVE_PRIVATE_LOOPBACK_PREVIEW:-}" == YES ]] || exit 4
[[ ! -e "$ROLLBACK" && ! -L "$ROLLBACK" && ! -e "$NEW_UNIT" ]] || exit 4
install -m 0700 -d "$BASE"
install -m 0700 "$SRC/target/debug/control-api" "$BASE/control-api"
install -m 0600 "$UNIT" "$ROLLBACK"
python3 - "$ROLLBACK" "$NEW_UNIT" "$BASE" <<'PY'
from pathlib import Path
import sys
old=Path(sys.argv[1]).read_text()
new=old.replace('/home/openai/.cache/ipat/r934-release/src',sys.argv[3]+'/src').replace('/home/openai/.cache/ipat/r934-release/control-api',sys.argv[3]+'/control-api')
assert old!=new and sys.argv[3]+'/control-api' in new and sys.argv[3]+'/src' in new
p=Path(sys.argv[2]); fd=p.open('x'); fd.write(new); fd.close(); p.chmod(0o600)
PY
rollback(){
  trap - ERR
  install -m 0600 "$ROLLBACK" "$UNIT"
  systemctl --user daemon-reload
  systemctl --user reset-failed ipat-r911-preview.service
  systemctl --user restart ipat-r911-preview.service
  echo R936_ROLLED_BACK_TO_R934 >&2
}
trap rollback ERR
install -m 0600 "$NEW_UNIT" "$UNIT"
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
ready=false
for _ in $(seq 1 24); do
  if curl --noproxy '*' -fsS --max-time 1 http://127.0.0.1:3002/lab/device-workbench >/dev/null 2>&1; then ready=true; break; fi
  sleep 0.25
done
if [[ "$ready" != true ]]; then rollback; exit 7; fi
IPAT_R936_PRIVATE_SMOKE=YES python3 "$BASE/private_ont_pre_adoption_smoke.py"
if [[ $(systemctl --user is-active ipat-r911-preview.service) != active ]]; then rollback; exit 7; fi
curl --noproxy '*' -fsS --max-time 3 http://127.0.0.1:3000/healthz >/dev/null
trap - ERR
echo R936_PRIVATE_LOOPBACK_PREVIEW_PASS_ZERO_DEVICE_WRITES
