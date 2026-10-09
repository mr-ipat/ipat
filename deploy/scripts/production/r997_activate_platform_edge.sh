#!/usr/bin/env bash
# R9.97 reviewed Platform Owner public-edge activation after production PITR.
# Customer OIDC and customer HTTPS ingress are deliberately separate.
set -Eeuo pipefail
umask 077
rollback_armed=NO
edge_rollback=

rollback(){
  set +e
  if [[ -n ${edge_rollback:-} && -x $edge_rollback ]]; then
    "$edge_rollback" >/dev/null 2>&1 || true
    edge_rollback=
  fi
  systemctl disable --now ipat-domain-ownership-verifier.service >/dev/null 2>&1 || true
  systemctl disable --now ipat-tenant-api.service >/dev/null 2>&1 || true
  systemctl disable --now ipat-platform-api.service ipat-platform-oidc.service >/dev/null 2>&1 || true
  rm -rf /var/lib/ipat-domain-ownership-verifier
  rm -f /etc/ipat/tenant-api.env /etc/ipat/platform-api.env /etc/ipat/platform-oidc.env
  systemctl daemon-reload >/dev/null 2>&1 || true
  echo "R997_ROLLBACK_PLATFORM_RUNTIME_DISABLED_PUBLIC_EDGE_SUBSCRIPT_HANDLES_OWN_TLS_ROLLBACK" >&2
}
die(){
  echo "R997_ACTIVATION_REFUSED: $*" >&2
  if [[ ${rollback_armed:-NO} == YES ]]; then rollback; rollback_armed=NO; fi
  exit 4
}

[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R997_APPLY:-} == ACTIVATE_REVIEWED_PLATFORM_EDGE ]] || die "explicit reviewed activation opt-in missing"
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
foundation=${IPAT_R997_FOUNDATION_MARKER:-/var/lib/ipat/r996-foundation.json}
pitr=${IPAT_R997_PITR_ATTESTATION:-}
ip=${IPAT_R997_PUBLIC_IPV4:-}
api_env=${IPAT_R997_PLATFORM_API_ENV_SOURCE:-}
oidc_env=${IPAT_R997_PLATFORM_OIDC_ENV_SOURCE:-}
tenant_api_env=${IPAT_R997_TENANT_API_ENV_SOURCE:-}
db=${IPAT_R997_DB_NAME:-}
sock=${IPAT_R997_DB_SOCKET:-}
email=${IPAT_R997_ACME_EMAIL:-}
[[ -n $pitr && -n $ip && -n $api_env && -n $oidc_env && -n $tenant_api_env && -n $db && -n $sock && -n $email ]] || die "all reviewed activation inputs required"

for c in python3 systemctl ss curl stat runuser psql; do command -v "$c" >/dev/null || die "missing tool: $c"; done
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu Server 26.04 required"
python3 "$root/deploy/scripts/production/r997_validate_pitr_acceptance.py" --foundation "$foundation" --pitr "$pitr" >/dev/null \
  || die "production PITR acceptance gate rejected"

python3 - "$ip" <<'PY' || die "invalid global public IPv4"
import ipaddress,sys
try:v=ipaddress.IPv4Address(sys.argv[1])
except ValueError:raise SystemExit(1)
if not v.is_global:raise SystemExit(1)
PY
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe production PostgreSQL target"
[[ -x /opt/ipat/bin/control-api && ! -L /opt/ipat/bin/control-api ]] || die "R9.96 foundation runtime binary missing"
[[ -f /etc/systemd/system/ipat-platform-api.service && -f /etc/systemd/system/ipat-platform-oidc.service ]] || die "platform units missing"
[[ -f /etc/systemd/system/ipat-tenant-api.service ]] || die "tenant API unit missing"
[[ -f /etc/systemd/system/ipat-domain-ownership-verifier.service ]] || die "ownership verifier unit missing"

for f in "$api_env" "$oidc_env" "$tenant_api_env" "$pitr" "$foundation"; do
  [[ -f $f && ! -L $f ]] || die "unsafe staged/evidence file: $f"
  [[ $(stat -c '%u:%a:%h' "$f") == 0:600:1 ]] || die "staged/evidence file must be root-owned 0600 single-link: $f"
done
for unit in ipat-platform-api.service ipat-platform-oidc.service ipat-tenant-api.service ipat-domain-ownership-verifier.service; do
  [[ $(systemctl is-active "$unit" 2>/dev/null || true) != active ]] || die "runtime already active: $unit"
done
[[ -z $(ss -H -ltn '( sport = :80 or sport = :443 or sport = :3003 or sport = :3005 or sport = :3006 )' 2>/dev/null || true) ]] \
  || die "public/runtime listener already active before reviewed activation"

# Rollback may remove ONLY artifacts not present before this operation.
# A stale file, directory or dangling symlink requires independent recovery;
# never arm cleanup that could erase an earlier deployment's state.
. "$root/deploy/scripts/production/r1014_require_absent.sh"
r1014_require_absent \
  "/etc/ipat/platform-api.env" \
  "/etc/ipat/platform-oidc.env" \
  "/etc/ipat/tenant-api.env" \
  "/var/lib/ipat-domain-ownership-verifier" \
  "/var/lib/ipat/r997-platform-edge.json" \
  || die "R997_PREEXISTING_ACTIVATION_ARTIFACT requires explicit recovery review"

