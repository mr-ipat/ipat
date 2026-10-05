# R9.91 — Remote-only offsite Restic evidence gate

R9.90 proves encrypted PostgreSQL18 base+WAL recovery mechanics using a disposable local Restic repository. R9.91 prevents that local test from being misreported as an independent offsite production backup.

The production verifier accepts only a nonsecret remote repository identifier and distinct active/backup failure-domain labels. Supported initial Restic repository forms are SFTP and HTTP(S)-backed REST/S3 forms. Local paths, file repositories, localhost, loopback/private literal targets and URL-inline passwords are rejected.

Actual repository credentials are never accepted as command arguments. RESTIC_REPOSITORY and an absolute owner-only RESTIC_PASSWORD_FILE must already exist in the executing host environment. Inline secret environment variables such as RESTIC_PASSWORD, cloud secret keys, or credential-bearing repository URLs make the gate fail closed. Output hashes repository/failure-domain identifiers and never prints the repository, password-file path, snapshot ID, hostname or token.

With the query flag, the gate executes only a read-only Restic snapshots JSON query with no lock and an approved production tag. It parses a current tagged snapshot and emits only a SHA256 of its ID, UTC time, age and matching count. Backup, restore, forget, prune, unlock, init, repair and repository mutation are not allowed in this gate.

A remote snapshot listing is not disaster recovery. Even with a valid snapshot, R9.91 returns production_status BLOCKED and public_go false. Production AC-13 still requires independently controlled offsite storage, encrypted base backup plus required WAL, restore on a distinct approved recovery host, pg_verifybackup, explicit PITR correctness, tenant/RLS/audit validation, measured RPO/RTO, retention policy and credential separation.

R9.91 cannot turn a self-declared failure-domain label into independent proof. The labels exist so evidence is fail-closed and reviewable; provider/storage ownership and distinct-host restore still require external observation.
