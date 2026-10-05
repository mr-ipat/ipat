#!/usr/bin/env bash
# R9.84 privileged prerequisite installer for nginx + current Certbot snap.
# Does not open firewall, enable nginx, issue a certificate, alter DNS or touch devices.
set -Eeuo pipefail
umask 077

die(){ echo "R984_PREP_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R984_PREPARE:-} == INSTALL_REVIEWED_IP_TLS_PREREQS ]] || die "explicit reviewed prerequisite opt-in missing"
[[ -r /etc/os-release ]] || die "OS identity unavailable"
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "requires Ubuntu Server 26.04"

for cmd in apt-get dpkg-query systemctl snap python3 install ln readlink ss; do
  command -v "$cmd" >/dev/null || die "missing prerequisite bootstrap tool: $cmd"
done
[[ ! -e /usr/sbin/policy-rc.d ]] || die "existing policy-rc.d requires manual review"

if dpkg-query -W -f='${Status}' certbot 2>/dev/null | grep -q 'install ok installed'; then
  die "Ubuntu apt Certbot package detected; do not mix OS Certbot with current snap Certbot"
fi

# Simulate first and refuse any package removals.
simulation=$(DEBIAN_FRONTEND=noninteractive apt-get -s install --no-install-recommends nginx snapd ca-certificates)
if grep -Eq '^Remv ' <<<"$simulation"; then
  die "APT simulation proposes package removal"
fi

cleanup_policy(){
  rm -f /usr/sbin/policy-rc.d
}
trap cleanup_policy EXIT
cat > /usr/sbin/policy-rc.d <<'POLICY'
#!/bin/sh
# R9.84 temporary package-install guard: prevent daemon auto-start.
exit 101
POLICY
chmod 0755 /usr/sbin/policy-rc.d

DEBIAN_FRONTEND=noninteractive apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends nginx snapd ca-certificates
cleanup_policy
trap - EXIT

systemctl stop nginx.service 2>/dev/null || true
if ss -ltnH '( sport = :80 or sport = :443 )' 2>/dev/null | grep -q .; then
  die "unexpected public web listener after prerequisite installation"
fi

if snap list certbot >/dev/null 2>&1; then
  snap refresh certbot
else
  snap install --classic certbot
fi
[[ -x /snap/bin/certbot ]] || die "Certbot snap executable unavailable"
python3 - /snap/bin/certbot <<'PY' || die "Certbot snap must be >=5.4 for IP certificates"
import re,subprocess,sys
out=subprocess.check_output([sys.argv[1],"--version"],text=True,stderr=subprocess.STDOUT)
m=re.search(r"(\d+)\.(\d+)(?:\.(\d+))?",out)
if not m or tuple(map(int,m.groups(default="0"))) < (5,4,0):
    raise SystemExit(1)
PY

link=/usr/local/bin/certbot
if [[ -e $link || -L $link ]]; then
  [[ -L $link && $(readlink "$link") == /snap/bin/certbot ]] || die "existing certbot command path requires manual review"
else
  ln -s /snap/bin/certbot "$link"
fi

command -v nginx >/dev/null || die "nginx install missing"
nginx -v
"$link" --version
systemctl is-active --quiet nginx && die "nginx must remain stopped until reviewed R9.84 edge apply"
echo R984_PREREQUISITES_INSTALLED_NGINX_STOPPED_NO_PUBLIC_LISTENER
