#!/usr/bin/env bash
# Real VPS PostgreSQL18 test for two synthetic tenants, disposable cluster only.
set -Eeuo pipefail
umask 077
abort(){ echo "R1053_REFUSED: $*" >&2; exit 4; }
[[ $(id -u) -ne 0 ]] || abort "root forbidden"
[[ $(printenv IPAT_R1053_EXACT_DISPOSABLE || :) == YES ]] || abort "explicit opt-in required"
for k in PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD PGSERVICE PGSERVICEFILE DOCKER_HOST KUBECONFIG;do
 [[ -z $(printenv "$k" || :) ]] || abort "inherited environment key: $k"
done
bin=/usr/lib/postgresql/18/bin
for x in initdb pg_ctl postgres;do [[ -x $bin/$x ]] || abort "PG18 $x missing";done
for x in psql pg_dump pg_restore createdb ss python3 git;do command -v "$x" >/dev/null || abort "missing $x";done
[[ -z $(ss -H -ltn '( sport = :55482 )') ]] || abort "disposable test port occupied"
root=$(pwd -P)
expected=$(printenv IPAT_R1053_EXPECTED_COMMIT || :)
[[ $expected =~ ^[0-9a-f]{40}$ ]] || abort "exact reviewed commit not supplied"
[[ $(git rev-parse HEAD) == "$expected" ]] || abort "source commit mismatch"
[[ -z $(git status --porcelain) ]] || abort "unreviewed source changes"
[[ -f "$root/deploy/db/migrations/0001_lab_tenant_rls.sql" ]] || abort "canonical schema missing"
work=$(mktemp -d "$HOME/.local/share/ipat/r1053-pg18-disposable-XXXXXXXX")
armed=no
cleanup(){
 if [[ $armed == yes ]];then "$bin/pg_ctl" -D "$work/data" -m immediate stop >/dev/null 2>&1 || :;fi
 rm -rf -- "$work"
}
trap cleanup EXIT
mkdir -m 0700 "$work/sock"
"$bin/initdb" -D "$work/data" -U postgres --encoding=UTF8 --locale=C.UTF-8 --auth-local=trust --auth-host=trust --no-instructions >"$work/init.log" 2>&1 || abort "initdb failed"
"$bin/pg_ctl" -D "$work/data" -o "-h 127.0.0.1 -p 55482 -k $work/sock -c unix_socket_permissions=0700" -l "$work/pg.log" start >/dev/null 2>&1 || abort "disposable PostgreSQL start failed"
armed=yes
export PGHOST=127.0.0.1 PGPORT=55482 PGUSER=postgres PGDATABASE=ipat_synthetic
export IPAT_PG_EPHEMERAL_TEST=1 IPAT_PG_SYNTHETIC_PASSWORD=local_ci_synthetic_only
[[ $(psql -d postgres -XAtqc 'SHOW server_version_num') == 18???? ]] || abort "wrong PostgreSQL version"
[[ $(psql -d postgres -XAtqc 'SHOW data_directory') == "$work/data" ]] || abort "wrong data directory"
[[ $(psql -d postgres -XAtqc 'SHOW port') == 55482 ]] || abort "wrong port"
createdb ipat_synthetic
python3 -m unittest discover -s "$root/deploy/db/tests" -p test_postgres_rls_integration.py -v
echo R1053_REAL_VPS_PG18_TWO_SYNTHETIC_TENANTS_RLS_PASS
# Add ONLY canonical schema migrations in their reviewed dependency order.
for m in 0003_lab_identity_memberships.sql 0004_lab_scoped_identity_lookup.sql \
  0012_tenant_domains.sql 0013_tenant_domain_enrollment.sql \
  0014_tenant_domain_activation_lifecycle.sql 0016_managed_device_registry.sql \
  0017_operator_firmware_workflow.sql 0018_tenant_sites_staged.sql \
  0019_tenant_sites_validate_fk.sql 0020_managed_device_metadata_crud.sql \
  0021_durable_tenant_browser_sessions.sql 0022_commercial_tenant_api_roles.sql \
  0023_tenant_menu_entitlements.sql 0024_oidc_one_time_pending.sql \
  0025_commercial_noc_scopes.sql 0026_platform_tenant_onboarding.sql \
  0027_vendor_transport_catalog.sql 0028_independent_pop_master.sql \
  0029_validate_tenant_pop_site_fk.sql 0030_typed_noc_real_pop_scope.sql \
  0031_noc_real_pop_grant_lifecycle.sql 0042_tenant_safe_branding.sql; do
  psql -X -q -v ON_ERROR_STOP=1 -f "$root/deploy/db/migrations/$m" >/dev/null
done
[[ -x /home/openai/.cargo/bin/cargo ]] || abort "offline Rust cargo missing"
export CARGO_TARGET_DIR="$work/rust-target" CARGO_BUILD_JOBS=8
/home/openai/.cargo/bin/cargo test --locked --offline -p control-api \
  commercial_tenant_api::pg_integration::r970_real_pg_tenant_dashboard_current_role_catalog_and_logout -- --exact
# The existing R9.70 CI test now invokes the R10.53 two-company matrix.
echo R1053_VPS_REAL_POSTGRES18_TWO_COMPANY_AXUM_HOST_COOKIE_NEGATIVE_MATRIX_PASS
