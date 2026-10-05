# R9.88 — PostgreSQL 18 archived-WAL named restore-point PITR

R9.87 creates a local-socket-only PostgreSQL 18 staging database on Ubuntu 26.04. R9.88 adds a real disposable PITR rehearsal using the same selected migration manifest. It is deliberately isolated from production and does not publish any host PostgreSQL port.

## What is actually tested

The test starts a pinned PostgreSQL 18 container with WAL archiving enabled and two private Docker named volumes: one for archived WAL and one for the physical base backup. It validates the exact R9.87 migration SHA256 manifest and applies all selected migrations before creating synthetic tenant-scoped data.

The acceptance sequence is:

1. Create one synthetic tenant and one known-good device row.
2. Prove that exact row exists before backup; an empty SHA256 input is explicitly rejected.
3. Run actual pg_basebackup -X stream and pg_verifybackup.
4. Create PostgreSQL named restore point r988_before_bad_change.
5. Insert a second synthetic row marked BAD-AFTER-PITR, then force WAL switch and confirm archiving completes.
6. Stop the source PostgreSQL process.
7. Start a fresh PostgreSQL 18 process from only the base backup plus archived WAL using recovery.signal, restore_command, recovery_target_name and recovery_target_action=promote.
8. Require exactly one pre-point device row with the expected tenant ID after recovery.
9. Require the post-point synthetic bad row count to be exactly zero.
10. Require the restored pre-point SHA256 dataset to equal the source pre-point dataset.
11. Re-test FORCE RLS using the actual ipat_app_runtime database login: unscoped count must be 0; transaction-local exact tenant scope must expose exactly 1 row.

No container publishes PostgreSQL ports to the host. The test does not use SSH, firewall changes, systemd, K3s, physical devices, production credentials or a production database.

## False-positive correction found during development

An early test draft used docker exec without -i for heredoc-fed psql. The command exited successfully while stdin SQL was not delivered, which made an empty-dataset SHA256 look superficially valid. R9.88 now requires docker exec -i for both seed and post-restore-point mutation, explicitly checks the source row count before backup, rejects the SHA256 of empty input, checks restored row count and tenant identity, then validates RLS. This correction is part of the accepted test contract.

## Local evidence

Owner Mac disposable PostgreSQL 18 execution:
- source test contract: 5/5 PASS;
- physical base backup and pg_verifybackup: PASS;
- archived WAL named-point recovery and automatic promote: PASS;
- pre-point row identity/hash: PASS;
- post-point bad row count: 0;
- restored FORCE RLS: unscoped 0, scoped 1.

The emitted final marker is R988_POSTGRES18_BASEBACKUP_VERIFY_WAL_NAMED_POINT_PITR=PASS.

## What this does NOT prove

This is not offsite storage, not a separate physical failure domain, not standby streaming replication, not automatic failover, and not measured production RPO/RTO. It does not prove restoration of the actual production database or full VPS. Commercial go-live still requires independently controlled encrypted off-host base backups/WAL, restore on a distinct approved host, measured recovery time and loss window, reconciliation of outbox/device state, real human MFA, trusted HTTPS ingress and external two-tenant security acceptance.
