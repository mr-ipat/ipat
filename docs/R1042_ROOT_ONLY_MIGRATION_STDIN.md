# R10.42 — Actual first-install failure: root-owned migrations and restricted OS account

**Date:** 2026-10-17 WIB. **Host:** owner Ubuntu Server 26.04. **Public GO:** FALSE.

## Verified root-cause and status

After fixing R10.41 Ubuntu locale and rollback, the second owner-approved local-only R9.96 first install passed root Git object/source hash, binary custody and all application unit preparation. PostgreSQL18 cluster initialization succeeded, but migrations immediately failed: `psql: error: /root/ipat-r1041-reviewed-src/deploy/db/migrations/0001_lab_tenant_rls.sql: Permission denied`. The temporary OS account `ipatpgmigrate` cannot traverse `/root` (mode 0700), as required by source protection. R10.41 fail-closed traps correctly aborted with exit code 4, removed new runtime identities and database, and did **not** issue a false success marker. Independent post-run Ubuntu checks found PG clusters 0, socket absent, all runtime users absent, services absent and both C320 private units ACTIVE.

This failure does not justify making `/root` world-readable, weakening filesystem modes, copying unverified SQL to an unprotected location, or running the migrations outside the audited R9.96 framework.

## Correction

The root R9.87 process already verifies the complete pinned SHA256 manifest and owns the immutable source tree. R10.42 makes it open each reviewed SQL file as root and stream it via a pipe to `runuser -u ipatpgmigrate -- psql ... -f -`. `set -Eeuo pipefail` and `ON_ERROR_STOP=1` require both the privileged read and restricted migrator execution to succeed. No temporary plaintext SQL files and no group/other read permission are created.

The disposable Ubuntu26 rehearsal now bind-mounts the reviewed repository under `/root/ipat-src` rather than `/src`, accurately reproducing the untraversable root-owned source location. Four source/negative tests enforce the stream, manifest ordering, no chmod weakening and no false-success on a failed pipe. This is an implementation and testing correction, not a PRD reduction.

## Retrying on actual VPS

After exact-head CI and fresh reviewed commit:
1. Root forensic inspection must confirm no clusters/users/public listeners; quarantine only orphan directories created by the R10.41 failed attempt, preserving invalid evidence.
2. Clone exact merged Git source into new root-only review directory with clean-tree/inode and digest controls, and create a fresh time-bound owner-risk document for *local foundation only*.
3. Execute R9.96 once and independently verify PostgreSQL18 cluster, migrated schema, Unix socket, six restricted peer-auth users, no plaintext passwords, no public listeners, and C320 health.
4. PostgreSQL HA/PITR, human MFA, trusted HTTPS and real tenant/customer/physical tests remain separate.

This note records a **source-level remediation**. It must not be represented as actual successful VPS PG18 production installation before the retry is observed.
