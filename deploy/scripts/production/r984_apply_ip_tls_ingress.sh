#!/usr/bin/env bash
# R9.84 privileged reviewed two-phase literal-IP HTTPS edge installer.
# Never changes nftables/iptables/security groups/DNS/device configuration.
set -Eeuo pipefail
umask 077

die(){ echo "R984_REFUSED: $*" >&2; exit 4; }
rollback_armed=NO
rollback_script=

[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required through independently approved rescue-capable deployment path"
[[ ${IPAT_R984_APPLY:-} == APPLY_REVIEWED_IP_TLS_EDGE ]] || die "explicit reviewed deployment opt-in missing"
phase=${IPAT_R984_PHASE:-}
[[ $phase == staging || $phase == production ]] || die "IPAT_R984_PHASE must be staging or production"
ip=${IPAT_R984_PUBLIC_IPV4:-}
email=${IPAT_R984_ACME_EMAIL:-}
[[ -n $ip && -n $email && $email != *$'\n'* && $email != *$'\r'* ]] || die "exact IPv4 and ACME contact email required"
[[ $email == *@*.* ]] || die "invalid ACME contact email shape"

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
bootstrap="$repo_root/deploy/nginx/r984-platform-ip-bootstrap.conf.template"
https="$repo_root/deploy/nginx/r984-platform-ip-https.conf.template"
renew_service="$repo_root/deploy/systemd/ipat-ip-cert-renew.service.template"
renew_timer="$repo_root/deploy/systemd/ipat-ip-cert-renew.timer"
for file in "$bootstrap" "$https" "$renew_service" "$renew_timer"; do
  [[ -f $file && ! -L $file ]] || die "missing trusted repository template: $file"
done

[[ -r /etc/os-release ]] || die "OS identity unavailable"
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "requires Ubuntu Server 26.04"
for cmd in python3 nginx certbot openssl systemctl systemd-run install sed ss curl flock date readlink; do
  command -v "$cmd" >/dev/null || die "missing prerequisite: $cmd"
done
certbot=$(command -v certbot)
nginx=$(command -v nginx)

python3 - "$ip" <<'PY' || die "invalid or non-global IPv4"
import ipaddress,sys
try:
    value=ipaddress.IPv4Address(sys.argv[1])
except ValueError:
    raise SystemExit(1)
if not value.is_global:
    raise SystemExit(1)
PY
python3 - "$certbot" <<'PY' || die "Certbot >= 5.4 required for webroot IP certificates"
import re,subprocess,sys
out=subprocess.check_output([sys.argv[1],"--version"],text=True,stderr=subprocess.STDOUT)
m=re.search(r"(\d+)\.(\d+)(?:\.(\d+))?",out)
if not m or tuple(map(int,m.groups(default="0"))) < (5,4,0):
    raise SystemExit(1)
PY

site=/etc/nginx/sites-available/ipat-platform-ip
enabled=/etc/nginx/sites-enabled/ipat-platform-ip
default_enabled=/etc/nginx/sites-enabled/default
hook=/etc/letsencrypt/renewal-hooks/deploy/ipat-platform-ip-nginx
service=/etc/systemd/system/ipat-ip-cert-renew.service
timer=/etc/systemd/system/ipat-ip-cert-renew.timer
[[ ! -L $site ]] || die "managed site path must not be a symlink"
[[ ! -e $enabled || -L $enabled ]] || die "managed enabled-site path must be a symlink or absent"
[[ ! -e $default_enabled || -L $default_enabled ]] || die "existing default enabled site is not a symlink; manual review required"
# First production cutover only. Existing managed renewal artifacts require a
# dedicated reviewed upgrade path so this installer never overwrites unknown
# local policy or silently changes an already-running renewal schedule.
for managed in "$hook" "$service" "$timer"; do
  [[ ! -e $managed && ! -L $managed ]] || die "existing managed renewal artifact requires explicit upgrade review: $managed"
done
webroot=/var/lib/ipat/acme-webroot
stamp=$(date -u +%Y%m%dT%H%M%SZ)
backup="/var/backups/ipat-r984/$stamp"
rollback_unit="ipat-r984-rollback-$stamp"
install -d -m 0700 "$backup"
[[ ! -e $site ]] || cp -a -- "$site" "$backup/site.previous"
if [[ -L $enabled ]]; then readlink "$enabled" > "$backup/enabled.link"; fi
if [[ -L $default_enabled ]]; then readlink "$default_enabled" > "$backup/default.enabled.link"; fi
if systemctl is-active --quiet nginx; then echo active > "$backup/nginx.state"; else echo inactive > "$backup/nginx.state"; fi
install -d -m 0755 "$webroot"

render(){
  local src=$1 dst=$2
  sed "s/__IP__/$ip/g" "$src" > "$dst.pending"
  install -o root -g root -m 0644 "$dst.pending" "$dst"
  rm -f "$dst.pending"
}

rollback_script="$backup/rollback.sh"
cat > "$rollback_script" <<ROLLBACK
#!/usr/bin/env bash
set -Eeuo pipefail
site='$site'
enabled='$enabled'
backup='$backup'
nginx='$nginx'
if [[ -f "\$backup/site.previous" ]]; then cp -a -- "\$backup/site.previous" "\$site"; else rm -f -- "\$site"; fi
if [[ -f "\$backup/enabled.link" ]]; then
  target=\$(cat "\$backup/enabled.link")
  ln -sfn -- "\$target" "\$enabled"
else
  rm -f -- "\$enabled"
fi
default_enabled=/etc/nginx/sites-enabled/default
hook=/etc/letsencrypt/renewal-hooks/deploy/ipat-platform-ip-nginx
service=/etc/systemd/system/ipat-ip-cert-renew.service
timer=/etc/systemd/system/ipat-ip-cert-renew.timer
/bin/systemctl disable --now ipat-ip-cert-renew.timer 2>/dev/null || true
rm -f -- "\$hook" "\$service" "\$timer"
if [[ -f "\$backup/default.enabled.link" ]]; then
  default_target=\$(cat "\$backup/default.enabled.link")
  ln -sfn -- "\$default_target" "\$default_enabled"
fi
/bin/systemctl daemon-reload 2>/dev/null || true
if "\$nginx" -t; then
  if [[ \$(cat "\$backup/nginx.state") == active ]]; then
    systemctl reload nginx
  else
    systemctl stop nginx || true
  fi
fi
ROLLBACK
chmod 0700 "$rollback_script"

# Production cutover is forbidden unless BOTH separate Platform services are
# already healthy on exact loopback. Staging ACME validation needs only port80.
# This check occurs before any public nginx mutation or rollback timer.
if [[ $phase == production ]]; then
  listeners=$(ss -H -ltn '( sport = :3005 or sport = :3006 )' 2>/dev/null || true)
  [[ $(grep -cE '127[.]0[.]0[.]1:3005[[:space:]]' <<<"$listeners") -eq 1 ]]     || die "Platform Owner API must listen exactly on 127.0.0.1:3005"
  [[ $(grep -cE '127[.]0[.]0[.]1:3006[[:space:]]' <<<"$listeners") -eq 1 ]]     || die "Platform Owner OIDC issuer must listen exactly on 127.0.0.1:3006"
  if grep -Eq '(^|[[:space:]])(0[.]0[.]0[.]0|[[]::[]]|[[]::0[]]):(3005|3006)([[:space:]]|$)' <<<"$listeners"; then
    die "Platform services must never bind wildcard public interfaces"
  fi
  curl --fail --silent --show-error --max-time 3     -H "Host: $ip" "http://127.0.0.1:3005/" >/dev/null     || die "Platform Owner API exact-Host preflight failed"
  issuer_status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 3     -H "Host: $ip" "http://127.0.0.1:3006/__r984_ingress_probe" || true)
  [[ $issuer_status == 404 ]] || die "Platform Owner OIDC issuer loopback preflight failed"
