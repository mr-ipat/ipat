#!/usr/bin/env bash
# R9.92 first-install tenant runtime preparation. No service start, DB mutation,
# firewall, DNS, certificate, or physical-device action.
set -Eeuo pipefail
umask 077
die(){ echo "R992_PREP_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R992_PREPARE:-} == PREPARE_REVIEWED_TENANT_RUNTIME ]] || die "explicit reviewed opt-in missing"
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu Server 26.04 required"
for cmd in install useradd getent systemctl stat; do command -v "$cmd" >/dev/null || die "missing tool: $cmd"; done
[[ -x /opt/ipat/bin/control-api && ! -L /opt/ipat/bin/control-api ]] || die "R9.85 checksum-pinned control-api binary required first"
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
for unit in ipat-tenant-api.service ipat-tenant-oidc@.service; do
  [[ -f "$root/deploy/systemd/$unit" && ! -L "$root/deploy/systemd/$unit" ]] || die "trusted unit missing: $unit"
  [[ ! -e "/etc/systemd/system/$unit" && ! -L "/etc/systemd/system/$unit" ]] || die "existing unit requires upgrade review: $unit"
done
for u in ipattapi ipattoidc; do
  if getent passwd "$u" >/dev/null; then
    shell=$(getent passwd "$u" | cut -d: -f7)
    [[ $shell == /usr/sbin/nologin || $shell == /bin/false ]] || die "existing $u is not nologin"
  else
    useradd --system --home-dir /nonexistent --shell /usr/sbin/nologin "$u"
  fi
done
install -d -o root -g root -m 0700 /etc/ipat /etc/ipat/tenant-oidc
install -d -o ipattoidc -g ipattoidc -m 0700 /var/lib/ipat-tenant-oidc /var/lib/ipat-tenant-oidc/secrets
for unit in ipat-tenant-api.service ipat-tenant-oidc@.service; do
  install -o root -g root -m 0644 "$root/deploy/systemd/$unit" "/etc/systemd/system/$unit"
done
systemctl daemon-reload
systemctl disable --now ipat-tenant-api.service 2>/dev/null || true
echo R992_TENANT_RUNTIME_PREPARED_SERVICES_DISABLED
