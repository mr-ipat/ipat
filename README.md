# IPAT — Integrated Provisioning, Automation & Telemetry

Private multi-tenant ISP operations platform (IP@). This repository is a **laboratory-MVP work in progress**, not a production service.

## Binding project decisions

- Ubuntu Server 26.04 LTS; backend Rust/Tokio/Axum.
- Original Rust ACS supporting CWMP/TR-069. Do **not** substitute GenieACS.
- Native, separately bounded TR-369/USP Controller is required. MQTT remains a proposal pending ADR approval.
- OLT/ONT/MikroTik adapters require exact model + firmware test evidence before compatibility claims.
- Tenant separation across API, data, queue, telemetry, artifacts and device identities.
- RBAC + ABAC deny-by-default, including hidden unauthorized menus and backend denial.
- K3s heterogeneous horizontal scaling with bounded worker concurrency and idempotent side effects.
- PostgreSQL HA and independently tested backups are commercial requirements, not properties of this initial scaffold.

The immutable transferred source baseline is [IPAT_PROJECT_BRIEF.md](IPAT_PROJECT_BRIEF.md).
The previously prepared full v0.1 documentation package must be imported and reconciled before any proposed ADR is finalized. No experimental code below implies approval of unresolved ADRs.

## Day-one scaffold (not device-ready)

- `crates/tenant-core`: canonical syntactic tenant IDs.
- `crates/authz-core`: pure deny-by-default authorization example with synthetic two-tenant tests.
- `apps/control-api`: local-only Axum health endpoint. Protected resource route always rejects unauthenticated requests.
- Rust CI workflow is prepared locally but **not yet publishable**: the connected GitHub OAuth credential lacks the `workflow` scope. GitHub-hosted CI has not run.

Run when Rust is installed: `cargo fmt --all -- --check && cargo test --workspace`.
Start local API: `cargo run -p control-api`; do not expose it to the internet.
No persistent tenant storage, OIDC validation, ACS, USP, physical adapters, migrations or deployment is implemented yet.

See [bootstrap status](docs/PROJECT_STATUS.md) for evidence and blockers.
