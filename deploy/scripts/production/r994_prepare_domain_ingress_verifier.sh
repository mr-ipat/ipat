#!/usr/bin/env bash
# R9.94 first-install verifier identity/service preparation. No DB/network mutation.
set -Eeuo pipefail
umask 077
die(){ echo "R994_PREP_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R994_PREPARE:-} == PREPARE_REVIEWED_DOMAIN_INGRESS_VERIFIER ]] || die "explicit prepare opt-in missing"
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu Server 26.04 required"
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
for f in deploy/scripts/production/r994_verify_customer_ingress.sh deploy/systemd/ipat-domain-ingress-verifier.service deploy/systemd/ipat-domain-ingress-verifier.timer; do
  [[ -f "$root/$f" && ! -L "$root/$f" ]] || die "trusted source missing: $f"
done
for c in install useradd getent systemctl; do command -v "$c" >/dev/null || die "missing tool: $c"; done
[[ -d /opt/ipat/bin && ! -L /opt/ipat/bin ]] || die "R9.85 runtime directory required"
if getent passwd ipatdverify >/dev/null; then
  shell=$(getent passwd ipatdverify | cut -d: -f7)
  [[ $shell == /usr/sbin/nologin || $shell == /bin/false ]] || die "existing verifier user is not nologin"
else
  useradd --system --home-dir /nonexistent --shell /usr/sbin/nologin ipatdverify
fi
for path in /opt/ipat/bin/r994-verify-customer-ingress /etc/systemd/system/ipat-domain-ingress-verifier.service /etc/systemd/system/ipat-domain-ingress-verifier.timer; do
  [[ ! -e $path && ! -L $path ]] || die "existing verifier artifact requires upgrade review"
done
install -o root -g root -m 0755 "$root/deploy/scripts/production/r994_verify_customer_ingress.sh" /opt/ipat/bin/r994-verify-customer-ingress
install -d -o root -g root -m 0700 /etc/ipat
install -o root -g root -m 0644 "$root/deploy/systemd/ipat-domain-ingress-verifier.service" /etc/systemd/system/ipat-domain-ingress-verifier.service
install -o root -g root -m 0644 "$root/deploy/systemd/ipat-domain-ingress-verifier.timer" /etc/systemd/system/ipat-domain-ingress-verifier.timer
systemctl daemon-reload
systemctl disable --now ipat-domain-ingress-verifier.timer 2>/dev/null || true
echo R994_DOMAIN_INGRESS_VERIFIER_PREPARED_DISABLED
