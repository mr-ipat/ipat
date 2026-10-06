#!/usr/bin/env bash
# R9.98 activate exactly one customer OIDC + HTTPS custom domain.
# Domain becomes DB-active only after exact TXT+A+trusted TLS/OIDC rechecks.
set -Eeuo pipefail
umask 077
rollback_armed=NO
customer_edge_rollback=
rollback_export_file=
installed_verifier=NO

rollback(){
  set +e
  if [[ -z ${customer_edge_rollback:-} && -n ${rollback_export_file:-} && -f $rollback_export_file && ! -L $rollback_export_file ]]; then
    if [[ $(stat -c '%u:%a:%h' "$rollback_export_file" 2>/dev/null) == 0:600:1 ]]; then
      customer_edge_rollback=$(cat "$rollback_export_file" 2>/dev/null || true)
    fi
  fi
  if [[ -n ${customer_edge_rollback:-} && -x $customer_edge_rollback ]]; then
    "$customer_edge_rollback" >/dev/null 2>&1 || true
    customer_edge_rollback=
  fi
  [[ -z ${rollback_export_file:-} ]] || rm -f "$rollback_export_file"
  if [[ -n ${instance:-} ]]; then
    systemctl disable --now "ipat-tenant-oidc@$instance.service" >/dev/null 2>&1 || true
    rm -f "/etc/ipat/tenant-oidc/$instance.env"
  fi
  if [[ ${installed_verifier:-NO} == YES ]]; then
    rm -f /opt/ipat/bin/r998-verify-exact-customer-ingress
  fi
  systemctl daemon-reload >/dev/null 2>&1 || true
  echo "R998_ROLLBACK_CUSTOMER_PUBLIC_INGRESS_AND_OIDC_REMOVED_NONACTIVE_DB_STATE_PRESERVED" >&2
}
die(){
  echo "R998_ACTIVATION_REFUSED: $*" >&2
  if [[ ${rollback_armed:-NO} == YES ]]; then rollback; rollback_armed=NO; fi
  exit 4
}