# Subscripts independently revalidate their own exact env, peer roles and rollback.
rollback_armed=YES
trap rollback ERR INT TERM

IPAT_R985_APPLY=ACTIVATE_REVIEWED_PLATFORM_RUNTIME \
IPAT_R985_PUBLIC_IPV4="$ip" \
IPAT_R985_API_ENV_SOURCE="$api_env" \
IPAT_R985_OIDC_ENV_SOURCE="$oidc_env" \
  "$root/deploy/scripts/production/r985_apply_platform_runtime.sh"

IPAT_R992_APPLY_API=ACTIVATE_REVIEWED_TENANT_API \
IPAT_R992_API_ENV_SOURCE="$tenant_api_env" \
  "$root/deploy/scripts/production/r992_apply_tenant_api.sh"

IPAT_R995_ENABLE=ENABLE_REVIEWED_DOMAIN_OWNERSHIP_VERIFIER \
IPAT_R995_DB_NAME="$db" IPAT_R995_DB_SOCKET="$sock" \
  "$root/deploy/scripts/production/r995_enable_domain_ownership_verifier.sh"

IPAT_R984_PREPARE=INSTALL_REVIEWED_IP_TLS_PREREQS \
  "$root/deploy/scripts/production/r984_prepare_ip_tls_prereqs.sh"

IPAT_R984_APPLY=APPLY_REVIEWED_IP_TLS_EDGE \
IPAT_R984_PHASE=staging IPAT_R984_PUBLIC_IPV4="$ip" IPAT_R984_ACME_EMAIL="$email" \
  "$root/deploy/scripts/production/r984_apply_ip_tls_ingress.sh"

before_edge_rollbacks=$(find /var/backups/ipat-r984 -mindepth 2 -maxdepth 2 -type f -name rollback.sh -print 2>/dev/null | sort || true)
IPAT_R984_APPLY=APPLY_REVIEWED_IP_TLS_EDGE \
IPAT_R984_PHASE=production IPAT_R984_PUBLIC_IPV4="$ip" IPAT_R984_ACME_EMAIL="$email" \
  "$root/deploy/scripts/production/r984_apply_ip_tls_ingress.sh"
after_edge_rollbacks=$(find /var/backups/ipat-r984 -mindepth 2 -maxdepth 2 -type f -name rollback.sh -print 2>/dev/null | sort || true)
new_edge_rollbacks=$(comm -13 <(printf '%s\n' "$before_edge_rollbacks") <(printf '%s\n' "$after_edge_rollbacks") | sed '/^$/d')
[[ $(wc -l <<<"$new_edge_rollbacks" | tr -d ' ') -eq 1 ]] || die "cannot identify exactly one new R9.84 rollback"
edge_rollback=$new_edge_rollbacks
[[ -x $edge_rollback ]] || die "new R9.84 rollback is not executable"

# Public edge is intentionally NOT production-go until a real human MFA browser
# login plus revocation/replay/Host checks are independently accepted.
curl --fail --silent --show-error --max-time 5 "https://$ip/" >/dev/null \
  || die "trusted public platform HTTPS probe failed after cutover"
status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 5 \
  "https://$ip/platform/auth/oidc/start" || true)
[[ $status == 303 ]] || die "public OIDC start route unavailable after cutover"

install -d -o root -g root -m 0700 /var/lib/ipat
marker=/var/lib/ipat/r997-platform-edge.json
[[ ! -e $marker && ! -L $marker ]] || die "existing R9.97 activation marker requires explicit review"
tmp=$(mktemp /var/lib/ipat/r997-platform-edge.pending.XXXXXX)
python3 - "$tmp" "$ip" <<'PY'
import json,sys
from datetime import datetime,timezone
p,ip=sys.argv[1:]
with open(p,"w") as f:
 json.dump({
  "schema":"ipat.platform-edge.v1","public_ipv4":ip,
  "activated_at_utc":datetime.now(timezone.utc).isoformat(),
  "platform_https":"ACTIVE_CA_TRUSTED_SHORTLIVED_IP_CERT",
  "platform_runtime":"ACTIVE",
  "shared_tenant_api":"ACTIVE_LOOPBACK_ONLY",
  "dns_ownership_verifier":"ACTIVE",
  "customer_oidc":"NOT_ACTIVATED",
  "customer_https_ingress":"NOT_ACTIVATED",
  "human_mfa_browser_acceptance":"REQUIRED",
  "public_go":False,
 },f,sort_keys=True)
PY
install -o root -g root -m 0600 "$tmp" "$marker"
rm -f "$tmp"
rollback_armed=NO
trap - ERR INT TERM
echo "R997_PLATFORM_EDGE_ACTIVE_PENDING_REAL_HUMAN_MFA_BROWSER_ACCEPTANCE"
