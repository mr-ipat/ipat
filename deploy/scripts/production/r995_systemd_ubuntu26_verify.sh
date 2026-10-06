#!/usr/bin/env bash
# Disposable Ubuntu26 systemd syntax/hardening verification for R9.95.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R995_DISPOSABLE_SYSTEMD:-} == YES ]] || { echo 'Requires disposable-only opt-in' >&2; exit 2; }
command -v docker >/dev/null || exit 2
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
name="ipat-r995-systemd-$RANDOM-$$"
cleanup(){ docker rm -f "$name" >/dev/null 2>&1 || :; }
trap cleanup EXIT
docker run --rm --name "$name" -v "$root:/src:ro" ubuntu:26.04@sha256:f144425ff09be612d6d9ad965196e9cdc23dae1f42110a8a11a3e9a8198759f7 bash -lc '
set -Eeuo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends systemd ca-certificates >/dev/null
useradd --system --home-dir /nonexistent --shell /usr/sbin/nologin ipatdnsverify
install -d -o root -g root -m 0755 /opt/ipat/bin
printf "#!/bin/sh\nexit 0\n" >/opt/ipat/bin/control-api
chmod 0755 /opt/ipat/bin/control-api
install -d -o ipatdnsverify -g ipatdnsverify -m 0700 /var/lib/ipat-domain-ownership-verifier
printf "host=/run/postgresql dbname=ipat_prod user=ipat_domain_verifier_login\n" >/var/lib/ipat-domain-ownership-verifier/postgres.conninfo
chown ipatdnsverify:ipatdnsverify /var/lib/ipat-domain-ownership-verifier/postgres.conninfo
chmod 0600 /var/lib/ipat-domain-ownership-verifier/postgres.conninfo
install -m 0644 /src/deploy/systemd/ipat-domain-ownership-verifier.service /etc/systemd/system/
systemd-analyze verify /etc/systemd/system/ipat-domain-ownership-verifier.service
grep -Fq "User=ipatdnsverify" /etc/systemd/system/ipat-domain-ownership-verifier.service
grep -Fq "ProtectSystem=strict" /etc/systemd/system/ipat-domain-ownership-verifier.service
grep -Fq "RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6" /etc/systemd/system/ipat-domain-ownership-verifier.service
echo R995_UBUNTU26_OWNERSHIP_SYSTEMD_VERIFY_PASS
'
