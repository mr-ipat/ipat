#!/usr/bin/env bash
# R9.96 first-install production foundation orchestrator.
# Creates NO public listener and activates NO application HTTP/OIDC/verifier service.
set -Eeuo pipefail
umask 077

rollback_armed=NO
die(){
  echo "R996_FOUNDATION_REFUSED: $*" >&2
  if [[ ${rollback_armed:-NO} == YES ]] && declare -F rollback >/dev/null; then
    rollback
    rollback_armed=NO
  fi
  exit 4
}
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R996_APPLY:-} == APPLY_REVIEWED_PRODUCTION_FOUNDATION ]] || die "explicit reviewed foundation opt-in missing"

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
expected=${IPAT_R996_SOURCE_COMMIT:-}
att=${IPAT_R996_RECOVERY_ATTESTATION:-}
binary=${IPAT_R996_BINARY_SOURCE:-}
binary_sha=${IPAT_R996_BINARY_SHA256:-}
[[ $expected =~ ^[0-9a-f]{40}$ ]] || die "exact source commit required"
[[ $binary_sha =~ ^[0-9a-f]{64}$ ]] || die "exact binary SHA256 required"
[[ -n $att && -n $binary ]] || die "recovery attestation and binary source required"

for c in git python3 sha256sum stat ss getent systemctl userdel rmdir rm install mktemp; do
  command -v "$c" >/dev/null || die "missing base tool: $c"
done
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu Server 26.04 required"

actual_source=$(git -C "$root" rev-parse HEAD 2>/dev/null || true)
[[ $actual_source == "$expected" ]] || die "running repository commit mismatch"
[[ -z $(git -C "$root" status --porcelain --untracked-files=no) ]] || die "tracked working tree must be clean"
[[ -f $binary && ! -L $binary ]] || die "binary source must be regular non-symlink"
[[ $(sha256sum "$binary" | awk '{print $1}') == "$binary_sha" ]] || die "binary SHA256 mismatch"
python3 "$root/deploy/scripts/production/r996_validate_recovery_attestation.py" \
  --path "$att" --expected-source "$expected" >/dev/null \
  || die "reviewed recovery attestation rejected"

users=(ipatpapi ipatpoidc ipattapi ipattoidc ipatdverify ipatdnsverify ipatpgmigrate)
for u in "${users[@]}"; do
  ! getent passwd "$u" >/dev/null || die "existing IPAT runtime identity requires upgrade/recovery review: $u"
done
artifacts=(
 /opt/ipat/bin/control-api
 /opt/ipat/bin/r994-verify-customer-ingress
 /etc/systemd/system/ipat-platform-api.service
 /etc/systemd/system/ipat-platform-oidc.service
 /etc/systemd/system/ipat-tenant-api.service
 /etc/systemd/system/ipat-tenant-oidc@.service
 /etc/systemd/system/ipat-domain-ingress-verifier.service
 /etc/systemd/system/ipat-domain-ingress-verifier.timer
 /etc/systemd/system/ipat-domain-ownership-verifier.service
 /var/lib/ipat/r996-foundation.json
)
for p in "${artifacts[@]}"; do
  [[ ! -e $p && ! -L $p ]] || die "existing artifact requires explicit upgrade/recovery review: $p"
done
if command -v pg_lsclusters >/dev/null; then
  [[ -z $(pg_lsclusters --no-header 2>/dev/null || true) ]] || die "existing PostgreSQL cluster requires separate migration review"
fi
listeners=$(ss -H -ltn '( sport = :80 or sport = :443 or sport = :5432 or sport = :3003 or sport = :3005 or sport = :3006 )' 2>/dev/null || true)
[[ -z $listeners ]] || die "managed/public/runtime port already has a listener"

