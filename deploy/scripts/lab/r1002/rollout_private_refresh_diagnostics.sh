#!/usr/bin/env bash
set -Eeuo pipefail
umask 077
[[ "$(printenv IPAT_R1002_PRIVATE_ROLLOUT 2>/dev/null || true)" == YES ]] || { echo "explicit private rollout opt-in required" >&2; exit 3; }
[[ $(id -u) -ne 0 ]] || { echo "root forbidden" >&2; exit 4; }
command -v systemctl >/dev/null
command -v python3 >/dev/null
repo=$(git rev-parse --show-toplevel 2>/dev/null) || exit 4
[[ -f "$repo/deploy/scripts/lab/r945/persistent_c320_connector.py" ]] || exit 4
[[ -z $(git -C "$repo" status --porcelain) ]] || { echo "source checkout must be clean" >&2; exit 4; }
unit="$HOME/.config/systemd/user/ipat-r945-connector.service"
[[ -f "$unit" && ! -L "$unit" ]] || exit 4
systemctl --user is-active --quiet ipat-r945-connector.service || exit 4
state="$HOME/.local/state/ipat/r1002"
mkdir -p "$state"; chmod 700 "$state"
[[ ! -e "$state/ipat-r945-connector.service.before" ]] || { echo "prior R10.02 rollout evidence exists" >&2; exit 4; }
cp -p -- "$unit" "$state/ipat-r945-connector.service.before"
sha256sum "$unit" > "$state/unit.before.sha256"
systemctl --user show ipat-r945-connector.service -p ExecStart --value > "$state/execstart.before.txt"
chmod 600 "$state/"*

rollback(){
  set +e
  if [[ -f "$state/ipat-r945-connector.service.before" ]]; then
    cp -- "$state/ipat-r945-connector.service.before" "$unit"
    chmod 600 "$unit"
    systemctl --user daemon-reload
    systemctl --user restart ipat-r945-connector.service
  fi
}
trap rollback ERR INT TERM

cat > "$unit.pending" <<EOF
[Unit]
Description=IPAT private persistent C320 read-only management connector R10.02
After=network-online.target
[Service]
Type=simple
WorkingDirectory=$repo
ExecStart=/usr/bin/python3 $repo/deploy/scripts/lab/r945/persistent_c320_connector.py --serve
Restart=on-failure
RestartSec=5
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
Environment=PYTHONDONTWRITEBYTECODE=1
[Install]
WantedBy=default.target
EOF
chmod 600 "$unit.pending"
mv -- "$unit.pending" "$unit"
systemctl --user daemon-reload
systemctl --user restart ipat-r945-connector.service
ok=NO
for _ in {1..30}; do
  if systemctl --user is-active --quiet ipat-r945-connector.service; then
    if python3 - <<'PY'
import json,socket,sys
path="/home/openai/.local/share/ipat/r940-live-agent/live.sock"
try:
    s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);s.settimeout(2);s.connect(path)
    s.sendall(b"STATUS\n");s.shutdown(socket.SHUT_WR)
    data=b""
    while len(data)<=1024:
        x=s.recv(1024)
        if not x: break
        data+=x
    v=json.loads(data)
    ok=(v.get("mode")=="OWNER_SUPERVISED_REAL_C320_READ_ONLY"
        and v.get("persistent_connector") is True
        and v.get("credentials_enrolled") is True
        and v.get("device_writes")==0)
    sys.exit(0 if ok else 1)
except Exception:
    sys.exit(1)
PY
    then ok=YES; break; fi
  fi
  sleep 1
done
[[ "$ok" == YES ]] || exit 4

python3 - <<'PY'
import json,socket,sys
path="/home/openai/.local/share/ipat/r940-live-agent/live.sock"
s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);s.settimeout(65);s.connect(path)
s.sendall(b"REFRESH\n");s.shutdown(socket.SHUT_WR)
data=b""
while len(data)<=2048:
    x=s.recv(2048)
    if not x: break
    data+=x
v=json.loads(data)
allowed={
"C320_REFRESH_BOUNDARY_REJECTED","C320_UNCONFIGURED_TABLE_UNSUPPORTED",
"C320_UNCONFIGURED_ROWS_OUT_OF_BOUNDS","C320_STATE_HEADER_UNSUPPORTED",
"C320_STATE_ROWS_UNSUPPORTED","C320_STATE_TOTALS_INCONSISTENT",
"C320_PON_CONFIG_SHAPE_INCOMPLETE","C320_STATE_CONFIG_COUNT_MISMATCH",
"DEVICE_CONNECTION_FAILED_UNCLASSIFIED"}
if v.get("error") in allowed:
    print("R1002_REFRESH_DIAGNOSTIC="+v["error"])
else:
    required={"mode","snapshot_is_live","port","unconfigured","configured","online",
              "offline","configuration_rows","serials_returned","device_adopted",
              "provisioning_enabled","device_writes","read_at_utc"}
    if not required.issubset(v) or v.get("serials_returned") is not False or v.get("device_writes")!=0:
        sys.exit(2)
    print("R1002_REFRESH_RESULT=BOUNDED_COUNTS_SUCCESS")
print("R1002_PHYSICAL_WRITES=0")
PY
systemctl --user is-active --quiet ipat-r945-connector.service
trap - ERR INT TERM
echo "R1002_PRIVATE_CONNECTOR_DIAGNOSTIC_ROLLOUT_PASS"
