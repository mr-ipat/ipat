#!/usr/bin/env bash
# R9.89: disposable PostgreSQL18 physical streaming standby + controlled promotion.
# No host PostgreSQL ports, no live production database, no automatic failback.
set -Eeuo pipefail
umask 077
set +u
opt=$IPAT_R989_PG18_STREAMING_DOCKER
set -u
[[ "$opt" == YES ]] || { echo 'Requires explicit disposable streaming-HA opt-in' >&2; exit 2; }
command -v docker >/dev/null && command -v sha256sum >/dev/null || exit 2
root=$(cd "$(dirname "$0")/../../.." && pwd -P)
image='postgres:18-alpine@sha256:77f585114c32fbca283dc835b0596f4e52b51b4c6662d7810b2f4084f60a1873'
tag="r989-$RANDOM-$$"
network="ipat-"$tag"-net"
primary="ipat-"$tag"-primary"
standby="ipat-"$tag"-standby"
primary_volume="ipat-"$tag"-primary-data"
standby_volume="ipat-"$tag"-standby-data"
cleanup(){
  docker rm -f "$primary" "$standby" >/dev/null 2>&1 || :
  docker network rm "$network" >/dev/null 2>&1 || :
  docker volume rm -f "$primary_volume" "$standby_volume" >/dev/null 2>&1 || :
}
trap cleanup EXIT
docker network create "$network" >/dev/null
docker volume create "$primary_volume" >/dev/null
docker volume create "$standby_volume" >/dev/null

docker run --rm -d --name "$primary" --network "$network" \
  --label ipat.test.disposable=r989 \
  -e POSTGRES_PASSWORD=local_ci_synthetic_only \
  -e POSTGRES_DB=ipat_synthetic \
  -e PGDATA=/pgdata \
  -v "$primary_volume:/pgdata" \
  -v "$root:/src:ro" \
  "$image" \
  -c wal_level=replica \
  -c max_wal_senders=5 \
  -c max_replication_slots=5 \
  -c hot_standby=on >/dev/null

ready=NO
for _ in $(seq 1 100); do
  database=$(docker exec -u postgres "$primary" psql -XAtq -U postgres -d ipat_synthetic \
    -c 'select current_database()' 2>/dev/null || true)
  if [[ "$database" == ipat_synthetic ]]; then
    ready=YES
    break
  fi
  sleep .25
done
[[ "$ready" == YES ]] || { docker logs "$primary" >&2 || true; echo 'primary PostgreSQL18 target database did not become ready' >&2; exit 3; }
[[ -z $(docker port "$primary" 2>/dev/null) ]] || { echo 'primary host port publishing forbidden' >&2; exit 3; }
version=$(docker exec -u postgres "$primary" psql -XAtq -U postgres -d ipat_synthetic -c 'show server_version_num')
[[ "$version" =~ ^18[0-9]{4}$ ]] || { echo "Expected PostgreSQL18, got $version" >&2; exit 3; }

while read -r sum file; do
  [[ -n "$sum" && -n "$file" ]] || continue
  actual=$(sha256sum "$root/$file" | awk '{print $1}')
  [[ "$actual" == "$sum" ]] || { echo "migration hash mismatch: $file" >&2; exit 3; }
  docker exec -u postgres "$primary" psql -X -q -v ON_ERROR_STOP=1 \
    -U postgres -d ipat_synthetic -f "/src/$file" >/dev/null
done < "$root/deploy/db/production/r987_migrations.sha256"

docker exec -u postgres "$primary" psql -Xq -v ON_ERROR_STOP=1 -U postgres -d ipat_synthetic \
  -c "CREATE ROLE ipat_replica LOGIN REPLICATION PASSWORD 'local_ci_replica_only';" >/dev/null
docker exec -u 0 "$primary" sh -ceu \
  "printf '%s\n' 'host replication ipat_replica all scram-sha-256' >> /pgdata/pg_hba.conf"
