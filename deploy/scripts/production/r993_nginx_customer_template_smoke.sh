#!/usr/bin/env bash
set -Eeuo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
image='nginx:1.28-alpine@sha256:a8b39bd9cf0f83869a2162827a0caf6137ddf759d50a171451b335cecc87d236'
host=portal.customer.co.id
instance=customer
port=31001
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r993-nginx-XXXXXX")
trap 'rm -rf "$tmp"' EXIT

sed -e "s/__HOST__/$host/g" -e "s/__INSTANCE__/$instance/g" -e "s/__OIDC_PORT__/$port/g"  "$root/deploy/nginx/r993-tenant-bootstrap.conf.template" > "$tmp/default.conf"
docker run --rm -v "$tmp/default.conf:/etc/nginx/conf.d/default.conf:ro" "$image" nginx -t

mkdir -p "$tmp/live/ipat-tenant-$instance"
openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj "/CN=$host"  -addext "subjectAltName=DNS:$host"  -keyout "$tmp/live/ipat-tenant-$instance/privkey.pem"  -out "$tmp/live/ipat-tenant-$instance/fullchain.pem" >/dev/null 2>&1
openssl x509 -in "$tmp/live/ipat-tenant-$instance/fullchain.pem" -noout -checkhost "$host" >/dev/null
sed -e "s/__HOST__/$host/g" -e "s/__INSTANCE__/$instance/g" -e "s/__OIDC_PORT__/$port/g"  "$root/deploy/nginx/r993-tenant-https.conf.template" > "$tmp/default.conf"
docker run --rm -v "$tmp/default.conf:/etc/nginx/conf.d/default.conf:ro"  -v "$tmp/live:/etc/letsencrypt/live:ro" "$image" nginx -t

echo R993_CUSTOMER_NGINX_EXACT_HOST_TEMPLATE_PASS
