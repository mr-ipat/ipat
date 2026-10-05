#!/usr/bin/env bash
# Disposable Ubuntu 26.04 full first-install rehearsal for R9.87.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R987_DISPOSABLE_DOCKER:-} == YES ]] || { echo 'Requires disposable-only opt-in' >&2; exit 2; }
command -v docker >/dev/null || exit 2
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
name="ipat-r987-u26-$RANDOM-$$"
cleanup(){ docker rm -f "$name" >/dev/null 2>&1 || :; }
trap cleanup EXIT
docker run --rm --name "$name" -v "$root:/src:ro" ubuntu:26.04@sha256:f144425ff09be612d6d9ad965196e9cdc23dae1f42110a8a11a3e9a8198759f7 bash -lc '
set -Eeuo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends iproute2 ca-certificates >/dev/null
useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin ipatpapi
useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin ipatpoidc
useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin ipattapi
useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin ipattoidc
IPAT_R987_BOOTSTRAP=CREATE_REVIEWED_LOCAL_POSTGRES18 IPAT_R987_SOURCE_ROOT=/src bash /src/deploy/scripts/production/r987_bootstrap_postgres18.sh
[[ $(pg_lsclusters --no-header | awk '"'"'{print $1"/"$2"/"$4}'"'"') == 18/ipat/online ]]
[[ $(ss -H -ltn "( sport = :5432 )" | wc -l) -eq 0 ]]
[[ $(runuser -u ipatpapi -- psql -X -w -h /run/postgresql -U ipat_platform_session_api_login -d ipat_prod -Atqc "select current_user") == ipat_platform_session_api_login ]]
[[ $(runuser -u ipatpoidc -- psql -X -w -h /run/postgresql -U ipat_platform_session_issuer_login -d ipat_prod -Atqc "select current_user") == ipat_platform_session_issuer_login ]]
[[ $(runuser -u ipattapi -- psql -X -w -h /run/postgresql -U ipat_tenant_api_login -d ipat_prod -Atqc "select current_user") == ipat_tenant_api_login ]]
[[ $(runuser -u ipattoidc -- psql -X -w -h /run/postgresql -U ipat_oidc_session_issuer_login -d ipat_prod -Atqc "select current_user") == ipat_oidc_session_issuer_login ]]
! getent passwd ipatpgmigrate >/dev/null
echo R987_UBUNTU26_DISPOSABLE_BOOTSTRAP_PASS
'
