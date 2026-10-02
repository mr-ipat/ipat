#!/usr/bin/env bash
# R9.67 exact disposable PostgreSQL 16 durable Host-bound session proof.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R967_SYNTHETIC_DOCKER:-} == YES ]] || { echo 'Set IPAT_R967_SYNTHETIC_DOCKER=YES for disposable-only proof.' >&2; exit 2; }
command -v docker >/dev/null && command -v python3 >/dev/null || { echo 'Docker + Python required' >&2; exit 2; }
[[ -z ${PGSERVICE:-} && -z ${PGSERVICEFILE:-} ]] || { echo 'Refusing inherited PostgreSQL service profile' >&2; exit 2; }
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
name="ipat-r967-disposable-$RANDOM-$$"; tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r967-XXXXXX")
cleanup(){ docker rm -f -- "$name" >/dev/null 2>&1 || :; rm -rf -- "$tmp"; }
trap cleanup EXIT
docker run --rm -d --name "$name" --label ipat.test.disposable=r967 \
 -e POSTGRES_PASSWORD=local_ci_synthetic_only -e POSTGRES_DB=ipat_synthetic \
 -v "$root:/src:ro" postgres:16-alpine >/dev/null
ready=NO
for _ in {1..50}; do docker exec "$name" pg_isready -h 127.0.0.1 -U postgres -d ipat_synthetic >/dev/null 2>&1 && { ready=YES; break; }; sleep .2; done
[[ $ready == YES ]] || { echo 'Disposable PostgreSQL unavailable' >&2; exit 3; }
cat > "$tmp/psql" <<'WRAP'
#!/usr/bin/env bash
set -Eeuo pipefail
[[ ${IPAT_PG_EPHEMERAL_TEST:-} == 1 && ${PGDATABASE:-} == ipat_synthetic && ${PGHOST:-} == 127.0.0.1 ]] || exit 3
exec docker exec -e PGPASSWORD=local_ci_synthetic_only "$IPAT_R967_CONTAINER_NAME" \
 psql -h 127.0.0.1 -U postgres -d ipat_synthetic "$@"
WRAP
chmod 0700 "$tmp/psql"
export PATH="$tmp:$PATH" IPAT_R967_CONTAINER_NAME="$name" IPAT_PG_EPHEMERAL_TEST=1
export PGHOST=127.0.0.1 PGDATABASE=ipat_synthetic PGUSER=postgres IPAT_PG_SYNTHETIC_PASSWORD=local_ci_synthetic_only
for m in 0001_lab_tenant_rls.sql 0003_lab_identity_memberships.sql 0004_lab_scoped_identity_lookup.sql \
 0012_tenant_domains.sql 0013_tenant_domain_enrollment.sql 0014_tenant_domain_activation_lifecycle.sql; do
 psql -X -q -v ON_ERROR_STOP=1 -f "/src/deploy/db/migrations/$m" >/dev/null
done
psql -X -q -v ON_ERROR_STOP=1 -f /src/deploy/db/migrations/0021_durable_tenant_browser_sessions.sql >/dev/null
python3 -m unittest "$root/deploy/db/tests/test_durable_tenant_browser_sessions_integration.py" -v
echo R967_DISPOSABLE_REAL_PG16_HOST_BOUND_SESSIONS=PASS
