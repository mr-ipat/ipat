#!/usr/bin/env bash
# Independent real PostgreSQL 16 validation for the R9.64 two-phase Site FK.
# Disposable ONLY. Docker daemon access is powerful; run on a lab computer
# with approved local Docker, NOT a production node or business database.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R964_SYNTHETIC_DOCKER:-} == YES ]] || {
  echo 'Refusing: explicitly set IPAT_R964_SYNTHETIC_DOCKER=YES for a disposable local DB.' >&2
  exit 2
}
[[ -z ${PGSERVICE:-} && -z ${PGSERVICEFILE:-} ]] || { echo 'Refusing inherited production PostgreSQL service profile' >&2; exit 2; }
command -v docker >/dev/null && command -v python3 >/dev/null || { echo 'Requires local Docker and Python' >&2; exit 2; }
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
name="ipat-r964-disposable-$RANDOM-$$"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r964-XXXXXX")
cleanup(){
  docker rm -f -- "$name" >/dev/null 2>&1 || :
  rm -rf -- "$tmp"
}
trap cleanup EXIT
# No -p option: the synthetic PostgreSQL instance is not exposed at all.
docker run --rm -d --name "$name" --label ipat.test.disposable=r964 \
  -e POSTGRES_PASSWORD=local_ci_synthetic_only -e POSTGRES_DB=ipat_synthetic \
  -v "$root:$root:ro" postgres:16-alpine >/dev/null
ready=NO
for _ in {1..50};do
  if docker exec "$name" pg_isready -h 127.0.0.1 -U postgres -d ipat_synthetic >/dev/null 2>&1;then ready=YES; break; fi
  sleep 0.2
done
[[ "$ready" == YES ]] || { echo 'Disposable PostgreSQL did not become ready' >&2; exit 3; }
# The historical integration test helper invokes external psql. Route those
# operations exclusively into this exact no-port disposable container.
cat > "$tmp/psql" <<'WRAPPER'
#!/usr/bin/env bash
set -Eeuo pipefail
[[ "${IPAT_PG_EPHEMERAL_TEST:-}" == '1' && "${PGDATABASE:-}" == 'ipat_synthetic' && "${PGHOST:-}" == '127.0.0.1' ]] || exit 3
exec docker exec -e PGHOST=127.0.0.1 -e PGPORT=5432 -e PGDATABASE=ipat_synthetic -e "PGUSER=${PGUSER:-postgres}" \
 -e PGPASSWORD=local_ci_synthetic_only "$IPAT_R964_CONTAINER_NAME" psql "$@"
WRAPPER
chmod 0700 "$tmp/psql"
export PATH="$tmp:$PATH" PGHOST=127.0.0.1 PGPORT=5432 PGDATABASE=ipat_synthetic PGUSER=postgres
export IPAT_PG_EPHEMERAL_TEST=1 IPAT_PG_SYNTHETIC_PASSWORD=local_ci_synthetic_only
export IPAT_R964_CONTAINER_NAME="$name"
for migration in 0001_lab_tenant_rls.sql 0003_lab_identity_memberships.sql \
  0004_lab_scoped_identity_lookup.sql 0016_managed_device_registry.sql;do
  psql -X -q -v ON_ERROR_STOP=1 -f "$root/deploy/db/migrations/$migration" >/dev/null
done
psql -X -q -v ON_ERROR_STOP=1 -f "$root/deploy/db/tests/r964_synthetic_seed.sql" >/dev/null
python3 -m unittest discover "$root/deploy/db/tests" -p test_tenant_sites_integration.py -v
echo 'R964_DISPOSABLE_REAL_PG16_TWO_PHASE_SITE_FK_TESTS=PASS'
