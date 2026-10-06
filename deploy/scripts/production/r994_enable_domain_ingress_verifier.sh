#!/usr/bin/env bash
# R9.94 enable prepared ingress verifier after reviewed PostgreSQL/edge readiness.
# No firewall, DNS, schema, package or physical-device mutation.
set -Eeuo pipefail
umask 077

die(){ echo "R994_ENABLE_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R994_ENABLE:-} == ENABLE_REVIEWED_DOMAIN_INGRESS_VERIFIER ]] || die "explicit enable opt-in missing"

db=${IPAT_R994_DB_NAME:-}
sock=${IPAT_R994_DB_SOCKET:-}
ip=${IPAT_R994_PUBLIC_IPV4:-}
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe PostgreSQL target"
python3 - "$ip" <<'PY' || die "invalid reviewed public IPv4"
import ipaddress,sys
try:
    v=ipaddress.IPv4Address(sys.argv[1])
except ValueError:
    raise SystemExit(1)
if not v.is_global:
    raise SystemExit(1)
PY

for c in install getent runuser psql systemctl systemd-analyze; do
  command -v "$c" >/dev/null || die "missing tool: $c"
done
getent passwd ipatdverify >/dev/null || die "R9.94 verifier identity not prepared"
[[ -x /opt/ipat/bin/r994-verify-customer-ingress ]] || die "verifier executable not prepared"
for f in /etc/systemd/system/ipat-domain-ingress-verifier.service /etc/systemd/system/ipat-domain-ingress-verifier.timer; do
  [[ -f $f && ! -L $f ]] || die "prepared systemd unit missing"
done
[[ ! -e /etc/ipat/domain-ingress-verifier.env && ! -L /etc/ipat/domain-ingress-verifier.env ]] || die "existing verifier env requires explicit upgrade review"

role=$(runuser -u ipatdverify -- psql -X -w -h "$sock" -d "$db"   -U ipat_domain_ingress_verifier_login -Atqc 'select current_user' 2>/dev/null || true)
[[ $role == ipat_domain_ingress_verifier_login ]] || die "restricted ingress verifier peer auth failed"
raw=$(runuser -u ipatdverify -- psql -X -w -h "$sock" -d "$db"   -U ipat_domain_ingress_verifier_login -Atqc   "select has_table_privilege(current_user,'ipat_platform.tenant_domains','SELECT')::int" 2>/dev/null || true)
[[ $raw == 0 ]] || die "ingress verifier unexpectedly has raw tenant-domain table access"

systemd-analyze verify /etc/systemd/system/ipat-domain-ingress-verifier.service   /etc/systemd/system/ipat-domain-ingress-verifier.timer >/dev/null

tmp=$(mktemp /etc/ipat/domain-ingress-verifier.env.pending.XXXXXX)
rollback(){
  systemctl disable --now ipat-domain-ingress-verifier.timer >/dev/null 2>&1 || true
  rm -f "$tmp" /etc/ipat/domain-ingress-verifier.env
}
trap rollback ERR INT TERM
cat >"$tmp" <<ENV
IPAT_R994_VERIFY_INGRESS=YES
IPAT_R994_ALLOW_ACTIVATE=YES
IPAT_R994_DB_NAME=$db
IPAT_R994_DB_SOCKET=$sock
IPAT_R994_PUBLIC_IPV4=$ip
ENV
install -o root -g root -m 0600 "$tmp" /etc/ipat/domain-ingress-verifier.env
rm -f "$tmp"
systemctl daemon-reload
systemctl enable --now ipat-domain-ingress-verifier.timer >/dev/null
systemctl is-enabled --quiet ipat-domain-ingress-verifier.timer
systemctl is-active --quiet ipat-domain-ingress-verifier.timer
trap - ERR INT TERM
echo R994_DOMAIN_INGRESS_VERIFIER_ENABLED_REVIEWED_TARGET
