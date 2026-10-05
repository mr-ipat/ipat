#!/usr/bin/env bash
# R9.85 first-install preparation for separate nonroot Platform Owner runtime.
# No service start, no PostgreSQL changes, no firewall/DNS/device changes.
set -Eeuo pipefail
umask 077

die(){ echo "R985_PREP_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R985_PREPARE:-} == PREPARE_REVIEWED_PLATFORM_RUNTIME ]] || die "explicit reviewed prepare opt-in missing"
[[ -r /etc/os-release ]] || die "OS identity unavailable"
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "requires Ubuntu Server 26.04"

for cmd in install useradd getent sha256sum systemctl python3; do command -v "$cmd" >/dev/null || die "missing tool: $cmd"; done
src=${IPAT_R985_BINARY_SOURCE:-}
expected=${IPAT_R985_BINARY_SHA256:-}
[[ -n $src && -f $src && ! -L $src && -n $expected ]] || die "regular binary source and expected SHA256 required"
[[ $expected =~ ^[0-9a-f]{64}$ ]] || die "invalid expected SHA256"
actual=$(sha256sum "$src" | awk '{print $1}')
[[ $actual == "$expected" ]] || die "binary SHA256 mismatch"

api_user=ipatpapi
oidc_user=ipatpoidc
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
[[ ! -e /opt/ipat/bin/control-api && ! -L /opt/ipat/bin/control-api ]] || die "existing runtime binary requires explicit upgrade review"
for unit in ipat-platform-api.service ipat-platform-oidc.service; do
  source_unit="$repo_root/deploy/systemd/$unit"
  [[ -f $source_unit && ! -L $source_unit ]] || die "missing trusted unit $source_unit"
  target="/etc/systemd/system/$unit"
  [[ ! -e $target && ! -L $target ]] || die "existing $target requires explicit upgrade review"
done

for u in "$api_user" "$oidc_user"; do
  if getent passwd "$u" >/dev/null; then
    shell=$(getent passwd "$u" | cut -d: -f7)
    [[ $shell == /usr/sbin/nologin || $shell == /bin/false ]] || die "existing $u is not a system nologin identity"
  else
    useradd --system --home-dir /nonexistent --shell /usr/sbin/nologin "$u"
  fi
done

install -d -o root -g root -m 0755 /opt/ipat /opt/ipat/bin
install -o root -g root -m 0755 "$src" /opt/ipat/bin/control-api
install -d -o root -g root -m 0700 /etc/ipat
install -d -o "$oidc_user" -g "$oidc_user" -m 0700 /var/lib/ipat-platform-oidc/secrets

for unit in ipat-platform-api.service ipat-platform-oidc.service; do
  source_unit="$repo_root/deploy/systemd/$unit"
  target="/etc/systemd/system/$unit"
  install -o root -g root -m 0644 "$source_unit" "$target"
done
systemctl daemon-reload
systemctl disable --now ipat-platform-api.service ipat-platform-oidc.service 2>/dev/null || true
echo "R985_PREPARED_BINARY_SHA256=$actual"
echo "R985_SECRET_DIR=/var/lib/ipat-platform-oidc/secrets"
echo "R985_ENV_DIR=/etc/ipat"
echo "R985_PLATFORM_RUNTIME_PREPARED_SERVICES_DISABLED"
