#!/usr/bin/env bash
# R9.94 nonroot exact-domain ingress verifier and ordered activator.
# No firewall, package, service, schema or physical-device mutation.
set -Eeuo pipefail
umask 077

die(){ echo "R994_VERIFY_REFUSED: $*" >&2; exit 4; }

[[ ${EUID:-$(id -u)} -ne 0 ]] || die "must run nonroot"
[[ $(id -un) == ipatdverify ]] || die "dedicated ipatdverify OS identity required"
[[ ${IPAT_R994_VERIFY_INGRESS:-} == YES ]] || die "explicit verifier opt-in missing"
[[ ${IPAT_R994_ALLOW_ACTIVATE:-} == YES ]] || die "activation opt-in missing"

db=${IPAT_R994_DB_NAME:-}
sock=${IPAT_R994_DB_SOCKET:-}
ip=${IPAT_R994_PUBLIC_IPV4:-}
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe PostgreSQL target"
python3 - "$ip" <<'PY' || die "invalid public IPv4"
import ipaddress,sys
try:
    v=ipaddress.IPv4Address(sys.argv[1])
except ValueError:
    raise SystemExit(1)
if not v.is_global:
    raise SystemExit(1)
PY

for c in psql dig curl openssl sha256sum cut sort sed python3; do
  command -v "$c" >/dev/null || die "missing tool: $c"
done

psql_base=(psql -X -w -v ON_ERROR_STOP=1 -h "$sock" -d "$db" -U ipat_domain_ingress_verifier_login)
role=$("${psql_base[@]}" -Atqc 'select current_user' 2>/dev/null || true)
[[ $role == ipat_domain_ingress_verifier_login ]] || die "restricted ingress verifier PostgreSQL peer auth failed"

rows=$("${psql_base[@]}" -AtF $'\t' -c   "SELECT id,hostname,routing_mode,verification_name,verification_value,activation_state
     FROM ipat_platform.list_tenant_domain_ingress_checks(25)")   || die "cannot list ingress verification candidates"

hash_material(){
  printf '%s' "$1" | sha256sum | cut -d' ' -f1
}

record_error(){
  local id=$1 code=$2
  "${psql_base[@]}" -Atqc     "SELECT ipat_platform.record_tenant_domain_ingress_check('$id'::uuid,'check_failed',NULL,'$code')"     >/dev/null || true
}

transition(){
  local id=$1 event=$2 evidence=$3 expected=$4
  local got
  got=$("${psql_base[@]}" -Atqc     "SELECT ipat_platform.record_tenant_domain_ingress_check('$id'::uuid,'$event','$evidence',NULL)")     || return 1
  [[ $got == "$expected" ]]
}

txt_ok(){
  local name=$1 expected=$2
  dig @1.1.1.1 +short TXT "$name" 2>/dev/null |
    python3 -c 'import shlex,sys
expected=sys.argv[1]
values=[]
for line in sys.stdin:
    try:
        values.append("".join(shlex.split(line.strip())))
    except ValueError:
        raise SystemExit(2)
raise SystemExit(0 if expected in values else 1)' "$expected"
}

routing_ok(){
  local host=$1
  local result
  result=$(dig @1.1.1.1 +short A "$host" | sed '/^$/d' | sort -u) || return 1
  [[ $result == "$ip" ]]
}

tls_evidence(){
  local host=$1
  curl --fail --silent --show-error --max-time 5 --resolve "$host:443:$ip" "https://$host/" >/dev/null || return 1
  local status
  status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 5     --resolve "$host:443:$ip" "https://$host/auth/oidc/start" || true)
  [[ $status == 303 ]] || return 1
  local fingerprint
  fingerprint=$(openssl s_client -connect "$ip:443" -servername "$host"       -verify_hostname "$host" -verify_return_error       -CAfile /etc/ssl/certs/ca-certificates.crt </dev/null 2>/dev/null |
    openssl x509 -noout -fingerprint -sha256) || return 1
  [[ $fingerprint == SHA256\ Fingerprint=* ]] || return 1
  hash_material "TLS:$host:$ip:$fingerprint"
}

advanced=0
while IFS=$'\t' read -r id host routing_mode verification_name verification_value state; do
  [[ -n $id ]] || continue
  [[ $id =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "invalid candidate id"
  python3 - "$host" <<'PY' || die "invalid candidate hostname"
import re,sys
host=sys.argv[1]
if host != host.lower() or len(host)>253 or "." not in host or ".." in host:
    raise SystemExit(1)
if host.endswith((".local",".localhost",".invalid",".test",".example")):
    raise SystemExit(1)
for label in host.split("."):
    if not re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?",label):
        raise SystemExit(1)
PY
  [[ $routing_mode == a_record ]] || die "unsupported candidate routing mode"
  [[ $verification_name == "_ipat-verify.$host" ]] || die "invalid candidate verification name"
  [[ $verification_value =~ ^ipat-domain=[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "invalid candidate verification value"
  [[ $state == ownership_verified || $state == routing_ready || $state == tls_ready ]] || die "invalid candidate state"

  if ! txt_ok "$verification_name" "$verification_value"; then
    record_error "$id" DNS_TXT_OWNERSHIP_RECHECK_FAILED
    continue
  fi

  if [[ $state == ownership_verified ]]; then
    if ! routing_ok "$host"; then
      record_error "$id" DNS_A_TARGET_MISMATCH
      continue
    fi
    a_hash=$(hash_material "TXT:$verification_name:$verification_value|A:$host:$ip")
    if ! transition "$id" routing_ready "$a_hash" routing_ready; then
      record_error "$id" ROUTING_STATE_TRANSITION_FAILED
      continue
    fi
    state=routing_ready
    advanced=$((advanced+1))
  fi

  if [[ $state == routing_ready ]]; then
    evidence=$(tls_evidence "$host") || {
      record_error "$id" TLS_HTTPS_VERIFY_FAILED
      continue
    }
    tls_hash=$(hash_material "TXT:$verification_name:$verification_value|A:$host:$ip|$evidence")
    if ! transition "$id" tls_ready "$tls_hash" tls_ready; then
      record_error "$id" TLS_STATE_TRANSITION_FAILED
      continue
    fi
    state=tls_ready
    advanced=$((advanced+1))
  fi

  if [[ $state == tls_ready ]]; then
    if ! txt_ok "$verification_name" "$verification_value"; then
      record_error "$id" DNS_TXT_OWNERSHIP_CHANGED
      continue
    fi
    if ! routing_ok "$host"; then
      record_error "$id" DNS_A_TARGET_CHANGED
      continue
    fi
    evidence=$(tls_evidence "$host") || {
      record_error "$id" TLS_HTTPS_RECHECK_FAILED
      continue
    }
    activation=$(hash_material "ACTIVATE:TXT:$verification_name:$verification_value|A:$host:$ip|$evidence")
    if transition "$id" activate "$activation" active; then
      advanced=$((advanced+1))
    else
      record_error "$id" ACTIVATE_STATE_TRANSITION_FAILED
    fi
  fi
done <<<"$rows"

echo "R994_INGRESS_VERIFIER_COMPLETE advanced=$advanced"
