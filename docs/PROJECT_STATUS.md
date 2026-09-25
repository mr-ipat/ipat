# IPAT — Project Status

**As-of:** 2026-09-25 Asia/Jakarta  
**Milestone:** R4.2 — restricted lab user-only Rust setup and guarded privileged stage prepared  
**Milestone state:** canonical v0.1 docs and 23 GitHub-hosted synthetic/unit tests verified; on Ubuntu 26.04.1 the user-only pinned Rust toolchain and private code mirror are installed and rustfmt passes, while local unit-test execution is BLOCKED by missing system C linker. No privileged stage, hardening, firewall change, K3s, database deployment or physical compatibility test has been completed.

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
| `docs/LAB_SERVER_READ_ONLY_2026-09-25.md` | Real SSH/Ubuntu, security boundary and capacity inspection | READ-ONLY VERIFIED; no host modification |
| `README.md` | Package index and setup/handoff | DRAFTED v0.1 |
| `IPAT_PROJECT_BRIEF.md` | Exact uploaded source snapshot | PRESERVED (not edited) |

## 3. Decisions retained and proposals registered

**Binding retained:** original ACS Rust (no GenieACS engine), separate native USP Controller TR-369, Rust/Tokio/Axum, Ubuntu 26.04 LTS, modular monolith + workers, heterogeneous K3s scheduling and idempotent device operations, PostgreSQL HA/PITR production requirement, per-company verified domain dashboards and full tenant isolation, RBAC+ABAC deny-by-default with invisible prohibited menus and backend denial, diagnostics with evidence/uncertainty, initial physical target list. Initial pilot sizing baseline from brief: 16 vCPU, 64 GiB-class RAM, ~1 TB NVMe **provisional**.

**Verified public reference version (not implementation):** BBF TR-069a6c1, TR-369a5, TR-181i2a21; Ubuntu 26.04 LTS released 2026-04-23; actual target device firmware remains unknown.

**PROPOSED:** shared tenant tables + RLS/controlled DB role, Next.js/Keycloak/RabbitMQ/Prometheus, MQTT USP MTP and broker, role threshold. **OPEN:** actual physical inventory/access, DB tenancy and backup model final approval, USP broker+features, secrets/KMS, data residency/custom domains, RPO/RTO/SLO, HA provider design, commercial role matrix.

## 4. Execution and evidence status