docker exec -u postgres "$primary" psql -XAtq -U postgres -d ipat_synthetic \
  -c 'select pg_reload_conf()' | grep -Fxq t

tenant='89898989-8989-4989-8989-898989898981'
row_a='89898989-8989-4989-8989-898989898982'
row_b='89898989-8989-4989-8989-898989898983'
row_c='89898989-8989-4989-8989-898989898984'
docker exec -i -u postgres "$primary" psql -Xq -v ON_ERROR_STOP=1 -U postgres -d ipat_synthetic <<SQL
INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
VALUES('$tenant','r989-ha','active');
INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor,exact_model,firmware)
VALUES('$tenant','$row_a','R989-POP','olt','SYNTHETIC','PRIMARY-SEEDED','synthetic-1');
SQL
source_count=$(docker exec -u postgres "$primary" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT count(*) FROM ipat_ops.devices WHERE tenant_id='$tenant'" | tr -d '[:space:]')
[[ "$source_count" == 1 ]] || { echo 'primary seed missing' >&2; exit 3; }

docker run --rm -v "$standby_volume:/standby" "$image" sh -ceu \
  'chown postgres:postgres /standby'
docker run --rm --user postgres --network "$network" \
  -e PGPASSWORD=local_ci_replica_only \
  -v "$standby_volume:/standby" "$image" \
  pg_basebackup -h "$primary" -U ipat_replica -D /standby -Fp -Xs -R -c fast >/dev/null

docker run --rm --user postgres -v "$standby_volume:/standby" "$image" sh -ceu \
  "printf '%s\n' \"primary_conninfo = 'host=$primary port=5432 user=ipat_replica password=local_ci_replica_only application_name=r989_standby'\" >> /standby/postgresql.auto.conf"

docker run --rm -d --name "$standby" --network "$network" \
  --label ipat.test.disposable=r989 \
  -e PGDATA=/standby \
  -v "$standby_volume:/standby" \
  "$image" -c hot_standby=on >/dev/null
[[ -z $(docker port "$standby" 2>/dev/null) ]] || { echo 'standby host port publishing forbidden' >&2; exit 3; }

ready=NO
for _ in $(seq 1 100); do
  state=$(docker exec -u postgres "$standby" psql -XAtq -U postgres -d ipat_synthetic \
    -c "SELECT current_database()||'|'||CASE WHEN pg_is_in_recovery() THEN '1' ELSE '0' END" 2>/dev/null || true)
  if [[ "$state" == 'ipat_synthetic|1' ]]; then
    ready=YES
    break
  fi
  sleep .25
done
[[ "$ready" == YES ]] || { docker logs "$standby" >&2 || true; echo 'standby target database did not enter recovery' >&2; exit 3; }

streaming=NO
for _ in $(seq 1 80); do
  state=$(docker exec -u postgres "$primary" psql -XAtq -U postgres -d ipat_synthetic \
    -c "SELECT state FROM pg_stat_replication WHERE application_name='r989_standby' LIMIT 1" 2>/dev/null || true)
  if [[ "$state" == streaming ]]; then
    streaming=YES
    break
  fi
  sleep .25
done
[[ "$streaming" == YES ]] || { echo 'standby never reached streaming state' >&2; exit 3; }

set +e
docker exec -u postgres "$standby" psql -Xq -v ON_ERROR_STOP=1 -U postgres -d ipat_synthetic \
  -c "INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor) VALUES('$tenant','$row_c','R989-POP','router','SHOULD-FAIL')" >/dev/null 2>&1
prepromote_write=$?
set -e
[[ "$prepromote_write" -ne 0 ]] || { echo 'standby accepted write before promotion' >&2; exit 1; }

docker exec -u postgres "$primary" psql -Xq -v ON_ERROR_STOP=1 -U postgres -d ipat_synthetic \
  -c "INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor,exact_model,firmware) VALUES('$tenant','$row_b','R989-POP','router','SYNTHETIC','REPLICATED-BEFORE-FAILOVER','synthetic-2');" >/dev/null

replicated=NO
for _ in $(seq 1 100); do
  count=$(docker exec -u postgres "$standby" psql -XAtq -U postgres -d ipat_synthetic \
    -c "SELECT count(*) FROM ipat_ops.devices WHERE tenant_id='$tenant'" 2>/dev/null | tr -d '[:space:]')
  if [[ "$count" == 2 ]]; then
    replicated=YES
    break
  fi
  sleep .25
done
[[ "$replicated" == YES ]] || { echo 'standby did not receive committed pre-failover row' >&2; exit 3; }

docker stop -t 10 "$primary" >/dev/null
[[ -z $(docker ps -q --filter "name=^/$primary$") ]] || { echo 'primary still running before promotion' >&2; exit 3; }

docker exec -u postgres "$standby" pg_ctl -D /standby promote -w >/dev/null
promoted=NO
for _ in $(seq 1 80); do
  state=$(docker exec -u postgres "$standby" psql -XAtq -U postgres -d ipat_synthetic \
    -c "SELECT (NOT pg_is_in_recovery())::int||'|'||current_database()||'|'||(SELECT count(*) FROM ipat_ops.devices WHERE tenant_id='$tenant')" 2>/dev/null || true)
  if [[ "$state" == '1|ipat_synthetic|2' ]]; then
    promoted=YES
    break
  fi
  sleep .25
done
[[ "$promoted" == YES ]] || { docker logs "$standby" >&2 || true; echo 'standby promotion readiness failed' >&2; exit 3; }

promoted_count=$(docker exec -u postgres "$standby" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT count(*) FROM ipat_ops.devices WHERE tenant_id='$tenant'" | tr -d '[:space:]')
[[ "$promoted_count" == 2 ]] || { echo 'promoted node lost pre-failover data' >&2; exit 1; }

unscoped=$(docker exec -u postgres "$standby" psql -XAtq -U ipat_app_runtime -d ipat_synthetic \
  -c 'SELECT count(*) FROM ipat_ops.devices' | tr -d '[:space:]')
[[ "$unscoped" == 0 ]] || { echo 'promoted runtime sees unscoped data' >&2; exit 1; }
scoped=$(docker exec -u postgres "$standby" psql -XAtq -U ipat_app_runtime -d ipat_synthetic \
  -c "BEGIN; SET LOCAL ipat.tenant_id='$tenant'; SELECT count(*) FROM ipat_ops.devices; COMMIT" \
  | grep -E '^[0-9]+$' | tail -1)
[[ "$scoped" == 2 ]] || { echo 'promoted tenant RLS count mismatch' >&2; exit 1; }

docker exec -u postgres "$standby" psql -Xq -v ON_ERROR_STOP=1 -U postgres -d ipat_synthetic \
  -c "INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor,exact_model,firmware) VALUES('$tenant','$row_c','R989-POP','router','SYNTHETIC','WRITE-AFTER-PROMOTE','synthetic-3');" >/dev/null
final_count=$(docker exec -u postgres "$standby" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT count(*) FROM ipat_ops.devices WHERE tenant_id='$tenant'" | tr -d '[:space:]')
[[ "$final_count" == 3 ]] || { echo 'promoted node not writable after controlled promotion' >&2; exit 1; }

echo R989_POSTGRES18_STREAMING_STANDBY_CONTROLLED_PROMOTION=PASS
echo R989_PREPROMOTE_WRITE_DENIED=PASS
echo "R989_PROMOTED_FORCE_RLS_UNSCOPED=$unscoped SCOPED=$scoped"
echo "R989_PROMOTED_FINAL_TENANT_ROWS=$final_count"
echo NOT_AUTOMATIC_FAILOVER_NOT_FAILBACK_NOT_PRODUCTION_HA
