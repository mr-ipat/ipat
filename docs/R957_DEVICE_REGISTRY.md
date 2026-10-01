# R9.57 — Production Device Registry implementation and gates

**Scope:** additive source-only metadata + restricted DB functions + an unmounted Rust BFF, no production traffic or physical device commands.

## Files
- `deploy/db/migrations/0016_managed_device_registry.sql`: additive table, audit, sealed Save/List, roles, FORCE RLS. Must apply AFTER canonical 0001..0015 in order, under a reviewed migration account. Never run on a customer DB without verified backups and authorized maintenance.
- `deploy/db/tests/test_managed_device_registry_integration.py`: synthetic-only disposable PostgreSQL test; CI runs after other baseline migration fixtures in one throwaway PostgreSQL 16 database.
- `apps/control-api/src/managed_device_bff.rs`: session/origin/CSRF-guarded transport-neutral Save/List. No public route; cannot perform adoption.
- `.github/workflows/ci.yml`: targeted PostgreSQL negative tests.

## How to test safely
1. On a disposable PostgreSQL 16 instance named **exactly** `ipat_synthetic`, run prior ordered CI integration tests that apply 0001..0015; synthetic fixtures are intentionally distinct from real customers. See the existing `postgres-rls-restore` CI job for the exact full command sequence and environment safety assertions.
2. Set only synthetic CI password `local_ci_synthetic_only`, `PGHOST=127.0.0.1`, `PGDATABASE=ipat_synthetic`, `IPAT_PG_EPHEMERAL_TEST=1`; run `python3 -m unittest discover deploy/db/tests -p test_managed_device_registry_integration.py -v`.
3. Run `cargo fmt --all -- --check` and `cargo test --locked -p control-api managed_device_bff` on the pinned Ubuntu 26.04 Rust toolchain; then full locked workspace CI.
4. Expect direct DB read/write to fail, inactive/revoked identity denial, same-tenant Vault reference only, retry to return existing ID and `SAVED` to remain unconnected.

## Unfinished acceptance gates
- Genuine customer HTTPS + Host→tenant comparison + MFA OIDC authorization + horizontally shared opaque session store.
- Production migration on backed-up PostgreSQL with HA/PITR plan; tenant-verified Add/List HTTP/UI and exact backend denial tests.
- Secret vault provision/read ACL and read-only network adapter worker with fresh model/firmware/capability evidence and per-device locking.
- C320 physical onboarding through new product path, RouterOS API-SSL verified capability discovery, original CWMP and native USP tenant-bound persistence.

No device action, firewall, certificate, DNS, K3s or live PostgreSQL modification is authorized by this milestone.
