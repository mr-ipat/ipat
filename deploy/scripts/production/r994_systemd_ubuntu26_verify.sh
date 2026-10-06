#!/usr/bin/env bash
# Disposable Ubuntu26 systemd unit syntax/hardening verification for R9.94.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R994_DISPOSABLE_SYSTEMD:-} == YES ]] || { echo 'Requires disposable-only opt-in' >&2; exit 2; }
command -v docker >/dev/null || exit 2
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
name="ipat-r994-systemd-$RANDOM-$$"
cleanup(){ docker rm -f "$name" >/dev/null 2>&1 || :; }
trap cleanup EXIT
docker run --rm --name "$name" -v "$root:/src:ro" ubuntu:26.04@sha256:f144425ff09be612d6d9ad965196e9cdc23dae1f42110a8a11a3e9a8198759f7 bash -lc '
set -Eeuo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends systemd ca-certificates >/dev/null
useradd --system --home-dir /nonexistent --shell /usr/sbin/nologin ipatdverify
install -d -o root -g root -m 0755 /opt/ipat/bin
install -d -o root -g root -m 0700 /etc/ipat
cat >/opt/ipat/bin/r994-verify-customer-ingress <<EOF
#!/bin/sh
exit 0
EOF
chmod 0755 /opt/ipat/bin/r994-verify-customer-ingress
cat >/etc/ipat/domain-ingress-verifier.env <<EOF
IPAT_R994_VERIFY_INGRESS=YES
IPAT_R994_ALLOW_ACTIVATE=YES
IPAT_R994_DB_NAME=ipat_prod
IPAT_R994_DB_SOCKET=/run/postgresql
IPAT_R994_PUBLIC_IPV4=198.18.0.1
EOF
install -m 0644 /src/deploy/systemd/ipat-domain-ingress-verifier.service /etc/systemd/system/
install -m 0644 /src/deploy/systemd/ipat-domain-ingress-verifier.timer /etc/systemd/system/
systemd-analyze verify /etc/systemd/system/ipat-domain-ingress-verifier.service /etc/systemd/system/ipat-domain-ingress-verifier.timer
grep -Fq "User=ipatdverify" /etc/systemd/system/ipat-domain-ingress-verifier.service
grep -Fq "ProtectSystem=strict" /etc/systemd/system/ipat-domain-ingress-verifier.service
grep -Fq "MemoryDenyWriteExecute=true" /etc/systemd/system/ipat-domain-ingress-verifier.service
grep -Fq "RandomizedDelaySec=20s" /etc/systemd/system/ipat-domain-ingress-verifier.timer
echo R994_UBUNTU26_SYSTEMD_VERIFY_PASS
'
