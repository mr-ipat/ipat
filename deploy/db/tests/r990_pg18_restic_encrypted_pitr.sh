#!/usr/bin/env bash
# R9.90: disposable PG18 physical backup+WAL -> encrypted Restic -> clean restore -> PITR.
# Local repository only proves encryption/recovery mechanics, NOT an actual offsite failure domain.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R990_RESTIC_PITR_DOCKER:-} == YES ]] || {
  echo 'Requires explicit disposable encrypted-PITR opt-in' >&2; exit 2; }
for cmd in docker restic sha256sum mktemp python3; do command -v "$cmd" >/dev/null || exit 2; done
root=$(cd "$(dirname "$0")/../../.." && pwd -P)
host_uid=$(id -u)
host_gid=$(id -g)
image='postgres:18-alpine@sha256:77f585114c32fbca283dc835b0596f4e52b51b4c6662d7810b2f4084f60a1873'
tag="r990-$RANDOM-$$"
source_name="ipat-$tag-source"
restore_name="ipat-$tag-restore"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r990-XXXXXX")
plain="$tmp/plain"
restored="$tmp/restored"
mkdir -p "$plain/base" "$plain/wal" "$restored"
password_file="$tmp/restic-password"
printf '%s\n' 'r990-local-synthetic-password-never-production' > "$password_file"
chmod 0600 "$password_file"
export RESTIC_REPOSITORY="$tmp/restic-repo"
export RESTIC_PASSWORD_FILE="$password_file"
cleanup(){
  docker rm -f "$source_name" "$restore_name" >/dev/null 2>&1 || :
  rm -rf "$tmp"
}
trap cleanup EXIT

docker run --rm -d --name "$source_name" \
  --label ipat.test.disposable=r990 \
  -e POSTGRES_PASSWORD=local_ci_synthetic_only \
  -e POSTGRES_DB=ipat_synthetic \
  -v "$root:/src:ro" \
  -v "$plain/base:/backup" \
  -v "$plain/wal:/archive" \
  "$image" \
  -c wal_level=replica \
  -c archive_mode=on \
  -c "archive_command=test ! -f /archive/%f && cp %p /archive/%f" \
  -c archive_timeout=5s >/dev/null

docker exec -u 0 "$source_name" sh -ceu 'chown postgres:postgres /backup /archive'
ready=NO
for _ in $(seq 1 80); do
  if docker exec -u postgres "$source_name" pg_isready -h 127.0.0.1 -U postgres -d ipat_synthetic >/dev/null 2>&1; then
    ready=YES; break
  fi
  sleep .25
done
[[ "$ready" == YES ]] || { echo 'source PostgreSQL18 did not start' >&2; exit 3; }
[[ -z $(docker port "$source_name" 2>/dev/null) ]] || { echo 'host DB port publishing forbidden' >&2; exit 3; }
version=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic -c 'show server_version_num')
[[ "$version" =~ ^18[0-9]{4}$ ]] || { echo "Expected PostgreSQL18, got $version" >&2; exit 3; }

while read -r sum file; do
  [[ -n "$sum" && -n "$file" ]] || continue
  actual=$(sha256sum "$root/$file" | awk '{print $1}')
  [[ "$actual" == "$sum" ]] || { echo "migration hash mismatch: $file" >&2; exit 3; }
  docker exec -u postgres "$source_name" psql -X -q -v ON_ERROR_STOP=1 \
    -U postgres -d ipat_synthetic -f "/src/$file" >/dev/null
done < "$root/deploy/db/production/r987_migrations.sha256"

tenant='90909090-9090-4090-8090-909090909091'
good='90909090-9090-4090-8090-909090909092'
bad='90909090-9090-4090-8090-909090909093'
docker exec -i -u postgres "$source_name" psql -Xq -v ON_ERROR_STOP=1 \
  -U postgres -d ipat_synthetic <<SQL
INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
VALUES('$tenant','r990-restic','active');
INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor,exact_model,firmware)
VALUES('$tenant','$good','R990-POP','olt','SYNTHETIC','GOOD-BEFORE-ENCRYPTED-PITR','synthetic-1');
SQL

docker exec -u postgres -e PGPASSWORD=local_ci_synthetic_only "$source_name" \
  pg_basebackup -h 127.0.0.1 -U postgres -D /backup -Fp -X stream -c fast >/dev/null
docker exec -u postgres "$source_name" pg_verifybackup /backup >/dev/null
point=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT pg_create_restore_point('r990_before_bad_change')")
[[ "$point" =~ ^[0-9A-F]+/[0-9A-F]+$ ]] || { echo 'restore point not created' >&2; exit 3; }

docker exec -i -u postgres "$source_name" psql -Xq -v ON_ERROR_STOP=1 \
  -U postgres -d ipat_synthetic <<SQL
INSERT INTO ipat_ops.devices(tenant_id,id,pop_id,device_kind,vendor,exact_model,firmware)
VALUES('$tenant','$bad','R990-POP','router','SYNTHETIC','BAD-AFTER-RESTORE-POINT','synthetic-bad');
SELECT pg_switch_wal();
SQL

archived=NO
for _ in $(seq 1 80); do
  wal_count=$(find "$plain/wal" -maxdepth 1 -type f 2>/dev/null | wc -l | tr -d '[:space:]')
  last=$(docker exec -u postgres "$source_name" psql -XAtq -U postgres -d ipat_synthetic \
    -c "SELECT coalesce(last_archived_wal,'') FROM pg_stat_archiver")
  if [[ "$wal_count" -ge 1 && -n "$last" ]]; then archived=YES; break; fi
  sleep .25
