# R9.90 — Restic-encrypted PostgreSQL 18 base+WAL recovery rehearsal

R9.88 proves PostgreSQL PITR from plaintext disposable base-backup and archived WAL. R9.90 adds encryption and recovery separation: a real PostgreSQL18 base backup and WAL archive are captured, encrypted into a Restic repository, the original plaintext staging is deleted, only the encrypted snapshot is restored, pg_verifybackup is rerun, then named-point PITR and tenant RLS are revalidated.

Acceptance sequence:
1. Pinned PostgreSQL18 with the exact R9.87 migration manifest.
2. Synthetic tenant and pre-restore-point device row.
3. Real pg_basebackup -Fp -X stream and pg_verifybackup.
4. Named restore point, post-point bad row, WAL switch and verified archiving.
5. Restic repository initialization with a synthetic test-only password file; base+WAL backup and restic check.
6. Delete plaintext base/WAL staging before restore.
7. Restore only from Restic; require base manifest/WAL and exact artifact SHA256 equality.
8. Re-run pg_verifybackup on the restored base backup.
9. Recover with restored WAL to the named point and promote.
10. Require good row count 1, bad row 0, FORCE-RLS unscoped 0 and scoped 1.

Actual owner-Mac execution passed with Restic 0.19.1. The local snapshot ID and artifact hash are test evidence only and are not production identifiers.

This does NOT prove an offsite failure domain because the rehearsal repository is local and disposable. Production acceptance still requires a repository controlled outside the primary VPS/provider failure domain, credentials stored outside Git, retention/immutability policy, actual production base+WAL upload, independent-host restore, measured RPO/RTO and restore evidence retained without exposing secrets.
