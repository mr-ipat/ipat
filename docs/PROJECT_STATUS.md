# IPAT — Project Status

**As-of:** 2026-09-25 Asia/Jakarta  
**Milestone:** R1 — v0.1 canonical documents imported into private Git repository  
**Milestone state:** M0 documentation drafted; R0 initial Rust scaffold committed; R1 documents mirrored. Rust tests/CI, physical interoperability and Ubuntu deployment are not verified.

## 1. Source of truth and provenance

- Binding source: `../IPAT_PROJECT_BRIEF.md` user-uploaded approved design baseline and Project Instructions.
- At M0, no existing PRD/ARCHITECTURE/DEVICE_MATRIX/SECURITY/DECISIONS/PROJECT_STATUS was available in the currently accessible Project files. This document package initializes v0.1; if an earlier Git canonical version exists elsewhere, compare/merge before overwriting.
- This ZIP/download is a **snapshot**, not a Git push or automatic Project attachment/update. Canonicalize by committing documents to project Git and attaching/updating latest project files as appropriate.

## 2. Deliverables created in this milestone

| Path | Deliverable | Status |
|---|---|---|
| `docs/PRD.md` | Functional/nonfunctional scope; lab S1 vs commercialization; AC-01..16; risks/assumptions/open decisions | DRAFTED v0.1 |
| `docs/ARCHITECTURE.md` | Rust services, multi-tenant persistence proposal, CWMP/USP, safety of jobs, K3s/HA, Mermaid diagrams | DRAFTED v0.1 |
| `docs/SECURITY.md` | Threat model T-01..15, SEC-01..10, roles, approval and security gates | DRAFTED v0.1 |
| `docs/DEVICE_MATRIX.md` | Eight physical target rows, exact inventory questions, per-feature test ledger | DRAFTED; **all untested** |
| `docs/DECISIONS.md` | ADR-001..017: binding decisions vs PROPOSED/OPEN | DRAFTED; decision proposals **not approved** |
| `docs/SPRINT_BACKLOG.md` | D1–D7 critical path, implementation stories, gates and cut-line | PLANNED only |
| `docs/DEPLOYMENT.md` | Planned lifecycle, restore/HA and file-path expectations | PLANNED procedures, not executable scripts |
| `README.md` | Package index and setup/handoff | DRAFTED v0.1 |
| `IPAT_PROJECT_BRIEF.md` | Exact uploaded source snapshot | PRESERVED (not edited) |

## 3. Decisions retained and proposals registered

**Binding retained:** original ACS Rust (no GenieACS engine), separate native USP Controller TR-369, Rust/Tokio/Axum, Ubuntu 26.04 LTS, modular monolith + workers, heterogeneous K3s scheduling and idempotent device operations, PostgreSQL HA/PITR production requirement, per-company verified domain dashboards and full tenant isolation, RBAC+ABAC deny-by-default with invisible prohibited menus and backend denial, diagnostics with evidence/uncertainty, initial physical target list. Initial pilot sizing baseline from brief: 16 vCPU, 64 GiB-class RAM, ~1 TB NVMe **provisional**.

**Verified public reference version (not implementation):** BBF TR-069a6c1, TR-369a5, TR-181i2a21; Ubuntu 26.04 LTS released 2026-04-23; actual target device firmware remains unknown.

**PROPOSED:** shared tenant tables + RLS/controlled DB role, Next.js/Keycloak/RabbitMQ/Prometheus, MQTT USP MTP and broker, role threshold. **OPEN:** actual physical inventory/access, DB tenancy and backup model final approval, USP broker+features, secrets/KMS, data residency/custom domains, RPO/RTO/SLO, HA provider design, commercial role matrix.

## 4. Execution and evidence status

| Area | Actual status | Next action |
|---|---|---|
| Rust repo/build & CWMP code | Minimal Rust workspace and Axum loopback stub written; CWMP NOT STARTED; compile NOT RUN | Run formatting/tests; strict XML parser + Inform simulator remain next |
| Native USP code/protocol tests | NOT STARTED | Select schema/version subset and establish isolated PoC |
| Frontend/tenant auth & policies | Two synthetic tenant IDs and pure deny-by-default policy test source written; no verified OIDC or frontend | Implement actual trusted identity and negative API/data access tests |
| PostgreSQL schema/migrations and DB RLS | NOT STARTED | Design ADR-005 pilot and write/test migrations |
| Queue/worker and MikroTik bulk demo | NOT STARTED | Job states, immutable dry-run/approvals, idempotency tests before write |
| OLT/ONT/MikroTik physical interoperability | NO TEST EXECUTED; ALL UNTESTED | Record exact models/firmware, get authorized lab access, test read-only |
| Diagnostics | NOT STARTED | Synthetic fixtures for three independent fault domains |
| K3s hetero second-node testing | NOT STARTED; AVAILABILITY UNKNOWN | Secure second node and network, follow deployment runbook |
| Database backup+restore | NOT EXECUTED | Implement scripts and verify isolated restore during S1 |
| Security pen test / production HA | NOT EXECUTED | Commercial gate after core readiness |

**M0 documentation checks executed (2026-09-25, container Python 3): PASS.** Checked that eight required canonical `docs/*.md` exist, ten Markdown files have headings/nontrivial content and balanced fenced code blocks, local file links resolve, all expected PRD/acceptance/matrix markers are present, the transferred brief is byte-identical, and the ZIP passed `unzip -t`. These are **documentation/packaging checks only**. No product tests, physical tests, security penetration tests, cluster tests or deployment runs have occurred. Future test results must report command, environment/version, pass/fail/blocked and evidence. No claim about device compatibility, throughput, uptime or deployed code can be inferred from M0.

