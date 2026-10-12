#!/usr/bin/env bash
# R10.17: reversible, rootless loopback preview rollout; never public SaaS.
set -Eeuo pipefail
umask 077
refuse(){ echo "R1017_PREVIEW_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -ne 0 ]] || refuse "nonroot only"
[[ ${IPAT_R1017_APPLY:-} == UPGRADE_PRIVATE_PREVIEW_ONLY ]] || refuse "explicit opt-in required"
[[ $(uname -s) == Linux ]] || refuse "Linux only"
commit=331a443db47d0447cb6dba947e7b5db85c0c36e3
digest=8b763e4e4dd35b5c259204cec66a4cf5e0992bcffbac2be8d266d5214d3d0dfb
bundle="$HOME/.local/share/ipat/release-candidates/$commit"
binary="$bundle/control-api"
unit=ipat-r911-preview.service
dropdir="$HOME/.config/systemd/user/$unit.d"
override="$dropdir/zzzz-r1017-native-main.conf"
for cmd in systemctl curl sha256sum file stat ss mktemp mv rm; do
  command -v "$cmd" >/dev/null || refuse "missing command: $cmd"
done
[[ -d $bundle && ! -L $bundle && -f $binary && ! -L $binary ]] || refuse "unsafe binary"
[[ -f $bundle/SHA256SUMS && ! -L $bundle/SHA256SUMS ]] || refuse "unsafe manifest"
[[ $(cat "$bundle/SOURCE_COMMIT") == "$commit" ]] || refuse "source mismatch"
(cd "$bundle"; sha256sum --status -c SHA256SUMS) || refuse "bundle checksum failure"
[[ $(sha256sum "$binary" | awk '{print $1}') == "$digest" ]] || refuse "binary checksum failure"
file "$binary" | grep -q 'ELF 64-bit.*x86-64' || refuse "wrong architecture"
[[ $(stat -c %a "$binary") == 500 ]] || refuse "binary permissions changed"
[[ -d $dropdir && ! -L $dropdir && ! -e $override && ! -L $override ]] || refuse "preexisting override"
systemctl --user is-active --quiet "$unit" || refuse "current preview inactive"
systemctl --user is-active --quiet ipat-r945-connector.service || refuse "connector status unsafe"
current=$(systemctl --user show "$unit" -p ExecStart --value | sed -n 's/.*path=\([^ ;]*\).*/\1/p')
[[ $current == "$HOME/.cache/ipat/r1003-release/control-api" ]] || refuse "unreviewed previous executable"
curl -fsS --max-time 4 http://127.0.0.1:3002/lab/device-workbench >/dev/null || refuse "existing dashboard unhealthy"
[[ -z $(ss -H -ltn '( sport = :443 )' 2>/dev/null) ]] || refuse "public ingress appeared"

armed=NO
temp=
rollback(){
  if [[ $armed == YES ]]; then
    rm -f -- "$override"
    systemctl --user daemon-reload || true
    systemctl --user restart "$unit" || true
    echo R1017_ROLLBACK_ORIGINAL_PREVIEW_RESTORED >&2
  fi
  [[ -z $temp ]] || rm -f -- "$temp"
}
trap rollback EXIT INT TERM ERR
temp=$(mktemp "$dropdir/.r1017.pending.XXXXXXXX")
printf '[Service]\nExecStart=\nExecStart=%s\n' "$binary" >"$temp"
chmod 0600 "$temp"
[[ ! -e $override && ! -L $override ]] || refuse "override collision"
mv "$temp" "$override"
temp=
armed=YES
systemctl --user daemon-reload
resolved=$(systemctl --user show "$unit" -p ExecStart --value | sed -n 's/.*path=\([^ ;]*\).*/\1/p')
[[ $resolved == "$binary" ]] || refuse "override order mismatch"
systemctl --user restart "$unit"
healthy=NO
for _ in {1..12}; do
  if systemctl --user is-active --quiet "$unit" &&
     curl -fsS --max-time 2 http://127.0.0.1:3002/healthz >/dev/null 2>&1; then
    healthy=YES; break
  fi
  sleep 1
done
[[ $healthy == YES ]] || refuse "canary restart unhealthy"
for endpoint in /healthz /lab/device-workbench /lab/platform-admin/domains /lab/c320-owner-connection /lab/owner/devices /lab/owner/platform-domains; do
  status=$(curl -s -o /dev/null -w '%{http_code}' --max-time 4 "http://127.0.0.1:3002$endpoint")
  [[ $status == 200 ]] || refuse "endpoint failure $endpoint $status"
done
systemctl --user is-active --quiet ipat-r945-connector.service || refuse "connector changed"
[[ -z $(ss -H -ltn '( sport = :443 )' 2>/dev/null) ]] || refuse "public listener unexpected"
ss -H -ltn '( sport = :3002 )' | grep -Fq '127.0.0.1:3002' || refuse "not bound loopback"
armed=NO
trap - EXIT INT TERM ERR
echo R1017_NEW_NATIVE_PRIVATE_PREVIEW_VERIFIED
echo "BUILD_COMMIT=$commit"
echo "PRIVATE_URL=http://127.0.0.1:3002/lab/device-workbench"
echo PUBLIC_GO_FALSE
