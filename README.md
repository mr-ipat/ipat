# IPAT — Integrated Provisioning, Automation & Telemetry

IPAT (`IP@`) is a planned commercial multi-tenant ISP network operations platform. This repository is **work in progress**, currently an integrated laboratory MVP effort, **not production-ready**.

**Developer:** Mr. iPat · [Project attribution](AUTHORS.md)

## Official product documentation v0.1

- [Project binding brief](IPAT_PROJECT_BRIEF.md)
- [Product requirements and acceptance criteria](docs/PRD.md)
- [R5.1 synthetic PostgreSQL tenant RLS and isolated logical restore](docs/POSTGRES_TENANT_R51.md)
- [R5.2 synthetic evidence-led Rust diagnostics and negative scenarios](docs/DIAGNOSTICS_R52.md)
- [R5.3 sealed non-executable provisioning job simulator and test evidence](docs/PROVISIONING_SIMULATOR_R53.md)
- [R5.4 disposable PostgreSQL job/outbox isolation and restore test plan](docs/POSTGRES_JOB_OUTBOX_R54.md)
- [R5.5 read-only production readiness checks, recovery and staged DB/firewall/K3s plan](docs/PRODUCTION_INFRA_RECOVERY_R55.md)
- [R5.6 real disposable Ubuntu 26.04 K3s node, networking smoke and etcd snapshot](docs/K3S_UBUNTU26_R56.md)
- [R5.7 cross-host K3s restore, stale-node reconciliation and precise nft rollback](docs/K3S_CROSS_HOST_RECOVERY_R57.md)
- [Technical architecture](docs/ARCHITECTURE.md)
- [Physical device and firmware test matrix](docs/DEVICE_MATRIX.md)
- [Security, tenant isolation and threat model](docs/SECURITY.md)
- [Architecture decision register](docs/DECISIONS.md)
- [Deployment/restore requirements](docs/DEPLOYMENT.md)
- [Seven-day sprint backlog](docs/SPRINT_BACKLOG.md)
- [Actual project status and verification evidence](docs/PROJECT_STATUS.md)
- [2026-09-25 Ubuntu lab server read-only inspection](docs/LAB_SERVER_READ_ONLY_2026-09-25.md)
- [Restricted laboratory stage-1 bootstrap and no-snapshot safety plan](deploy/scripts/lab/README.md)
- [Stage-2 SSH key-only hardening proposal and six-minute recovery timer](deploy/scripts/lab/STAGE2-SSH.md)
- [Verified Stage-2 SSH results and read-only K3s prerequisites](docs/LAB_K3S_READ_ONLY_2026-09-25.md)
- [R4.5 encrypted temporary Mac backup, real isolated restore and historical external-ingress observations](docs/LAB_ENCRYPTED_BACKUP_EDGE_R45.md)
- [R4.6 verified FileVault and live IPv6 exposure; generic perimeter dependency (historical evidence only)](docs/EDGE_SECURITY_GROUP_R46.md)
- [R4.7 root-config direct-encrypted streaming preparation and host recovery gate](docs/ROOT_CONFIG_STREAM_R47.md)
- [R4.8 optional first-party host firewall architecture and safety gates](docs/FIREWALL_CONTROL_PLANE.md)
- [R4.9 original Rust offline CWMP authenticated-identity boundary and bounded session simulator](docs/CWMP_ADMISSION_R49.md)
- [R5.0 separate native Rust USP Controller boundary and strictly synthetic tenant-safe correlation](docs/USP_SYNTHETIC_R50.md)

**Authority:** The approved Project Master Brief is binding. In `DECISIONS.md`, `PROPOSED` and `OPEN` items require explicit approval/evidence. A documentation draft, test fixture or protocol placeholder does not imply implementation, certification, device support or production readiness.

## Binding technical requirements

- Ubuntu Server 26.04 LTS, backend Rust/Tokio/Axum, original Rust TR-069/CWMP engine (not GenieACS).
- Native TR-369/USP Controller is mandatory as a separate boundary; initial MTP/broker require confirmation.
- Heterogeneous K3s worker scaling, bounded queues/leases and idempotent high-impact operations.
- PostgreSQL with independently designed/tested commercial HA, PITR and external backups.
- End-to-end multi-tenant isolation; verified identity, RBAC+ABAC deny-by-default; unauthorized UI menus invisible and backend/API requests rejected.
- Evidence-aware network diagnostics across distribution, OLT/PON/ONT, router/PPPoE and subscriber layers.
- Physical compatibility is always per exact model, firmware, interface and proven feature.

## Initial Rust workspace (source scaffold only)

`crates/tenant-core` stores syntactic tenant IDs; `crates/authz-core` demonstrates pure policy checks and synthetic negative unit tests. `apps/control-api` exposes only loopback health and a fail-closed placeholder data path. No verified OIDC adapter, tenant database, CWMP/USP runtime or real device control has shipped.

`crates/cwmp-protocol` adds **offline-only** bounded CWMP 1.0 Inform parsing and a pure response serializer, with synthetic tests. `apps/cwmp-gateway` still has no authenticated transport/session listener. `apps/usp-controller` has only an optional loopback health route; native USP trust and correlation remain synthetic. `crates/diagnostic-core` and `crates/provisioning-core` are non-networked synthetic test modules, with no live router commands, OIDC-backed privileges or persistent job store. Device interoperability remains untested.

On a prepared development host: `cargo fmt --all -- --check && cargo test --workspace --locked`; run local demo with `cargo run -p control-api`. See `docs/PROJECT_STATUS.md` for **actual** test results; never infer success solely from source presence.
