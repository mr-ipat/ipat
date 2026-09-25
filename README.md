# IPAT — Integrated Provisioning, Automation & Telemetry

IPAT (`IP@`) is a planned commercial multi-tenant ISP network operations platform. This repository is **work in progress**, currently an integrated laboratory MVP effort, **not production-ready**.

## Official product documentation v0.1

- [Project binding brief](IPAT_PROJECT_BRIEF.md)
- [Product requirements and acceptance criteria](docs/PRD.md)
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

`crates/cwmp-protocol` adds **offline-only** bounded CWMP 1.0 Inform parsing and a pure response serializer, with synthetic tests. `apps/cwmp-gateway` still has no authenticated transport/session listener; `apps/usp-controller` remains planned. Device interoperability remains untested.

On a prepared development host: `cargo fmt --all -- --check && cargo test --workspace --locked`; run local demo with `cargo run -p control-api`. See `docs/PROJECT_STATUS.md` for **actual** test results; never infer success solely from source presence.