## 5. Dependencies / blockers to resolve on D1

1. Exact C-DATA model; ZTE C320 board/firmware; VSOL/ZTE ONT model/firmware and whether any truly runs USP; MikroTik SKUs/RouterOS versions/accessible secure management interfaces.
2. Team and lab operator assignment; isolated lab permission, recovery path, restricted device test credentials and maintenance windows.
3. Access to proposed lab server baseline and possible second heterogeneous node; private routing and external backup destination.
4. Product-owner selection/approval of PROPOSED architecture decisions and exact tests/feature cut-line where resources conflict.
5. Target tenant/device scale, Inform intervals, telemetry retention, SLO and commercial data-residency/legal constraints.

## 6. Next execution steps

- **First:** review v0.1 PRD/ADRs and approve or record changes; establish project Git canonical docs and update Project attachments so new chats consume the latest versions.
- **Then D1:** capture lab device intake and credentials safely; initialize Rust workspace, migrations, CI and two synthetic tenants; no network writes yet.
- **D2–D7:** follow `SPRINT_BACKLOG.md` and update this file after each day's measurable result, blocked state and file changes.
- **M1+:** update `DEVICE_MATRIX.md` with exact per-feature test evidence; no blanket vendor support claims; progress to commercial HA/security gates only on measured results.

## 7. Milestone changelog template

```text
Date / milestone:
Commit and environment:
Decisions made or changed (ADR IDs):
Paths changed:
Automated tests run and result:
Simulator tests run and result:
Physical tuple(s) tested, capability/result/evidence:
Failures/security issues and mitigations:
Outstanding blockers, owner and next priority:
```

## 8. GitHub R0 and documentation R1 (verified 2026-09-25)

- GitHub repository `mr-ipat/ipat` is private; the R0 main commit `6ac0b098ed9094e029973a4d1fe41e682435fd51` was verified through GitHub API.
- R0 committed scaffold: `Cargo.toml`, `tenant-core`, `authz-core`, loopback-only `control-api`, placeholder module boundaries for independent native CWMP and USP; synthetic authorization tests are **source only**, not proof of runtime authorization.
- The existing local `~/Projects/ipat` was not overwritten. The disposable R0 checkout vanished between sessions; R1 uses isolated `~/Projects/ipat-current` cloned from private GitHub.
- GitHub CLI account reauthorization now reports `workflow` OAuth scope. This is a **permission verification**, not a successful workflow run.
- Seven full canonical v0.1 documents now transferred byte-for-byte, each SHA-256 matched to the synchronized ZIP: PRD, ARCHITECTURE, DEVICE_MATRIX, SECURITY, DECISIONS, DEPLOYMENT and SPRINT_BACKLOG. This status file was imported from original M0 status and updated to preserve actual R0/R1 execution evidence.
- All physical target rows in DEVICE_MATRIX remain **untested**. No real device writes, server access, production deployment or database migration has occurred.
- Open GitHub issues: #1 official doc import; #2 workflow publication/test run; #3 verified OIDC/tenant negative integration tests; #4 actual Ubuntu host and device intake.

### R1 test and approval gates

- Documentation/package checks: validate tracked doc inventory, local Markdown file links, fenced blocks and `git diff --check`; preserve check results in the subsequent commit.
- Rust `cargo fmt`, `cargo test`, and GitHub Actions: NOT YET RUN at document import time. Source-only policy tests do not establish tenant isolation.
- No newly PROPOSED or OPEN architecture ADR was approved by this import; hold DB tenant strategy, USP transport/broker, secrets management, role matrix and production HA for explicit decisions.
- Next milestone: publish restricted read-only workflow in a reviewed PR using new GitHub scope, run actual tests and record evidence, then implement first safe CWMP simulator slice. Ubuntu deployment waits for nonsecret host details and least-privilege authorized access.

### R1 executed documentation checks

- PASS on authorized Mac: all seven synchronized v0.1 documents SHA-256 matched the source ZIP; transferred brief SHA-256 verified; README and eight documentation files have balanced fenced code blocks and resolvable local Markdown links. Intended Markdown two-space line breaks are preserved; Git whitespace check must allow Markdown blank-at-EOL. This verifies document integrity/links, not software behavior.

## 9. R2 — initial GitHub CI evidence (2026-09-25)

- GitHub CLI now advertises `workflow` scope, confirmed by GitHub response headers. Initial workflow push was still rejected because Git consulted an older macOS keychain credential; pushing with explicitly reset Git credential helpers and the current `gh auth git-credential` **succeeded**. No token value was copied into the repository or conversation.
- PR #6 introduces restricted GitHub Actions on **Ubuntu 24.04 runners**, not validation of the target Ubuntu Server 26.04 LTS deployment.
- Initial real GitHub Actions run `36095862500`: **FAILED on rustfmt** in `apps/control-api/src/main.rs` and `crates/authz-core/src/lib.rs`; unit tests were **not executed** in that run.
- Applied the exact CI-generated rustfmt diff to both source files after successful `git apply --check`; the fix remains **unverified until the next workflow run passes**.
- GitHub PR #5 for full official v0.1 documents was merged; GitHub issue #1 closed. New architecture ADRs remain proposed/open pending sign-off.