done
[[ "$archived" == YES ]] || { echo 'WAL archive did not complete' >&2; exit 3; }
docker stop -t 10 "$source_name" >/dev/null
# Linux bind-mount ownership follows the postgres container UID. Return the
# stopped disposable staging tree to the invoking runner before Restic/delete.
docker run --rm -u 0 -v "$plain:/plain" "$image" sh -ceu   "chown -R $host_uid:$host_gid /plain/base /plain/wal"

pre_backup_hash=$(cd "$plain" && find base wal -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | awk '{print $1}')
[[ -n "$pre_backup_hash" ]] || { echo 'backup artifact hash unavailable' >&2; exit 3; }

restic init >/dev/null
(
  cd "$plain"
  restic backup --tag ipat-r990-encrypted-pg18 base wal >/dev/null
)
restic check >/dev/null
snapshot=$(restic snapshots --tag ipat-r990-encrypted-pg18 --latest 1 --json | \
  python3 -c 'import json,sys; x=json.load(sys.stdin); print(x[0]["short_id"] if x else "")')
[[ -n "$snapshot" ]] || { echo 'encrypted snapshot missing' >&2; exit 3; }

rm -rf "$plain/base" "$plain/wal"
[[ ! -e "$plain/base" && ! -e "$plain/wal" ]] || { echo 'plaintext staging deletion failed' >&2; exit 3; }
restic restore latest --tag ipat-r990-encrypted-pg18 --target "$restored" >/dev/null
[[ -f "$restored/base/backup_manifest" ]] || { echo 'restored base backup manifest missing' >&2; exit 3; }
[[ -n $(find "$restored/wal" -maxdepth 1 -type f -print -quit) ]] || { echo 'restored WAL missing' >&2; exit 3; }

post_restore_hash=$(cd "$restored" && find base wal -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | awk '{print $1}')
[[ "$post_restore_hash" == "$pre_backup_hash" ]] || { echo 'encrypted restore artifact hash mismatch' >&2; exit 1; }

docker run --rm --user postgres -v "$restored/base:/verify:ro" "$image" pg_verifybackup /verify >/dev/null

docker run --rm -u 0 -v "$restored/base:/restore" -v "$restored/wal:/archive:ro" "$image" sh -ceu '
chown -R postgres:postgres /restore
cat >> /restore/postgresql.auto.conf <<EOF
restore_command = '"'"'cp /archive/%f %p'"'"'
recovery_target_name = '"'"'r990_before_bad_change'"'"'
recovery_target_action = '"'"'promote'"'"'
EOF
touch /restore/recovery.signal
chown postgres:postgres /restore/postgresql.auto.conf /restore/recovery.signal
'
docker run --rm -d --name "$restore_name" \
  --label ipat.test.disposable=r990 \
  -e PGDATA=/restore \
  -v "$restored/base:/restore" \
  -v "$restored/wal:/archive:ro" \
  "$image" -c listen_addresses='' >/dev/null
[[ -z $(docker port "$restore_name" 2>/dev/null) ]] || { echo 'restore DB host port publishing forbidden' >&2; exit 3; }

ready=NO
for _ in $(seq 1 120); do
  state=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic \
    -c "SELECT CASE WHEN pg_is_in_recovery() THEN '1' ELSE '0' END" 2>/dev/null || true)
  if [[ "$state" == 0 ]]; then ready=YES; break; fi
  sleep .25
done
[[ "$ready" == YES ]] || { docker logs "$restore_name" >&2 || true; echo 'encrypted PITR did not promote' >&2; exit 3; }

good_count=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT count(*) FROM ipat_ops.devices WHERE id='$good'" | tr -d '[:space:]')
bad_count=$(docker exec -u postgres "$restore_name" psql -XAtq -U postgres -d ipat_synthetic \
  -c "SELECT count(*) FROM ipat_ops.devices WHERE id='$bad'" | tr -d '[:space:]')
[[ "$good_count" == 1 && "$bad_count" == 0 ]] || { echo 'encrypted PITR row boundary mismatch' >&2; exit 1; }
unscoped=$(docker exec -u postgres "$restore_name" psql -XAtq -U ipat_app_runtime -d ipat_synthetic \
  -c 'SELECT count(*) FROM ipat_ops.devices' | tr -d '[:space:]')
scoped=$(docker exec -u postgres "$restore_name" psql -XAtq -U ipat_app_runtime -d ipat_synthetic \
  -c "BEGIN; SET LOCAL ipat.tenant_id='$tenant'; SELECT count(*) FROM ipat_ops.devices; COMMIT" \
  | grep -E '^[0-9]+$' | tail -1)
[[ "$unscoped" == 0 && "$scoped" == 1 ]] || { echo 'restored FORCE-RLS mismatch' >&2; exit 1; }

echo R990_RESTIC_ENCRYPTED_PG18_BASE_WAL_PITR=PASS
echo "R990_RESTIC_SNAPSHOT=$snapshot"
echo "R990_ARTIFACT_SHA256=$post_restore_hash"
echo "R990_BAD_ROW=$bad_count FORCE_RLS_UNSCOPED=$unscoped SCOPED=$scoped"
echo LOCAL_RESTIC_REPOSITORY_ONLY_NOT_ACTUAL_OFFSITE_NOT_PRODUCTION_RPO_RTO
