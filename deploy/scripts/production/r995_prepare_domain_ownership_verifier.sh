#!/usr/bin/env bash
# R9.95 prepare dedicated DNS ownership verifier identity/service.
# No PostgreSQL schema, firewall, DNS, customer or device mutation.
set -Eeuo pipefail
umask 077

die(){ echo "R995_PREP_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R995_PREPARE:-} == PREPARE_REVIEWED_DOMAIN_OWNERSHIP_VERIFIER ]] || die "explicit prepare opt-in missing"
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu Server 26.04 required"

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
for f in deploy/systemd/ipat-domain-ownership-verifier.service; do
  [[ -f "$root/$f" && ! -L "$root/$f" ]] || die "trusted source missing: $f"
done
for c in install useradd getent systemctl; do command -v "$c" >/dev/null || die "missing tool: $c"; done
[[ -d /opt/ipat/bin && ! -L /opt/ipat/bin ]] || die "runtime directory required"
[[ -x /opt/ipat/bin/control-api && ! -L /opt/ipat/bin/control-api ]] || die "reviewed control-api binary required"

if getent passwd ipatdnsverify >/dev/null; then
  shell=$(getent passwd ipatdnsverify | cut -d: -f7)
  [[ $shell == /usr/sbin/nologin || $shell == /bin/false ]] || die "existing ownership verifier is not nologin"
else
  useradd --system --home-dir /nonexistent --shell /usr/sbin/nologin ipatdnsverify
fi

unit=/etc/systemd/system/ipat-domain-ownership-verifier.service
[[ ! -e $unit && ! -L $unit ]] || die "existing ownership-verifier unit requires upgrade review"
install -o root -g root -m 0644 "$root/deploy/systemd/ipat-domain-ownership-verifier.service" "$unit"
systemctl daemon-reload
systemctl disable --now ipat-domain-ownership-verifier.service >/dev/null 2>&1 || true
echo R995_DOMAIN_OWNERSHIP_VERIFIER_PREPARED_DISABLED
