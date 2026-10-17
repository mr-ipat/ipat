# R10.42 — Actual successful local-only PostgreSQL18 production foundation on owner VPS

**Observed:** 2026-10-18, Asia/Jakarta. **Execution:** on real owner VPS running Ubuntu 26.04 (not Docker). **Scope:** local production database foundation with all platform/tenant services disabled and NO public ingress.

## Exact release

- Reviewed, merged main source: `279ab141ded8a93ddf4ea18f7b6b3482470625fc` (PR #240, GitHub CI 4/4 SUCCESS).
- Rust binary SHA256: `7e041f843668e77d46c89c2fb7cc61972b794af12f3b674beb284cba3ecf91cd` (no change to Rust binary in this shell-only migration fix).
- Root-owned reviewed source `/root/ipat-r1042-reviewed-src`, verified clean Git objects from a fresh user-owned clone of exact source and privileged `git fsck`.
- Root-owned binary `/root/ipat-r1042-reviewed-bin/control-api`; new root-controlled owner risk acceptance `/root/ipat-evidence/r1042-owner-recovery-risk.json` bound to the exact release, validated by existing root-only R10.40 risk validator.
- Original wrapper initially refused to overwrite existing remnants. Inspected failures and quarantined only confirmed empty `/etc/ipat/tenant-oidc` remnant under root evidence `/root/ipat-evidence/r1042-aborted-empty-oidc-20261018`. No persistent customer data deleted.

## Actual successful installation

Executed the reviewed R9.96 production foundation orchestrator on the real VPS with strict `LANG=C.UTF-8`/`LC_ALL=C.UTF-8`, explicit first-install opt-in, exact source and binary digests, and **local-only owner-risk waiver**. Earlier failure while reading SQL in `/root` was corrected in R10.42: privileged root streams SHA256-verified migrations to the temporary `ipatpgmigrate` Unix-socket peer identity over stdin, with `pipefail` and SQL `ON_ERROR_STOP=1`. The temporary identity does not gain root directory traversal permissions.

Installer completed **exit code 0**:
- `R996_PRODUCTION_FOUNDATION_PREPARED_LOCAL_DB_ONLY_NO_PUBLIC_EDGE`
- `R1042_LOCAL_FOUNDATION_MARKER=PASS`
- `R1042_ROOT_LOCAL_DB_FOUNDATION_DONE_NO_PUBLIC_EDGE`

### Independently verified after installation

- `pg_lsclusters --no-header`: PostgreSQL **18/ipat online** on local socket.
- Six actual runtime OS identities present: `ipatpapi`, `ipatpoidc`, `ipattapi`, `ipattoidc`, `ipatdverify`, `ipatdnsverify`.
- Six independent `runuser -u <OS-user> -- psql -X -w -h /run/postgresql -U <restricted DB role> -d ipat_prod -Atqc 'select current_user'` checks PASS.
- Temporary `ipatpgmigrate` identity absent.
- `ss` confirms **no TCP listeners** on 80, 443, 5432, 3003, 3005, 3006.
- Root-owned `/var/lib/ipat/r996-foundation.json` is present and truthfully records `recovery_gate=OWNER_ACCEPTED_UNTESTED_BOOT_RESTORE_LOCAL_ONLY`, `independent_full_host_restore_verified=false`, `public_go=false`. This is not a forged disaster-recovery pass.
- Existing private C320 dashboard and connector remain active; no OLT/ONT/MikroTik write was issued.

## Unmet commercial acceptance (do not conceal)

The installed database is a **single-node local-only** foundation. There is no independently verified whole-host boot restore, production PostgreSQL PITR/HA, real Platform Owner and customer MFA login, trusted TLS/tenant Host-SNI ingress, cross-tenant authenticated browser acceptance or physical provisioning approval. R9.97 correctly prohibits opening the public edge from an R10.40 local-only waiver marker; do not bypass without an independently reviewed decision and implemented controls. Customer-facing production is **NOT GO**.

## Next implementation sequence

1. Establish independent PostgreSQL18 PITR and standby/HA with off-host backup destination and actual restore/failover receipts. Keep the verified local database without exposing TCP.
2. Enroll Platform Owner identity with human MFA; verify real browser OIDC, CSRF, backend role rejection, audit and two-tenant isolation over private connections.
3. Review remaining recovery/public-ingress risk policy with the Product Owner and Security Architect, then implement a valid production-readiness gate. R9.97 must not silently treat the local-only waiver as public approval.
4. Activate trusted HTTPS, scoped custom domains and tenant dashboards only after the conditions above. Follow with approved physical device-read/write interoperability and full PRD acceptance.
