# R9.86 — PostgreSQL 18 compatibility baseline for Ubuntu Server 26.04

Ubuntu 26.04 LTS currently provides PostgreSQL 18 packages. IPAT historically exercised most tenant/RLS integration on PostgreSQL16, so production planning must not assume those migrations behave identically on the Ubuntu26 baseline without running them.

R9.86 keeps PostgreSQL16 regression coverage and adds a separately pinned PostgreSQL18 disposable compatibility gate. It applies the current selected canonical security/tenant migration chain through migration 0032 and executes the same 45 database authorization, tenant isolation, POP/Site, catalog, Platform Owner and maker/checker regression tests.

The Docker test uses a pinned `postgres:18-alpine` repository digest, publishes no host database port, rejects inherited `PGSERVICE` configuration, verifies `server_version_num` begins with 18, and destroys the disposable database on exit.

Actual owner-Mac execution on 2026-10-05: selected migrations through 0032 applied successfully; PostgreSQL reported an 18.x server version; all 45/45 existing database security regressions PASS. A second rerun using the pinned image digest also PASS 45/45.

This is a **compatibility gate**, not PostgreSQL production HA/PITR evidence. It does not establish primary/standby failover, WAL archive durability, offsite retention, encrypted backups, measured RPO/RTO, real application service authentication mapping or recovery on an independent host. Those remain separate production requirements.
