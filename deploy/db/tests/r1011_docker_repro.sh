#!/usr/bin/env bash
set -Eeuo pipefail
umask 077
[[ ${IPAT_R1011_SYNTHETIC_DOCKER:-} == YES ]] || { echo 'explicit disposable opt-in required' >&2; exit 2; }
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
name="ipat-r1011-$RANDOM-$$"; tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r1011-XXXXXX")
cleanup(){ docker rm -f "$name" >/dev/null 2>&1 || :; rm -rf "$tmp"; }
trap cleanup EXIT
docker run --rm -d --name "$name" --label ipat.test.disposable=r1011 \
 -e POSTGRES_PASSWORD=local_ci_synthetic_only -e POSTGRES_DB=ipat_synthetic \
 -v "$root:/src:ro" postgres:16-alpine >/dev/null
ready=NO
for _ in {1..80}; do
  if docker exec "$name" psql -U postgres -d ipat_synthetic -Atqc 'select 1' 2>/dev/null | grep -Fxq 1; then ready=YES; break; fi
  sleep .25
done
[[ $ready == YES ]] || { echo "final target database not ready" >&2; exit 3; }
cat > "$tmp/psql" <<'WRAP'
#!/usr/bin/env bash
set -Eeuo pipefail
exec docker exec -i -e PGPASSWORD=local_ci_synthetic_only "$IPAT_R1011_CONTAINER" \
 psql -h 127.0.0.1 -U postgres -d ipat_synthetic "$@"
WRAP
chmod 0700 "$tmp/psql"
export PATH="$tmp:$PATH" IPAT_R1011_CONTAINER="$name" IPAT_PG_EPHEMERAL_TEST=1 \
 PGHOST=127.0.0.1 PGDATABASE=ipat_synthetic PGUSER=postgres \
 IPAT_PG_SYNTHETIC_PASSWORD=local_ci_synthetic_only
count=0
while read -r _ path; do
  [[ -n "${path:-}" ]] || continue
  docker exec -i "$name" psql -X -q -v ON_ERROR_STOP=1 -U postgres -d ipat_synthetic < "$root/$path" >/dev/null
  count=$((count+1))
done < "$root/deploy/db/production/r987_migrations.sha256"
[[ $count -eq 32 ]] || { echo "unexpected migration count: $count" >&2; exit 3; }
python3 -m unittest "$root/deploy/db/tests/test_company_activation_lifecycle_integration.py" -v
python3 -m unittest "$root/deploy/db/tests/test_company_activation_session_wrapper_integration.py" -v
echo R1011_DISPOSABLE_PG16_COMPANY_ACTIVATION=PASS
