#!/usr/bin/env bash
# R9.85 validated first activation of Platform Owner API+OIDC loopback services.
# Requires already-provisioned env + OIDC secret/key files and PostgreSQL auth.
# Does not mutate PostgreSQL roles/config, firewall, DNS, Nginx or devices.
set -Eeuo pipefail
umask 077

die(){ echo "R985_APPLY_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R985_APPLY:-} == ACTIVATE_REVIEWED_PLATFORM_RUNTIME ]] || die "explicit reviewed activation opt-in missing"
ip=${IPAT_R985_PUBLIC_IPV4:-}
api_src=${IPAT_R985_API_ENV_SOURCE:-}
oidc_src=${IPAT_R985_OIDC_ENV_SOURCE:-}
[[ -n $ip && -n $api_src && -n $oidc_src ]] || die "exact public IPv4 and both staged env files required"

python3 - "$ip" <<'PY' || die "invalid global IPv4"
import ipaddress,sys
try: v=ipaddress.IPv4Address(sys.argv[1])
except ValueError: raise SystemExit(1)
if not v.is_global: raise SystemExit(1)
PY

for cmd in python3 install systemctl systemd-run runuser psql ss curl stat readlink; do command -v "$cmd" >/dev/null || die "missing tool: $cmd"; done
[[ -x /opt/ipat/bin/control-api ]] || die "prepared control-api binary missing"

secure_root_file(){
  local path=$1
  [[ -f $path && ! -L $path ]] || die "unsafe staged env file: $path"
  [[ $(stat -c '%u:%a:%h' "$path") == 0:600:1 ]] || die "staged env must be root-owned 0600 single-link: $path"
}
secure_root_file "$api_src"
secure_root_file "$oidc_src"

parse_env(){
  python3 - "$1" "$2" <<'PY'
import pathlib,re,sys
p=pathlib.Path(sys.argv[1]); prefix=sys.argv[2]
data={}
for raw in p.read_text().splitlines():
    if not raw or raw.startswith('#'): continue
    if '=' not in raw: raise SystemExit(2)
    k,v=raw.split('=',1)
    if not re.fullmatch(r'[A-Z0-9_]+',k) or any(c in v for c in '\r\n\0'): raise SystemExit(2)
    if k in data: raise SystemExit(2)
    data[k]=v
req={
'api':['IPAT_R981_PLATFORM_SERVICE','IPAT_R981_REVIEWED_TRUSTED_HTTPS_EDGE','IPAT_R981_EXACT_PLATFORM_HOST','IPAT_R982_TEMPORARY_IPV4_MODE','IPAT_R982_IP_SAN_TLS_REVIEWED','IPAT_R981_PLATFORM_DB_USER','IPAT_R981_PLATFORM_DB_NAME','IPAT_R981_PLATFORM_DB_SOCKET'],
'oidc':['IPAT_R983_PLATFORM_OIDC_ISSUER_SERVICE','IPAT_R983_PLATFORM_TRUSTED_HTTPS_EDGE','IPAT_R983_VERIFIED_PLATFORM_CONFIDENTIAL_IDP','IPAT_R983_SINGLE_INSTANCE_OIDC','IPAT_R983_HOST','IPAT_R982_TEMPORARY_IPV4_MODE','IPAT_R982_IP_SAN_TLS_REVIEWED','IPAT_R983_DB_USER','IPAT_R983_DB_NAME','IPAT_R983_DB_SOCKET','IPAT_R983_CLIENT_SECRET_FILE','IPAT_R983_PUBLIC_KEY_FILE','IPAT_R983_ISSUER','IPAT_R983_CLIENT_ID','IPAT_R983_KID']
}[prefix]
if any(k not in data for k in req): raise SystemExit(3)
for k in req: print(k+'='+data[k])
PY
}
api_parsed=$(parse_env "$api_src" api) || die "invalid API env"
oidc_parsed=$(parse_env "$oidc_src" oidc) || die "invalid OIDC env"
mapfile -t api_lines <<<"$api_parsed"
mapfile -t oidc_lines <<<"$oidc_parsed"

value(){ local key=$1; shift; local line; for line in "$@"; do [[ $line == "$key="* ]] && { printf '%s' "${line#*=}"; return 0; }; done; return 1; }
[[ $(value IPAT_R981_PLATFORM_SERVICE "${api_lines[@]}") == YES ]] || die "API mode missing"
[[ $(value IPAT_R981_REVIEWED_TRUSTED_HTTPS_EDGE "${api_lines[@]}") == YES ]] || die "API reviewed edge gate missing"
[[ $(value IPAT_R981_EXACT_PLATFORM_HOST "${api_lines[@]}") == "$ip" ]] || die "API exact Host mismatch"
[[ $(value IPAT_R982_TEMPORARY_IPV4_MODE "${api_lines[@]}") == YES && $(value IPAT_R982_IP_SAN_TLS_REVIEWED "${api_lines[@]}") == YES ]] || die "API temporary IP gates missing"
[[ $(value IPAT_R981_PLATFORM_DB_USER "${api_lines[@]}") == ipat_platform_session_api_login ]] || die "wrong API DB identity"

[[ $(value IPAT_R983_PLATFORM_OIDC_ISSUER_SERVICE "${oidc_lines[@]}") == YES ]] || die "OIDC mode missing"
[[ $(value IPAT_R983_PLATFORM_TRUSTED_HTTPS_EDGE "${oidc_lines[@]}") == YES ]] || die "OIDC reviewed edge gate missing"
[[ $(value IPAT_R983_VERIFIED_PLATFORM_CONFIDENTIAL_IDP "${oidc_lines[@]}") == YES ]] || die "verified confidential IdP gate missing"
[[ $(value IPAT_R983_SINGLE_INSTANCE_OIDC "${oidc_lines[@]}") == YES ]] || die "current bounded single-instance issuer gate missing"
[[ $(value IPAT_R983_HOST "${oidc_lines[@]}") == "$ip" ]] || die "OIDC exact Host mismatch"
[[ $(value IPAT_R982_TEMPORARY_IPV4_MODE "${oidc_lines[@]}") == YES && $(value IPAT_R982_IP_SAN_TLS_REVIEWED "${oidc_lines[@]}") == YES ]] || die "OIDC temporary IP gates missing"
[[ $(value IPAT_R983_DB_USER "${oidc_lines[@]}") == ipat_platform_session_issuer_login ]] || die "wrong OIDC DB identity"

secret=$(value IPAT_R983_CLIENT_SECRET_FILE "${oidc_lines[@]}")
key=$(value IPAT_R983_PUBLIC_KEY_FILE "${oidc_lines[@]}")
for path in "$secret" "$key"; do
  [[ $path == /var/lib/ipat-platform-oidc/secrets/* && -f $path && ! -L $path ]] || die "OIDC secret/key must be in dedicated secure directory"
  [[ $(stat -c '%U:%a:%h' "$path") == ipatpoidc:600:1 ]] || die "OIDC secret/key owner/mode/link mismatch"
done
[[ $(stat -c '%U:%a' /var/lib/ipat-platform-oidc/secrets) == ipatpoidc:700 ]] || die "OIDC secret directory owner/mode mismatch"

api_db_socket=$(value IPAT_R981_PLATFORM_DB_SOCKET "${api_lines[@]}")
api_db_name=$(value IPAT_R981_PLATFORM_DB_NAME "${api_lines[@]}")
oidc_db_socket=$(value IPAT_R983_DB_SOCKET "${oidc_lines[@]}")
oidc_db_name=$(value IPAT_R983_DB_NAME "${oidc_lines[@]}")
for socket in "$api_db_socket" "$oidc_db_socket"; do
  [[ $socket == /* && -d $socket && ! -L $socket ]] || die "database socket directory unavailable or unsafe: $socket"
done
[[ $api_db_name =~ ^[A-Za-z0-9_]{1,63}$ && $oidc_db_name =~ ^[A-Za-z0-9_]{1,63}$ ]] || die "unsafe database name"

api_role=$(PGCONNECT_TIMEOUT=3 runuser -u ipatpapi -- psql -X -w -h "$api_db_socket" -d "$api_db_name" -U ipat_platform_session_api_login -Atqc 'select current_user' 2>/dev/null || true)
[[ $api_role == ipat_platform_session_api_login ]] || die "API restricted PostgreSQL login failed"
oidc_role=$(PGCONNECT_TIMEOUT=3 runuser -u ipatpoidc -- psql -X -w -h "$oidc_db_socket" -d "$oidc_db_name" -U ipat_platform_session_issuer_login -Atqc 'select current_user' 2>/dev/null || true)
[[ $oidc_role == ipat_platform_session_issuer_login ]] || die "OIDC ISSUE-only PostgreSQL login failed"

api_final=/etc/ipat/platform-api.env
oidc_final=/etc/ipat/platform-oidc.env
[[ ! -e $api_final && ! -L $api_final && ! -e $oidc_final && ! -L $oidc_final ]] || die "existing runtime env requires explicit upgrade review"
[[ $(systemctl is-active ipat-platform-api.service 2>/dev/null || true) != active ]] || die "API already active"
[[ $(systemctl is-active ipat-platform-oidc.service 2>/dev/null || true) != active ]] || die "OIDC already active"

stamp=$(date -u +%Y%m%dT%H%M%SZ)
rollback_unit="ipat-r985-runtime-rollback-$stamp"
rollback=/run/ipat-r985-runtime-rollback-$stamp.sh
cat > "$rollback" <<'ROLLBACK'
#!/usr/bin/env bash
set -Eeuo pipefail
systemctl disable --now ipat-platform-api.service ipat-platform-oidc.service 2>/dev/null || true
rm -f /etc/ipat/platform-api.env /etc/ipat/platform-oidc.env
ROLLBACK
chmod 0700 "$rollback"
systemd-run --quiet --collect --unit="$rollback_unit" --on-active=10m "$rollback"
armed=YES
rollback_now(){ [[ ${armed:-NO} == YES ]] && "$rollback" || true; }
trap rollback_now ERR

install -o root -g root -m 0600 "$api_src" "$api_final"
install -o root -g root -m 0600 "$oidc_src" "$oidc_final"
systemctl daemon-reload
systemctl enable --now ipat-platform-api.service
systemctl enable --now ipat-platform-oidc.service

for _ in {1..30}; do
  listeners=$(ss -H -ltn '( sport = :3005 or sport = :3006 )' 2>/dev/null || true)
  if grep -qE '127[.]0[.]0[.]1:3005[[:space:]]' <<<"$listeners" && grep -qE '127[.]0[.]0[.]1:3006[[:space:]]' <<<"$listeners"; then break; fi
  sleep .2
done
listeners=$(ss -H -ltn '( sport = :3005 or sport = :3006 )' 2>/dev/null || true)
[[ $(grep -cE '127[.]0[.]0[.]1:3005[[:space:]]' <<<"$listeners") -eq 1 ]] || die "API listener not exact loopback"
[[ $(grep -cE '127[.]0[.]0[.]1:3006[[:space:]]' <<<"$listeners") -eq 1 ]] || die "OIDC listener not exact loopback"
if grep -Eq '(^|[[:space:]])(0[.]0[.]0[.]0|[[]::[]]|[[]::0[]]):(3005|3006)([[:space:]]|$)' <<<"$listeners"; then die "wildcard listener forbidden"; fi
curl --fail --silent --show-error --max-time 3 -H "Host: $ip" http://127.0.0.1:3005/ >/dev/null
status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 3 -H "Host: $ip" http://127.0.0.1:3006/__r985_probe || true)
[[ $status == 404 ]] || die "OIDC no-side-effect probe failed"

systemctl stop "$rollback_unit.timer" "$rollback_unit.service" 2>/dev/null || true
rm -f "$rollback"
armed=NO
trap - ERR
echo R985_PLATFORM_API_AND_OIDC_LOOPBACK_RUNTIME_ACTIVE_READY_FOR_R984_TLS_CUTOVER
