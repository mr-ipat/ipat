# R10.38 — PostgreSQL 18 migration, tenant isolation, PITR, HA and Restic QA

**Date:** 2026-10-16 WIB. **Scope:** isolated disposable database containers on authorized operator Mac. **Not** the production VPS database or a provider rescue/full-host restore.

## Actual execution evidence

Using the exact canonical source migrations and existing source-controlled scripts:

| Scenario | Observed result |
| --- | --- |
| R9.86 PostgreSQL18 migration, multi-tenant RLS/ABAC, owner browser, independent POP/Site and vendor catalog | 51 tests: 50 PASS, 1 legacy-role SKIP, 0 FAIL |
| R9.88 base backup, archived WAL, named-point PITR and post-restore tenant RLS | PASS; synthetic post-point row absent; pre-point checksum verified |
| R9.89 PostgreSQL18 physical streaming standby and controlled promotion | PASS; write to standby denied before promotion; tenant RLS after promotion passed |
| R9.90 encrypted Restic backup of PG18 base + WAL, integrity verification and clean PITR | PASS; restored bad-row count zero, tenant RLS enforced |

The PostgreSQL 18 checks were executed using Docker Desktop on the Mac after root release custody verification. The tests use only fixed synthetic identities and secrets, separate disposable containers, and publish no database ports to the host.

## R10.38 implementation

`deploy/scripts/production/r1038/run_pg18_qa.py` is a single **local-only** orchestration entry point. Invoke:

```bash
python3 deploy/scripts/production/r1038/run_pg18_qa.py --run
```

The runner requires an explicit opt-in; refuses root, pre-existing production PostgreSQL/Restic environment variables, remote Docker endpoints, missing scripts and incomplete PASS markers. It runs all four reviewed scripts sequentially; an error in any suite prevents the overall PASS. Output is a public-safe JSON report with exact source commit, script digests and actual results. It always states `full_host_restore_verified=false`, `production_postgresql_ha_pitr_verified=false`, and `public_go=false`.

Five negative/positive unit tests are included in the existing R9.96 CI contract without modifying workflow permissions. The runner deliberately creates no production database, infrastructure service, firewall change, network route or vendor device write.

## Production acceptance still separate

- Production VPS **does not have PostgreSQL18 installed** at this milestone.
- Existing reviewed source/binary already root owned and verified; do not repeat handoff. For first-install the reviewed immutable release commit remains `a01811457c45d1be4bef959d34f691f1636ea87b`, even if documentation/QA-only changes later advance `main`. The R9.96 installer validates this exact source and binary; a newer documentation commit does not automatically invalidate a separately reviewed release. Any subsequent runtime change requires new source/binary review and checksums.
- Production recovery attestation, independent host backup/restore receipt, live PITR+HA, tenant MFA and TLS Host/SNI still require real acceptance. The local Mac has no verified independent offsite backup destination connected, and its free storage is not adequate to hold a trustworthy full copy of the ~124 GiB production filesystem *and* an independently restored whole-host VM simultaneously.
- The successful disposable PostgreSQL tests cannot be substituted for owner-provider rescue access or full-host restore on separate VPS/VM.

No existing PRD MUST requirement has been waived.

## Orchestrator execution result

The new R10.38 runner was **actually executed** on the Mac on 2026-10-16 UTC 05:20:50 with explicit `--run`. Exit code 0, all four JSON entries `PASS`, exact source commit `c5f12bcb7a900875009b493cb7abda499561f758`. Script digests:

```text
r986  8c4cda3dbdd83ef62d91d7748030f951311a2fe138e9872584df28c0c081f7a4
r988  59a74eed32d09225dd683ceebc1fdcdf03231ceceb1964d191a309130ecd1782
r989  2ce4c39ee85b7f897d28605daab89e6401f27b4c7902bc278b7c9cbc10697605
r990  5100c57a46c6c3ee9b45d6240fa8960eead47dd1500d8016d7e98df166b3588b
```

The full four-stage run took approximately 66 seconds. No production PostgreSQL or offsite host restore was executed.
