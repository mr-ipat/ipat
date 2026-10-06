#!/usr/bin/env bash
# Same isolated no-published-port R9.68 pre-requisites + new R9.72 exact NOC POP scope permissions.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R980_SYNTHETIC_DOCKER:-} == YES ]] || { echo 'Requires explicit disposable-only opt-in' >&2; exit 2; }
command -v docker >/dev/null && command -v python3 >/dev/null || exit 2
[[ -z ${PGSERVICE:-} && -z ${PGSERVICEFILE:-} ]] || exit 2
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
name="ipat-r980-disposable-$RANDOM-$$";tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r980-XXXXXX")
cleanup() { docker rm -f "$name" >/dev/null 2>&1 || :; rm -rf "$tmp"; }; trap cleanup EXIT
docker run --rm -d --name "$name" --label ipat.test.disposable=r980 -e POSTGRES_PASSWORD=local_ci_synthetic_only -e POSTGRES_DB=ipat_synthetic -v "$root:/src:ro" postgres:16-alpine >/dev/null
ready=NO
for _ in {1..50};do if docker exec "$name" pg_isready -h 127.0.0.1 -U postgres -d ipat_synthetic >/dev/null 2>&1;then ready=YES; break; fi;sleep .2;done
[[ "$ready" == YES ]] || exit 3
cat > "$tmp/psql" <<'WRAP'
#!/usr/bin/env bash
set -Eeuo pipefail
[[ ${IPAT_PG_EPHEMERAL_TEST:-} == 1 && ${PGDATABASE:-} == ipat_synthetic ]] || exit 3
exec docker exec -e PGPASSWORD=local_ci_synthetic_only "$IPAT_R980_CONTAINER_NAME" psql -h 127.0.0.1 -U postgres -d ipat_synthetic "$@"
WRAP
chmod 0700 "$tmp/psql"
export PATH="$tmp:$PATH" IPAT_R980_CONTAINER_NAME="$name" IPAT_PG_EPHEMERAL_TEST=1 PGHOST=127.0.0.1 PGDATABASE=ipat_synthetic PGUSER=postgres IPAT_PG_SYNTHETIC_PASSWORD=local_ci_synthetic_only
for m in 0001_lab_tenant_rls.sql 0003_lab_identity_memberships.sql 0004_lab_scoped_identity_lookup.sql 0012_tenant_domains.sql 0013_tenant_domain_enrollment.sql 0014_tenant_domain_activation_lifecycle.sql 0016_managed_device_registry.sql 0017_operator_firmware_workflow.sql 0018_tenant_sites_staged.sql 0019_tenant_sites_validate_fk.sql 0020_managed_device_metadata_crud.sql 0021_durable_tenant_browser_sessions.sql 0022_commercial_tenant_api_roles.sql 0023_tenant_menu_entitlements.sql 0024_oidc_one_time_pending.sql 0025_commercial_noc_scopes.sql 0026_platform_tenant_onboarding.sql 0027_vendor_transport_catalog.sql 0028_independent_pop_master.sql 0029_validate_tenant_pop_site_fk.sql 0030_typed_noc_real_pop_scope.sql 0031_noc_real_pop_grant_lifecycle.sql 0032_platform_owner_browser_sessions.sql 0033_tenant_domain_ingress_verifier.sql;do
 psql -X -q -v ON_ERROR_STOP=1 -f "/src/deploy/db/migrations/$m" >/dev/null
done
python3 -m unittest "$root/deploy/db/tests/test_commercial_tenant_api_roles_integration.py" "$root/deploy/db/tests/test_tenant_menu_entitlements_integration.py" "$root/deploy/db/tests/test_noc_scope_integration.py" "$root/deploy/db/tests/test_platform_tenant_onboarding_integration.py" "$root/deploy/db/tests/test_vendor_transport_catalog_integration.py" "$root/deploy/db/tests/test_independent_pop_master_integration.py" "$root/deploy/db/tests/test_typed_noc_real_pop_scope_integration.py" "$root/deploy/db/tests/test_noc_real_pop_grant_lifecycle_integration.py" "$root/deploy/db/tests/test_platform_owner_browser_sessions_integration.py" "$root/deploy/db/tests/test_tenant_domain_ingress_verifier_integration.py" -v
echo R980_DISPOSABLE_PG16_PLATFORM_OWNER_SESSIONS=PASS