| Area | Actual status | Next action |
|---|---|---|
| Rust repo/build & CWMP code | Rust workspace and **offline-only** CWMP 1.0 Inform parser/serializer merged; GitHub-hosted Ubuntu 24.04 CI: **23 tests PASS** | Authenticated/tenant-bound CWMP session + safe RPC simulator remain required |
| Native USP code/protocol tests | NOT STARTED; independent protocol boundary documented | Select schema/version subset and establish isolated PoC (issue #9) |
| Frontend/tenant auth & policies | Two synthetic tenant IDs and pure deny-by-default policy test source written; no verified OIDC or frontend | Implement actual trusted identity and negative API/data access tests |
| PostgreSQL schema/migrations and DB RLS | NOT STARTED | Design ADR-005 pilot and write/test migrations |
| Queue/worker and MikroTik bulk demo | NOT STARTED | Job states, immutable dry-run/approvals, idempotency tests before write |
| OLT/ONT/MikroTik physical interoperability | NO TEST EXECUTED; ALL UNTESTED | Record exact models/firmware, get authorized lab access, test read-only |
| Diagnostics | NOT STARTED | Synthetic fixtures for three independent fault domains |
| K3s hetero second-node testing | NOT STARTED; first Ubuntu 26.04.1 host verified as 16-vCPU/30-GiB/250G KVM VPS; no K3s service | Approve restricted lab sizing and staged change plan, then validate node setup |
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

### R2 CI test evidence and dependency lock

- GitHub Actions run `36095959951` on GitHub-hosted Ubuntu 24.04 completed **SUCCESS** after applying CI rustfmt diff: workspace format check passed and **8 synthetic/unit tests passed** (4 `authz-core`, 2 `tenant-core`, 2 `control-api`); no integration/physical/Ubuntu 26.04 tests.
- GitHub Actions run `36096084672` also completed **SUCCESS**, produced initial `Cargo.lock` as a short-retention private workflow artifact. Downloaded and verified artifact SHA-256 `bc4d918e6131c00a5e48a25c32f22542462e75b2c52389bdce257bf5bdf98ece` (12,602 bytes).
- CI runner reported Rust `1.98.1`; project now proposes to pin it in `rust-toolchain.toml` and use `cargo test --workspace --locked`. Final locked GitHub Actions run `36096203196` on Ubuntu 24.04 **PASSED**: format check and all **8 unit/synthetic tests** (4 authz, 2 tenant, 2 API), using Rust 1.98.1 and the committed `Cargo.lock`. This still does not validate Ubuntu 26.04 deployment or real-device behavior.

## 10. R3 — original Rust CWMP parser / simulator slice (in progress)

- Feature branch: `feat/cwmp-inform-parser` from the verified CI-enabled main branch.
- New crate `crates/cwmp-protocol` implements *offline* bounded SOAP 1.1 / CWMP 1.0 Inform parsing and a pure InformResponse XML serializer; simulated fixture test sources cover valid envelope, correlations, XML escaping, Doctype, body/depth/size/identity rejection and strict namespace checks. **No real CPE, authorization, networked ACS service or USP is claimed.**
- Added dependency `roxmltree` as a parser choice for this limited offline slice. Final protocol feature/method/firmware coverage remains subject to the recorded ADR and physical test matrix.
- R3 tests: **not yet executed** at source-writing time; first refresh lockfile through controlled GitHub runner artifact and obtain a passing final pinned `--locked` run before merging.

### R3 initial CI / lock resolution evidence

- GitHub Actions run `36096625485` on the CWMP feature PR generated the new `roxmltree = 0.21.1` lockfile artifact, then **FAILED on rustfmt** before parser tests ran. These failures are recorded; no parser unit test is yet marked passed.
- Downloaded and verified the new `Cargo.lock` from the private job artifact (SHA-256 `ae574ffbb8488eebdc79d3802793cd8e5a9e47d82f9319bf0dd68a68183688f0`). Applied CI-produced rustfmt diff for the new parser and simulator test sources with `git apply --check` before application.
- Removed the temporary artifact-export workflow step and restored locked/least-privilege CI; new test run remains **PENDING** at this status update.

### R3 actual locked-test failure and correction

- GitHub Actions run `36096726738` passed rustfmt and compiled the workspace, but the new CWMP integration suite **FAILED** 1 of 15 tests (14 passed). Existing 8 tests also passed. The failing assertion expected `WrongMethod` for an unsupported CWMP version, whereas the strict parser rejected its header first as `UnsupportedHeader`.
- Adjusted the simulator test to assert rejection regardless of which validation gate detects the unsupported version first. No parser acceptance behavior was relaxed and no real device was tested. A fresh CI run is required before any R3 pass claim or merge.

### R3 verified simulator-only result

- GitHub Actions run `36096847980` on GitHub-hosted Ubuntu 24.04 completed **SUCCESS** with pinned Rust 1.98.1 and `cargo test --workspace --locked`: rustfmt passed; **23 unit/simulator tests passed, zero failed** (4 authorization, 2 tenant IDs, 2 Control API, 15 offline CWMP Inform/InformResponse cases). The CWMP test rejection invariant was corrected before this successful run.
- Scope is strictly **offline SOAP 1.1 / CWMP 1.0 synthetic parser and XML response serializer**. This is NOT an authenticated HTTPS ACS listener, CWMP session engine, native USP implementation, physical-device compatibility evidence, or Ubuntu Server 26.04 deployment.
- Next R4 requires per-device tenant identity verification, authenticated session design, SOAP fault and RPC interoperability tests before enabling any network listener; physical access and exact firmware are still prerequisites for vendor claims. All PROPOSED/OPEN architecture ADRs remain pending approval.

## 11. R3 finalized on protected workflow — 2026-09-25

- GitHub PR #7 (original Rust **offline-only** CWMP 1.0 SOAP/Inform parser + pure InformResponse serializer and synthetic negative tests) was **MERGED** to `main` in commit `7ae27f0fe4aad1e5b03443b7e7aeb58ce13dbe02`. This closes R3 source integration, **not** ACS network interoperability or Sprint 1 acceptance AC-03.
- Final pull-request CI run `36096943097` **PASSED** formatting and locked unit/simulator tests. Post-merge `main` CI run `36097042718` independently **PASSED**, using pinned Rust 1.98.1 on GitHub-hosted Ubuntu **24.04**. The previously verified suite comprises **23 synthetic/unit tests, zero failed**; no tests on target Ubuntu Server 26.04 LTS or real devices.
- R4 tracked as GitHub issue #8: authenticated/tenant-bound native CWMP gateway and simulator-based session/fault/RPC slice; #9: native independent USP Controller simulator PoC. Security issue #3 and physical device/Ubuntu intake issue #4 remain open.
- Remaining blockers: no authorized Ubuntu server SSH target/non-root username supplied; exact lab model/hardware revision/firmware absent. Physical device support, USP runtime, end-to-end OIDC isolation, PostgreSQL HA and heterogeneous K3s deployment remain **NOT IMPLEMENTED / NOT VALIDATED**.
- `DECISIONS.md`: no new architecture decision is approved by this milestone. Do not change choices marked PROPOSED/OPEN without explicit review.

## 12. R4 lab SSH access preflight — 2026-09-25 (blocked on one-time account authorization)

- Authorized connection path: Remote Desktop Commander on `MisteriPat.local`; target `openai@ipat.fadly.id:22`. DNS resolved and the server's ED25519 host key matched the existing local `known_hosts` record.
- **Observed SSH result:** noninteractive login with the Mac's existing RSA identity returned `Permission denied (publickey,password)`. Ubuntu release, CPU/RAM/disk, packages, firewall, login shell and server permissions have **NOT BEEN READ**, so reported Ubuntu 26.04 installation remains user-reported, not verified.
- With project-owner authorization, a new single-purpose **lab** Ed25519 private/public key was created **only on the Mac** as `~/.ssh/id_ed25519_ipat_openai{,.pub}` with local private-key mode `0600`; public fingerprint `SHA256:UlAgH1utoMzK3Ld4QghPfVwQr873CAaKpJjCmnCeA3Q`. It has **no passphrase** to support unattended lab read-only preflight; treat as a privileged local secret, use only with the dedicated non-root lab account and rotate/replace with a reviewed agent/keychain approach before production. Neither key was added to Git.
- Added local SSH alias `ipat-lab` with a pinned identity, `IdentitiesOnly=yes`, `StrictHostKeyChecking=yes`, `BatchMode=yes`, and preserved original config in `~/.ssh/config.before-ipat-lab`. Prepared local user-run helper `~/.ssh/install-ipat-lab-key.sh` (mode `0700`); shell syntax verified. This helper requires the **real Linux account password typed on the Mac locally**, not a key passphrase or a password pasted into chat.
- **Remote status: BLOCKED.** No new public key could be authorized on the Ubuntu host without an existing authenticated server session or one-time account-owner console/password action. No server configuration, SSH authorization, packages, or firewall was changed by the assistant. Do not mark the SSH check or Ubuntu inspection as passed.
- Next: account owner runs the prepared helper in their Mac Terminal and enters the Linux account password at the local SSH prompt (or adds the displayed public key through the VPS console). Then retry a strict-key, noninteractive SSH login and **read-only** OS/hardware/network/package inspection, record evidence, and seek separate approval before server modifications.

## 13. R4.1 — first successful read-only Ubuntu server inspection (2026-09-25)

**Recorded evidence:** [LAB_SERVER_READ_ONLY_2026-09-25.md](LAB_SERVER_READ_ONLY_2026-09-25.md). Target: `openai@ipat.fadly.id` via verified existing `ipat-lab` alias from authorized Mac. This milestone supersedes the previous *SSH blocked* state; retain R4 blocked history as an accurate earlier result.

- **SSH / OS:** PASS. Dedicated Ed25519 public-key `BatchMode=yes`, pinned known-host key and non-root `openai` authenticated; directly verified `Ubuntu 26.04.1 LTS`, kernel `7.0.0-34-generic`, `x86_64` KVM, time synchronized in `Asia/Jakarta`. No Ubuntu configuration was changed by the assistant.
- **Capacity:** 16 KVM vCPUs, `free -h` 30 GiB total memory, no swap, 250G single observed disk with 246G ext4 root file system (about 233G free, 2% used). This is the constrained lab VPS and **does not amend** the existing 16-vCPU / 64-GiB-class / ~1-TB provisional pilot sizing ADR-013.
- **Network / security:** SSH TCP/22 listening on all IPv4/IPv6 addresses; observed private `eth0` address, DNS stub on loopback; SSH service and time sync active. `authorized_keys` mode `0600`, `~/.ssh` `0700`; noninteractive `sudo -n -l` denied and nonprivileged `sshd -T` cannot validate effective security settings. `ufw`, `nft`, `fail2ban-client` not on PATH; **provider firewall and root/password SSH rules remain UNKNOWN**.
- **Software:** Git/curl/OpenSSL/Python/jq found. Rust/Cargo, PostgreSQL clients, Docker/containerd CLI, K3s/kubectl/Helm/Ansible absent on PATH, no respective active daemons observed. No package or container installation, deployment, privilege escalation, or device access was attempted.
- **Scope:** PASS for read-only SSH/OS, capacity, baseline network and account permission inspection; **BLOCKED/PENDING APPROVAL** for host hardening, firewall, secret handling, K3s, PostgreSQL, backup/restore and app deployment. All physical-device capabilities remain untested; no native USP implementation claim.
- **Changed repository paths (documentation only):** `docs/LAB_SERVER_READ_ONLY_2026-09-25.md`, `docs/PROJECT_STATUS.md`, `README.md`; no architecture ADR changed.
- **Next mandatory gate:** present the above results to the project owner; obtain explicit authorization and recovery plan **before writing to Ubuntu**. Then proceed with security-first staged lab bootstrap and measure storage/memory under restricted pilot workload.

## 14. R4.2 — restricted lab user-only bootstrap and guarded privileged plan (2026-09-25)

**Authorization and risk gate:** The project owner authorized secure laboratory configuration and confirmed they can access the VPS console, but explicitly reported **no VPS snapshot**. No evidence of a successful rescue-console restore or encrypted complete offsite host backup is available. Therefore this stage made **NO privileged Ubuntu changes**, SSH/password/firewall policy changes, K3s or database deployment; do not describe the host as hardened.

### Actually executed on Ubuntu (non-root only)

- Fetched the official `x86_64-unknown-linux-gnu` Rust `rustup-init` and its published SHA-256 over certificate-validated HTTPS; `sha256sum --check` **PASS**. Verified installer SHA-256: `dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71`. Installed user-local pinned Rust **1.98.1**, Cargo **1.98.1** and rustfmt with `--no-modify-path`; added a guarded Cargo environment source to the `openai` user's `~/.profile` after creating a 0600 local profile backup. `bash -lc 'cargo --version'` **PASS**.
- Transferred the exact canonical private GitHub `main` revision `9359bf5edc9ec0882e9b7926ae1e372d57d3e8c9` from the Mac into `/home/openai/workspaces/ipat` via a Git bundle and verified the commit hash. **No GitHub token, Mac private SSH key or deploy credentials** were copied to Ubuntu. Removed the transient bundle and its invalid repository remote; next update is a new hash-verified bundle from the owner Mac.
- Real Ubuntu `cargo fmt --all -- --check` **PASS**. Real Ubuntu `cargo test --workspace --locked` **FAILED/BLOCKED** at the first compiled build script: `linker cc not found`. Apt simulation on this minimal Ubuntu image proposed 33 additional packages for `build-essential`; the privileged package install has **NOT RUN**. Existing GitHub-hosted Ubuntu 24.04 simulator tests are independent and cannot substitute for this real-host validation.
- Created a **PARTIAL** off-host, 0600-mode readable-config archive under the Mac's 0700-mode `~/IPAT-secure-backups/`, successfully verified archive integrity; SHA-256 `2d74782d0e418262701107be01e91c0ea87d3b7406803ca58acdb91a57338a77`. Does not include the Ubuntu root-only `50-cloud-init.conf`, SSH private host keys, block storage or application state; NOT a full disaster recovery backup.
- From read-only world-readable `/etc/ssh/sshd_config`, **observed text** `PermitRootLogin yes` and `PasswordAuthentication yes`, with an earlier root-only cloud-init drop-in that non-root cannot read. **Effective SSH policy UNKNOWN** until approved privileged read-only `sshd -T` and matching real SSH context. Keep a verified provider console path before any SSH or firewall change.
- Postflight `ss -lnt`: unchanged SSH 22 and loopback resolver listeners; no public IPAT service, K3s, database or firewall modifications. Root file system remained ~233 GiB available.

### Stage-1 implementation and tests (root action NOT YET run)

- Added reviewable `deploy/scripts/lab/stage1-root-preflight-and-toolchain.sh`, `deploy/scripts/lab/apply-stage1-from-mac.sh`, and accompanying `deploy/scripts/lab/README.md`. The Mac helper requires a verified partial external archive, clean canonical source, valid SSH key, manual **APPLY** confirmation and a sudo password entered **only into the owner's Terminal**; it installs a SHA-256-validated root-owned script before execution.
- Privileged script will perform an Ubuntu/space/account guard, validated read-only SSH preflight, root-only local backup of complete accessible SSH/apt configs and package manifest, signed Ubuntu apt index update, removal/upgrade-free simulation and **only** `build-essential` install, followed by service/compiler verification. It will **not** rewrite SSH, firewall, routing, databases, root keys or K3s.
- **Executed tests:** macOS `bash -n` for both scripts **PASS**; actual Ubuntu `bash -n` + non-root `--check` **PASS**; actual Ubuntu non-root `--apply` **correctly DENIED** (exit 7). The privileged `--apply`, actual `cc` installation and subsequent Ubuntu `cargo test` remain **NOT RUN** until the owner provides the sudo password locally.
- Updated `.github/workflows/ci.yml` to syntax-check both shell scripts; `README.md` and `docs/DEPLOYMENT.md` link the guarded bootstrap runbook. Existing architectural decisions, especially pilot sizing ADR-013, USP transport and commercial PostgreSQL HA, remain unchanged/pending as recorded.
- **Next gated action:** merge the reviewed PR after CI; owner runs `bash ~/Projects/ipat-current/deploy/scripts/lab/apply-stage1-from-mac.sh` from an interactive Mac Terminal and types sudo password locally. The helper automatically verifies compiler installation and real Ubuntu Rust tests, then reports actual results for the next milestone. Before stateful/K3s or key-only SSH changes, obtain a complete encrypted independent backup, validate the provider console rescue and agree on safe network/CNI/provider firewall design.