fi

# Arm rollback BEFORE changing public nginx state.
systemd-run --quiet --collect --unit="$rollback_unit" --on-active=10m "$rollback_script"
rollback_armed=YES
rollback_now(){ [[ $rollback_armed == YES ]] && "$rollback_script" || true; }
trap 'rollback_now' ERR

# Refuse to seize an unrelated existing listener.
if ss -ltnp '( sport = :80 or sport = :443 )' 2>/dev/null | grep -q LISTEN && [[ ! -e $enabled ]]; then
  die "port 80/443 already has an unmanaged listener"
fi

rm -f -- "$default_enabled"
render "$bootstrap" "$site"
ln -sfn "$site" "$enabled"
"$nginx" -t
if systemctl is-active --quiet nginx; then systemctl reload nginx; else systemctl start nginx; fi

if [[ $phase == staging ]]; then
  cert_name=ipat-platform-ip-staging
else
  cert_name=ipat-platform-ip
fi
args=(certonly --non-interactive --agree-tos --email "$email"
  --preferred-profile shortlived
  --webroot --webroot-path "$webroot"
  --ip-address "$ip"
  --cert-name "$cert_name")
[[ $phase == staging ]] && args+=(--staging)
"$certbot" "${args[@]}"

cert="/etc/letsencrypt/live/$cert_name/fullchain.pem"
key="/etc/letsencrypt/live/$cert_name/privkey.pem"
[[ -s $cert && -s $key ]] || die "ACME client did not produce expected certificate lineage"
openssl x509 -in "$cert" -noout -checkip "$ip" >/dev/null || die "certificate lacks exact iPAddress SAN"

