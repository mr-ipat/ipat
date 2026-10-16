# R10.40 — Product Owner accepts untested-VM-restore risk for local production foundation only

**Decision date:** 2026-10-17 WIB. **Public production release: not approved.** The product owner explicitly asked on 2026-10-17 to consider the recovery phase complete and prioritize the remaining PRD and production implementation. That is recorded here as a bounded **risk acceptance**, not as proof a separately booted VM/VPS restoration succeeded. No independent boot test was executed.

## Verified evidence and residual risk

The owner Mac holds encrypted Restic snapshots rootfs `2d767d09` (31,759,953,920 bytes logical archive) and EFI `af1a0129` (9,000,960 bytes). Both were independently read from the encrypted repository, `restic check --read-data` exited 0 and `restic dump | tar -tf -` returned 0 for both archives (R10.39, merged #237). Original reviewed binary/source from R10.37 remain controlled by root. This does **not** demonstrate bootable recovery, ownership/ACL reconstruction on new storage, successful provider rescue login, or recovery RPO/RTO.

Risk acknowledged: a VPS failure after first-install may be unrecoverable or take longer than the business tolerates. The risk exception is limited to a **local Unix-socket PostgreSQL18 first-install with application and public edge disabled**. The product owner cannot truthfully mark independent VM restore verified without an actual test.

## Enforced alternate recovery *input*, not a fake recovery attestation

The R9.96 installer still accepts its original strictly validated, root-owned real recovery attestation. R10.40 adds a distinct root-only `ipat.owner-risk-acceptance.v1` input, bound to exact source commit, encrypted backup snapshot references and actual pack/tar readability evidence. Fields require:

- An unexpired (24h) unique product-owner approval, exact typed statement `ACCEPT_UNTESTED_INDEPENDENT_BOOT_RESTORE_RISK_FOR_LOCAL_DB_FOUNDATION_ONLY`.
- `independent_full_host_boot_restore_verified=false`, `public_edge_activation_authorized=false` and `production_pitr_ha_verified=false`.
- Immutable snapshot IDs `2d767d09` / `af1a0129` and explicit reviewed archive-integrity flags.
- Root-owned mode 0600, single-link regular-file inode, trusted root-only ancestor directory and a no-symlink, no-TOCTOU bounded FD read.

Invocation requires `IPAT_R996_RECOVERY_ATTESTATION` and `IPAT_R1040_OWNER_EXCEPTION` to be the **same** root-owned waiver path and separately sets `IPAT_R1040_ACCEPT_LOCAL_DB_RISK` to the exact owner phrase. Both opt-ins are required, alongside existing R9.96 explicit foundation opt-in and exact hash inputs. If missing or invalid, first-install fails before changing a service.

The resulting foundation marker records `recovery_gate=OWNER_ACCEPTED_UNTESTED_BOOT_RESTORE_LOCAL_ONLY`, `independent_full_host_restore_verified=false`, and `public_go=false`. R9.97 PITR/public-edge validator explicitly **refuses** any such marker regardless of PostgreSQL backup results. It can only be replaced through a separately reviewed controlled recovery/migration path after actual independent restore evidence. The optional exception is **not** automatically convertible to public readiness.

## Acceptance and rollback

R10.40 unit tests cover honest unverified-state markers, invalid approval, stale expiry, original-recovery path retention, privilege guard, bad ownership, symlinks and hardlinks, duplicate keys and malicious public activation. The existing CI suite executes them, plus unchanged original R9.96/R9.97 acceptance tests. A production run must inspect exact-source root checkout and binary checksum, unused runtime identities and ports, and execute R9.96 with automatic rollback after backup/access readiness checks.

**MUST requirements still open:** provider rescue/second VM recovery, production PostgreSQL PITR/HA and failover, actual human Platform Owner MFA, trusted certificate/custom Host routing, two real tenants with API-level isolation, physical ACS/USP/OLT/ONT/MikroTik interop and approved device writes. No commercial customer release is authorized by R10.40.
