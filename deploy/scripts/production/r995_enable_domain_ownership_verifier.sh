#!/usr/bin/env bash
# R9.95 enable prepared DNS ownership verifier after local PostgreSQL peer mapping.
set -Eeuo pipefail
umask 077

die(){ echo "R995_ENABLE_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R995_ENABLE:-} == ENABLE_REVIEWED_DOMAIN_OWNERSHIP_VERIFIER ]] || die "explicit enable opt-in missing"

db=${IPAT_R995_DB_NAME:-}
sock=${IPAT_R995_DB_SOCKET:-}
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock =~ ^/[A-Za-z0-9_./-]{1,180}$ && $sock != *..* && -d $sock && ! -L $sock ]] || die "unsafe PostgreSQL target"
for c in install getent runuser psql systemctl systemd-analyze; do command -v "$c" >/dev/null || die "missing tool: $c"; done
getent passwd ipatdnsverify >/dev/null || die "ownership verifier identity not prepared"
[[ -x /opt/ipat/bin/control-api && ! -L /opt/ipat/bin/control-api ]] || die "control-api runtime missing"
unit=/etc/systemd/system/ipat-domain-ownership-verifier.service
[[ -f $unit && ! -L $unit ]] || die "ownership-verifier unit missing"

role=$(runuser -u ipatdnsverify -- psql -X -w -h "$sock" -d "$db"   -U ipat_domain_verifier_login -Atqc 'select current_user' 2>/dev/null || true)
[[ $role == ipat_domain_verifier_login ]] || die "ownership verifier peer auth failed"
raw=$(runuser -u ipatdnsverify -- psql -X -w -h "$sock" -d "$db"   -U ipat_domain_verifier_login -Atqc   "select has_table_privilege(current_user,'ipat_platform.tenant_domains','SELECT')::int" 2>/dev/null || true)
[[ $raw == 0 ]] || die "ownership verifier unexpectedly has raw tenant-domain access"
broad=$(runuser -u ipatdnsverify -- psql -X -w -h "$sock" -d "$db"   -U ipat_domain_verifier_login -Atqc   "select has_function_privilege(current_user,'ipat_platform.record_tenant_domain_check(uuid,text,text,text)','EXECUTE')::int" 2>/dev/null || true)
[[ $broad == 0 ]] || die "ownership verifier still has broad lifecycle authority"

systemd-analyze verify "$unit" >/dev/null
dir=/var/lib/ipat-domain-ownership-verifier
[[ ! -e $dir && ! -L $dir ]] || die "existing ownership-verifier config requires upgrade review"
install -d -o ipatdnsverify -g ipatdnsverify -m 0700 "$dir"
tmp=$(mktemp "$dir/postgres.conninfo.pending.XXXXXX")
rollback(){
  systemctl disable --now ipat-domain-ownership-verifier.service >/dev/null 2>&1 || true
  rm -rf "$dir"
}
trap rollback ERR INT TERM
printf 'host=%s dbname=%s user=ipat_domain_verifier_login\n' "$sock" "$db" >"$tmp"
chown ipatdnsverify:ipatdnsverify "$tmp"
chmod 0600 "$tmp"
mv "$tmp" "$dir/postgres.conninfo"
chown ipatdnsverify:ipatdnsverify "$dir/postgres.conninfo"
chmod 0600 "$dir/postgres.conninfo"
systemctl daemon-reload
systemctl enable --now ipat-domain-ownership-verifier.service >/dev/null
systemctl is-enabled --quiet ipat-domain-ownership-verifier.service
systemctl is-active --quiet ipat-domain-ownership-verifier.service
trap - ERR INT TERM
echo R995_DOMAIN_OWNERSHIP_VERIFIER_ENABLED
