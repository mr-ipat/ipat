#!/usr/bin/env bash
# R9.92 add one exact-customer-host OIDC issuer instance on a reviewed loopback port.
# No DNS/TLS/firewall/PostgreSQL/device mutation.
set -Eeuo pipefail
umask 077
die(){ echo "R992_OIDC_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R992_ADD_OIDC:-} == ADD_REVIEWED_TENANT_OIDC_INSTANCE ]] || die "explicit add-instance opt-in missing"
src=${IPAT_R992_OIDC_ENV_SOURCE:-}; instance=${IPAT_R992_OIDC_INSTANCE:-}
[[ $instance =~ ^[a-z0-9][a-z0-9-]{0,47}$ ]] || die "invalid nonsecret instance ID"
[[ -n $src && -f $src && ! -L $src ]] || die "staged OIDC env required"
[[ $(stat -c '%u:%a:%h' "$src") == 0:600:1 ]] || die "staged env must be root-owned 0600 single-link"
for cmd in python3 install systemctl systemd-run runuser psql ss curl stat; do command -v "$cmd" >/dev/null || die "missing tool: $cmd"; done
parsed=$(python3 - "$src" <<'PY'
import ipaddress,pathlib,re,sys
data={}
for raw in pathlib.Path(sys.argv[1]).read_text().splitlines():
    if not raw or raw.startswith("#"): continue
    if "=" not in raw: raise SystemExit(2)
    k,v=raw.split("=",1)
    if not re.fullmatch(r"[A-Z0-9_]+",k) or k in data or any(c in v for c in "\r\n\0"): raise SystemExit(2)
    data[k]=v
fixed={"IPAT_R970_OIDC_ISSUER_SERVICE":"YES","IPAT_TRUSTED_HTTPS_EDGE":"YES",
       "IPAT_R970_VERIFIED_CONFIDENTIAL_IDP":"YES","IPAT_R971_DURABLE_PENDING":"YES",
       "IPAT_R970_DB_USER":"ipat_oidc_session_issuer_login"}
if any(data.get(k)!=v for k,v in fixed.items()): raise SystemExit(3)
if data.get("IPAT_R970_SINGLE_INSTANCE_OIDC")=="YES": raise SystemExit(3)
for k in ("IPAT_R992_TENANT_OIDC_PORT","IPAT_R970_ISSUER","IPAT_R970_HOST","IPAT_R970_CLIENT_ID",
          "IPAT_R970_CLIENT_SECRET_FILE","IPAT_R970_PUBLIC_KEY_FILE","IPAT_R970_KID",
          "IPAT_R970_DB_NAME","IPAT_R970_DB_SOCKET"):
    if k not in data: raise SystemExit(3)
port=int(data["IPAT_R992_TENANT_OIDC_PORT"])
if not 31000 <= port <= 31999: raise SystemExit(4)
host=data["IPAT_R970_HOST"]
if host != host.lower() or len(host)>253 or "." not in host or ":" in host: raise SystemExit(4)
if any(not label or label[0]=="-" or label[-1]=="-" or
       not all(c.isalnum() or c=="-" for c in label) for label in host.split(".")): raise SystemExit(4)
try:
    ipaddress.ip_address(host)
    raise SystemExit(4)
except ValueError:
    pass
for k,v in data.items(): print(f"{k}={v}")
PY
) || die "invalid tenant OIDC env"
value(){ local key=$1; grep -m1 "^$key=" <<<"$parsed" | cut -d= -f2-; }
port=$(value IPAT_R992_TENANT_OIDC_PORT); host=$(value IPAT_R970_HOST)
db=$(value IPAT_R970_DB_NAME); sock=$(value IPAT_R970_DB_SOCKET)
secret=$(value IPAT_R970_CLIENT_SECRET_FILE); key=$(value IPAT_R970_PUBLIC_KEY_FILE)
for path in "$secret" "$key"; do
  [[ $path == /var/lib/ipat-tenant-oidc/secrets/* && -f $path && ! -L $path ]] || die "secret/key outside dedicated tree"
  [[ $(stat -c '%U:%a:%h' "$path") == ipattoidc:600:1 ]] || die "secret/key owner/mode/link mismatch"
done
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe DB target"
role=$(PGCONNECT_TIMEOUT=3 runuser -u ipattoidc -- psql -X -w -h "$sock" -d "$db" -U ipat_oidc_session_issuer_login -Atqc 'select current_user' 2>/dev/null || true)
[[ $role == ipat_oidc_session_issuer_login ]] || die "OIDC restricted PostgreSQL peer login failed"
! ss -H -ltn "( sport = :$port )" 2>/dev/null | grep -q . || die "assigned OIDC port already listening"
final="/etc/ipat/tenant-oidc/$instance.env"
[[ ! -e $final && ! -L $final ]] || die "instance already exists"
service="ipat-tenant-oidc@$instance.service"
stamp=$(date -u +%Y%m%dT%H%M%SZ); rollback=/run/ipat-r992-oidc-rollback-$stamp.sh; unit=ipat-r992-oidc-rollback-$stamp
cat >"$rollback" <<RB
#!/usr/bin/env bash
systemctl disable --now '$service' 2>/dev/null || true
rm -f '$final'
RB
chmod 0700 "$rollback"
systemd-run --quiet --collect --unit="$unit" --on-active=10m "$rollback"
armed=YES
rollback_now(){ [[ ${armed:-NO} == YES ]] && "$rollback" || true; }
trap rollback_now ERR
install -o root -g root -m 0600 "$src" "$final"
systemctl enable --now "$service"
for _ in {1..30}; do
  ss -H -ltn "( sport = :$port )" 2>/dev/null | grep -qE "127[.]0[.]0[.]1:$port[[:space:]]" && break
  sleep .2
done
listeners=$(ss -H -ltn "( sport = :$port )" 2>/dev/null || true)
[[ $(grep -cE "127[.]0[.]0[.]1:$port[[:space:]]" <<<"$listeners") -eq 1 ]] || die "OIDC listener not exact loopback"
status=$(curl -sS -o /dev/null -w '%{http_code}' --max-time 3 -H "Host: $host" "http://127.0.0.1:$port/__r992_probe" || true)
[[ $status == 404 ]] || die "OIDC no-side-effect exact-Host probe failed"
systemctl stop "$unit.timer" "$unit.service" 2>/dev/null || true
rm -f "$rollback"; armed=NO; trap - ERR
echo "R992_TENANT_OIDC_INSTANCE_ACTIVE instance=$instance host=$host port=$port"
