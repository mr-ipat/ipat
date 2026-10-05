#!/usr/bin/env bash
# Disposable syntax-only nginx validation for R9.84 templates.
set -Eeuo pipefail
umask 077
command -v docker >/dev/null
command -v openssl >/dev/null
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
image='nginx:1.28-alpine@sha256:a8b39bd9cf0f83869a2162827a0caf6137ddf759d50a171451b335cecc87d236'
ip=202.162.204.121
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r984-nginx-XXXXXX")
cleanup(){ rm -rf "$tmp"; }
trap cleanup EXIT

sed "s/__IP__/$ip/g" "$root/deploy/nginx/r984-platform-ip-bootstrap.conf.template" > "$tmp/default.conf"
docker run --rm -v "$tmp/default.conf:/etc/nginx/conf.d/default.conf:ro" "$image" nginx -t

mkdir -p "$tmp/live/ipat-platform-ip"
openssl req -x509 -newkey rsa:2048 -nodes -days 1   -subj '/CN=synthetic-r984'   -addext "subjectAltName=IP:$ip"   -keyout "$tmp/live/ipat-platform-ip/privkey.pem"   -out "$tmp/live/ipat-platform-ip/fullchain.pem" >/dev/null 2>&1
openssl x509 -in "$tmp/live/ipat-platform-ip/fullchain.pem" -noout -checkip "$ip" >/dev/null
sed "s/__IP__/$ip/g" "$root/deploy/nginx/r984-platform-ip-https.conf.template" > "$tmp/default.conf"
docker run --rm   -v "$tmp/default.conf:/etc/nginx/conf.d/default.conf:ro"   -v "$tmp/live:/etc/letsencrypt/live:ro"   "$image" nginx -t

echo R984_DISPOSABLE_NGINX_BOOTSTRAP_AND_TLS_SYNTAX_PASS
