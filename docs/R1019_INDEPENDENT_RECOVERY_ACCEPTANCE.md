# R10.19 — Actual sudo proof; recovery before production

Date: 2026-10-12 (Asia/Jakarta). This is an operational acceptance guide, **not** a public release approval.

## Verified now

The owner executed a real `ssh -tt ipat-lab` command and entered their Linux sudo password locally. The reviewed root-owned R10.18 probe reported:
- `ROOT_AUTHENTICATION=PASS`: interactive Linux administrative authorization now independently demonstrated through the owner's Terminal.
- `SOURCE_EXACT_MAIN=PASS`, `CONTROL_API_HASH=PASS` at the earlier pinned R10.18 baseline.
- `FIRST_INSTALL_RUNTIME_UNOCCUPIED=PASS`, `POSTGRES_CLUSTER_UNOCCUPIED=PASS`, `PRODUCTION_PORTS_UNOCCUPIED=PASS`.
- **`ROOT_OWNED_RECOVERY_ATTESTATION_METADATA=BLOCKED`** and `PROVIDER_RESCUE_AND_INDEPENDENT_HOST_RESTORE=EXTERNAL_EVIDENCE_REQUIRED`.
- `MUTATIONS_ATTEMPTED=false` and `PUBLIC_GO=false`.

Noninteractive assistant SSH still requires sudo authentication. Do not bypass sudo, add `NOPASSWD: ALL`, paste secrets or equate ordinary SSH sudo with an independent provider rescue console.

GitHub main `9f61ea5587ad6d7157cce7c79e582bad88dba5f8`, postmerge CI `38196768089` SUCCESS 4/4. On the authorized real Ubuntu26.04 x86_64 VPS, the exact main was cloned rootlessly to `/home/openai/ipat-release-src-r1019`, and `cargo build --locked --offline --release -p control-api` PASSED. Resulting ELF x86_64 binary SHA256: `1753d706b6d076fb67ba9509782e1e5c82a5aa673f901e9e809935bd98a66f20`. It was staged in `/home/openai/.local/share/ipat/release-candidates/9f61ea5587ad6d7157cce7c79e582bad88dba5f8`; `sha256sum -c SHA256SUMS` returned `control-api: OK`. This bundle is **owned by the unprivileged SSH user and is not yet a root-owned deployable source tree**.

A read-only Restic snapshot listing on the authorized Mac showed historical encrypted source, partial and selected root-readable configuration backups. These are **not** a complete server image restored on an independent replacement host. No whole-host restore, provider rescue login or real PostgreSQL WAL recovery was independently verified in this milestone.

## P0: operator-controlled evidence required before running R9.96

1. **Provider rescue console:** Using the actual VPS provider's out-of-band console for the correct VPS, establish a real rescue/root login and run `id -u` to prove UID 0. Preserve redacted date/time, instance reference, recovery route and independent reviewer reference in a private vault. Do not reboot the running owner preview merely to create a screenshot.
2. **Complete encrypted host backup:** Capture and retain an independently controlled bootable/fully reconstructable VPS disk/host configuration backup, including file ownership, boot partition layout and a documented method for restoring SSH access. Backup custody and encryption keys must survive loss of the live VPS and of the one Mac. A tiny subset of `/etc` or a Git archive is not a full-host backup.
3. **Real distinct-host restore:** Restore the actual encrypted full-host archive or image onto an **isolated disposable replacement host** with separate failure domain, no live subscriber management network and no public traffic. Demonstrate boot, approved login, SSH/service configuration, file permissions and controlled rollback. Record a backup manifest digest, redacted destination/failure-domain label, restore timestamps and clear PASS/FAIL checks. A local Mac folder extraction alone is insufficient.
4. **Independent review:** Different authorized maker and checker review provider-console access, encrypted backup integrity, offsite failure-domain independence, full-host restore logs and secrets/rollback handling. Only then create a genuine, root-owned 0600 single-link attestation with schema `ipat.recovery-readiness.v1` and truthful fields `provider_rescue_root_console_verified`, `independent_full_host_restore_verified`, `independent_backup_destination_verified`, reviewed timestamp, reviewer/evidence IDs and the exact **final installer source commit**. Do not set booleans true without underlying independently reviewed evidence. The existing `r996_validate_recovery_attestation.py` validates its format and policy; it does not independently prove off-host restoration.
5. **Root-owned source and binary:** Stage the exact reviewed main commit and matching compiled binary under a root-owned controlled location (not root-execute user-writable checkout scripts). Keep source and binary digests verified; complete preflight and rollback review. The R10.18 root probe hardcodes an **older** source commit and is not a substitute for a newly reviewed source pin.
6. **Controlled R9.96 foundation only:** Once gates pass, run the existing R9.96 first-install orchestrator with the real reviewed recovery attestation, exact SHA and explicit opt-in. Foundation must leave Platform/Tenant HTTP/OIDC services disabled and open **no public TCP listener**. Independently verify PostgreSQL18 Unix-socket peer roles, 33 immutable SHA256-pinned migrations, audit/RLS and rollback.
7. **Post-foundation gates:** Establish separately recovered and encrypted production PostgreSQL base+WAL with `pg_verifybackup`, named PITR target, tenant/RLS/audit restore, measured RPO/RTO, streaming standby promotion and fencing with maker/checker. Only then plan R9.97 literal-IP CA-trusted HTTPS/real operator MFA and R9.98/R9.99 actual per-company Host/SNI/DNS browser tests. Complete native ACS TR-069, USP TR-369, physical OLT/ONT/MikroTik interop, PPPoE controlled execution and scaling tests separately before ALL MUST PRD may be marked GO.

## Checks safe to repeat now

```bash
ssh ipat-lab 'cd ~/.local/share/ipat/release-candidates/9f61ea5587ad6d7157cce7c79e582bad88dba5f8 && sha256sum -c SHA256SUMS'
ssh ipat-lab 'systemctl --user is-active ipat-r911-preview.service'
```

Neither command requires sudo or mutates production. Keep raw backup credentials, root keys, proprietary device configs and actual host restore archives out of GitHub and chat. See `docs/R996_PRODUCTION_FOUNDATION.md`, `docs/R997_PLATFORM_EDGE_ACTIVATION.md` and `docs/PRODUCTION_INFRA_RECOVERY_R55.md`.

**Verdict:** interactive root PASS; exact main binary PASS; independent host rescue/full restore **BLOCKED**; real production database PITR/HA, MFA, TLS and hardware acceptance untested; **PUBLIC GO=FALSE**.
