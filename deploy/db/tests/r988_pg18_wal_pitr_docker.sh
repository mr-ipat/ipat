#!/usr/bin/env bash
# R9.88: disposable PostgreSQL18 base-backup + archived WAL named-point PITR.
# No host ports, no production credentials, no live PostgreSQL/host/device changes.
set -Eeuo pipefail
umask 077
set +u
opt=$IPAT_R988_PG18_PITR_DOCKER
set -u
[[ "$opt" == YES ]] || { echo 'Requires explicit disposable PITR opt-in' >&2; exit 2; }
command -v docker >/dev/null && command -v sha256sum >/dev/null || exit 2
root=$(cd "$(dirname "$0")/../../.." && pwd -P)
image='postgres:18-alpine@sha256:77f585114c32fbca283dc835b0596f4e52b51b4c6662d7810b2f4084f60a1873'
tag="r988-$RANDOM-$$"
source_name="ipat-"$tag"-source"
restore_name="ipat-"$tag"-restore"
archive_volume="ipat-"$tag"-archive"
backup_volume="ipat-"$tag"-backup"
cleanup(){
  docker rm -f "$source_name" "$restore_name" >/dev/null 2>&1 || :
  docker volume rm -f "$archive_volume" "$backup_volume" >/dev/null 2>&1 || :
}
trap cleanup EXIT
docker volume create "$archive_volume" >/dev/null
docker volume create "$backup_volume" >/dev/null
docker run --rm -d --name "$source_name" \
  --label ipat.test.disposable=r988 \
  -e POSTGRES_PASSWORD=local_ci_synthetic_only \
  -e POSTGRES_DB=ipat_synthetic \
  -v "$root:/src:ro" \
  -v "$archive_volume:/archive" \
  -v "$backup_volume:/backup" \
  "$image" \
  -c wal_level=replica \
  -c archive_mode=on \
  -c "archive_command=test ! -f /archive/%f && cp %p /archive/%f" \
  -c archive_timeout=5s >/dev/null

docker exec -u 0 "$source_name" sh -ceu 'chown postgres:postgres /archive /backup'
ready=NO
for _ in $(seq 1 80); do
  if docker exec -u postgres "$source_name" pg_isready -h 127.0.0.1 -U postgres -d ipat_synthetic >/dev/null 2>&1; then
    ready=YES
    break
  fi
  sleep .25
done
[[ "$ready" == YES ]] || { echo 'source PostgreSQL18 did not start' >&2; exit 3; }
[[ -z $(docker port "$source_name" 2>/dev/null) ]] || { echo 'host port publishing forbidden' >&2; exit 3; }
version=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic -c 'show server_version_num')
[[ "$version" =~ ^18[0-9]{4}$ ]] || { echo "Expected PostgreSQL18, got $version" >&2; exit 3; }

while read -r sum file; do
  [[ -n "$sum" && -n "$file" ]] || continue
  actual=$(sha256sum "$root/$file" | awk '{print $1}')
  [[ "$actual" == "$sum" ]] || { echo "migration hash mismatch: $file" >&2; exit 3; }
  docker exec -u postgres "$source_name" psql -X -q -v ON_ERROR_STOP=1 \
    -U postgres -d ipat_synthetic -f "/src/$file" >/dev/null
done < "$root/deploy/db/production/r987_migrations.sha256"

tenant='88888888-8888-4888-8888-888888888881'
good='88888888-8888-4888-8888-888888888882'
bad='88888888-8888-4888-8888-888888888883'
docker exec -i -u postgres "$source_name" psql -X -q -v ON_ERROR_STOP=1 \
  -U postgres -d ipat_synthetic <<SQL
INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
VALUES('$tenant','r988-pitr','active');
INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor,exact_model,firmware)
VALUES('$tenant','$good','R988-POP','olt','SYNTHETIC','GOOD-BEFORE-PITR','synthetic-1');
SQL

query="SELECT tenant_id||'|'||id||'|'||pop_id||'|'||device_kind||'|'||vendor||'|'||coalesce(exact_model,'')||'|'||coalesce(firmware,'') FROM ipat_ops.devices ORDER BY tenant_id,id"
source_good_count=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic -c "SELECT count(*) FROM ipat_ops.devices WHERE id='$good'" | tr -d '[:space:]')
[[ "$source_good_count" == 1 ]] || { echo 'pre-backup synthetic good row missing' >&2; exit 3; }
good_hash=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic -c "$query" | sha256sum | awk '{print $1}')
[[ "$good_hash" != e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 ]] || { echo 'pre-backup query unexpectedly empty' >&2; exit 3; }

docker exec -u postgres -e PGPASSWORD=local_ci_synthetic_only "$source_name" \
  pg_basebackup -h 127.0.0.1 -U postgres -D /backup -Fp -X stream -c fast >/dev/null
docker exec -u postgres "$source_name" pg_verifybackup /backup >/dev/null

