#!/usr/bin/env bash
# R9.92 activate shared tenant BFF after local PostgreSQL peer auth.
# No PostgreSQL mutation, firewall, DNS, Nginx, TLS, or device action.
set -Eeuo pipefail
umask 077
die(){ echo "R992_API_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R992_APPLY_API:-} == ACTIVATE_REVIEWED_TENANT_API ]] || die "explicit activation opt-in missing"
src=${IPAT_R992_API_ENV_SOURCE:-}
[[ -n $src && -f $src && ! -L $src ]] || die "root-owned staged tenant API env required"
[[ $(stat -c '%u:%a:%h' "$src") == 0:600:1 ]] || die "staged API env must be root-owned 0600 single-link"
for cmd in python3 install systemctl systemd-run runuser psql ss stat; do command -v "$cmd" >/dev/null || die "missing tool: $cmd"; done
parsed=$(python3 - "$src" <<'PY'
import pathlib,re,sys
data={}
for raw in pathlib.Path(sys.argv[1]).read_text().splitlines():
    if not raw or raw.startswith("#"): continue
    if "=" not in raw: raise SystemExit(2)
    k,v=raw.split("=",1)
    if not re.fullmatch(r"[A-Z0-9_]+",k) or k in data or any(c in v for c in "\r\n\0"): raise SystemExit(2)
    data[k]=v
fixed={"IPAT_R969_COMMERCIAL_SERVICE":"YES","IPAT_PRODUCTION_TENANT_API":"YES",
       "IPAT_TRUSTED_HTTPS_EDGE":"YES","IPAT_TENANT_API_DB_USER":"ipat_tenant_api_login"}
if any(data.get(k)!=v for k,v in fixed.items()): raise SystemExit(3)
for k in ("IPAT_TENANT_API_DB_NAME","IPAT_TENANT_API_DB_SOCKET"):
    if k not in data: raise SystemExit(3)
for k,v in data.items(): print(f"{k}={v}")
PY
) || die "invalid tenant API env"
value(){ local key=$1; grep -m1 "^$key=" <<<"$parsed" | cut -d= -f2-; }
db=$(value IPAT_TENANT_API_DB_NAME); sock=$(value IPAT_TENANT_API_DB_SOCKET)
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe DB target"
role=$(PGCONNECT_TIMEOUT=3 runuser -u ipattapi -- psql -X -w -h "$sock" -d "$db" -U ipat_tenant_api_login -Atqc 'select current_user' 2>/dev/null || true)
[[ $role == ipat_tenant_api_login ]] || die "tenant API restricted PostgreSQL peer login failed"
final=/etc/ipat/tenant-api.env
[[ ! -e $final && ! -L $final ]] || die "existing tenant API env requires upgrade review"
[[ $(systemctl is-active ipat-tenant-api.service 2>/dev/null || true) != active ]] || die "tenant API already active"
stamp=$(date -u +%Y%m%dT%H%M%SZ); rollback=/run/ipat-r992-api-rollback-$stamp.sh; unit=ipat-r992-api-rollback-$stamp
cat >"$rollback" <<'RB'
#!/usr/bin/env bash
systemctl disable --now ipat-tenant-api.service 2>/dev/null || true
rm -f /etc/ipat/tenant-api.env
RB
chmod 0700 "$rollback"
systemd-run --quiet --collect --unit="$unit" --on-active=10m "$rollback"
armed=YES
rollback_now(){ [[ ${armed:-NO} == YES ]] && "$rollback" || true; }
trap rollback_now ERR
install -o root -g root -m 0600 "$src" "$final"
systemctl enable --now ipat-tenant-api.service
for _ in {1..30}; do
  ss -H -ltn '( sport = :3003 )' 2>/dev/null | grep -qE '127[.]0[.]0[.]1:3003[[:space:]]' && break
  sleep .2
done
listeners=$(ss -H -ltn '( sport = :3003 )' 2>/dev/null || true)
[[ $(grep -cE '127[.]0[.]0[.]1:3003[[:space:]]' <<<"$listeners") -eq 1 ]] || die "tenant API listener not exact loopback"
! grep -Eq '(0[.]0[.]0[.]0|[[]::[]]):3003' <<<"$listeners" || die "wildcard listener forbidden"
systemctl stop "$unit.timer" "$unit.service" 2>/dev/null || true
rm -f "$rollback"; armed=NO; trap - ERR
echo R992_SHARED_TENANT_API_LOOPBACK_ACTIVE