[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R998_APPLY:-} == ACTIVATE_REVIEWED_SINGLE_CUSTOMER ]] || die "explicit single-customer activation opt-in missing"
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
edge=${IPAT_R998_PLATFORM_EDGE_MARKER:-/var/lib/ipat/r997-platform-edge.json}
platform_mfa=${IPAT_R998_PLATFORM_MFA_ACCEPTANCE:-}
domain=${IPAT_R998_DOMAIN_ID:-}
expected_host=${IPAT_R998_HOST:-}
instance=${IPAT_R998_INSTANCE:-}
oidc_port=${IPAT_R998_OIDC_PORT:-}
oidc_env=${IPAT_R998_OIDC_ENV_SOURCE:-}
ip=${IPAT_R998_PUBLIC_IPV4:-}
email=${IPAT_R998_ACME_EMAIL:-}
db=${IPAT_R998_DB_NAME:-}
sock=${IPAT_R998_DB_SOCKET:-}
[[ $domain =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "invalid exact domain UUID"
[[ $instance =~ ^[a-z0-9][a-z0-9-]{0,47}$ ]] || die "invalid customer instance"
[[ $oidc_port =~ ^[0-9]+$ && $oidc_port -ge 31000 && $oidc_port -le 31999 ]] || die "OIDC port outside 31000-31999"
[[ -n $platform_mfa && -n $oidc_env && -n $ip && -n $email && $email == *@*.* && -n $expected_host ]] || die "all reviewed customer inputs required"
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe PostgreSQL target"

for c in python3 install sha256sum stat runuser psql systemctl ss curl dig sort wc tr mv mktemp awk cat sed rm; do
  command -v "$c" >/dev/null || die "missing tool: $c"
done
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu Server 26.04 required"

python3 "$root/deploy/scripts/production/r998_validate_platform_mfa_acceptance.py" \
  --edge "$edge" --acceptance "$platform_mfa" --ip "$ip" >/dev/null \
  || die "Platform Owner real-human MFA acceptance gate rejected"

for f in "$edge" "$platform_mfa" "$oidc_env"; do
  [[ -f $f && ! -L $f ]] || die "unsafe reviewed input file: $f"
  [[ $(stat -c '%u:%a:%h' "$f") == 0:600:1 ]] || die "reviewed input must be root-owned 0600 single-link: $f"
done
python3 - "$expected_host" "$ip" <<'PY' || die "invalid exact customer hostname/public IPv4"
import ipaddress,re,sys
host,ip=sys.argv[1:]
if host!=host.lower() or len(host)>253 or "." not in host or ":" in host or host.endswith((".local",".localhost",".invalid",".test",".example")):
    raise SystemExit(1)
for label in host.split("."):
    if not re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?",label):raise SystemExit(1)
if not ipaddress.IPv4Address(ip).is_global:raise SystemExit(1)
PY

get_result(){
  runuser -u ipatdverify -- psql -X -w -v ON_ERROR_STOP=1 -h "$sock" -d "$db" \
    -U ipat_domain_ingress_verifier_login -AtF $'\t' -c \
    "SELECT id,hostname,verification_name,verification_value,activation_state,COALESCE(activated_at::text,'')
       FROM ipat_platform.get_tenant_domain_activation_result('$domain'::uuid)"
}
role=$(runuser -u ipatdverify -- psql -X -w -h "$sock" -d "$db" \
  -U ipat_domain_ingress_verifier_login -Atqc 'select current_user' 2>/dev/null || true)
[[ $role == ipat_domain_ingress_verifier_login ]] || die "restricted ingress verifier peer auth failed"
raw=$(runuser -u ipatdverify -- psql -X -w -h "$sock" -d "$db" \
  -U ipat_domain_ingress_verifier_login -Atqc \
  "select has_table_privilege(current_user,'ipat_platform.tenant_domains','SELECT')::int" 2>/dev/null || true)
[[ $raw == 0 ]] || die "ingress verifier unexpectedly has raw tenant-domain access"

row=$(get_result) || die "cannot read exact customer domain"
[[ -n $row && $(wc -l <<<"$row" | tr -d ' ') -eq 1 ]] || die "customer domain not eligible"
IFS=$'\t' read -r got_id host verification_name verification_value state activated_at <<<"$row"
[[ $got_id == "$domain" && $host == "$expected_host" ]] || die "domain id/Host mismatch"
[[ $verification_name == "_ipat-verify.$host" ]] || die "verification name mismatch"
[[ $verification_value =~ ^ipat-domain=[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "persisted TXT token invalid"
[[ $state == ownership_verified || $state == routing_ready || $state == tls_ready || $state == active ]] || die "domain state not eligible"

# Existing active domain path is read-only verification; never re-create OIDC/cert.
if [[ $state == active ]]; then
  dig @1.1.1.1 +short TXT "$verification_name" 2>/dev/null |
    python3 -c 'import shlex,sys
expected=sys.argv[1]; vals=[]
for line in sys.stdin:
    try: vals.append("".join(shlex.split(line.strip())))
    except ValueError: raise SystemExit(2)
raise SystemExit(0 if expected in vals else 1)' "$verification_value" \
    || die "active domain TXT no longer matches"
  addresses=$(dig @1.1.1.1 +short A "$host" | sed '/^$/d' | sort -u)
  [[ $addresses == "$ip" ]] || die "active domain A no longer matches"
  curl --fail --silent --show-error --max-time 5 --resolve "$host:443:$ip" "https://$host/" >/dev/null || die "active customer HTTPS unavailable"
  code=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 5 --resolve "$host:443:$ip" "https://$host/auth/oidc/start" || true)
  [[ $code == 303 ]] || die "active customer OIDC start unavailable"
  echo "R998_CUSTOMER_DOMAIN_ALREADY_ACTIVE_REVERIFIED domain_id=$domain host=$host"
  exit 0
fi

# First mutation starts only after all exact authority and MFA preflight passed.
rollback_armed=YES
trap rollback ERR INT TERM

verifier_src="$root/deploy/scripts/production/r998_verify_exact_customer_ingress.sh"
[[ -f $verifier_src && ! -L $verifier_src ]] || die "trusted exact verifier source missing"
verifier=/opt/ipat/bin/r998-verify-exact-customer-ingress
expected_verifier_sha=$(sha256sum "$verifier_src" | awk '{print $1}')
if [[ -e $verifier || -L $verifier ]]; then
  [[ -f $verifier && ! -L $verifier && $(stat -c '%u:%a:%h' "$verifier") == 0:755:1 ]] || die "existing exact verifier artifact unsafe"
  [[ $(sha256sum "$verifier" | awk '{print $1}') == "$expected_verifier_sha" ]] || die "existing exact verifier checksum mismatch"
else
  install -o root -g root -m 0755 "$verifier_src" "$verifier"
  installed_verifier=YES
fi

[[ ! -e "/etc/ipat/tenant-oidc/$instance.env" && ! -L "/etc/ipat/tenant-oidc/$instance.env" ]] || die "customer OIDC instance already exists"
[[ $(systemctl is-active "ipat-tenant-oidc@$instance.service" 2>/dev/null || true) != active ]] || die "customer OIDC service already active"

IPAT_R992_ADD_OIDC=ADD_REVIEWED_TENANT_OIDC_INSTANCE \
IPAT_R992_OIDC_ENV_SOURCE="$oidc_env" IPAT_R992_OIDC_INSTANCE="$instance" \
  "$root/deploy/scripts/production/r992_add_tenant_oidc_instance.sh"

IPAT_R993_APPLY=APPLY_REVIEWED_CUSTOMER_INGRESS IPAT_R993_PHASE=staging \
IPAT_R993_HOST="$host" IPAT_R993_INSTANCE="$instance" IPAT_R993_OIDC_PORT="$oidc_port" \
IPAT_R993_PUBLIC_IPV4="$ip" IPAT_R993_ACME_EMAIL="$email" \
IPAT_R993_VERIFICATION_VALUE="$verification_value" \
  "$root/deploy/scripts/production/r993_apply_customer_ingress.sh"

rollback_export_file="/run/ipat-r998-r993-rollback-$$-$RANDOM.path"
[[ ! -e $rollback_export_file && ! -L $rollback_export_file ]] || die "rollback export collision"
IPAT_R993_APPLY=APPLY_REVIEWED_CUSTOMER_INGRESS IPAT_R993_PHASE=production \
IPAT_R993_HOST="$host" IPAT_R993_INSTANCE="$instance" IPAT_R993_OIDC_PORT="$oidc_port" \
IPAT_R993_PUBLIC_IPV4="$ip" IPAT_R993_ACME_EMAIL="$email" \
IPAT_R993_VERIFICATION_VALUE="$verification_value" \
IPAT_R993_ROLLBACK_EXPORT_FILE="$rollback_export_file" \
  "$root/deploy/scripts/production/r993_apply_customer_ingress.sh"
[[ -f $rollback_export_file && ! -L $rollback_export_file && $(stat -c '%u:%a:%h' "$rollback_export_file") == 0:600:1 ]] \
  || die "exact customer rollback export missing or unsafe"
customer_edge_rollback=$(cat "$rollback_export_file")
[[ $customer_edge_rollback == /var/backups/ipat-r993/*/rollback.sh && -x $customer_edge_rollback ]] \
  || die "exported customer ingress rollback is invalid"
rm -f "$rollback_export_file"
rollback_export_file=

runuser -u ipatdverify -- env \
  IPAT_R998_VERIFY_ONE=YES IPAT_R998_DOMAIN_ID="$domain" \
  IPAT_R998_DB_NAME="$db" IPAT_R998_DB_SOCKET="$sock" IPAT_R998_PUBLIC_IPV4="$ip" \
  "$verifier"

# DB active is the authoritative boundary. Disarm destructive rollback now.
# Any later marker failure is evidence-only and must not desynchronize DB/edge.
rollback_armed=NO
trap - ERR INT TERM
customer_edge_rollback=

# Evidence marker is non-authoritative. From this point the database and public
# edge are already active; logging failure must not trigger rollback/desync.
set +e
install -d -o root -g root -m 0700 /var/lib/ipat/customer-activations
marker="/var/lib/ipat/customer-activations/$domain.json"
if [[ $? -eq 0 && ! -e $marker && ! -L $marker ]]; then
  tmp=$(mktemp /var/lib/ipat/customer-activations/.r998.pending.XXXXXX)
  python3 - "$tmp" "$domain" "$host" "$instance" "$oidc_port" <<'PY'
import json,sys
from datetime import datetime,timezone
p,domain,host,instance,port=sys.argv[1:]
with open(p,"w") as f:
 json.dump({"schema":"ipat.customer-domain-activation.v1","domain_id":domain,"hostname":host,
  "oidc_instance":instance,"oidc_port":int(port),"activated_at_utc":datetime.now(timezone.utc).isoformat(),
  "domain_activation_state":"active","tenant_human_mfa_browser_acceptance":"REQUIRED",
  "customer_release_go":False},f,sort_keys=True)
PY
  install -o root -g root -m 0600 "$tmp" "$marker" || true
  rm -f "$tmp" || true
fi
set -e
echo "R998_SINGLE_CUSTOMER_DOMAIN_ACTIVE_PENDING_TENANT_HUMAN_MFA_ACCEPTANCE domain_id=$domain host=$host"
