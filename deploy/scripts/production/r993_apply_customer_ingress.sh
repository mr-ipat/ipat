#!/usr/bin/env bash
# R9.93 two-phase exact customer-owned Host ingress.
# Does NOT modify firewall/security groups, PostgreSQL or physical devices.
set -Eeuo pipefail
umask 077
die(){ echo "R993_REFUSED: $*" >&2; exit 4; }

[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R993_APPLY:-} == APPLY_REVIEWED_CUSTOMER_INGRESS ]] || die "explicit reviewed opt-in missing"
phase=${IPAT_R993_PHASE:-}
[[ $phase == staging || $phase == production ]] || die "phase must be staging or production"
host=${IPAT_R993_HOST:-}
instance=${IPAT_R993_INSTANCE:-}
oidc_port=${IPAT_R993_OIDC_PORT:-}
ip=${IPAT_R993_PUBLIC_IPV4:-}
email=${IPAT_R993_ACME_EMAIL:-}
verification=${IPAT_R993_VERIFICATION_VALUE:-}
rollback_export=${IPAT_R993_ROLLBACK_EXPORT_FILE:-}
[[ $instance =~ ^[a-z0-9][a-z0-9-]{0,47}$ ]] || die "invalid instance"
[[ $oidc_port =~ ^[0-9]+$ && $oidc_port -ge 31000 && $oidc_port -le 31999 ]] || die "OIDC port outside 31000-31999"
[[ -n $host && -n $ip && -n $email && $email == *@*.* ]] || die "host/public IPv4/contact required"
[[ $verification =~ ^ipat-domain=[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "exact persisted ownership token required"
if [[ -n $rollback_export ]]; then
  [[ $phase == production ]] || die "rollback export is production-only"
  [[ $rollback_export == /run/ipat-r998-r993-rollback-*.path && ! -e $rollback_export && ! -L $rollback_export ]] \
    || die "unsafe rollback export path"
fi

. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu Server 26.04 required"
for c in python3 nginx openssl systemctl systemd-run install sed ss curl dig readlink flock certbot stat; do
  command -v "$c" >/dev/null || die "missing tool: $c"
done
certbot=$(command -v certbot)
nginx=$(command -v nginx)
systemctl is-active --quiet nginx || die "reviewed nginx edge must already be active"

python3 - "$host" "$ip" <<'PY' || die "invalid exact customer Host or public IPv4"
import ipaddress,re,sys
host,ip=sys.argv[1:]
if host != host.lower() or len(host)>253 or "." not in host or ":" in host:
    raise SystemExit(1)
if host.endswith((".local",".localhost",".invalid",".test",".example")):
    raise SystemExit(1)
for label in host.split("."):
    if not re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?",label):
        raise SystemExit(1)
try:
    value=ipaddress.IPv4Address(ip)
except ValueError:
    raise SystemExit(1)
if not value.is_global:
    raise SystemExit(1)
PY

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
boot="$repo_root/deploy/nginx/r993-tenant-bootstrap.conf.template"
tls="$repo_root/deploy/nginx/r993-tenant-https.conf.template"
[[ -f $boot && ! -L $boot && -f $tls && ! -L $tls ]] || die "trusted ingress templates missing"

listeners=$(ss -H -ltn "( sport = :3003 or sport = :$oidc_port )" 2>/dev/null || true)
[[ $(grep -cE '127[.]0[.]0[.]1:3003[[:space:]]' <<<"$listeners") -eq 1 ]] || die "tenant API not exact loopback"
[[ $(grep -cE "127[.]0[.]0[.]1:$oidc_port[[:space:]]" <<<"$listeners") -eq 1 ]] || die "tenant OIDC not exact loopback"
! grep -Eq "(0[.]0[.]0[.]0|[[]::[]]|[[]::0[]]):(3003|$oidc_port)([[:space:]]|$)" <<<"$listeners" || die "wildcard tenant backend forbidden"
curl --fail --silent --show-error --max-time 3 -H "Host: $host" http://127.0.0.1:3003/ >/dev/null || die "tenant API exact-Host preflight failed"
code=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 3 -H "Host: $host" "http://127.0.0.1:$oidc_port/__r993_probe" || true)
[[ $code == 404 ]] || die "tenant OIDC exact-Host preflight failed"

python3 - "$host" "$verification" <<'PY' || die "public ownership TXT does not match persisted IPAT token"
import shlex,subprocess,sys
host,expected=sys.argv[1:]
out=subprocess.check_output(
    ["dig","@1.1.1.1","+short","TXT",f"_ipat-verify.{host}"],
    text=True,stderr=subprocess.DEVNULL,timeout=5,
)
records=[]
for line in out.splitlines():
    try:
        records.append("".join(shlex.split(line)))
    except ValueError:
        raise SystemExit(1)
if expected not in records:
    raise SystemExit(1)
PY
mapfile -t resolved < <(dig @1.1.1.1 +short A "$host" | sed '/^$/d' | sort -u)
[[ ${#resolved[@]} -eq 1 && ${resolved[0]} == "$ip" ]] || die "public DNS A must resolve exactly to reviewed VPS IPv4"

site="/etc/nginx/sites-available/ipat-tenant-$instance"
enabled="/etc/nginx/sites-enabled/ipat-tenant-$instance"
cert_name="ipat-tenant-$instance"
[[ ! -e $site && ! -L $site && ! -e $enabled && ! -L $enabled ]] || die "customer ingress already exists"
[[ ! -e /etc/letsencrypt/renewal/$cert_name.conf ]] || die "certificate lineage already exists"
webroot=/var/lib/ipat/acme-webroot
install -d -m 0755 "$webroot"
stamp=$(date -u +%Y%m%dT%H%M%SZ)
backup="/var/backups/ipat-r993/$stamp"
install -d -m 0700 "$backup"
rollback="$backup/rollback.sh"
unit="ipat-r993-rollback-$stamp"
cat >"$rollback" <<RB
#!/usr/bin/env bash
set -Eeuo pipefail
rm -f '$enabled' '$site'
'$certbot' delete --non-interactive --cert-name '$cert_name' >/dev/null 2>&1 || true
'$nginx' -t && /bin/systemctl reload nginx || true
RB
chmod 0700 "$rollback"
if [[ -n $rollback_export ]]; then
  printf '%s\n' "$rollback" > "$rollback_export"
  chmod 0600 "$rollback_export"
  [[ $(stat -c '%u:%a:%h' "$rollback_export") == 0:600:1 ]] || die "rollback export ownership/mode mismatch"
fi
systemd-run --quiet --collect --unit="$unit" --on-active=10m "$rollback"
armed=YES
rollback_now(){ [[ ${armed:-NO} == YES ]] && "$rollback" || true; }
trap rollback_now ERR

render(){
  local src=$1 dst=$2
  sed -e "s/__HOST__/$host/g" -e "s/__INSTANCE__/$instance/g" -e "s/__OIDC_PORT__/$oidc_port/g" "$src" > "$dst.pending"
  install -o root -g root -m 0644 "$dst.pending" "$dst"
  rm -f "$dst.pending"
}
render "$boot" "$site"
ln -s "$site" "$enabled"
"$nginx" -t
systemctl reload nginx

args=(certonly --non-interactive --agree-tos --email "$email" --webroot --webroot-path "$webroot" -d "$host" --cert-name "$cert_name"
      --deploy-hook "$nginx -t && /bin/systemctl reload nginx")
[[ $phase == staging ]] && args+=(--staging)
"$certbot" "${args[@]}"
cert="/etc/letsencrypt/live/$cert_name/fullchain.pem"
key="/etc/letsencrypt/live/$cert_name/privkey.pem"
[[ -s $cert && -s $key ]] || die "expected certificate lineage missing"
openssl x509 -in "$cert" -noout -checkhost "$host" >/dev/null || die "certificate lacks exact customer DNS SAN"

if [[ $phase == staging ]]; then
  rollback_now
  armed=NO
  systemctl stop "$unit.timer" "$unit.service" 2>/dev/null || true
  trap - ERR
  echo R993_STAGING_CUSTOMER_ACME_VALIDATED_AND_ROLLED_BACK
  exit 0
fi

render "$tls" "$site"
"$nginx" -t
systemctl reload nginx
curl --fail --silent --show-error --max-time 5 --resolve "$host:443:$ip" "https://$host/" >/dev/null || die "trusted customer HTTPS does not reach tenant API"
oidc_status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 5 --resolve "$host:443:$ip" "https://$host/auth/oidc/start" || true)
[[ $oidc_status == 303 ]] || die "trusted customer HTTPS does not reach exact tenant OIDC start"
"$certbot" renew --cert-name "$cert_name" --dry-run --run-deploy-hooks
"$nginx" -t
systemctl stop "$unit.timer" "$unit.service" 2>/dev/null || true
armed=NO
trap - ERR
echo "R993_CUSTOMER_HTTPS_INGRESS_ACTIVE host=$host instance=$instance oidc_port=$oidc_port"