rollback(){
  set +e
  for unit in ipat-domain-ownership-verifier.service ipat-domain-ingress-verifier.timer \
              ipat-domain-ingress-verifier.service ipat-tenant-api.service \
              ipat-platform-api.service ipat-platform-oidc.service; do
    systemctl disable --now "$unit" >/dev/null 2>&1 || true
  done
  systemctl disable --now 'ipat-tenant-oidc@*.service' >/dev/null 2>&1 || true
  if command -v pg_lsclusters >/dev/null && pg_lsclusters --no-header 2>/dev/null | awk '$1=="18"&&$2=="ipat"{found=1} END{exit !found}'; then
    command -v pg_dropcluster >/dev/null && pg_dropcluster --stop 18 ipat >/dev/null 2>&1 || true
  fi
  rm -f /etc/systemd/system/ipat-platform-api.service /etc/systemd/system/ipat-platform-oidc.service \
        /etc/systemd/system/ipat-tenant-api.service /etc/systemd/system/ipat-tenant-oidc@.service \
        /etc/systemd/system/ipat-domain-ingress-verifier.service /etc/systemd/system/ipat-domain-ingress-verifier.timer \
        /etc/systemd/system/ipat-domain-ownership-verifier.service \
        /opt/ipat/bin/r994-verify-customer-ingress /opt/ipat/bin/control-api
  rm -rf /var/lib/ipat-platform-oidc /var/lib/ipat-tenant-oidc /var/lib/ipat-domain-ownership-verifier
  rmdir /opt/ipat/bin /opt/ipat /etc/ipat /var/lib/ipat 2>/dev/null || true
  for u in ipatpgmigrate ipatdnsverify ipatdverify ipattoidc ipattapi ipatpoidc ipatpapi; do
    getent passwd "$u" >/dev/null && userdel "$u" >/dev/null 2>&1 || true
  done
  systemctl daemon-reload >/dev/null 2>&1 || true
  echo "R996_ROLLBACK_SAFE_STATE_APPLICATIONS_DISABLED_NO_PUBLIC_EDGE" >&2
}
rollback_armed=YES
trap rollback ERR INT TERM

IPAT_R985_PREPARE=PREPARE_REVIEWED_PLATFORM_RUNTIME \
IPAT_R985_BINARY_SOURCE="$binary" IPAT_R985_BINARY_SHA256="$binary_sha" \
  "$root/deploy/scripts/production/r985_prepare_platform_runtime.sh"

IPAT_R992_PREPARE=PREPARE_REVIEWED_TENANT_RUNTIME \
  "$root/deploy/scripts/production/r992_prepare_tenant_runtime.sh"

IPAT_R994_PREPARE=PREPARE_REVIEWED_DOMAIN_INGRESS_VERIFIER \
  "$root/deploy/scripts/production/r994_prepare_domain_ingress_verifier.sh"

IPAT_R995_PREPARE=PREPARE_REVIEWED_DOMAIN_OWNERSHIP_VERIFIER \
  "$root/deploy/scripts/production/r995_prepare_domain_ownership_verifier.sh"

"$root/deploy/scripts/production/r987_bootstrap_postgres18.sh"

for unit in ipat-platform-api.service ipat-platform-oidc.service ipat-tenant-api.service \
            ipat-domain-ingress-verifier.timer ipat-domain-ownership-verifier.service; do
  [[ $(systemctl is-active "$unit" 2>/dev/null || true) != active ]] || die "application/verifier unexpectedly active: $unit"
done
[[ -z $(ss -H -ltn '( sport = :80 or sport = :443 or sport = :5432 or sport = :3003 or sport = :3005 or sport = :3006 )' 2>/dev/null || true) ]] \
  || die "foundation unexpectedly exposes managed TCP listener"

install -d -o root -g root -m 0700 /var/lib/ipat
tmp=$(mktemp /var/lib/ipat/r996-foundation.pending.XXXXXX)
python3 - "$tmp" "$expected" "$binary_sha" <<'PY'
import json,sys
from datetime import datetime,timezone
p,source,binary=sys.argv[1:]
with open(p,"w") as f:
 json.dump({
  "schema":"ipat.production-foundation.v1",
  "source_commit":source,
  "control_api_sha256":binary,
  "created_at_utc":datetime.now(timezone.utc).isoformat(),
  "postgresql":"18/ipat-local-unix-socket-only",
  "application_services":"DISABLED",
  "public_http_https":"NOT_CONFIGURED",
  "production_pitr_acceptance":"REQUIRED_BEFORE_PUBLIC_ACTIVATION",
  "public_go":False,
 },f,sort_keys=True)
PY
install -o root -g root -m 0600 "$tmp" /var/lib/ipat/r996-foundation.json
rm -f "$tmp"
rollback_armed=NO
trap - ERR INT TERM
echo "R996_PRODUCTION_FOUNDATION_PREPARED_LOCAL_DB_ONLY_NO_PUBLIC_EDGE"
