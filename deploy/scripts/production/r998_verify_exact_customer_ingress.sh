#!/usr/bin/env bash
# R9.98 exact one-customer ingress verifier/activator.
# Must run only as ipatdverify and can touch only one explicit domain UUID.
set -Eeuo pipefail
umask 077
die(){ echo "R998_VERIFY_REFUSED: $*" >&2; exit 4; }

[[ ${EUID:-$(id -u)} -ne 0 ]] || die "must run nonroot"
[[ $(id -un) == ipatdverify ]] || die "dedicated ipatdverify identity required"
[[ ${IPAT_R998_VERIFY_ONE:-} == YES ]] || die "explicit exact-domain verifier opt-in missing"
domain=${IPAT_R998_DOMAIN_ID:-}
db=${IPAT_R998_DB_NAME:-}
sock=${IPAT_R998_DB_SOCKET:-}
ip=${IPAT_R998_PUBLIC_IPV4:-}
[[ $domain =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "invalid exact domain UUID"
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe PostgreSQL target"
python3 - "$ip" <<'PY' || die "invalid global IPv4"
import ipaddress,sys
try:v=ipaddress.IPv4Address(sys.argv[1])
except ValueError:raise SystemExit(1)
if not v.is_global:raise SystemExit(1)
PY
for c in psql dig curl openssl sha256sum cut sort sed python3 wc tr; do command -v "$c" >/dev/null || die "missing tool: $c"; done

psql_base=(psql -X -w -v ON_ERROR_STOP=1 -h "$sock" -d "$db" -U ipat_domain_ingress_verifier_login)
role=$("${psql_base[@]}" -Atqc 'select current_user' 2>/dev/null || true)
[[ $role == ipat_domain_ingress_verifier_login ]] || die "restricted ingress verifier peer auth failed"

result(){
  "${psql_base[@]}" -AtF $'\t' -c \
    "SELECT id,hostname,verification_name,verification_value,activation_state,COALESCE(last_error_code,''),COALESCE(activated_at::text,'')
       FROM ipat_platform.get_tenant_domain_activation_result('$domain'::uuid)"
}
row=$(result) || die "cannot read exact activation result"
[[ -n $row && $(wc -l <<<"$row" | tr -d ' ') -eq 1 ]] || die "exact domain is not eligible"
IFS=$'\t' read -r got_id host verification_name verification_value state last_error activated_at <<<"$row"
[[ $got_id == "$domain" ]] || die "returned domain id mismatch"
[[ $verification_name == "_ipat-verify.$host" ]] || die "verification name mismatch"
[[ $verification_value =~ ^ipat-domain=[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "verification token invalid"
[[ $state == ownership_verified || $state == routing_ready || $state == tls_ready || $state == active ]] || die "unexpected activation state"

python3 - "$host" <<'PY' || die "invalid exact customer hostname"
import re,sys
h=sys.argv[1]
if h!=h.lower() or len(h)>253 or "." not in h or ".." in h or h.endswith((".local",".localhost",".invalid",".test",".example")):raise SystemExit(1)
for label in h.split("."):
    if not re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?",label):raise SystemExit(1)
PY

txt_ok(){
  dig @1.1.1.1 +short TXT "$verification_name" 2>/dev/null |
    python3 -c 'import shlex,sys
expected=sys.argv[1];vals=[]
for line in sys.stdin:
    try:vals.append("".join(shlex.split(line.strip())))
    except ValueError:raise SystemExit(2)
raise SystemExit(0 if expected in vals else 1)' "$verification_value"
}
a_ok(){
  local addresses
  addresses=$(dig @1.1.1.1 +short A "$host" | sed '/^$/d' | sort -u) || return 1
  [[ $addresses == "$ip" ]]
}
tls_evidence(){
  curl --fail --silent --show-error --max-time 5 --resolve "$host:443:$ip" "https://$host/" >/dev/null || return 1
  local status fingerprint
  status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 5 --resolve "$host:443:$ip" "https://$host/auth/oidc/start" || true)
  [[ $status == 303 ]] || return 1
  fingerprint=$(openssl s_client -connect "$ip:443" -servername "$host" -verify_hostname "$host" -verify_return_error -CAfile /etc/ssl/certs/ca-certificates.crt </dev/null 2>/dev/null |
    openssl x509 -noout -fingerprint -sha256) || return 1
  [[ $fingerprint == SHA256\ Fingerprint=* ]] || return 1
  printf '%s' "$fingerprint"
}
hash(){ printf '%s' "$1" | sha256sum | cut -d' ' -f1; }
transition(){
  local event=$1 evidence=$2 expected=$3 got
  got=$("${psql_base[@]}" -Atqc "SELECT ipat_platform.record_tenant_domain_ingress_check('$domain'::uuid,'$event','$evidence',NULL)") || return 1
  [[ $got == "$expected" ]]
}

# If already active, independently verify public reality before reporting success.
if [[ $state == active ]]; then
  txt_ok || die "active DB state but ownership TXT no longer matches"
  a_ok || die "active DB state but public A no longer matches"
  tls_evidence >/dev/null || die "active DB state but trusted HTTPS/OIDC no longer matches"
  echo "R998_EXACT_CUSTOMER_DOMAIN_ALREADY_ACTIVE_AND_REVERIFIED domain_id=$domain host=$host"
  exit 0
fi

txt_ok || die "ownership TXT recheck failed"
if [[ $state == ownership_verified ]]; then
  a_ok || die "public A target mismatch"
  transition routing_ready "$(hash "TXT:$verification_name:$verification_value|A:$host:$ip")" routing_ready || die "routing transition failed"
  state=routing_ready
fi
if [[ $state == routing_ready ]]; then
  txt_ok || die "ownership TXT changed before TLS"
  a_ok || die "public A changed before TLS"
  fp=$(tls_evidence) || die "trusted HTTPS/OIDC verification failed"
  transition tls_ready "$(hash "TXT:$verification_name:$verification_value|A:$host:$ip|TLS:$fp")" tls_ready || die "TLS transition failed"
  state=tls_ready
fi
if [[ $state == tls_ready ]]; then
  txt_ok || die "ownership TXT changed before activation"
  a_ok || die "public A changed before activation"
  fp=$(tls_evidence) || die "trusted HTTPS/OIDC recheck failed before activation"
  # The SECURITY DEFINER transition returning exactly "active" is the final
  # authoritative commit acknowledgement. Do not perform any fallible action
  # afterwards; otherwise an unrelated read failure could roll back public
  # ingress while the database has already committed active.
  transition activate "$(hash "ACTIVATE:TXT:$verification_name:$verification_value|A:$host:$ip|TLS:$fp")" active || die "activation transition failed"
  echo "R998_EXACT_CUSTOMER_DOMAIN_ACTIVE domain_id=$domain host=$host"
  exit 0
fi
die "exact domain state did not reach activation boundary"
