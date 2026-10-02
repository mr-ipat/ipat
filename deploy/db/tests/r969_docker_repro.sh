#!/usr/bin/env bash
# R9.69 isolated PostgreSQL 16 platform company lifecycle proof.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R969_SYNTHETIC_DOCKER:-} == YES ]] || { echo 'Set IPAT_R969_SYNTHETIC_DOCKER=YES for disposable test.' >&2; exit 2; }
command -v docker >/dev/null && command -v python3 >/dev/null || exit 2
[[ -z ${PGSERVICE:-} && -z ${PGSERVICEFILE:-} ]] || exit 2
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P); name="ipat-r969-disposable-$RANDOM-$$"; tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r969-XXXXXX")
cleanup(){ docker rm -f "$name" >/dev/null 2>&1||:;rm -rf "$tmp"; };trap cleanup EXIT
docker run --rm -d --name "$name" --label ipat.test.disposable=r969 -e POSTGRES_PASSWORD=local_ci_synthetic_only -e POSTGRES_DB=ipat_synthetic -v "$root:/src:ro" postgres:16-alpine >/dev/null
for _ in {1..50};do docker exec "$name" pg_isready -h 127.0.0.1 -U postgres -d ipat_synthetic >/dev/null 2>&1&&break;sleep .2;done
cat > "$tmp/psql" <<'WRAP'
#!/usr/bin/env bash
set -Eeuo pipefail
exec docker exec -e PGPASSWORD=local_ci_synthetic_only "$IPAT_R969_CONTAINER_NAME" psql -h 127.0.0.1 -U postgres -d ipat_synthetic "$@"
WRAP
chmod 0700 "$tmp/psql";export PATH="$tmp:$PATH" IPAT_R969_CONTAINER_NAME="$name" IPAT_PG_EPHEMERAL_TEST=1 PGHOST=127.0.0.1 PGDATABASE=ipat_synthetic PGUSER=postgres IPAT_PG_SYNTHETIC_PASSWORD=local_ci_synthetic_only
for m in 0001_lab_tenant_rls.sql 0003_lab_identity_memberships.sql 0004_lab_scoped_identity_lookup.sql 0023_platform_company_lifecycle.sql;do psql -X -q -v ON_ERROR_STOP=1 -f "/src/deploy/db/migrations/$m" >/dev/null;done
python3 -m unittest "$root/deploy/db/tests/test_platform_company_lifecycle_integration.py" -v
echo R969_DISPOSABLE_PG16_PLATFORM_COMPANY_LIFECYCLE=PASS
