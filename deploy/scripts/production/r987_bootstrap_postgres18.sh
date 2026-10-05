#!/usr/bin/env bash
# R9.87 first-install ONLY local-socket PostgreSQL18 bootstrap.
# No TCP listener, no runtime passwords, no firewall/DNS/device mutation.
set -Eeuo pipefail
umask 077
die(){ echo "R987_BOOTSTRAP_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R987_BOOTSTRAP:-} == CREATE_REVIEWED_LOCAL_POSTGRES18 ]] || die "explicit reviewed opt-in missing"
root=${IPAT_R987_SOURCE_ROOT:-}
[[ -n $root && $root == /* && -d $root && ! -L $root ]] || die "absolute reviewed source root required"
manifest="$root/deploy/db/production/r987_migrations.sha256"
[[ -f $manifest && ! -L $manifest ]] || die "migration manifest missing"
for cmd in apt-get apt-cache sha256sum runuser useradd userdel getent install stat ss; do
  command -v "$cmd" >/dev/null || die "missing base tool: $cmd"
done
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || die "Ubuntu 26.04 required"
getent passwd ipatpapi >/dev/null || die "R9.85 ipatpapi user required first"
getent passwd ipatpoidc >/dev/null || die "R9.85 ipatpoidc user required first"
! getent passwd ipatpgmigrate >/dev/null || die "temporary migrator OS user already exists"
if command -v pg_lsclusters >/dev/null; then
  [[ -z $(pg_lsclusters --no-header 2>/dev/null || true) ]] || die "existing PostgreSQL cluster requires separate migration review"
fi

(
  cd "$root"
  sha256sum -c deploy/db/production/r987_migrations.sha256 >/dev/null
) || die "migration SHA256 manifest mismatch"

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
candidate=$(apt-cache policy postgresql-18 | sed -n 's/^[[:space:]]*Candidate:[[:space:]]*//p' | head -1)
[[ $candidate == 18.* ]] || die "Ubuntu PostgreSQL18 candidate unavailable"
apt-get install -y --no-install-recommends postgresql-common postgresql-client-common >/dev/null
for cmd in pg_createcluster pg_dropcluster pg_ctlcluster pg_lsclusters psql; do
  command -v "$cmd" >/dev/null || die "missing PostgreSQL tool after postgresql-common install: $cmd"
done
[[ -z $(pg_lsclusters --no-header 2>/dev/null || true) ]] || die "existing PostgreSQL cluster requires separate migration review"
cfg=/etc/postgresql-common/createcluster.conf
bak=/run/ipat-r987-createcluster.conf.backup
had_cfg=NO
if [[ -e $cfg ]]; then
  [[ -f $cfg && ! -L $cfg ]] || die "unsafe postgresql-common createcluster config"
  cp -a "$cfg" "$bak"
  had_cfg=YES
fi
printf 'create_main_cluster = false\n' > "$cfg"
restore_cfg(){
  if [[ $had_cfg == YES ]]; then mv -f "$bak" "$cfg"; else rm -f "$cfg"; fi
}
trap restore_cfg EXIT
apt-get install -y --no-install-recommends postgresql-18 postgresql-client-18 >/dev/null
restore_cfg
trap - EXIT
[[ -z $(pg_lsclusters --no-header 2>/dev/null || true) ]] || die "package install unexpectedly created a cluster"

pg_createcluster 18 ipat --start-conf=auto --port=5432 -- --auth-local=peer --auth-host=reject >/dev/null
created=YES
rollback(){
  if [[ ${created:-NO} == YES ]]; then pg_dropcluster --stop 18 ipat >/dev/null 2>&1 || true; fi
  if getent passwd ipatpgmigrate >/dev/null; then userdel ipatpgmigrate || true; fi
}
trap rollback ERR INT TERM
conf=/etc/postgresql/18/ipat
[[ -d $conf && ! -L $conf ]] || die "cluster config missing"
cat >> "$conf/postgresql.conf" <<'CFG'

# IPAT R9.87 local-primary staging boundary.
listen_addresses = ''
unix_socket_directories = '/run/postgresql'
ssl = off
password_encryption = 'scram-sha-256'
log_connections = on
log_disconnections = on
log_line_prefix = '%m [%p] db=%d user=%u app=%a '
wal_level = replica
max_wal_senders = 5
CFG
cat > "$conf/pg_ident.conf" <<'IDENT'
ipat_bootstrap  ipatpgmigrate  postgres
ipat_bootstrap  postgres       postgres
ipat_runtime    ipatpapi       ipat_platform_session_api_login
ipat_runtime    ipatpoidc      ipat_platform_session_issuer_login
IDENT
cat > "$conf/pg_hba.conf" <<'HBA'
local   all        postgres                            peer map=ipat_bootstrap
local   ipat_prod  ipat_platform_session_api_login    peer map=ipat_runtime
local   ipat_prod  ipat_platform_session_issuer_login peer map=ipat_runtime
local   all        all                                 reject
HBA

useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin ipatpgmigrate
pg_ctlcluster --skip-systemctl-redirect 18 ipat start
ready=NO
for _ in {1..30}; do
  if runuser -u postgres -- psql -X -Atqc 'select 1' postgres >/dev/null 2>&1; then ready=YES; break; fi
  sleep .2
done
[[ $ready == YES ]] || die "cluster did not become locally ready"
runuser -u ipatpgmigrate -- psql -X -U postgres -d postgres -v ON_ERROR_STOP=1 -q -c 'CREATE DATABASE ipat_prod'

while read -r sum file; do
  [[ -n ${sum:-} && -n ${file:-} ]] || continue
  runuser -u ipatpgmigrate -- psql -X -U postgres -d ipat_prod -v ON_ERROR_STOP=1 -q -f "$root/$file"
done < "$manifest"

# Replace bootstrap superuser mapping with final runtime-only auth.
cat > "$conf/pg_ident.conf.final" <<'IDENT'
ipat_runtime    ipatpapi       ipat_platform_session_api_login
ipat_runtime    ipatpoidc      ipat_platform_session_issuer_login
IDENT
install -o postgres -g postgres -m 0640 "$conf/pg_ident.conf.final" "$conf/pg_ident.conf"
rm -f "$conf/pg_ident.conf.final"
cat > "$conf/pg_hba.conf.final" <<'HBA'
local   all        postgres                            peer
local   ipat_prod  ipat_platform_session_api_login    peer map=ipat_runtime
local   ipat_prod  ipat_platform_session_issuer_login peer map=ipat_runtime
local   all        all                                 reject
HBA
install -o postgres -g postgres -m 0640 "$conf/pg_hba.conf.final" "$conf/pg_hba.conf"
rm -f "$conf/pg_hba.conf.final"
userdel ipatpgmigrate
pg_ctlcluster --skip-systemctl-redirect 18 ipat reload
! getent passwd ipatpgmigrate >/dev/null || die "temporary migration OS identity removal failed"
[[ -z $(ss -H -ltn '( sport = :5432 )' 2>/dev/null || true) ]] || die "TCP PostgreSQL listener forbidden"

api=$(runuser -u ipatpapi -- psql -X -w -h /run/postgresql -U ipat_platform_session_api_login -d ipat_prod -Atqc 'select current_user' 2>/dev/null || true)
oidc=$(runuser -u ipatpoidc -- psql -X -w -h /run/postgresql -U ipat_platform_session_issuer_login -d ipat_prod -Atqc 'select current_user' 2>/dev/null || true)
[[ $api == ipat_platform_session_api_login ]] || die "restricted Platform API peer auth failed"
[[ $oidc == ipat_platform_session_issuer_login ]] || die "restricted Platform OIDC peer auth failed"
created=NO
trap - ERR INT TERM
echo R987_POSTGRES18_LOCAL_SOCKET_SCHEMA_READY_NOT_HA_NOT_PITR
