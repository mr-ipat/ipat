# R9.89 — PostgreSQL 18 physical streaming standby and controlled promotion

R9.88 proves base backup plus archived-WAL named-point PITR in a disposable environment. R9.89 adds PostgreSQL 18 physical streaming standby built from pg_basebackup, streaming WAL from a separate primary process, remaining read-only while in recovery, and promoted only after the primary is deliberately stopped.

Acceptance: pinned PostgreSQL18 primary with exact R9.87 migrations; replication-only login; synthetic tenant row; standby physical basebackup; explicit recovery and pg_stat_replication streaming; standby write denial; second row replicated; primary stopped and proven absent; standby promoted; two rows retained; FORCE-RLS unscoped 0/scoped 2; third write succeeds after promotion.

First execution found a false readiness failure: PostgreSQL logs showed streaming, but concatenated boolean text was true while the harness expected t. The harness now emits explicit 1/0. Replication and authorization were not weakened. Clean rerun passed.

This is not automatic failover, split-brain fencing, automatic failback, quorum, cross-VPS reliability, or production RPO/RTO. Production still needs independent failure domains, encrypted replication, monitored lag, stable endpoint, fencing and actual offsite PITR recovery.