point=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT pg_create_restore_point('r988_before_bad_change')")
[[ "$point" =~ ^[0-9A-F]+/[0-9A-F]+$ ]] || { echo 'restore point not created' >&2; exit 3; }

docker exec -i -u postgres "$source_name" psql -X -q -v ON_ERROR_STOP=1 \
  -U postgres -d ipat_synthetic <<SQL
INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor,exact_model,firmware)
VALUES('$tenant','$bad','R988-POP','router','SYNTHETIC','BAD-AFTER-PITR','synthetic-bad');
SELECT pg_switch_wal();
SQL

archived=NO
for _ in $(seq 1 80); do
  count=$(docker exec -u postgres "$source_name" sh -c 'find /archive -maxdepth 1 -type f | wc -l' | tr -d '[:space:]')
  last=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic \
    -c "SELECT coalesce(last_archived_wal,'') FROM pg_stat_archiver")
  if [[ "$count" -ge 1 && -n "$last" ]]; then
    archived=YES
    break
  fi
  sleep .25
done
[[ "$archived" == YES ]] || { echo 'WAL archive did not complete' >&2; exit 3; }
docker stop -t 10 "$source_name" >/dev/null

docker run --rm --user postgres \
  -v "$backup_volume:/restore" -v "$archive_volume:/archive:ro" "$image" sh -ceu '
cat >> /restore/postgresql.auto.conf <<EOF
restore_command = '"'"'cp /archive/%f %p'"'"'
recovery_target_name = '"'"'r988_before_bad_change'"'"'
recovery_target_action = '"'"'promote'"'"'
EOF
touch /restore/recovery.signal
'

docker run --rm -d --name "$restore_name" \
  --label ipat.test.disposable=r988 \
  -e PGDATA=/restore \
  -v "$backup_volume:/restore" \
  -v "$archive_volume:/archive:ro" \
  "$image" -c listen_addresses='' >/dev/null
[[ -z $(docker port "$restore_name" 2>/dev/null) ]] || { echo 'restore host port publishing forbidden' >&2; exit 3; }

ready=NO
for _ in $(seq 1 120); do
  if docker exec -u postgres "$restore_name" pg_isready -U postgres -d ipat_synthetic >/dev/null 2>&1; then
    recovering=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic -c 'select pg_is_in_recovery()' 2>/dev/null || true)
    if [[ "$recovering" == f ]]; then
      ready=YES
      break
    fi
  fi
  sleep .25
done
[[ "$ready" == YES ]] || {
  docker logs "$restore_name" >&2 || true
  echo 'PITR restore did not promote at named restore point' >&2
  exit 3
}

restored_count=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic -c "SELECT count(*) FROM ipat_ops.devices" | tr -d '[:space:]')
restored_tenant=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic -c "SELECT tenant_id FROM ipat_ops.devices WHERE id='$good'" | tr -d '[:space:]')
[[ "$restored_count" == 1 && "$restored_tenant" == "$tenant" ]] || { echo 'restored prepoint row identity mismatch' >&2; exit 1; }
restored_hash=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic -c "$query" | sha256sum | awk '{print $1}')
[[ "$restored_hash" == "$good_hash" ]] || { echo 'pre-restore-point data hash mismatch' >&2; exit 1; }
bad_count=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT count(*) FROM ipat_ops.devices WHERE id='$bad'" | tr -d '[:space:]')
[[ "$bad_count" == 0 ]] || { echo 'post-restore-point synthetic bad row survived PITR' >&2; exit 1; }

unscoped=$(docker exec -u postgres "$restore_name" psql -XAtq -U ipat_app_runtime -d ipat_synthetic \
  -c "SELECT count(*) FROM ipat_ops.devices" | tail -1 | tr -d '[:space:]')
[[ "$unscoped" == 0 ]] || { echo 'restored unscoped runtime unexpectedly sees rows' >&2; exit 1; }
scoped=$(docker exec -u postgres "$restore_name" psql -XAtq -U ipat_app_runtime -d ipat_synthetic \
  -c "BEGIN; SET LOCAL ipat.tenant_id='$tenant'; SELECT count(*) FROM ipat_ops.devices; COMMIT" \
  | grep -E '^[0-9]+$' | tail -1)
[[ "$scoped" == 1 ]] || { echo 'restored tenant scoped runtime row count mismatch' >&2; exit 1; }

echo "R988_POSTGRES18_BASEBACKUP_VERIFY_WAL_NAMED_POINT_PITR=PASS"
echo "R988_RESTORED_PREPOINT_SHA256=$restored_hash"
echo "R988_POSTPOINT_BAD_ROW_COUNT=$bad_count"
echo "R988_FORCE_RLS_UNSCOPED=$unscoped SCOPED=$scoped"
echo "NOT_OFFSITE_NOT_HA_NOT_PRODUCTION_RPO_RTO"
