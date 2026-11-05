#!/usr/bin/env bash
# Read-only metadata/role audit on actual PG18. No tenants, session secrets or writes.
set -Eeuo pipefail
umask 077
die(){ echo "R1050_READONLY_REFUSED: $*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} -eq 0 ]] || die 'privileged operator required for restricted peer identity'
[[ ${IPAT_R1050_READ_ONLY:-} == EXACT_PRODUCTION_ROLE_AUDIT ]] || die 'explicit read-only opt-in required'
for x in pg_lsclusters runuser psql timeout; do command -v "$x" >/dev/null || die "missing $x"; done
[[ $(pg_lsclusters --no-header 2>/dev/null | awk '$1=="18" && $2=="ipat" && $4=="online"{n++}END{print n+0}') == 1 ]] || die 'expected PG18 ipat cluster not online'
query="BEGIN READ ONLY;
SELECT current_user = 'ipat_tenant_api_login' AS peer_role,
  NOT EXISTS (
    SELECT 1 FROM pg_roles WHERE rolname = current_user
      AND (rolsuper OR rolbypassrls OR rolcreaterole)
  ) AS role_least_privilege,
  NOT has_table_privilege(current_user,'ipat_platform.tenants','SELECT') AS raw_tenants_denied;
SELECT COUNT(*) = 2 AND COUNT(*) FILTER (WHERE relrowsecurity AND relforcerowsecurity) =
       2 AS all_target_relations_force_rls
 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname='ipat_ops' AND c.relname IN ('devices','subscribers');
COMMIT;"
# No tenant records are selected; catalog/role metadata only.
result=$(timeout 12s runuser -u ipattapi -- psql -X -w -h /run/postgresql -U ipat_tenant_api_login -d ipat_prod -v ON_ERROR_STOP=1 -Atq -c "$query") || die 'restricted peer audit failed'
# psql emits two boolean result rows, plus no secret data.
[[ "$result" == $'t|t|t\nt' ]] || die 'production role/RLS catalogue audit mismatch'
echo R1050_ACTUAL_PG18_ROLE_AND_RLS_METADATA_READONLY_PASS
echo R1050_CROSS_TENANT_RUNTIME_LOGIN_AND_BROWSER=NOT_TESTED