if [[ $phase == staging ]]; then
  "$certbot" delete --non-interactive --cert-name "$cert_name"
  rollback_now
  rollback_armed=NO
  systemctl stop "$rollback_unit.timer" "$rollback_unit.service" 2>/dev/null || true
  trap - ERR
  echo R984_STAGING_IP_ACME_VALIDATED_AND_ROLLED_BACK
  exit 0
fi

render "$https" "$site"
"$nginx" -t
systemctl reload nginx

install -d -m 0755 "$(dirname "$hook")"
cat > "$hook" <<HOOK
#!/usr/bin/env bash
set -Eeuo pipefail
/usr/bin/openssl x509 -in '$cert' -noout -checkip '$ip' >/dev/null
'$nginx' -t
/bin/systemctl reload nginx
HOOK
chmod 0755 "$hook"

sed "s|__CERTBOT__|$certbot|g" "$renew_service" > "$service.pending"
install -o root -g root -m 0644 "$service.pending" "$service"
rm -f "$service.pending"
install -o root -g root -m 0644 "$renew_timer" "$timer"
systemctl daemon-reload
systemctl enable --now ipat-ip-cert-renew.timer

# Dry-run verifies renewal and deploy hook; it does not force a production issuance.
"$certbot" renew --cert-name "$cert_name" --dry-run --run-deploy-hooks
"$nginx" -t
curl --fail --silent --show-error --max-time 5 --cacert /etc/ssl/certs/ca-certificates.crt "https://$ip/" >/dev/null
oidc_status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 5   --cacert /etc/ssl/certs/ca-certificates.crt "https://$ip/platform/auth/oidc/start" || true)
[[ $oidc_status == 303 ]] || die "public trusted HTTPS does not reach the dedicated OIDC start route"

systemctl stop "$rollback_unit.timer" "$rollback_unit.service" 2>/dev/null || true
rollback_armed=NO
trap - ERR
echo R984_PRODUCTION_IP_TLS_EDGE_INSTALLED_LOCAL_CA_VERIFIED_EXTERNAL_VERIFICATION_REQUIRED
