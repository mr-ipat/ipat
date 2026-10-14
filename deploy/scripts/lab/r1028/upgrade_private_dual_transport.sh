#!/usr/bin/env bash
# R10.28: reversible nonroot rollout of a source-pinned private :3002 diagnostic.
# This script never authorizes production SaaS, OLT CLI or firmware actions.
set -Eeuo pipefail
umask 077
refuse(){ echo "R1028_PREVIEW_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -ne 0 ]] || refuse "nonroot only"
[[ ${IPAT_R1028_APPLY:-} == UPGRADE_REVIEWED_PRIVATE_DUAL_TRANSPORT ]] || refuse "explicit reviewed preview opt-in absent"
commit=${IPAT_R1028_RELEASE_COMMIT:-}
digest=${IPAT_R1028_RELEASE_SHA256:-}
[[ $commit =~ ^[0-9a-f]{40}$ && $digest =~ ^[0-9a-f]{64}$ ]] || refuse "exact main commit and SHA256 required"
bundle="$HOME/.local/share/ipat/release-candidates/$commit"
binary="$bundle/control-api"
source_root=${IPAT_R1028_SOURCE_ROOT:-}
[[ $source_root == /* && -d $source_root && ! -L $source_root ]] || refuse "reviewed absolute source root required"
[[ $(git -C "$source_root" rev-parse HEAD) == "$commit" ]] || refuse "source commit mismatch"
[[ -z $(git -C "$source_root" status --porcelain --untracked-files=all) ]] || refuse "source must be clean"
[[ -d $bundle && ! -L $bundle && -f $binary && ! -L $binary ]] || refuse "release candidate missing or unsafe"
[[ $(cat "$bundle/SOURCE_COMMIT") == "$commit" ]] || refuse "bundle commit mismatch"
(cd "$bundle" && sha256sum --status -c SHA256SUMS) || refuse "candidate manifest SHA256 mismatch"
[[ $(sha256sum "$binary" | awk '{print $1}') == "$digest" ]] || refuse "reviewed binary SHA256 mismatch"
[[ $(stat -c %a "$binary") == 500 ]] || refuse "candidate file permission drift"
file "$binary" | grep -q 'ELF 64-bit.*x86-64' || refuse "wrong executable architecture"
unit=ipat-r911-preview.service
dropdir="$HOME/.config/systemd/user/$unit.d"
override="$dropdir/zzzzz-r1028-dual-transport.conf"
[[ -d $dropdir && ! -L $dropdir && ! -e $override && ! -L $override ]] || refuse "existing override/unsafe systemd directory"
systemctl --user is-active --quiet "$unit" || refuse "previous preview inactive"
systemctl --user is-active --quiet ipat-r945-connector.service || refuse "private connector inactive"
previous=$(systemctl --user show "$unit" -p ExecStart --value | sed -n 's/.*path=\([^ ;]*\).*/\1/p')
[[ $previous == "$HOME/.local/share/ipat/release-candidates/331a443db47d0447cb6dba947e7b5db85c0c36e3/control-api" ]] || refuse "unexpected previous executable"
[[ $(sha256sum "$previous" | awk '{print $1}') == 8b763e4e4dd35b5c259204cec66a4cf5e0992bcffbac2be8d266d5214d3d0dfb ]] || refuse "previous binary checksum drift"
for p in 80 443 3003 3005 3006 5432; do
 [[ -z $(ss -H -ltn "( sport = :$p )") ]] || refuse "public/commercial port conflict: $p"
done
[[ $(curl --noproxy '*' -s -o /dev/null -w '%{http_code}' --max-time 4 http://127.0.0.1:3002/healthz) == 200 ]] || refuse "previous preview unhealthy"
armed=NO
temp=
rollback(){
 if [[ $armed == YES ]]; then
   rm -f -- "$override"
   systemctl --user daemon-reload || true
   systemctl --user restart "$unit" || true
   echo R1028_PREVIOUS_PRIVATE_PREVIEW_RESTORED >&2
 fi
 [[ -z $temp ]] || rm -f -- "$temp"
}
trap rollback EXIT INT TERM ERR
temp=$(mktemp "$dropdir/.r1028.pending.XXXXXXXX")
printf '[Service]\nExecStart=\nExecStart=%s\n' "$binary" > "$temp"
chmod 0600 "$temp"
mv "$temp" "$override"
temp=
armed=YES
systemctl --user daemon-reload
resolved=$(systemctl --user show "$unit" -p ExecStart --value | sed -n 's/.*path=\([^ ;]*\).*/\1/p')
[[ $resolved == "$binary" ]] || refuse "override precedence mismatch"
systemctl --user restart "$unit"
healthy=NO
for _ in {1..16}; do
 if systemctl --user is-active --quiet "$unit" &&
    curl --noproxy '*' -fsS --max-time 2 http://127.0.0.1:3002/healthz >/dev/null 2>&1; then
    healthy=YES
    break
 fi
 sleep 1
done
[[ $healthy == YES ]] || refuse "private preview failed restart"
python3 - <<'PY'
import json,urllib.request
base="http://127.0.0.1:3002"
def request(path):
 with urllib.request.urlopen(base+path,timeout=8) as response:
  assert response.status==200, (path,response.status)
  return response.read()
dashboard=request("/lab/device-workbench")
assert b'id="ipat-c320-check-network"' in dashboard
assert b"Periksa jaringan SSH" in dashboard
v=json.loads(request("/lab/c320-owner-network-probe"))
assert v["target"]=="DEV-01"
for key,port,marker in (("ssh",321,"server_banner_observed"),("telnet",323,"negotiation_observed")):
 item=v[key]
 assert item["port"]==port
 assert item["tcp_reachable"] is True and item["probe_stage"]=="TCP_REACHABLE_AUTH_NOT_TESTED",(key,item)
 assert item[marker] is True,(key,item)
for key in ("ssh_authentication_verified","telnet_authentication_verified","production_adopted","physical_writes_enabled"):
 assert v[key] is False,key
connection=json.loads(request("/lab/c320-owner-connection"))
assert connection["production_adopted"] is False
assert connection["physical_writes_enabled"] is False
print("R1028_TWO_PORTS_TCP_PROTOCOL_NEGOTIATION_PRIVATE_HTTP_PASS")
PY
systemctl --user is-active --quiet ipat-r945-connector.service || refuse "connector changed after restart"
[[ $(ss -H -ltn '( sport = :3002 )' | awk '{print $4}') == 127.0.0.1:3002 ]] || refuse "preview not loopback-only"
for p in 80 443 3003 3005 3006 5432; do
 [[ -z $(ss -H -ltn "( sport = :$p )") ]] || refuse "unexpected new commercial/public port: $p"
done
armed=NO
trap - EXIT INT TERM ERR
echo R1028_PRIVATE_TWO_TRANSPORT_DASHBOARD_LIVE_VERIFIED
echo PUBLIC_GO_FALSE
