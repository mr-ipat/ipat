# IPAT — Project Status

**As-of:** 2026-09-28 Asia/Jakarta
**Current milestone:** R9.2 VERIFIED SOFTWARE — real durable IMMUTABLE NONEXECUTABLE first-read intents with signed opaque CSRF session and latest four-gate PostgreSQL checks; R9.2 feature CI 36379223832 and independent post-code-main CI 36379530246 each 4/4 SUCCESS. Final code SHA 9e02ded020d697b8c5b82bc6ad3e3357c2eda2bb synced clean GitHub/owner Mac/nonroot Ubuntu26 VPS. Actual physical OLT/ONT probe, live customer MFA/BFF and full production DR remain MUST/OPEN. Detailed final evidence appended.
**Historical R7.1 state (superseded by later entries):** R7.1 original Rust ZTE C320 strict offline read-only evidence parser, owner-private non-network importer, hard-disabled firmware and large RED unfulfilled-PRD warning across all three private dashboard previews FEATURE PR #71 MERGED at code SHA d452d1be6943bb4b3685b2136ad30b587e6af1e9. Both independent feature-PR GitHub CI 36249894205 and post-feature-main CI 36250108104 completed SUCCESS in all four independent jobs (locked Rust/security/UI/OLT offline, disposable Ubuntu26 K3s and two isolated synthetic PostgreSQL recovery jobs, none deployed live). Actual clean unchanged Ubuntu 26.04.1 VPS canonical source, private GitHub and owner FileVault Mac main synchronized at feature SHA; actual VPS rustfmt and 145/145 locked OFFLINE whole-workspace Rust tests, 5/5 synthetic offline CLI permission/data tests, 5/5 source PRD red warning + disabled firmware checks and 7/7 existing R6.8 Python dashboard tests PASS. Existing R6.8 synthetic Node DOM proof rerun on Mac Node v22.22, actual authorized Mac private dashboard served LARGE red PRD alert via real HTTP 200/no-store/CSS high contrast while 3 business APIs remained HTTP401. FileVault Mac encrypted exact merged-feature-source Restic snapshot 63c7c461 independently isolated SHA256 restored plus full encrypted pack read PASS; selected historical root-readable config separately restored, NOT full VPS or real PostgreSQL PITR. Clean main production readiness 8/8 automatic gates PASS, ALL 7/7 independent external safety gates BLOCKED, production NO_GO; live VPS K3s/PostgreSQL/nftables inactive. ZTE C320 actual model/boards/running firmware, authenticated private device route and owner recovery/maintenance approval still unavailable; TC-OLT-01 physical NOT RUN, no firmware update or device changes. Separate physical blocker Issue #72 and high-risk firmware Issue #73 OPEN. Final docs-only checkpoint independent CI, exact updated source synchronization and backup to be logged immutably after docs merge to avoid recursive SHA-changing commits. Developer Mr. iPat.

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
- Privileged script will perform an Ubuntu/space/account guard, validated read-only SSH preflight, root-only local backup of complete accessible SSH/apt configs and package manifest, sanitized synthetic-context SSH policy report, signed Ubuntu apt index update, removal/upgrade-free simulation and **only** `build-essential` install, followed by service/compiler verification. It will **not** rewrite SSH, firewall, routing, databases, root keys or K3s.
- **Executed tests:** macOS `bash -n` for both scripts **PASS**; actual Ubuntu `bash -n` + non-root `--check` **PASS**; actual Ubuntu non-root `--apply` **correctly DENIED** (exit 7). The privileged `--apply`, actual `cc` installation and subsequent Ubuntu `cargo test` remain **NOT RUN** until the owner provides the sudo password locally.
- Updated `.github/workflows/ci.yml` to syntax-check both shell scripts; `README.md` and `docs/DEPLOYMENT.md` link the guarded bootstrap runbook. Existing architectural decisions, especially pilot sizing ADR-013, USP transport and commercial PostgreSQL HA, remain unchanged/pending as recorded.
- **Next gated action:** merge the reviewed PR after CI; owner runs `bash ~/Projects/ipat-current/deploy/scripts/lab/apply-stage1-from-mac.sh` from an interactive Mac Terminal and types sudo password locally. The helper automatically verifies compiler installation and real Ubuntu Rust tests, then reports actual results for the next milestone. Before stateful/K3s or key-only SSH changes, obtain a complete encrypted independent backup, validate the provider console rescue and agree on safe network/CNI/provider firewall design.

### R4.2 supply-chain guard and Ubuntu diagnostic follow-up

- GitHub PR #14 merged the first guarded Stage-1 helper into canonical `main` (commit `61f8be2a0a2b2d55a3cbf4201597a19a4d16aacd`). Final PR and post-merge GitHub Actions on Ubuntu 24.04 **passed** shell script syntax, rustfmt and the existing locked **23 synthetic/unit tests**. Verified that the private GitHub source and the Ubuntu non-root local Git checkout initially matched that exact commit. This is CI for source; it does NOT prove a root bootstrap ran.
- A further **non-standard diagnostic**, using the bundled `rust-lld` linker without installing compiler dependencies, **FAILED** on the actual Ubuntu server because development linker libraries (including `-lgcc_s`) were unavailable. No production compiler override was installed; the standard `build-essential` gate still applies. This experiment created only ordinary unprivileged Cargo build cache.
- To avoid a privilege-escalation supply-chain problem, the Mac Stage-1 helper now refuses to auto-update code: it requires a clean local `main` matching the current private GitHub `main`; it checks the **pre-reviewed** root-script SHA-256 `c2f65afb09d0a8f2509f8e6e27530e5d3d7c1c692f502513835620738414e8e4` and the **pre-verified partial archive** SHA-256 `2d74782d0e418262701107be01e91c0ea87d3b7406803ca58acdb91a57338a77`. An unexpected change blocks privileged execution pending new review; the script is still checked a second time after installation in a root-owned destination.
- **No snapshot remains available; privileged package installation, SSH policy changes, network hardening and system services remain NOT EXECUTED.** Root script `--check` and non-root denial were already actually verified. Review provider-console rescue and full encrypted offsite backup before proceeding beyond this narrow compiler-bootstrap gate.

## 15. R4.3 — Stage 1 complete on actual Ubuntu; key-only SSH Stage 2 prepared (2026-09-25)

**Completed actual Stage-1 execution (owner's Mac sudo prompt):**
- The operator ran `deploy/scripts/lab/apply-stage1-from-mac.sh` and supplied a terminal transcript including `STAGE1_LOCAL_TESTS_PASS`; the assistant independently reconnected in a fresh strict-host-key SSH session and confirmed `build-essential 12.12ubuntu2.26.04.2`, `cc (Ubuntu 15.2.0-16ubuntu1) 15.2.0`, `cargo 1.98.1`, and the original `ssh` service `active`. Stage-1 created a root-owned local backup directory (`/var/backups/ipat-lab`, mode `0700`); its full internal file integrity has NOT been independently audited.
- The assistant independently ran `cargo fmt --all -- --check` and `cargo test --workspace --locked --offline --quiet` on the **actual Ubuntu 26.04.1 LTS host**, using private repository revision `cf8586f271695150e5f806656e0b2dcd83053995`. **PASS:** formatting and 23 synthetic/unit tests (4 authz, 2 control API, 15 CWMP Inform fixture, 2 tenant). All 0-test doc/crate groups passed too. These do **not** test physical devices, real ACS sessions or native USP.
- Root-stage sanitized effective `sshd -T -C` report exists and was read through the `openai` account: synthetic-loopback `openai` and `root` both showed `permitrootlogin yes`, `passwordauthentication yes`, `pubkeyauthentication yes`, `kbdinteractiveauthentication no`. This confirms security exposure in those evaluated contexts; real remote-user or source-address Match contexts still require independent verification.
- Postflight SSH remains accessible through the original approved Mac Ed25519 key; port 22 and resolver are the only sampled TCP listeners. Disk ~232 GiB free after Rust builds. No firewall, routing, port, user-key or SSH policy modification had been executed.

**Stage 2 prepared but NOT APPLIED:**
- New `deploy/scripts/lab/stage2-ssh-key-only.sh` and `apply-stage2-from-mac.sh` propose only an early lexical OpenSSH drop-in with `PermitRootLogin no`, `PasswordAuthentication no`, `PubkeyAuthentication yes` and `KbdInteractiveAuthentication no`. This globally disables password-based SSH for ALL accounts if later applied; no firewall or port change is part of this stage.
- Before any change, the script creates a root-only local backup and arms a six-minute systemd-run automatic rollback; it requires effective policy checks and a NEW independent key-only SSH connection before owner-entered confirmation can disarm the timer. A reboot during the transient timer window is unsafe. The Mac helper requires separate explicit console and global-password-SSH acknowledgements, reviewed root-script hash, private Git main match and the preserved partial Mac backup hash. The provider console must be TESTED by the owner, not merely available in a menu.
- **Tests run now:** macOS Bash parse for both Stage-2 scripts **PASS**, source hash matches helper **PASS**, five Python standard-library static safety-contract tests **PASS**, Ubuntu rootless Stage-2 `--check` **PASS**, and Ubuntu non-root `--apply` correctly denied. **NOT RUN:** real root application, transient rollback drill, post-application independent SSH verification, timed confirmation, complete encrypted offsite recovery/restore, firewall or K3s.
- New files: `deploy/scripts/lab/stage2-ssh-key-only.sh`, `deploy/scripts/lab/apply-stage2-from-mac.sh`, `deploy/scripts/lab/STAGE2-SSH.md`, `deploy/scripts/lab/test_stage2_review.py`; updated project docs, CI and `.gitignore`. **No architecture ADR changes approved.**
- **Required next gate:** Stage-2 source merged as PR #16 after successful restricted CI, and exact private main SHA synchronized and verified on the Mac and Ubuntu. Next the owner must TEST VPS provider-console recovery and invoke the reviewed Mac Stage-2 helper, supplying their Linux sudo password ONLY locally. If any new SSH test fails, DO NOT confirm and allow automatic rollback. Verify the actual result before planning provider firewall/CNI/K3s. No snapshot and no complete offsite recovery copy exist; keep all stateful customer workloads disabled.


### R4.3 post-merge handoff verification (2026-09-25)

- GitHub PR #16 merged into private `main` as `774e707d120fa3c96310ce8a7e2ab8bedd9add8d`; CI workflow `36111890193` **PASS** (five Stage-2 static safety-contract tests, shell syntax, Rust formatting and 23 locked Rust unit/simulator tests) on GitHub-hosted Ubuntu 24.04.
- Mac Git main and the user's non-root Ubuntu 26.04.1 checkout were both hash-verified against that same private main commit via an authenticated Git bundle, without copying GitHub tokens/SSH private keys.
- **Independent re-run on the ACTUAL Ubuntu 26.04.1 host** after that merge: `cargo fmt --all -- --check` **PASS**, five Python standard-library Stage-2 static tests **PASS**, `cargo test --workspace --locked --offline --quiet` **PASS** (4 + 2 + 15 + 2 = 23 tests, zero failures; zero-test doc groups also passed). A separate Stage-2 rootless `--check` completed successfully. Verified OpenSSH service still active and no Stage-2 rollback marker exists.
- **Current live security state:** Stage-2 SSH hardening script is PREPARED but NOT APPLIED. The prior root effective policy report documented password/root SSH access in synthetic contexts. The Mac-local user helper will request explicit tested-console confirmation, consent to globally disable SSH passwords, and Linux sudo credentials interactively, followed by a fresh key-only connection test and six-minute rollback disarm.
- **Outstanding blockers:** no provider snapshot, no complete encrypted independent offsite recovery backup or tested full restore, firewall/provider ACL and K3s network design pending. Do NOT yet deploy stateful PostgreSQL/customer data, public CWMP/USP ingress, or production cluster. All actual physical vendor capabilities remain UNTESTED; no ADR was approved or amended in R4.3.

## 16. R4.4 — Stage-2 SSH hardening verified; K3s read-only preflight (2026-09-25)

- **Owner-provided privileged execution evidence:** `/root/ipat-bootstrap/stage2-ssh.sh` passed root SHA validation and read-only preflight, applied the managed key-only SSH snippet with a six-minute automatic rollback armed, then established an independent public-key SSH session. The owner entered `CONFIRM`, successfully stopped the rollback timer and completed the final fresh public-key login. `FIREWALL_AND_SSH_PORT_UNCHANGED` was reported by the staged script.
- **Assistant independently reverified:** from the authorized Mac, a *new non-multiplexed* strict-known-host SSH session using **public key only** succeeded as `openai`; the root-owned 0644 managed drop-in includes `PermitRootLogin no`, `PasswordAuthentication no`, `PubkeyAuthentication yes` and `KbdInteractiveAuthentication no`; `/run/ipat-ssh-stage2.active` is **absent**, SSH service **active**, and sampled TCP listeners remain port 22 and loopback resolver. A separate diagnostic without a public key offered **only publickey** for the real `openai` connection and was denied without sending a password. This verifies access from this tested source, NOT all possible SSH Match contexts or full firewall posture.
- **Actual rootless K3s host observation:** new `deploy/scripts/lab/k3s-readonly-preflight.sh --report` successfully ran against Ubuntu 26.04.1 KVM, 16 logical CPUs, ~30-GiB-class RAM, ~232-GiB remaining ext4 root, zero swap, cgroup v2 and synchronized time. `ip_forward=0`, and `vxlan`/bridge/filter modules were not currently visible (do not infer unsupported). No existing sampled K3s TCP/overlay UDP listener; K3s and kubectl absent on PATH. Guest `eth0` private; provider public ACL and NAT exposure **cannot be validated from guest data alone**.
- **Security/recovery constraints:** user reports VPS provider has **no snapshot**; the previous Mac archive is a PARTIAL configuration copy, not a complete independently encrypted and tested restore solution. **No host firewall, provider ACL, CNI, K3s, PostgreSQL, user-data, or public ACS/USP ingress modification was made in this milestone.** Keep persistent customer data disabled and the commercial pilot ADR-013 sizing baseline unchanged.
- **Changed repository paths:** `deploy/scripts/lab/k3s-readonly-preflight.sh`, `deploy/scripts/lab/test_k3s_readonly_review.py`, `docs/LAB_K3S_READ_ONLY_2026-09-25.md`, `docs/PROJECT_STATUS.md`, `docs/SECURITY.md`, `docs/DEPLOYMENT.md`, `deploy/scripts/lab/STAGE2-SSH.md`, `deploy/scripts/lab/README.md`, `README.md`, and `.github/workflows/ci.yml`. `DECISIONS.md` remains unchanged; ADR-017 network/datastore and ADR-008 backup/secret handling remain OPEN.
- **Verified checks before PR:** Mac `bash -n` and four K3s preflight static guard tests passed; real Ubuntu rootless `--report` completed without mutation. Existing five Stage-2 static tests, four new K3s static tests, and 23 Rust synthetic/unit tests were subsequently verified against the merged source revision (see R4.4 post-merge evidence below).
- **Next security gate:** independently verified provider firewall/management-source allowlist and tested provider console recovery; separate encrypted **independent off-host** config/datastore backup and isolated restore proof. Select/approve private K3s overlay and datastore as an explicit ADR-017 decision; only afterward prepare a SHA-pinned, rollback-tested **separate** K3s install milestone. K3s official documentation warns against exposing Flannel VXLAN UDP/8472 publicly; TCP/6443 and kubelet metrics similarly require private source restrictions.


### R4.4 post-merge test and repository confirmation

- GitHub PR #19 (live key-only SSH evidence + K3s read-only prerequisite report) **MERGED** to private `main` at `1f8574481d6f0c4904509465186faeb1462830fe`.
- GitHub CI run `36113101822` on the PR and **post-merge main run `36113206429` both SUCCESS**: Stage-2 five static tests, K3s four static guard tests, shell syntax, formatting and the 23 locked Rust synthetic/unit tests on GitHub-hosted Ubuntu 24.04.
- The assistant synchronized the **exact same private GitHub main SHA** across the Mac and real Ubuntu 26.04.1 host using a verified temporary Git bundle, without installing a GitHub token or copying an SSH private key to Ubuntu. The actual host passed a fresh rootless K3s `--report`, Rust `cargo fmt --all -- --check`, nine Python static tests (5 + 4), and **23 Rust tests, zero failed** via `cargo test --workspace --locked --offline --quiet` after the merge. SSH remained active, and no live Stage-2 rollback marker or visible revert timer remained.
- **R4.4 completion is READ-ONLY / LAB SOURCE ONLY.** Provider public firewall, full encrypted independent offsite backup plus isolated restore test, explicit ADR-017 network/datastore decision, K3s installation, PostgreSQL, native USP runtime and actual device interoperability remain BLOCKED/NOT DONE; no firewall or cluster changes are authorized by this milestone.

## 17. R4.5 — verified temporary Mac restic backup, isolated restores and external provider network preflight (2026-09-25)

**Actual work completed by assistant using authorized Mac (no Ubuntu root/provider rule changes):**
- Installed `restic 0.19.1` on the user's Mac with Homebrew. Generated a high-entropy password using local cryptographic randomness; stored it only in the user's macOS login Keychain (service `id.ipat.lab.restic.backup.v1`, account `ipat-lab-backup`), never printed the password or committed it to Git. Restic reads it via `RESTIC_PASSWORD_COMMAND`. Created mode-0700 encrypted repository `~/IPAT-secure-backups/restic-lab-v1` (ID prefix `8b539c4534bf`).
- Snapshot `c5447733`: encrypted prior **PARTIAL** Mac-local VPS configuration archive. Ran a real, *separately restored* test under a private disposable Mac directory, compared actual SHA-256 against the earlier known verified archive checksum `2d74782d0e418262701107be01e91c0ea87d3b7406803ca58acdb91a57338a77`, and checked the tar could be listed. **PASS**.
- Snapshot `f38403d0`: streamed Git `main` revision `d5ac374eec5fff27a2f58bbc1f8887768050e2f4` using `restic backup --stdin-from-command -- git archive`, which propagates command failures. Restored into a separate private disposable directory; independently computed the original Git archive SHA-256 and verified it matches the decrypted/restored archive byte-for-byte; brief/decisions/status files present. **PASS**. Re-backup the newly merged source after the current R4.5 PR lands (snapshot above only covers its stated revision).
- Ran real `restic check --read-data` across **both snapshots and four packs**: **PASS, no errors**. Removed temporary restored plaintext test directories; retained the old source PARTIAL plaintext archive from prior milestones pending explicit safe retirement.
- **Security limitations:** `fdesetup status` on Mac returned **FileVault Off**. The Keychain-held passphrase and encrypted restic repository reside on the **SAME MAC**, with no independently protected passphrase escrow or separate backup copy. The snapshots cover only the earlier partial readable VPS config and committed Git source; VPS root-only files, private SSH host keys, PostgreSQL and future K3s/datastore/server token are **NOT BACKED UP** in this milestone. This is useful temporary, separate-host laboratory redundancy; NOT production DR and NOT PRD AC-07 database restore.
- Public external network check from authorized Mac: DNS `ipat.fadly.id` returned IPv4 `202.162.204.121`, no DNS AAAA in sampled query; **TCP/22 accepted**. TCP 80, 443, 6443, 10250, 2379 and 2380 did not establish connections. Without listeners, this **cannot prove external provider Managed Firewall filtering**. UDP/8472 was not tested; IPv6 provider rules uninspected; host firewall, external provider panel rules and NAT details UNKNOWN.
- External-provider procedure notes confirms the Cloud VPS management UI exposes IPv4/IPv6 inbound/outbound custom rules, VNC Console and Rescue Mode. The official `Reset Rules` feature restores **allow-all IPv4 and IPv6** and must NOT be used as a security hardening shortcut. **No external provider panel/provider firewall rules or Ubuntu firewall were changed** during these tests.
- Changed code/docs: `deploy/scripts/lab/backup-mac-restic.sh`, `test_backup_mac_review.py`, `external-readonly-network-check.sh`, `test_external_network_review.py`, `docs/LAB_ENCRYPTED_BACKUP_EDGE_R45.md`, `docs/PROJECT_STATUS.md`, plus `docs/SECURITY.md`, `docs/DEPLOYMENT.md`, `deploy/scripts/lab/README.md`, `README.md` and CI. Mac initial static checks **PASS**: five backup-contract and four external provider source-inspection tests; the read-only external provider probe was actually run. Newly generated backup script `--verify` requires a clean `main` and must run again after the R4.5 PR merge.
- **No architecture ADR automatically approved:** user selected a temporary Mac backup destination for this restricted lab, not a final ADR-008 secret/backend storage strategy; ADR-010 production PostgreSQL PITR, and ADR-017 cluster network/provider/datastore remain OPEN.

**Next change gate:** obtain the owner's redacted **external provider Managed Firewall rule inventory** for both IPv4 and IPv6 and evidence of working VNC Console rescue login; the owner should enable Mac FileVault and safely escrow the restic recovery password outside this Mac *without sharing it*. Build a separately reviewed owner-interactive root-only configuration backup that streams sensitive data directly into encrypted restic with a successful isolated restore. Only after these gates can an approved provider ACL transaction and isolated/disposable K3s laboratory installation be considered. Until then, do not activate public Kubernetes/CWMP/USP ingress or stateful subscriber/tenant data.


### R4.5 post-merge independent backup/CI verification (2026-09-25)

- PR #22 merged into GitHub private `main` `deecfed13bfa6d453816e89af0b3592c8a541067`; its PR and post-merge GitHub main workflow `36115936092` **SUCCESS**, including nine newly added static Mac-restic/external provider source checks, previous nine static SSH/K3s checks, formatting and 23 locked Rust unit/simulator tests on GitHub-hosted Ubuntu.
- The freshly merged clean `main` was re-backed-up using the committed actual Mac helper, producing latest-encrypted-source snapshot `2ee0cc82` (the older initial source snapshot `f38403d0` remains as history). Actual helper `--verify` succeeded on **3 snapshots/6 packs**, decrypting and isolating both the latest source and earlier partial configuration, checking independent bytewise SHA-256 and tar entry integrity. Temporary test plaintext was deleted.
- Verified private GitHub, Mac and actual Ubuntu 26.04.1 `main` commit checksums match the merge commit via short-lived Git bundle; SSH service active and no Stage-2 rollback marker. **Mac FileVault still OFF**. Local Keychain password and repository lack independent escrow/second copy; ROOT-ONLY VPS config, K3s token/datastore and any future PostgreSQL database remain UNSAVED. external provider panel firewall rules remain UNKNOWN; no firewall or K3s installed/modified.
- A later docs-only merge will advance Git `main`; re-run the Mac backup helper against that new HEAD and log its snapshot in GitHub issue #18 without recursively amending this evidence. Full restore and AC-07 acceptance remain **BLOCKED**.

## 18. R4.6 — owner screenshots and independent FileVault/external provider dual-stack check (2026-09-25)

- **Owner evidence:** external provider panel screenshot displays global IPv6 address and `allow-all` security group permitting **all inbound IPv4 `0.0.0.0/0` and inbound IPv6 `::/0`**, as well as all outbound for both families. The image does NOT show whether `allow-all` is shared by other VMs/interfaces, whether additional security groups exist, or the full per-VPS Managed Firewall/NAT policy. The owner also provided a Mac FileVault screen reporting **encryption finished** and a recovery key set, but separate recovery-key custody was not verified.
- **Independently verified on authorized Mac:** `fdesetup status` now reports **FileVault is On** (supersedes older R4.5 FileVault OFF measurement); the Keychain password still decrypts the existing Restic repo. After enabling FileVault, the assistant re-ran the actual committed `backup-mac-restic.sh --verify` on current reviewed source `main`: `restic check --read-data` **PASS** over **4 encrypted snapshots/8 packs**, isolated temporary restore of latest canonical source and historic partial config matched independently computed SHA-256, and temporary test directories were removed. Same-Mac Keychain + backup location remain a correlated failure risk; independent password escrow and second encrypted copy are **NOT VERIFIED**.
- **Independently verified on Ubuntu:** actual guest `eth0` holds global IPv6 `2401:2900:2fc::114/128` and a default IPv6 route; SSH listens on `0.0.0.0:22` and `[::]:22` with previously verified root-owned key-only directives, even though earlier sampled DNS had no AAAA. Root login/password authentication is disabled in the installed host drop-in, but open public dual-stack provider ingress is still a significant risk for future listeners and source-agnostic SSH brute-force attempts.
- **Mac management-source observation:** two separate direct IPv4 echo checks agreed at inspection time; this **does not prove stable assignment**. Strict external Mac IPv6 source check did NOT establish a usable IPv6 management address; do not guess a /128. No externally reachable IPv6 probe from an independent source was performed. The tested Mac IPv4 /32 is only a candidate to confirm immediately before a staged firewall transaction.
- **R4.6 source prepared:** `docs/EDGE_SECURITY_GROUP_R46.md` includes exact current allow-all exposure, family-specific proposed default-deny inbound, documented rule/VM attachment inventory, fresh Mac IPv4 source + tested VNC console gate, and provider-specific rollback. New non-mutating `deploy/scripts/lab/edge-r46-gate-check.sh --report` and four accompanying static tests passed syntax and Mac run. No external provider panel access, host firewall modification, security-group change, sudo action, root-only backup or K3s install was executed. Existing Open ADR-008/010/017 remain unchanged.
- **Required follow-up before live network change:** (1) owner independently confirms **actual VNC console login**, not just that a console screen opens; (2) owner confirms the Keychain-held Restic password has been securely escrowed outside this Mac without sharing it; (3) review affected `allow-all` group attachment and all IPv4/IPv6 per-VPS rules to avoid disrupting another VM; (4) prepare and separately verify a privileged root config backup streamed directly into encrypted Restic without persisting plaintext on Mac; (5) only then perform one monitored provider rule change and fresh dual-stack testing with documented manual rollback. Avoid external provider `Reset Rules`, which official docs show restores global allow-all. K3s/PostgreSQL remain BLOCKED.

## 19. R4.7 — owner restrictions + tested failure-aware encrypted root-stream preparation (2026-09-25)

**Owner-confirmed input, not independently verified beyond cited probes:** real external provider VNC Console Linux login **UNSUCCESSFUL**; the permissive `allow-all` security group is **SHARED among multiple VPSs**; Restic password is safely escrowed outside Mac and available on another device (**OWNER ATTESTATION** only). Never modify shared `allow-all` rules. The live source-level facts from R4.6 (FileVault ON, global guest IPv6/default route, dual-stack SSH binding) still apply. No provider or guest firewall rule, SSH setting, package or K3s/DB service was changed by R4.7.

**Independent read-only VNC prerequisite inspection:** actual Ubuntu 26.04.1 `getty@tty1.service` returned `active`, `openai` reported password status `P` and interactive shell `/bin/bash`. This does not prove browser VNC input/authentication works; owner must retest the correct VPS browser Console with Ubuntu `openai` account credentials, not an SSH key passphrase, and seek external provider Support for VNC access if still unsuccessful. Do NOT trigger Rescue Mode (it reboots) or weaken the existing key-only SSH to debug console.

**Code and scope prepared:** `deploy/scripts/lab/root-config-stream.py` and `mac-root-config-backup.sh` generate a fixed-purpose Linux `sudo -S` read-only tar stream from selected root-accessible SSH/sudoers/network/cloud/apt config directly over strict authenticated SSH into the existing encrypted Restic Mac repo, without writing an unencrypted root archive during backup or putting the sudo password into CLI arguments/environment/Git. The sudo password must be typed *locally in the owner's interactive Mac Terminal*. The sensitive root archive intentionally EXCLUDES SSH host private keys and ALL real K3s/PostgreSQL/app state; thus it is a selected-root-config backup, not full host/PRD AC-07 DR. Its runner decrypts snapshots only into a private throwaway Mac directory to verify a real isolated restored tar, then removes plaintext.

**Actually executed test evidence BEFORE privileged backup:**
- `python3 -m py_compile` + `bash -n` **PASS**; actual target Ubuntu `bash -n` parsing of the fixed root tar payload **PASS** without running as root; five Python static source-contract tests **PASS**.
- Actual strict `ssh -T` `--smoke` from the Mac streamed a user-readable Ubuntu tar as **binary** via the same Python and Restic `--stdin-from-command` path; saved encrypted test snapshot `81da8cc6`, and restored it to a separate private Mac test directory. Valid gzip/tar archive entries and full repository data read **PASS**.
- A Mac cleanup trap scope issue appeared after the first smoke's successful restore and was corrected. The first unprivileged test's abandoned `restore-root-test.*` directory was inspected and explicitly deleted. Rerun with the repaired cleanup generated snapshot `ce1c8807`, successfully restored it, validated its gzip/tar listing, checked all Restic packs and **removed** the throwaway plaintext test directory; **PASS**.
- Actual forced `--fail-smoke`: intentionally incomplete remote stream, SSH producer exit 42 propagated as nonzero from the Python wrapper. Restic correctly **FAILED** the backup and a subsequent snapshot query verified **NO snapshot with the negative-test tag** was created. The documented `--stdin-from-command` failure check is materially safer than a blind pipe to Restic `--stdin`.
- **Not yet run:** real root `sudo` stream; encrypted privileged/root-config snapshot; independent actual root-config restore; any full-root SSH host-key/disk snapshot or PostgreSQL/K3s recovery. These require the reviewed helper merged into GitHub, Mac/VPS synchronized, and the owner's one-time LOCAL Ubuntu sudo prompt.

**Repository changes in this milestone:** `root-config-stream.py`, `mac-root-config-backup.sh`, `test_root_config_stream_review.py`, `docs/ROOT_CONFIG_STREAM_R47.md`, updates to CI, `README.md`, `docs/SECURITY.md`, `docs/DEPLOYMENT.md`, `docs/EDGE_SECURITY_GROUP_R46.md`, lab runbook and status. No ADR was auto-approved: ADR-008 encrypted backup/secret design, ADR-010 PostgreSQL HA/PITR and ADR-017 CNI/provider private network selection remain OPEN where recorded.

**Next operator action after PR/CI merge:** from Mac interactive Terminal only, run `bash ~/Projects/ipat-current/deploy/scripts/lab/mac-root-config-backup.sh --backup-root`, confirm independently escrowed Restic password and enter Ubuntu account `openai` sudo password when prompted. The script then directly encrypts the root config stream and independently restores/validates the selected root archive, without touching external provider rules. Ask for final nonsecret test lines; reverify snapshot and update status/issue #18 in a separate milestone. Keep K3s/firewall/production services blocked until actual VNC/equivalent rescue access and **dedicated IPAT-only SG** plus provider IPv4+IPv6/network restore review.


## 20. R4.7.1 — visibility fix after the owner's canceled backup attempt (2026-09-25)

The first interactive privileged backup attempt appeared stuck showing Restic at zero bytes without a visible sudo prompt. Inspection of process metadata showed Restic's Python producer running without a child SSH process. After the owner canceled the attempt, independent checks found neither process running and confirmed that no root-config snapshot had been saved.

The corrected Mac helper now prints clear guidance to the controlling terminal that the required credential is the existing Ubuntu `openai` sudo password, not a root SSH login or key passphrase. It uses Restic's verified quiet flag during that specific privileged capture to avoid progress output overwriting the prompt. The existing public-key-only SSH and server configuration remain unchanged.

Mac parser checks and six source safety tests passed. A fresh non-root strict SSH stream was encrypted in Restic snapshot `21c5cbde`, with a full data-integrity check of nine snapshots/sixteen packs, successful isolated archive restoration and cleanup. The actual privileged root-config backup and its independent restore have NOT been completed. external provider VNC login is still unavailable; the shared allow-all provider group and all host firewall/K3s configurations remain unchanged. The owner must run the updated reviewed helper after merge and enter their Linux sudo password only in their own Mac Terminal.


## 21. R4.8 — actual root-config backup VERIFIED and vendor-neutral native firewall policy proposed (2026-09-25)

**Owner-provided actual privileged result:** encrypted Restic selected-root-config snapshot `abaa9827` successfully created by a locally authorized `openai` sudo read-only tar stream. Independent assistant re-verification inspected the one matching `root-config` snapshot and executed the committed `mac-root-config-backup.sh --verify-root` on the Mac with no Linux sudo credential: `restic check --read-data` passed across **11 snapshots/20 packs**, the snapshot was restored to an isolated private temporary directory, gzip/tar listing passed, root-only `etc/sudoers` and expected managed SSH key-only directives were present, and temporary plaintext was removed. **PASS for selected root-readable configuration archive recovery ONLY.** This does not include SSH host private keys, whole-VPS filesystem, actual database records, K3s datastore/server tokens, or an independently rebuilt/restarted restored node; PRD AC-07 remains NOT DONE.

**Product boundary explicitly clarified by owner:** do not include a named hosting provider or integrate external hosting-provider firewall APIs into IPAT. This R4.8 change renames previously branded tracked scripts/tests/documents to provider-neutral `external`/`edge` names, removes provider-specific URLs from current project docs and updates crosslinks and test references; exact technical observations (IPv4/IPv6 allow-all ingress, shared external security group, and unavailable independent console) remain in generic historical reports. Old Git commit history is preserved and is not secretly rewritten.

**Proposed new feature, NOT an approved live network operation:** native IPAT Ubuntu host-firewall architecture in `docs/FIREWALL_CONTROL_PLANE.md` and `PROPOSED` ADR-018; no external provider API dependency. Pure Rust `crates/firewall-policy` validates in-memory per-service/source-CIDR proposals for explicit IPv4/IPv6 management, separately verified private-cluster services, and future verified public TLS. It rejects globally allowed public SSH/K3s ingress, duplicate/invalid rules, missing management source, unverified private/TLS trust scopes; every output has `executable=false`. There is **no privileged agent, nftables execution, firewall UI/API, network mutation or approval implementation yet**. A real agent would be separate, root-isolated, audit/ABAC/two-person controlled and requires tested console recovery, safe native nftables/CNI coexistence and independent dual-stack rollback drills. ADR-017 K3s network still OPEN.

**Executed pre-merge tests:** verified 28 existing provider-neutral Python static lab contract tests on Mac after renaming; on an isolated actual Ubuntu 26.04.1 Rust checkout with the proposed workspace crate, `cargo fmt --all -- --check` **PASS** and `cargo test --workspace --locked --offline --quiet` **32/32 PASS** (23 pre-existing synthetic/unit + nine new native firewall policy validator tests). `Cargo.lock` changed solely to register the local new crate, without dependency upgrades. These tests exercise pure policy validation, not host nftables/kernel packet filtering or physical ISP vendor support. Final GitHub PR/CI, post-merge exact source synchronization and fresh Mac backup of latest source must be verified as a separate finalization gate.

**Outstanding hard stops:** actual out-of-band console login not established; externally shared permissive security group must NOT be edited by IPAT; approved firewall execution safety design, K3s CNI/ADR-017, independent full host/DB recovery and physical interoperability are all NOT DONE. No guest firewall, external security group, K3s/DB/tenant services have been changed during this milestone.


### R4.8 reviewed source merge, real Ubuntu regression and dual Restic restore

- PR #27 **MERGED** to private `main` as `620651aedfbd273a35f5493ab2004283a4c9710e`; the PR and actual post-merge GitHub Actions workflow `36135626489` were **SUCCESS**. Current tracked source/docs have no references to the formerly named hosting provider, including filenames and URLs; historical Git commits remain unchanged for audit.
- After merge, the exact private `main` SHA was independently matched on the authorized Mac and actual Ubuntu 26.04.1 VPS using a short-lived, verified Git bundle. On that real VPS, `cargo fmt --all -- --check` **PASS**, `cargo test --workspace --locked --offline --quiet` **32/32 PASS** (23 pre-existing synthetic/unit + 9 new pure host firewall proposal validator), and all **28 provider-neutral Python lab static tests PASS**. SSH remained active with no pending Stage-2 rollback marker. No host firewall or K3s was changed.
- Latest source commit `620651...` was captured into the Mac encrypted Restic repo as snapshot `49f546b4`. Independent `backup-mac-restic.sh --verify` restored this exact source archive and earlier partial user-readable config with SHA-256 checks **PASS**; independent `mac-root-config-backup.sh --verify-root` separately re-restored actual privileged selected-root-config snapshot `abaa9827` (root `sudoers` + managed key-only SSH config) **PASS**. Full repository `restic check --read-data` covered **12 encrypted snapshots/22 packs**, no errors; temporary plaintext test directories removed.
- The older source/partial-only helper emitted an obsolete warning claiming the root configuration was entirely unbacked up. R4.8 documentation/utility follow-up updates the warning to distinguish **source+historical partial** verification mode from the separately and actually verified **selected root-config** archive. Full VPS/host SSH key, PostgreSQL data and K3s datastore/server-token recovery remain NOT VERIFIED.
- R4.8 acceptance is limited to vendor-neutral **source and non-executable dry-run firewall policy** plus partial encrypted root archive recovery. ADR-018 remains PROPOSED; ADR-017 K3s CNI/network still OPEN, and out-of-band console login remains unverified. Firewall/cluster activation is therefore still blocked. A later docs-only merge changes the canonical Git hash; re-backup that new latest source revision without recursively amending status.


## 22. R4.9 — native Rust offline CWMP admission with synthetic authenticated peer (2026-09-25)

**Scope:** Underlying original CWMP parser from R4 accepts bounded SOAP 1.1/CWMP 1.0 synthetic Inform; R4.9 adds `crates/cwmp-admission` as a separate, **non-networked** Rust trust boundary. A device enrollment binds one `tenant-core::TenantId`, exact OUI + ProductClass + SerialNumber and a pinned synthetic client-SPKI SHA-256. `AuthenticatedPeer` intentionally lacks any publicly accessible constructor; synthetic authenticated peer construction exists only inside Rust unit tests, not in a running gateway. The admission registry rejects cross-tenant, unknown device, mismatched peer, duplicate device/cert, malformed XML, missing CWMP ID, duplicate correlation, concurrent same-device admission, exhausted bounds and incorrect session-close peer. Only successfully matched trusted in-memory test proofs produce an `InformResponse` and ephemeral lease.

**Actual independent pre-merge test evidence:** 11 new Rust synthetic admission tests plus 32 existing unit/simulator tests **PASS** on a separate real Ubuntu 26.04 checkout, with no application listener or privileged operations. `cargo fmt --all` ran on the isolated checkout and the formatted source and minimal new-crate `Cargo.lock` entry were copied back to the Mac branch. Three Mac Python static trust-boundary assertions passed. The existing physical device matrix remains `untested`, while the CWMP unit/simulator scope is recorded separately. The real Github CI and post-merge actual Ubuntu re-test/synchronization will be recorded when verified.

**Non-claims:** This module has no real TLS/mTLS verifier, public TCP listener, live/device enrollment database, durable sessions/anti-replay, retry-safe CWMP state machine, parameter discovery/GetParameterValues, real SOAP faults, USP runtime, real ONT/OLT integration or actual tenant UI. R4.9 is **partial S1-04 implementation only**, not satisfaction of full FR-009/FR-010, AC-03 or production ACS claims. An externally controlled header or SOAP ID never establishes authentication. No firewall, K3s, provider service configuration, host SSH policy or databases were changed; out-of-band console recovery remains unverified.

**Files added/changed:** `crates/cwmp-admission/{Cargo.toml,src/lib.rs}`, `Cargo.toml`, `Cargo.lock`, `deploy/scripts/lab/test_cwmp_admission_review.py`, `.github/workflows/ci.yml`, `docs/CWMP_ADMISSION_R49.md`, `docs/PROJECT_STATUS.md`, `docs/{ARCHITECTURE,SECURITY,DEVICE_MATRIX,SPRINT_BACKLOG,DEPLOYMENT}.md`, `README.md`, `apps/cwmp-gateway/README.md`. No ADR baseline was changed; original ACS and distinct native USP controller remain binding.

**Next:** after this PR passes CI, sync canonical GitHub/Ubuntu/Mac source, independently re-run all locked Rust tests and lab contracts, create an encrypted backup of the latest merged source and separately reconfirm the selected-root-config snapshot integrity. Then implement a verified transport adapter in an isolated private simulation environment without exposing public traffic; native USP simulator work is next independent product track. Real K3s/firewall rollout remains blocked by independent recovery and ADR-017 network checks.


### R4.9 verified GitHub merge, live Ubuntu tests and encrypted recovery

- CWMP offline synthetic tenant/peer admission PR **#30** merged to private `main` `0e354e5445fc9ed3a1d6989f4ef2ae8712923afc`. Actual GitHub workflow `36137884232` **SUCCESS**. The exact canonical SHA was independently synchronized Mac → real Ubuntu non-root workspace using authenticated temporary Git bundle; SSH still active and Stage-2 rollback not pending.
- Real Ubuntu 26.04.1 canonical source checkout: `cargo fmt --all -- --check` **PASS**, `cargo test --workspace --locked --offline --quiet` **43/43 unit/synthetic PASS** (11 CWMP admission new + previous 32), and Python lab source-contract suite **31/31 PASS**. These are purely simulator/static results, no actual ONT/TLS protocol or live network listener.
- Final reviewed R4.9 Git `main` revision was encrypted to Mac Restic snapshot `167fe785`. Full encrypted repository read-data check **14 snapshots/26 packs PASS** and separate isolated SHA-256 recovery of source, earlier user-readable config and selected privileged-root-config `abaa9827` **PASS**; private decrypted restore files cleaned.
- Original CWMP actual HTTPS/mTLS/durable session/RPC and physical interop remain NOT DONE; #8 stays open with R4.9 sub-slice evidence.

## 23. R5.0 — distinct native Rust USP Controller synthetic domain and loopback-only health (2026-09-25)

**Actual code introduced on R5.0 branch:**
- `crates/usp-core` is a separate native Rust **domain-level synthetic** controller component. It models pre-enrolled exact synthetic endpoint/tenant/pinned client identity, sealed `VerifiedAgent` + `TrustedOperator` proofs with NO public constructors, read-only constrained `Device.` fixture parameter plans, bounded one-inflight-agent correlation and peer/tenant/replay validation. It intentionally does NOT parse or emit TR-369 protobuf, USP Record/Msg or any MTP. Synthetic test identities are constructed exclusively in Rust unit tests, not reachable by callers.
- `apps/usp-controller` is a standalone Tokio/Axum Rust binary only for **explicitly opted-in loopback `127.0.0.1:3100/healthz`**; all other paths return 503 including forged agent/tenant headers. Its binary has NOT been started on the live VPS. It is NOT a public USP Controller and there is no MQTT/broker or live CPE route.
- **Isolated pre-merge actual Ubuntu 26.04 tests:** `cargo test --workspace --offline` **58/58 synthetic/unit PASS** (43 existing + twelve `usp-core` + three private router tests). After correcting an endpoint-control-char negative fixture to contain a genuine newline, `cargo fmt --all -- --check` and `cargo test -p usp-core --locked --offline` **12/12 PASS** in the isolated Ubuntu checkout. Three separate Python static source-boundary checks **PASS** on Mac. Final GitHub CI, main-branch sync, source backup and canonical post-merge retest must be verified before marking the milestone merged.
- **Unimplemented and BLOCKED:** official pinned USP protocol amendment/protobuf schemas; actual real mTLS + MTP/broker/agent trust and tenant mapping; durable session/replay; real read RPC or genuine interop agent; actual user interface and API authorization; any real device/OLT/ONT compatibility. No PRD AC-04 pass claim; ADR-007 remains PROPOSED. Actual host firewall/K3s/DB not touched because independent console recovery and data restore gates are not met.

**Files:** `crates/usp-core/{Cargo.toml,src/lib.rs}`, `apps/usp-controller/{Cargo.toml,src/main.rs,README.md}`, workspace manifest/lock, `deploy/scripts/lab/test_usp_synthetic_review.py`, CI and relevant documentation/PROJECT_STATUS. No physical device tests or migration required for this pure Rust simulator.


## 24. R5.1 — actual GitHub ephemeral PostgreSQL tenant RLS and logical restore (2026-09-25)

- Added **LAB-only** `deploy/db/migrations/0001_lab_tenant_rls.sql`: `ipat_platform.tenants`, `ipat_ops.devices/subscribers`, distinct no-login schema owner and restricted login runtime role (non-superuser, no BYPASSRLS), `ENABLE`+`FORCE ROW LEVEL SECURITY`, missing/invalid tenant scope fail-closed, composite `(tenant_id,id)` keys and same-tenant subscriber↔device FK. It is **not deployed to the live Ubuntu VPS**; trusted OIDC backend context binding and pooling remain UNIMPLEMENTED.
- Initial PR #33 real GitHub Actions `36141744263` **SUCCESS** in both jobs. Its separate disposable PostgreSQL 16.9 service used synthetic tenants and no actual customer data. Five actual PostgreSQL integration tests **PASS**: runtime missing tenant zero-row and denied platform access, two-tenant read isolation including `SET LOCAL` clearing at transaction boundary, cross-tenant writes denied, `TRUNCATE` denied, invalid UUID denied, and a binary `pg_dump` restored to a **second clean database in the same CI cluster** with SHA-256-equivalent synthetic record content and scoped RLS check. Five local static migration contracts **PASS**; added static checks to main Rust workflow for future regressions.
- Scope warning: a second database in the same temporary CI cluster is NOT a second PostgreSQL server, full HA/PITR or complete user-facing AC-07 with real operations. No trusted OIDC/runtime DB adapter, live PostgreSQL installation, network/firewall/K3s change, real ISP data or offsite DB backup executed. Release blockers from R5.0 still apply. Final source PR/CI merge, Mac/Ubuntu sync and encrypted latest-source backup should be verified before marking R5.1 documentation fully finalized.


### R5.1 verified post-merge GitHub, Ubuntu, encrypted-source and selected-root recovery

- PR **#33** merged to canonical private `main` `7b69e6eb5851eb370beec76933ea82f32d5cae88`, and actual post-merge GitHub workflow **36142084702 SUCCESS** for both Rust/static job and separate PostgreSQL 16.9 integration job. Confirmed five real Postgres RLS/new-database restore tests PASS with expected negative policy cases and synthetic restored-data checksum match; five migration static checks PASS. The CI test server is a temporary GitHub service container, **not** the live IPAT host or independent PostgreSQL PITR/HA.
- Private GitHub/Mac/actual Ubuntu 26.04.1 Git `main` commit hashes matched identically; actual Ubuntu `cargo fmt --check`, **58 Rust synthetic/unit** tests, **34 Python existing static** checks and five new migration static checks PASS. PostgreSQL client/server and K3s still **NOT INSTALLED** on the live VPS; SSH remains active and unchanged.
- Latest reviewed source snapshot `5ca4ed6a` created in encrypted Mac Restic repository for the merged main SHA; actual full data read (17 snapshots/32 packs) **PASS**, isolated latest Git source + old readable config SHA-256 **PASS**, and separate independent selected root-configuration snapshot `abaa9827` isolated restore with sudoers/SSH policy checks **PASS**. No plaintext test restore retained.
- New docs-only evidence commit will advance the Git source HEAD; subsequent latest-source backup must be repeated. Remaining hard gates unchanged: independent console access/host disaster recovery, authorized firewall/CNI decision, OIDC-backed trusted runtime and production-grade PostgreSQL deployment/HA/PITR/customer data isolation are NOT COMPLETE. No external hosting-provider firewall integration is included.


## 25. R5.2 — offline Rust evidence-led diagnostics (2026-09-25)

- New `crates/diagnostic-core` distinct from ACS/USP transports accepts validated canonical tenant, explicit POP/distribution path, normalized synthetic observations with provenance and bounded timestamps. It returns hypotheses only, with operator review required and remediation disabled. R-DIAG-01 distribution: one down link and **two distinct reachability-lost subscribers** with explicitly caller-verified topology; R-DIAG-02 one ONT LOS plus independent uplink and neighbor healthy; R-DIAG-03 matching PPPoE auth reject plus independent normal optical and uplink healthy. R-DIAG-04 missing CWMP Inform-only yields insufficient evidence. Contradictory/stale/foreign-tenant inputs cannot trigger confident conclusions.
- Ten **new actual Rust offline unit tests PASS** on a disposable isolated checkout of actual Ubuntu 26.04.1; the existing Rust workspace tests passed there too. `Cargo.lock` only adds the local `diagnostic-core` with `tenant-core` dependency; no new external crates. No live CPE/OLT/router interface, alarm, operator dashboard, tenant database or production diagnostic accuracy has been tested.
- Files: `crates/diagnostic-core/Cargo.toml`, `src/lib.rs`, workspace `Cargo.toml` and `Cargo.lock`, `docs/DIAGNOSTICS_R52.md`, project source README/status/architecture/backlog updates. Planned final gate: GitHub PR CI, exact GitHub/Mac/Ubuntu source SHA sync, full actual Ubuntu tests, latest encrypted source capture and existing selected-root archive re-verification. All pre-existing K3s, independent-console and hosted firewall gating still apply.


### R5.2 independently verified GitHub, actual Ubuntu and encrypted recovery (2026-09-25)

- Synthetic diagnostics PR **#35 MERGED** into private GitHub `main` `0fa05a9d3a637019287fc3737b2927a2eae53395`. Actual post-merge GitHub workflow `36143747994` **SUCCESS**, including Rust/unit and separate PostgreSQL RLS/restore synthetic CI jobs. No changes to production networking.
- Private GitHub, Mac and real Ubuntu 26.04.1 canonical source SHA independently matched. Fresh actual Ubuntu `cargo fmt --check` and **68 Rust synthetic/unit tests PASS** (58 previous + ten new diagnostic tests), plus **34 existing Python lab static tests and five DB migration contract checks PASS**. Actual database RLS/new-database restore tests ran only on GitHub's separate ephemeral PostgreSQL service; database/K3s are NOT installed on the live VPS.
- Actual Mac encrypted Restic newest reviewed source snapshot `706129c3` for merged R5.2 SHA was independently restored with SHA-256 validation alongside the historical readable config. Real privileged selected-root-readable config snapshot `abaa9827` was separately re-restored with root-only sudoers and SSH hardening file tests. `restic check --read-data` on **19 snapshots/36 packs PASS**, zero errors; temporary test restores removed.
- This verifies deterministic **simulator-only diagnostic hypotheses** and source/config recoverability, NOT real topology authority, signal ingestion, device firmware compatibility, production diagnostic accuracy, full VPS snapshot, external DB PITR, HA, firewall agent operation or K3s network setup. The external shared perimeter remains untouched; independent out-of-band console login is not yet established. A docs-only evidence merge will advance HEAD; repeat latest-source encryption after final documentation merge.

## 26. R5.3 — non-executable sealed provisioning job simulator (2026-09-25; pre-merge)
- New `crates/provisioning-core` builds a **synthetic, memory-only** tenant/actor-scoped proposed-operation registry with bounded plans, exact idempotency keys, distinct checker approval/expiry, sealed test-only worker proofs, global per-router serialization and fenced finite leases. Any expired claimed work is `Unknown`, leaves its router quarantined and has **no automatic retry**; there is no real device command transport, executable integration or manual release API.
- Isolated nonprivileged checkout of the *actual* Ubuntu 26.04.1 VPS was created without touching its canonical `main` tree or services. Rust formatter/locked offline complete workspace unit tests **76/76 PASS** (68 pre-existing + 8 new); three new Mac Python static sealed/execution-boundary checks **PASS**. Initial review caught and fixed an unsafe router unlock on expired lease; a regression assertion now confirms the router remains quarantined.
- Changes under review: `crates/provisioning-core/{Cargo.toml,src/lib.rs}`, workspace manifest/lock, `deploy/scripts/lab/test_provisioning_synthetic_review.py`, CI, `docs/PROVISIONING_SIMULATOR_R53.md`, architecture/security/backlog/status. No database schema change or new ADR approval; existing ADR-005/006/009 remain proposed/mixed.
- Completion gates pending: GitHub PR CI including unchanged PostgreSQL ephemeral integration, merge, exact GitHub/Mac/VPS source synchronization, latest encrypted-source capture and separate selected-root Restic restore. R5.3 does **not** satisfy actual PPPoE AC-05, crash-safe durable FR-028 or verified trusted authentication; live PostgreSQL/host firewall/K3s are untouched, real out-of-band console still unavailable.

### R5.3 final source merge, live Ubuntu regression and encrypted selected recovery
- PR [#37](https://github.com/mr-ipat/ipat/pull/37) merged by reviewed squash to private `main` SHA `f4e2f8d0ed27f1bf1eeb039d2a751f0d4a5ad174`. GitHub PR workflow `36149621245` and new post-merge main workflow `36149777499` both returned SUCCESS in both Rust/static and isolated PostgreSQL 16.9 RLS/logical restore jobs. The PostgreSQL server existed **only on ephemeral GitHub CI**, not on the VPS.
- Exact GitHub and Mac `main` SHA verified. A short-lived, prerequisite-checked incremental Git bundle fast-forwarded only the clean nonprivileged Ubuntu source checkout to the same SHA, without changing running services. Actual Ubuntu `cargo fmt --check`, `cargo test --workspace --locked --offline` **76/76 PASS**, **37 lab Python static PASS** and **5 PostgreSQL migration static PASS**; Mac equivalent static source checks also passed. No physical ISP devices or protocol transports tested.
- Mac FileVault remained ON; new encrypted Restic source snapshot `18416643` captured exact SHA `f4e2f8d...`. Verified full pack reading, fresh isolated source+historical partial SHA-256 restores and **separate selected-root-readable config** snapshot `abaa9827` isolated recovery; plaintext restores cleaned. This is NOT full SSH host-key/whole-VPS recovery, K3s datastore backup, independently escrow-audited credentials or external PostgreSQL PITR.
- The merged R5.3 code is still an offline-only synthetic safety model with sealed identity proofs and no write path. No hosting-provider network control integration, external/shared allow-all Security Group mutation, host firewall apply, PostgreSQL/K3s installation, or SSH policy change occurred. Independent out-of-band console remains UNVERIFIED. A documentation-only evidence merge will require a fresh source snapshot for its new SHA.

## 27. R5.4 — synthetic PostgreSQL persisted job/outbox implementation (pre-CI)
- Scope: `deploy/db/migrations/0002_lab_job_outbox.sql` on the disposable CI PostgreSQL 16.9 service only, **not** the actual Ubuntu VPS. Job/outbox rows use force-enabled tenant+POP RLS with runtime SELECT only, same-tenant/POP router FK, per-tenant unique idempotency and unresolved-router partial unique index. Triggers restrict synthetic approval/state/lease transitions and emit each status outbox event in the same transaction. This is a trusted **synthetic test harness**, not an authenticated production worker or tenant-writable API.
- New seven-scenario Python integration suite includes denied scope/POP/direct runtime writes, cross-tenant FK, immutable plans, self-approval and expiry, ambiguous lease quarantine, rollback-atomic outbox and a *new database in the same disposable PostgreSQL cluster* logical restore. Five static migration source-contract tests check intended security controls. CI gains explicit separate execution after unchanged R5.1 tests, retaining no live PostgreSQL changes.
- Repo project/developer attribution is **Mr. iPat** in README, AUTHORS and Rust workspace/package metadata; repo-local new Git author uses Mr. iPat and the existing account's GitHub no-reply address. Prior Git history and required third-party licenses remain unchanged.
- Pending actual execution at this draft checkpoint: PR and real ephemeral PostgreSQL test results, source merge, final GitHub/Mac/Ubuntu hash synchronization, actual Ubuntu regression, fresh encrypted source backup and separate selected-root-config re-restore. No new ADR approval: shared RLS tenancy ADR-005, exact approval role matrix ADR-009 and OIDC stack ADR-006 remain PROPOSED/MIXED.

### R5.4 independently verified feature merge, actual Ubuntu tests and encrypted recoverability
- PR [#39](https://github.com/mr-ipat/ipat/pull/39) merged into private `main` `4026a77847c2d4c26c981ad843221d76c14ef5ee`; real GitHub PR workflow `36153298906` and post-merge workflow `36153496947` both SUCCESS in two jobs each. The disposable PostgreSQL 16.9 service really executed **5 existing R5.1 tests + 7 new R5.4 SQL job/outbox tests PASS**. The second synthetic database restore and scoped reread passed checksums; this is NOT independent PostgreSQL host restoration or PITR.
- GitHub/Mac/actual Ubuntu 26.04.1 reviewed source commit SHA matched. The real nonprivileged Ubuntu `main` checkout passed `cargo fmt --check`, **76 Rust offline tests**, **37 lab Python static contracts**, **5 R5.1 and 5 R5.4 SQL static contracts**. No PostgreSQL service, K3s, firewall or RouterOS equipment was installed/changed on the VPS; its pre-existing live network was not changed.
- Mac FileVault ON; encrypted exact reviewed merged-source Restic snapshot `53f2a224` independently passed latest SHA-256 isolated restore alongside the historical partial readable config. Selected privileged root-readable config snapshot `abaa9827` was **separately** re-restored and validated. Full Restic repository pack read PASS with decrypted test artifacts removed. Full VPS/disk/SSH host key backup, production DB PITR and independently rebuilt host remain UNVERIFIED.
- Project credit/metadata established **Mr. iPat** on all ten Rust package manifests, workspace, AUTHORS, README and new local Git author configuration; required third-party licenses and historical commit/audit provenance are preserved. R5.4 SQL constraints alone do **not** prove real actor verification, actual worker concurrency, true physical router ownership or a network provisioning write. PROPOSED ADRs retained, not silently approved. After the docs-only merge, repeat latest source encryption for its new SHA and record final evidence without an endless recursive documentation commit.

## 28. R5.5 read-only production infrastructure readiness (2026-09-25; execution pending review)

- Authorized live inspection reconfirmed private GitHub/Mac/Ubuntu 26.04.1 canonical source `fc322136045adca89db0e289b16008862db995a7`, latest GitHub main CI `36154111253` SUCCESS, current VPS PostgreSQL/K3s/nftables/UFW services inactive and SSH bound on IPv4 and IPv6. No host firewall/provider firewall/database/K3s modification. Prior Restic source `d5ee0eab` and selected-root `abaa9827` had succeeded in isolated restore, but complete independent full-host/PITR recovery and successful real out-of-band login remain **unverified**.
- `deploy/scripts/production/readiness.py` now implements a STRICTLY READ-ONLY Mac-to-GitHub/VPS gate report for clean canonical source hashes, SSH key-only reachability, Mac FileVault and exact-revision encrypted source snapshot. It deliberately **never authorizes** PostgreSQL, host firewall or K3s activation; real independent console login, dedicated IPv4+IPv6 perimeter, complete separate-host recovery, approved ADR-005/010/017/018 and independent PostgreSQL PITR remain blocked external proof.
- Five isolated fail-closed Python source/gate unit tests were added under `deploy/scripts/production/test_readiness.py`; four additional source-safety checks protect the CI-only `deploy/db/tests/physical_restore_ephemeral.sh` rehearsal. A separate disposable GitHub PostgreSQL job proposes a real physical base backup with streamed WAL, manifest verification and restart on a distinct isolated runner-local PostgreSQL 16 server process; its actual result remains PENDING CI at this checkpoint. This does **not** establish offsite/PITR production readiness. No privileged command on the live VPS, third-party hosting-provider API integration, live network configuration or production install has been added.
- `docs/PRODUCTION_INFRA_RECOVERY_R55.md` establishes independently evidenced prerequisites, phase-separated PostgreSQL HA/WAL/PITR, native host nftables and private multi-node K3s deployment/rollback design. This is a **proposal**, not an approved ADR or claimed real deployment. Physical ISP device compatibility remains untested. The current 16-vCPU/30-GiB/250-GiB VPS by itself is not independent production DB HA or a three-node K3s cluster.
- Pending at this source checkpoint: branch review, GitHub CI, independently verified read-only report against merged clean Git `main`, sync exact GitHub/Mac/VPS hashes, encrypted latest-source snapshot/recovery and final evidence report. Before any privileged infrastructure work, the operator must independently prove usable out-of-band rescue, full separate-host restore, dedicated perimeter controls and approved HA/private CNI topology.

### R5.5 reviewed source merge, actual Ubuntu and encrypted recoverability
- PR [#41](https://github.com/mr-ipat/ipat/pull/41) merged to private `main` `9c2187649714a00d09e0478c7ed3065a19ddaa19`. Verified actual GitHub PR CI `36156542785` and post-merge CI `36156779897` both returned SUCCESS for **all three jobs**: locked Rust/unit+static, existing PostgreSQL RLS/job logical restoration and new real disposable PostgreSQL physical recovery.
- Distinct ephemeral physical recovery CI job ran 5+7 synthetic database tests, took a PostgreSQL 16.9 `pg_basebackup -X stream`, verified its backup manifest in Docker and separately on the temporary GitHub runner, started another isolated PostgreSQL 16 server process, and **matched synthetic job/outbox record SHA-256 plus unscoped RLS denial**. This is physically restored state within one temporary CI runner, not independently rebuilt production infrastructure, encrypted off-host backups, archived WAL timestamp PITR, measured production RPO/RTO or HA failover.
- Private GitHub, Mac and actual Ubuntu 26.04.1 canonical source SHA matched. On the **real** VPS, `cargo fmt --check`, **76/76 locked/offline Rust tests**, **37/37 existing lab static**, **14/14 database static** and **5/5 new infrastructure static** tests PASS. Original production services remain INACTIVE, and no network port, firewall or cluster service was changed.
- Mac FileVault remained ON. Fresh merged-source encrypted Restic snapshot `11bc5931` separately passed isolated SHA-256 source recovery together with the historical partial readable config; existing selected-root-readable snapshot `abaa9827` was independently re-restored with expected root-only and SSH directives. Full repository data-read PASS and private plaintext test directories removed. These selected artifacts do not support independently booting a complete replacement server or reconstructing PostgreSQL PITR.
- Actual clean-`main` `deploy/scripts/production/readiness.py --json` against this merged source returned **8/8 automatic PASS and 7/7 external independent gates BLOCKED**, intentionally exiting **3 / NO_GO**. The script has no mode to apply PostgreSQL, nftables, external perimeter or K3s. Seven external blockers remain until real rescue access, complete independently tested host recovery, dedicated verified dual-stack ingress, approved network and database topology plus real independent PostgreSQL PITR/rollback proofs are delivered. No ADR was silently promoted to APPROVED.
- A documentation-only proof merge will change the canonical Git hash. Capture and reverify that exact final source in the existing encrypted Mac repository and publish immutable evidence in the merged PR discussion; avoid recursive evidence commits.

## 29. R5.6 — actual disposable Ubuntu 26.04 K3s validation (initial implementation; CI pending)

- Canonical preflight at start: private GitHub `main`, clean authorized Mac and actual nonprivileged Ubuntu 26.04.1 VPS still match final R5.5 `2c5aac6f5a08fc3cd4ac56b23694616f7babd6f5`; main CI `36157345732` SUCCESS. The real VPS remains K3s/PostgreSQL/nftables INACTIVE, its shared dual-stack perimeter unchanged and live nonprivileged K3s read-only preflight reports independent rescue, complete independently rebuildable backup, dedicated perimeter and ADR-017 CNI/datastore decisions still BLOCKED.
- Added `deploy/scripts/lab/k3s-ubuntu26-ephemeral-ci.sh` and five `test_k3s_ubuntu26_ephemeral_review.py` static/negative guard tests. The independent new GitHub job `k3s-ubuntu26-disposable` provisions **another disposable GitHub-hosted Ubuntu 26.04 x64 runner**, never the IPAT VPS. It pins stable `v1.37.0+k3s1` to the published binary SHA-256, runs a real bounded temporary single-node embedded-etcd server with API bound only to a verified runner RFC1918 IPv4, and proposes node readiness, CoreDNS, actual BusyBox pod DNS and embedded-etcd snapshot smoke verification. GitHub actual success/failure must be recorded separately after CI finishes.
- Initial actual GitHub PR #43 CI `36159690724`: existing Rust/static and both PostgreSQL jobs PASS. Separate Ubuntu 26.04.1 runner verified official K3s binary checksum and reached a real `Ready` control-plane+etcd node with API bound to its private RFC1918 runner address, but **CoreDNS readiness timed out**, so this K3s job FAILED; do not claim working pod DNS or etcd snapshot. Second actual CI `36160201256` repeated node Ready/private API PASS but identified **CoreDNS CrashLoopBackOff**, with one transient `FailedCreatePodSandBox` event and a packaged gateway-CRD installer error. No internal DNS or etcd snapshot milestone has passed. Third CI `36160606070` again reached Node Ready/private-only API but the CoreDNS check ran before the packaged Deployment existed, prematurely failing after ~17 seconds; this is a CI test ordering bug, not evidence that pod DNS is healthy or permanently broken. Fourth actual CI `36160878253` used the bounded Deployment-existence wait and confirmed a recurrent package-container failure: CoreDNS `CrashLoopBackOff` (last exit code 128), and the new v1.37.0 gateway-CRD installer also exited 128; CI classified a runtime `permission denied` error. Rust and both PostgreSQL jobs remained PASS. Comparative CI `36161398677` with independently pinned non-prerelease K3s v1.36.4+k3s1 also reached Node Ready/private-only API, but CoreDNS had last exit code 128 and `runc` executable/runtime permission errors. Changing versions alone did not fix Ubuntu 26.04 runner OCI execution. Actual follow-up CI `36161873043` confirmed that a root filesystem with no `noexec` did **not** resolve the error; CoreDNS still exited 128 with `runc` executable/permission classifications on Ubuntu 26.04. The next disposable CI changes only the K3s daemon's service umask from the secret-safe script wrapper's 077 to standard 022 (the wrapper keeps restrictive secret permissions), testing whether restrictive OCI layer-directory creation prevents non-root system containers from executing. Latest actual CI `36162282199` proved the root cause in the disposable runner: the secret-safe wrapper's inherited `umask 077` prevented non-root packaged containers from executing; keeping restrictive outer-file permissions and using a conventional `umask 022` for the K3s daemon restored **CoreDNS rollout PASS and an actual BusyBox pod's internal DNS PASS**. The only remaining CI failure was an experiment-specific etcd-snapshot CLI default `127.0.0.1:6443`, inconsistent with this disposable runner's deliberately private `$node_ip:16443` bind. Verified upstream v1.36.4 source supports `--etcd-server`; the script now specifies the private endpoint and reruns CI. No snapshot success or production installation should be claimed before that real CI completes; no production daemon setting is approved by this experiment. This is not an approved production version or reason to weaken the live host firewall.
- No K3s installer, systemd service, host firewall or third-party shared group change was attempted on the actual IPAT VPS. No new ADR-017 network/CNI/HA approval or physical device compatibility evidence is implied. This single disposable node will not demonstrate multi-node HA, separate-host datastore restore, actual private VPN/dual-stack perimeter or production failover.
- Changed files: new isolated K3s lab script, five guard tests, workflow Ubuntu 26.04 job, `docs/K3S_UBUNTU26_R56.md`, README, architecture/security/backlog/status. Gates after successful CI: production-safe network/rollback profile for independently recovered hosts, verified independent out-of-band login and complete host recovery, private node networking ADR approval, then supervised actual VPS install. Existing `NO_GO` production gate remains binding.

### R5.6 independently verified GitHub, actual Ubuntu and encrypted recovery
- Private feature PR [#43](https://github.com/mr-ipat/ipat/pull/43) merged to `main` `23295a7ebf1c0bdf6f5a0ae29bb428e336b0b8d6`. Final feature PR workflow `36163382926` and genuine new main workflow `36163547567` both returned **SUCCESS** in four independent GitHub jobs. Actual dedicated Ubuntu 26.04.1 disposable job independently performed every R5.6 K3s smoke acceptance: pinned upstream SHA-256, embedded-etcd node Ready, RFC1918-only API bind, CoreDNS deployment Ready, BusyBox pod/internal DNS and local etcd snapshot.
- Root cause of the earlier real CoreDNS container `runc` failure was a lab-process permission mistake, not a device/CNI fact: secret-bearing outer script `umask 077` was inadvertently inherited by K3s/containerd. A scoped standard `umask 022` for the disposable K3s daemon, while retaining `0600` Kubeconfig and restrictive wrapper, allowed genuine pod execution. An explicit upstream-documented `--etcd-server` private API endpoint fixed the follow-up local snapshot CLI port mismatch. Do not extrapolate this result to actual customer networking, Ubuntu package interactions, independent off-host datastore restore or production release selection.
- **Independent actual VPS Ubuntu 26.04.1 regression on canonical merged source:** `cargo fmt --all -- --check` PASS; `cargo test --workspace --locked --offline --quiet` **76/76** Rust tests PASS; **42/42** Python lab safety, **14/14** DB source contracts and **5/5** production NO-GO gate tests PASS. GitHub/Mac/VPS committed source SHA matched; live K3s and PostgreSQL remain INACTIVE and no host or shared external perimeter was changed.
- Mac FileVault ON. Encrypted Restic snapshot `bb7ad5ff` for precisely this merged source independently matched isolated SHA-256 restore; historic partial config and separately encrypted selected privileged-root config `abaa9827` also independently restored; repository `restic check --read-data` PASS, temporary decrypted restores removed. Neither a real alternate rescue console nor complete host backup/off-host etcd recovery, external production DB PITR, private cluster routing, host-native firewall/CNI coexistence or K3s multi-node HA was established. Those are HARD GATES before any live privileged K3s execution.
- A docs-only evidence merge will advance `main` SHA; re-encrypt and reverify the final exact source and record the last checkpoint in the merged docs PR discussion, rather than producing recurring metadata-only commits.

## 30. R5.7 — real cross-host K3s recovery + deterministic nftables rollback (2026-09-26; pre-merge)

- Official Ubuntu 26.04 arm64 cloud image was downloaded outside the repository and SHA-256 verified against the official release checksum before boot. Two fresh QEMU/HVF VMs were used as distinct source/restore hosts with exact marker guards and no customer/device data.
- Exact repository `deploy/scripts/lab/r57/source-systemd-k3s.sh` executed successfully on a fresh source VM: checksum-pinned K3s v1.36.4+k3s1, real systemd service, Node Ready, CoreDNS and BusyBox cluster DNS PASS, private-only API port 16443, embedded-etcd restore marker, snapshot, mode-0600 original server-token export and systemd restart PASS. Token content was never printed or committed.
- Exact repository restore procedure was iterated against fresh second VMs and exposed three real recovery defects that were corrected: new-host restore requires the original server token at the standard K3s path; stale Node/pod objects from the source host require reconciliation; and old Pod Ready state can be stale immediately after service restart. The final fresh run restored the synthetic etcd marker, removed stale source Node/pods, recreated CoreDNS/workload on VM B, passed DNS, private API, service restart via a newly created post-restart pod, and created a post-restore snapshot.
- Real nftables/K3s coexistence passed on the restored VM. A dedicated `inet ipat_r57_lab` default-drop table first preserved K3s/DNS and fresh SSH under an explicit management rule. The failure drill intentionally omitted fresh SSH access. Initial `systemd-run --on-active=20s` recovery was delayed because default timer `AccuracySec=1min`; the final exact repository drill uses 10 seconds, `AccuracySec=1s`, `RandomizedDelaySec=0`, blocks a fresh SSH connection, then automatically deletes only the IPAT lab table. New SSH, node Ready and fresh pod DNS subsequently PASS.
- This materially proves disposable single-node K3s service/etcd recovery and an IPAT-owned nftables rollback mechanism, but **does not** prove live VPS recoverability, provider-isolated node networking, production HA quorum, persistent storage recovery or production firewall safety. ADR-017 remains OPEN and ADR-018 PROPOSED. No live VPS K3s/PostgreSQL/nftables or shared external perimeter mutation occurred.
- Files: `deploy/scripts/lab/r57/{source-systemd-k3s.sh,restore-systemd-k3s.sh,nft-rollback-drill.sh,test_r57_review.py}`, `docs/K3S_CROSS_HOST_RECOVERY_R57.md`, README/architecture/security/deployment/backlog/status and CI static contracts. GitHub review/CI/merge and final encrypted source evidence remain pending.

## 31. R5.8 — isolated original Rust Control API + native USP stub K3s application packaging (pre-CI)
- Resumed the existing uncommitted feature branch rather than overwriting work. Based on verified identical GitHub/Mac/actual Ubuntu 26.04.1 source `3eef408742e0eeec450b00f4fc1507d7ac0ca5fe`; actual VPS K3s/PostgreSQL/nftables remain INACTIVE.
- Added explicit **lab-only** pod-network binding to both existing original Rust health-only processes without changing their default loopback behavior or denied data/USP routes. Updated the prior synthetic USP safety contract to explicitly check that pod wildcard binding is guarded by a distinct R5.8 K3s-lab flag. No live OIDC, TR-369 transport, device I/O or subscriber data.
- Added two static scratch OCI build definitions, strict private non-root Helm chart with exactly two Deployments/ClusterIP Services/no-token accounts/NetworkPolicies, rendered-manifest validator plus deliberate negative mutation tests, and a post-R5.6 K3s CI script that builds static-musl binaries, locally imports OCI images and probes health/401/real unapproved-pod network-policy denial. Every script explicitly refuses non-disposable environments; no external hosting-provider API integration.
- Existing 42 lab Python static tests, 6 R5.7 static, 5 R5.8 static and 14 DB static source contracts PASS on Mac prior to PR; source-only checks are not proof of pod execution. Actual GitHub Ubuntu 26.04 real K3s smoke and full Rust CI are pending. See `docs/K3S_APPLICATION_PACKAGING_R58.md`; R5.8 does not relax unresolved external production safety gates.

### R5.8 first real GitHub CI finding (before checksum correction)
- Actual PR #47 first workflow `36207087725`: Rust/static, PostgreSQL synthetic logical and PostgreSQL disposable physical-recovery jobs **PASS**. Disposable Ubuntu 26.04 K3s application job **FAILED BEFORE ANY K3s INSTALLATION** at the pinned Helm SHA-256 verification step. The workflow's initially copied v3.21.3 Linux amd64 checksum did not match the actual checksum separately fetched from the official `get.helm.sh` release `.sha256` endpoint using the authorized Mac; the exact expected release checksum is now corrected in `.github/workflows/ci.yml` to `15e041a93a590dce8100f39385cd98c84a765c9e36aeeb9e2dc6ff9e4769e2e0`. This is not evidence of working container pods or K3s failure. Re-run full GitHub CI; leave the real VPS untouched.

### R5.8 second actual disposable CI finding: root-only K3s state permissions
- Actual PR #47 workflow `36207214161`: Rust/static, PostgreSQL logical and disposable physical recovery jobs PASS. The Ubuntu 26.04 runner verified the corrected official Helm checksum, Helm manifest negative tests, checksum-pinned K3s Node Ready, private-only API, CoreDNS, pod DNS and embedded-etcd snapshot PASS. The new R5.8 application launcher then stopped with `R58_DENIED` because its unprivileged preflight tried to read an intentionally root-only K3s data directory. The reviewed correction uses a narrowly scoped noninteractive privileged Unix-socket existence probe instead; it does not relax K3s file permissions or change the actual VPS. Retest before claiming actual app pod success.

### R5.8 third CI finding: K3s containerd runtime socket is separate from the data directory
- Actual PR #47 workflow `36207346710`: Rust/static, synthetic PostgreSQL logical recovery and disposable PostgreSQL physical recovery jobs PASS. Ubuntu 26.04 K3s binary checksum, Ready node, RFC1918-only API, CoreDNS, real pod DNS and etcd snapshot again PASS. The R5.8 launcher still refused its runtime check because K3s places containerd's **runtime Unix socket** under `/run/k3s/containerd/containerd.sock`, not beneath the explicitly selected persistent `--data-dir`. The documented K3s bundled `k3s ctr` automatically targets that standard runtime socket. The next revision checks the correct Unix socket through scoped root access and lets bundled `k3s ctr` select its own address. Only source code and disposable CI were modified; no real-VPS services or provider/shared firewall changes. Actual Rust app pod tests remain PENDING re-run.

### R5.8 fourth disposable CI: physical container build/import succeeded; Helm rollout diagnosed next
- Actual PR #47 workflow `36207492977`: Rust/static, PostgreSQL RLS and physical recovery CI jobs all PASS. Real Ubuntu 26.04 K3s Node Ready, RFC1918 API, CoreDNS, pod DNS and embedded-etcd snapshot PASS. R5.8 then compiled both actual Rust binaries for static musl, built and imported both scratch OCI images into K3s containerd, and passed strict real Helm render verification. **The Helm deployment did NOT become Ready** within its original 240-second timeout (`context deadline exceeded`); real application pod health and network-policy behavior must NOT be claimed yet. The initial script discarded useful pod states on failure; a new 105-second bounded rollout plus redacted pod/deployment/event-reason-only diagnostics and explicit post-import normalized image-reference checks will distinguish image naming, scheduling, container failures and readiness or policy issues without exposing container logs, secrets or event messages. No external/shared security group, live Ubuntu K3s/PostgreSQL/nftables or device was changed. Further disposable CI remains required.

### R5.8 fifth real disposable CI finding: non-root OCI application binaries exit 128
- Actual PR #47 workflow `36207928773`: all three Rust/static and PostgreSQL CI jobs PASS. Real Ubuntu 26.04 K3s checksum-pinned node, private API, CoreDNS/pod DNS, etcd snapshot, static-musl Rust release build, imported normalized OCI image names and strict Helm-rendered no-public-exposure manifest validation **all PASS**. Actual Helm rollout still failed. Newly safe-redacted Kubernetes diagnostics prove **both** original Rust application pods entered `CrashLoopBackOff` with their previous container exit code **128**, and Kubernetes reported only allowlisted Failed/BackOff event reasons (no event message or app secret was logged). The feature's outer shell deliberately uses `umask 077` to protect temporary secrets; Docker `COPY` without an explicit destination mode can preserve an executable file unreadable to non-root UID 65532, which is a plausible cause (not proven until a rerun). Both `scratch` Dockerfiles now explicitly set `COPY --chmod=0755` on the **single non-secret app executable** while preserving root-only temporary K3s secrets and all pod confinement. The failure reporter was additionally extended to emit only fixed event error categories, never raw event messages or logs. Re-run real ephemeral CI; do not claim pod readiness yet. The actual VPS and shared external perimeter are unchanged.

## 32. R5.9 — browser preview safely accessible on authorized Mac (pre-review)

- Verified the clean R5.8 private GitHub, Mac and actual Ubuntu 26.04.1 VPS
  canonical `main` matches `914ec432626f103a66608af0e7dc831e2d49b10e`.
  Live VPS remains K3s, PostgreSQL and nftables INACTIVE; SSH rollback marker
  absent and no real customer/device protocols activated.
- On isolated unprivileged Ubuntu 26.04.1 source checkout, the new
  explicitly opted-in Rust local-only browser routes compiled and **six
  Control API unit tests PASS** (initial handler signature compile failure
  was corrected before any merge). A real, temporary local-only HTTP process
  served HTML, CSS, JS and JSON with CSP, returned 401 for anonymous device
  data and 405 for attempted status POST; actual listener bound
  `127.0.0.1:3000` ONLY. The temporary process was stopped.
- The first-party responsive Indonesian information-only UI is under
  `web/lab`; the server includes it only with `IPAT_LAB_WEB=1` and
  expressly masks it in K3s pod bind mode. The Mac private-tunnel launcher
  refuses mismatched Git/SSH/port/process identity and changes no host
  firewall or externally managed Security Group. Negative source tests
  and real Mac→VPS end-to-end tunnel are separate review/verification gates.
- Paths: `web/lab/*`, `apps/control-api/src/main.rs`,
  `deploy/scripts/lab/r59/*`, `.github/workflows/ci.yml`,
  `docs/WEB_PRIVATE_PREVIEW_R59.md` and affected canonical documents.
  All production public ingress/OIDC/real USP/customer-data gates remain
  OPEN; ADR-006/014/017/018 have not been silently approved. Only after
  PR CI, merge, exact source sync, encrypted restore and actual working
  SSH tunnel may the Mac-local browser preview be declared READY.

### R5.9 reviewed feature PR, actual Ubuntu regression, encrypted backup checkpoint

- [Feature PR #49](https://github.com/mr-ipat/ipat/pull/49) merged to private
  GitHub `main` `466ffcb67a87ba3f88330f26a13cebda5d624e7d`. Actual PR CI
  `36210434210` SUCCESS across all four independent GitHub jobs (Rust/static,
  real disposable K3s Ubuntu 26.04, isolated logical PostgreSQL and synthetic
  physical PostgreSQL backup/restore). Existing K3s health-only pod policies
  were unchanged; the new opt-in browser UI cannot be exposed by them.
- Private GitHub/Mac/actual Ubuntu 26.04.1 nonprivileged checkout exact SHA
  matched. Real VPS passed Rust formatting, **81/81 offline Rust workspace
  tests**, **42/42 existing lab**, **6/6 R5.7**, **6/6 R5.8**,
  **6/6 new R5.9**, **14/14 database static** and
  **5/5 production readiness static** tests without installing or activating
  K3s/PostgreSQL/nftables. On isolated VPS source, a *real* temporary
  private HTTP server passed HTML/assets/status/CSP/401/405 and exclusively
  loopback listener checks, then was stopped.
- Mac FileVault ON; exact reviewed feature-main source snapshot `8c40ccec`
  independently passed isolated SHA-256 restoration and historic partial
  restore. Separately encrypted selected privileged-root-readable snapshot
  `abaa9827` independently restored sudoers/SSH hardening metadata;
  repository complete Restic pack reading PASS; temporary plaintext removed.
  This is NOT a complete restored independent replacement VPS or PostgreSQL
  PITR rehearsal. Actual post-feature-main CI `36210550883` also returned
  SUCCESS in all four jobs. The real Mac tunnel is still pending at this
  checkpoint; after documentation merge, repeat final exact
  source snapshot and post that SHA/backup/CI/localhost proof in PR discussion
  rather than repeatedly changing `main`.

## 33. R6.0 — planned physical-device targets and credential-free intake (pre-CI)

- Source of truth remains the already verified clean private GitHub/Mac/
  Ubuntu 26.04.1 VPS `main` `1b649b90cd4920d0dddbbc125f928cf92b75286c`.
  The original R5.9 private Mac browser tunnel was verified against this
  source. Actual VPS K3s, PostgreSQL and nftables remain INACTIVE.
- The next product slice presents all **eight** original DEV-01..08
  targets in the private browser (ZTE C320, C-DATA OLT, VSOL/ZTE ONT,
  MikroTik x86/CCR/distribution RB and customer RB), not fictitious
  connected devices. Exact model/firmware, permissions and real lab
  reachability are missing. Lab JSON explicitly declares ZERO physical
  devices, ZERO interoperability, no network discovery and no
  compatibility claims. The browser rejects catalog provenance mismatch.
- The same original Rust lab-only API adds a read-only target-list endpoint,
  absent in default/K3s mode and rejecting mutations. An independent
  offline Python validator accepts only exact approved target IDs and
  strictly bounded observed model/board/firmware and *candidate* protocol,
  rejects secrets, serials, management addresses, duplicate keys,
  placeholders, unsafe paths and overwrites, and writes new mode-0600
  unapproved metadata **outside Git** only. It NEVER contacts equipment
  or automatically authorizes registration/tenant/device claims.
- Files: `web/lab/{device-targets.json,index.html,app.js,style.css}`,
  `apps/control-api/src/main.rs`,
  `deploy/scripts/lab/r60/{prepare-device-intake.py,test_r60_review.py}`,

  `.github/workflows/ci.yml`, `docs/DEVICE_TESTING_R60.md` and
  updated authoritative PRD/architecture/security/device matrix/backlog/
  deployment/status docs. Local six Python R6.0 review tests passed;
  exact final Rust/CI, reviewed merge, actual final laptop preview,
  new encrypted source backup and any REAL device evidence remain
  pending at this checkpoint. No external provider firewall integrations.
- NEXT actual hardware gate: operator supplies a true locally observed
  model, firmware and board for at least ONE specifically authorized,
  isolated reachable physical target. Independent review and a safe
  physical read-only evidence record must precede any notification that
  a real device was actually added for testing. No untested vendor,
  model or firmware may be marked `validated`.

### R6.0 reviewed feature, nonprivileged Ubuntu and recovery checkpoint

- Feature [PR #51](https://github.com/mr-ipat/ipat/pull/51) merged as
  `2184cda657bea315de455e8e9249c82aba8e55e4`; actual reviewed PR
  workflow `36219353933` returned SUCCESS in all four jobs: Rust+static,
  real isolated Ubuntu26 K3s Node/etcd/DNS + real existing health-only app pods
  and ingress policy denial, PostgreSQL synthetic logical+physical recovery.
- Private GitHub/Mac/actual Ubuntu 26.04.1 VPS clean source aligned exactly
  at the feature SHA. Actual host, with no new privileged installation,
  passed Rust formatting and 83 locked offline Rust tests, 42 base static,
  6 R5.7, 6 R5.8, 6 R5.9, 6 R6.0, 14 DB static and 5 admission static tests.
  Actual VPS K3s/PostgreSQL/nftables all remained INACTIVE.
- Mac FileVault ON. Encrypted feature-main Git source snapshot `4b2a0b99`
  and separately encrypted selected root-readable snapshot `abaa9827`
  were independently SHA-256/SSH-sudoers restored, with complete Restic data
  check PASS and plaintext temporary artifacts removed. This does NOT prove
  a complete replacement VPS/production PostgreSQL restore.
- Next actual physical test still requires real exact hardware+firmware
  evidence, equipment-owner authorization and isolated lab connectivity.
  Eight software candidate records are NOT eight connected devices;
  simulator/protocol tests must not be relabelled as physical evidence.

## 35. R6.1 first actual customer MikroTik target (PRE-CI, NO REAL DEVICE LOGIN)
- User reported DEV-08 as **RB951Ui-2HnD, RouterOS 7.23.7**. Canonical initial status is “owner reported, pending verification”; **zero actual physical devices have been enrolled, read or declared compatible**. The website only shows this explicitly caveated owner report; all other pilot targets remain unknown and read-only.
- Added standalone guarded “exactly one HTTPS GET /rest/system/resource” operator tool at `deploy/scripts/lab/r61/readonly-rest-probe.py`; code path defaults to offline preflight, only allows authorized Mac with two explicit run-time real-GET opt-ins, exact DEV-08/model/RouterOS, single operator-confirmed RFC1918 management target, trusted private CA, independently matched TLS SAN/DNS and locally stored owner-mode-0600 dedicated netrc credentials. It does **not** configure www-ssl, discover networks, scan customer devices, auto-enroll, start production PostgreSQL/K3s, modify the router or expose credentials. On successful later authorized read, all automated enrollment, tenant binding, compatibility and write fields remain FALSE pending human evidence review.
- Eleven targeted Python tests passed on authorized Mac (ten negative/synthetic source tests plus one ephemeral loopback TLS CA/hostname verification test) before final CI, including exact mocked GET, secret/serial/address suppression, bad model/firmware, missing opt-in, extra config/unsafe address, permission/duplicate input rejection and exclusive private-mode evidence output. Original R6.0 negative tests adapted to distinguish **owner reported** from physically verified without inflating real counts. Neither these tests nor the owner report is a physical test. Actual Ubuntu clean checkout/CI/PR, exact-merge backup and browser update must be independently verified before milestone can be marked complete. See `docs/MIKROTIK_CUSTOMER_R61.md`.
- Dependencies before TC-ROS-03 physical test: independent proof of specific customer's test authorization, correct physical management route on isolated lab Mac, existing non-disruptive recovery, restricted “read+rest-api only” group (the built-in read group has additional rights), verified router `www-ssl` certificate/CA, then local operator's separately approved one-GET read. Actual hardware revision and RouterBOOT may require a subsequent separately scoped read. Full router-management/PPPoE/firmware/tenant platform capability is NOT implemented or validated; live VPS remains K3s/PostgreSQL/nftables INACTIVE.

### R6.1 synthetic HTTPS trust evidence (still NO physical connection)
Added `deploy/scripts/lab/r61/test_r61_tls.py` with an ephemeral locally generated signing CA and HTTPS server. The first test-run failed because the synthetic CA lacked required certificate key-usage extensions; fixture corrected and 11 R6.1 Mac tests subsequently PASS. Successful ephemeral TLS handshake with matching signed DNS SAN and **rejection of the wrong server name** validate only the local probe code path, never the owner's RB951 physical device or its actual TLS certificate. GitHub CI, exact merge SHA, production-safe Mac browser rollout and exact source backup remain to be proven after review.

### R6.1 reviewed source, post-merge regression and actual Mac web evidence
- Real feature PR #53 CI `36221642643` and real post-feature-merge main CI `36221754351` were each fully successful across all four GitHub jobs (disposable real Ubuntu26 K3s with actual app pods/network policy, Rust/static, two isolated PostgreSQL recovery jobs). The detailed earlier canceled feature-branch runs were superseded after tightening evidence-path prevalidation and adding synthetic signed HTTPS CA/SAN and negative hostname tests.
- Matching SHA `dcacc1a5e93e04b5ce58d43a34b168b231d42da3` verified on private GitHub, clean authorized FileVault Mac and clean real unchanged Ubuntu 26.04.1 VPS. Actual VPS 83/83 locked offline Rust workspace tests, 11/11 guarded R6.1 (including actual loopback synthetic CA/SAN and wrong hostname denial), 6/6 R6.0, 42/42 base lab, 14/14 DB contracts PASS; K3s, PostgreSQL and nftables remain INACTIVE.
- The authorized Mac restarted the original loopback-only Rust browser/tunnel against matching source and verified real HTTP from the Mac for `GET /lab`, local JS, CSP, exact reported DEV-08 model/RouterOS, zero enrolled physical devices, POST rejection 405 and unauthenticated real device endpoint 401. This is **not** actual router traffic.
- Fresh encrypted exact feature-main Restic source snapshot `74308479` passed independently isolated SHA-256 restore and complete encrypted pack read; separately existing selected privileged root-readable snapshot `abaa9827` passed isolated recovery. This still is **not** independently rebuilt complete VPS, real production HA/PITR or OOB recovery.
- Physical TC-ROS-03 remains NOT RUN until the operator supplies a separate approved isolated Mac route to the actual customer router, independent trusted CA + restricted single-purpose REST identity locally, and explicit test permission and recovery. None of these sensitive artifacts go into Git or chat.

## 36. R6.2 — native Rust read-only RouterOS domain and private evidence bridge (pre-CI)

- Source baseline: previous clean, synchronised private GitHub/Mac/
  actual Ubuntu 26.04.1 VPS main SHA
  `35eda016c5fce55a73b33298bf472b0a5c57864a`, four-job
  post-docs-main CI `36221968613` fully successful, encrypted
  exact prior source Restic snapshot `eb439ff7` independently
  restored. Real owner-RB951 remains **NOT CONNECTED** and
  test TC-ROS-03 NOT RUN.
- Added original `crates/routeros-core` Rust crate to
  workspace with pinned `serde`/`serde_json` lockfile
  using available offline cached dependencies. The first
  attempted `serde/derive` dependency was not cached on
  the actual Ubuntu checkout; unnecessary derive feature
  removed before tests rather than silently accessing the
  Internet. Strict bounded field and duplicate validator
  strips unapproved response fields and allows only
  operator-reported DEV-08 RB951Ui-2HnD/mipsbe/7.23.7,
  optionally the same build with stable/long-term suffix.
  All outputs stay `UnreviewedInventory` without a
  verified-tenant or privileged constructor.
- Added distinct closed-schema `routeros-core::evidence`
  parser for PRIVATE sanitized R6.1 operator-local
  proof, rejecting redacted evidence with unexpected
  serial/secret/unknown fields, duplicate JSON fields,
  unapproved method/path, invalid identity/version,

  fabricated enrollment, self-declared approval or tenant
  claims. Added `routeros-lab-evidence` offline CLI with
  owner-only file-mode/private path checks and no network
  operations. The CLI explicitly reports syntactic schema
  success is **NOT** a real-device authenticity proof.
- Enhanced the original R6.1 Python HTTPS one-GET sanitizer
  for the identical narrow RouterOS version suffix policy
  while retaining RFC1918, separate TLS hostname/CA,
  dedicated mode-0600 credentials and two manual real
  network-access opt-ins. Added synthetic cross-contract
  script that constructs fake raw serial/password/IP fields,
  drops them in Python and verifies Rust staged evidence,
  then checks forged tenant and injected serial files are
  rejected. NO actual device address, password or customer
  data is needed, read or written by this suite.
- On an **isolated nonprivileged actual Ubuntu 26.04.1 VPS
  checkout**, initial `cargo test --workspace --locked
  --offline` reported **99/99 Rust tests PASS** (previous
  83 plus 15 routeros-core unit + 1 independent offline
  CLI integration), **12/12 R6.1 Python tests PASS**,
  and independently invoked actual Python-to-Rust
  offline synthetic cross-contract **PASS**. This is
  code-level test evidence only; final reviewed GitHub
  CI, exact merge SHA, final source backup and Mac
  preview rollout remain pending at this checkpoint.
- Actual authorized Mac's

  `~/.local/share/ipat/router-lab/` directory was absent
  during a read-only filename/mode preflight. Do not
  invent a target management IP, certificate, account,
  operator approval or non-disruptive backup. Real
  router probing remains blocked until the operator
  prepares verified facts locally and separately
  authorizes the first controlled GET. Public production
  K3s/PostgreSQL/nftables/network recovery gates unchanged.
  This new independent crate is within the existing
  modular-monolith ADR-003 design; DECISIONS.md unchanged
  because no new architecture decision was approved.

### R6.2 reviewed code, production guard, Mac template and encrypted recovery checkpoint

- [Feature PR #55](https://github.com/mr-ipat/ipat/pull/55)
  merged to private `main` SHA `9901627a1739a4cfa785ed899fed8fb2c55580fa`.
  Its **four** independent GitHub jobs `36222753736` and
  independently run post-feature-main CI `36222869650` all
  completed **SUCCESS**, including real disposable Ubuntu 26.04
  K3s smoke with in-cluster denial and both ephemeral PostgreSQL
  recovery jobs. No live infrastructure was changed.
- Private GitHub, clean authorized FileVault Mac, and unchanged
  actual Ubuntu 26.04.1 VPS canonical main source were all
  independently verified at exact feature SHA. Actual VPS passed
  **99/99** locked/offline Rust workspace tests, **12/12**
  R6.1 synthetic/negative first-read Python tests,
  **6/6** R6.0, **42/42** base lab,
  **6/6** each R5.7, R5.8, R5.9 and
  **14/14** DB static, **5/5** readiness static tests;
  the standalone synthetic Python -> Rust CLI cross-contract
  passed and refused fake serial/tenant elevation.
- Mac FileVault ON; exact merged-feature-source encrypted Restic
  snapshot `b13e3f49` independently isolated SHA-256 restored.
  Selected privileged root-readable config snapshot
  `abaa9827` recovered separately; full encrypted pack read

  PASS, plaintext removed. These **do not** constitute
  independently recovered whole VPS/real database PITR.
- Authorized Mac `~/.local/share/ipat/router-lab/` now exists
  mode-0700 and contains an owner-mode-0600 **deliberately unusable
  template only**, with reserved TEST-NET target, no real router
  IP, non-confirmed operator permission and false isolated
  route. Existing offline R6.1 validator DENIED this template
  as intended, with **zero network requests**; no actual
  probe.json, dedicated netrc or trusted CA was provided.
  Real RB951Ui-2HnD RouterOS 7.23.7 still owner-reported only;
  no physical identity, firmware, board, compatibility,
  enrollment, login or subscriber support has been verified.
- Final documentation-only PR, post-docs-main CI, fresh
  exact merged documentation-source encrypted backup and
  browser loopback re-verification will be captured in
  PR review discussion to avoid recursive SHA-changing commits.

## 37. R6.3 — customer-router public SSH server RSA key changed (authentication BLOCKED)

- The owner supplied explicit permission to attempt SSH to ONE
  public endpoint (address and port not source-controlled). A Mac
  TCP connection succeeded and an unauthenticated remote banner
  resembled MikroTik RouterOS SSH. The authorized Mac already
  holds a previously trusted RSA key for this exact endpoint,
  **but strict SSH returned REMOTE HOST IDENTIFICATION HAS CHANGED**
  before offering or sending any password. An independent
  unauthenticated one-host RSA key scan found a distinct current
  fingerprint. SSH fingerprint is public, but the old and new
  full endpoint-specific fingerprints were presented in the
  owner's private chat rather than persisted in Git.
- **NO password was transmitted or stored, no RouterOS command
  executed, no SSH host-key override or deletion, no router
  settings changed, and NO physical hardware model/firmware
  compatibility or customer tenant identity verified.**
  This might indicate a legitimate RouterOS host-key
  regeneration, different NAT destination or an adversarial
  SSH endpoint; a `ROSSSH` banner alone is insufficient.
- Added `deploy/scripts/lab/r63/ssh-host-trust-check.py`,
  a strictly unauthenticated single-public-IPv4/port SSH
  host-key observer that compares current RSA SHA-256 with
  the previous owner-Mac key. A mismatch exits 4; the optional

  independent fingerprint gate requires an owner-controlled
  mode-0600 file in a private mode-0700 folder and the explicit
  owner confirmation that the value came from a **separate
  trusted direct-LAN check**. It never logs in, sends or
  requests a password, updates `known_hosts`, retries
  authentication or accepts the current public scan as
  independent identity proof.
- Actual owner-Mac **live unauthenticated** test of the
  supplied exact public endpoint returned
  `HOST_KEY_CHANGED_UNVERIFIED` with expected exit 4
  and explicit no-password/no-RouterOS-command/no-hostkey-change
  evidence. Ten pure negative/mock tests passed on Mac and on isolated actual Ubuntu 26.04.1 VPS (mocked network, no credentials).
  The new project documentation details a separate verified
  direct-LAN fingerprint procedure, credential rotation,
  temporary strictly pinned host-key approach after explicit
  sign-off and zero client-configuration changes.
- NEXT: independently compare current public-endpoint
  fingerprint to actual RB951's direct-LAN SSH host key
  from trusted WinBox/LAN management. If inconsistent,
  do NOT authenticate; investigate forwarding and
  possible host-key compromise. If consistent, obtain
  owner approval for a separate, strictly host-pinned,
  single read-only login preferably via SSH public key
  after rotation of the already disclosed credential.

  Until then, real TC-ROS-03 remains NOT RUN and DEV-08
  remains operator-reported, zero physical enrollment.
- CI, exact reviewed merge, post-merge recovery/snapshot
  and final endpoint reprobe pending at this preliminary
  checkpoint. No changes to the live VPS provider/network
  perimeter, k3s, PostgreSQL, nftables or firewall.

### R6.3 evidence of an actual security stop (no real device authentication)

- Reviewed feature PR #57 merged SHA `017adfc382be86aec742cf24299eac794ba886be`; PR GitHub CI `36224036550` and independently run post-feature-main CI `36224159513` both SUCCESS on all four jobs, including disposable Ubuntu26 K3s and separately isolated synthetic PostgreSQL restore/failure tests.
- The Mac could open exactly the owner-provided SSH port and received only an **unverified** RouterOS-style banner. Its strict pinned-host connection failed BEFORE any user-supplied password could be sent, with unexpected RSA host-key change. An unauthenticated public RSA keyscan and the new exact-single-target R6.3 preflight independently reproduced the saved-vs-current mismatch; live helper exited 4 with explicit `PASSWORD_SENT=NO`, `ROUTER_COMMANDS_RUN=NO`, `KNOWN_HOSTS_MODIFIED=NO`. No other destination/protocol was probed and none of the exact endpoint identifiers or secrets are committed to source.
- Actual unchanged Ubuntu VPS canonical source matched clean GitHub/Mac at reviewed feature SHA and passed 99 locked/offline Rust workspace tests, 10 R6.3 mock/negative host-key tests, 12 R6.1 synthetic tests, 42 base-lab, 14 DB static plus original Python-to-Rust fake-secret/forged-tenant cross-contract. FileVault Mac encrypted exact merged-feature Restic source snapshot `6bc581b5` was independently isolated SHA256 restored; separately selected privileged-root readable backup `abaa9827` passed independent restore and full encrypted pack read PASS. These are **not** whole live server/datastore recovery.
- Actual trusted direct-LAN SSH fingerprint for **this exact physical RB951** has not been provided. The owner-posted credential was not used or stored in any script, documentation, repository, shell command or local credentials file. An authentic remote MikroTik read, TLS management setup, tenant assignment and all physical compatibility tests remain **NOT RUN**. The already functioning web preview must not display a false connected-device status. All live VPS K3s/PostgreSQL/nftables inactive; no externally shared provider firewall changes.

## 38. R6.4 — restricted key-only RouterOS SSH first-read lab path (pre-merge)

- Baseline source: reviewed R6.3 `main` SHA
  `cec40a5e3e602b79ef3654d371605f0c9bda1d7b`
  synchronized clean GitHub/authorized FileVault Mac/actual
  unchanged Ubuntu 26.04.1 VPS; last final four-job
  GitHub CI `36224396747` SUCCESS. R6.3 live
  unauthenticated single-host check recorded previous
  vs current RSA SSH host-key mismatch and failed
  closed without sending any password, running a
  router command or altering `known_hosts`.
- Product/UI advancement: `web/lab/device-targets.json`,
  `index.html`, `app.js`, `style.css` now display
  the previous R6.3 finding as a STATIC security block
  for planned DEV-08; never represent it as a live
  router session or real telemetry. Actual physical
  enrollment/interoperability counts stay ZERO, and
  customer operation controls remain absent.
- New `deploy/scripts/lab/r64/ssh-first-read.py`
  offers OFFLINE `--requirements` and locally
  validated `--preflight` paths with **no network**.
  Its optional one-device `--read` requires an
  owner-controlled mode-0600 fingerprint obtained from
  a separate trusted direct-LAN route, independently
  reviewed router recovery/permission, a dedicated

  short-lived restricted owner-private SSH key,
  an exact single public IP+port, and three explicit
  local owner opt-in environment flags. A live
  untrusted public RSA keyscan MUST match the
  independent trusted fingerprint BEFORE any
  public-key-only strict SSH login. Current historical
  key conflict remains preserved, and a separate
  acknowledgment is mandatory; no weak RSA/SHA-1
  fallback, password, SSH agent, proxy or forwarding.
- The ONLY allowed future physical SSH command reads
  board, architecture and RouterOS version; raw
  stdout is bounded, strict and never logged.
  A matching output writes a new private
  mode-0600 `UNREVIEWED` evidence record with
  physical enrollment, compatibility, operator review,
  tenant binding and configuration writes FALSE.
  The original Rust `routeros-core::evidence` verifier
  now accepts ONLY the exact REST or SSH read-only
  tuple and rejects method/resource swaps and
  forged privilege flags. It still CANNOT
  authenticate the provenance of local JSON.
- New `deploy/scripts/lab/r64/synthetic-ssh-cross-contract.sh`
  combines an entirely synthetic Python three-field
  CLI sanitizer and the Rust private-file evidence
  validator, rejecting fake serial, forged tenant

  trust, arbitrary method and write resource.
  Initial actual nonprivileged Ubuntu 26.04.1
  isolated checkout reported 100/100 Rust tests,
  10/10 early R6.4 Python mock/negative tests,
  10/10 R6.3 security tests and this independent
  fake SSH to Rust cross-contract PASS. Two
  additional R6.4 negative UI/ambiguous-key
  tests passed locally, expanded to 12/12.
- Owner-Mac owner-only folder
  `~/.local/share/ipat/router-lab`
  now contains ONLY a mode-0600 **unusable** R6.4
  `ssh-read.template.json` with TEST-NET placeholder,
  false human verification/backup/permission booleans,
  and nonexistent dedicated key and proof paths;
  the actual offline preflight refused it as intended.
  No real `ssh-read.json`, independent fingerprint
  record, dedicated SSH private key, actual authorized
  router backup/recovery or authenticated device test
  was created. The earlier chat-disclosed password
  must be rotated by the owner through independent
  trusted management and is never used in code.
- FINAL feature four-job GitHub CI, reviewed merge,
  exact SHA Mac/VPS source synchronization,
  new encrypted-source isolated restore and
  replayed private browser test are STILL PENDING

  at this milestone's preliminary checkpoint.
  Real device TC-ROS-03 and commercial
  RouterOS capabilities remain UNTESTED.
  `docs/DECISIONS.md` unchanged: no new
  approved architecture decision; a new
  lab transport is NOT a production stack choice.

## R6.5 — first-customer management path decision after R6.4 code merge

- User confirms permission for a temporary customer-router test with
  one previously provided public SSH endpoint; API-SSL or TR-069
  may alternatively be considered. **The previous unexpected
  host-key change remains unresolved**, so no supplied existing
  password was ever used in this milestone and no actual router
  management login or configuration occurred.
- Read-only source-of-truth recheck: private GitHub `main`,
  clean owner FileVault Mac `main` and clean unchanged
  Ubuntu 26.04.1 VPS `main` all matched
  `af1f12259957557e22a8306e767287d5dd57043e`.
  Actual GitHub CI `36226064609` and `36226186394` both
  independently SUCCESS for all four jobs. Real VPS K3s,
  PostgreSQL and nftables remain inactive.
- Owner Mac maintains the separately generated restricted
  DEV-08 SSH RSA public/private key in its owner-private
  router-lab folder. The `ssh-read.template.json` remains
  deliberately unusable until physical identity, actual
  authorized source, non-disruptive recovery, and trusted LAN
  fingerprint are confirmed. No real customer configuration
  file or new router-account secret was prepared or deployed.
- Additional unauthenticated, **single exact user-provided
  endpoint** two-port availability check: default binary
  API-SSL port and standard HTTPS port were not

  verified reachable. They may differ behind NAT/source
  restrictions; this result does not establish actual
  RouterOS service configuration. No passwords/certificates
  sent, TLS trust bypasses or arbitrary subnet scans.
- WinBox application is installed/running on owner Mac,
  but macOS Accessibility permission for remote UI
  automation was rejected; no known authenticated
  WinBox session or independent direct LAN identity
  is available to remote tooling. No GUI router edits.
- The documented first step is owner/trusted-LAN WinBox
  validation, private recovery proof, then new dedicated
  least-privileged read-only SSH account bound to the
  actual approved management-client source, using the
  already-generated owner-private public key. Existing
  SSH listener, NAT, firewall and subscriber connectivity
  remain untouched. The R6.4 strict key-only read
  cannot start without its independent proof and
  deliberate local owner authorizations.
- API-SSL and TR-069 are independent future branches,
  NOT silent substitutes: the former requires real
  TLS server identity, an actual binary API adapter
  and restricted private route; the latter requires
  actual verified installed tr069-client package
  matching RouterOS/architecture plus a tested,
  isolated and TLS-trusted custom IPAT ACS endpoint.

- This milestone is documents/protocol decision
  only. R6.3/6.4 and full-product acceptance gates
  remain unchanged: physical devices enrolled ZERO;
  actual reported customer firmware NOT VERIFIED,
  production readiness NO_GO. No new architecture
  decision requiring a DECISIONS.md change.

## R6.6 — native Rust bounded CWMP read RPC, synthetic sealed session, private actual Axum HTTP

- After confirming canonical R6.5 documents and approved design, extended original `crates/cwmp-protocol` with a bounded SOAP 1.1 / CWMP 1.0 `GetParameterValues` serializer and strict correlated response/CWMP numeric fault parser. Only exact `Device.DeviceInfo.SoftwareVersion` can be requested. DTD, oversized/deep XML, unexpected namespace/method/parameter/duplicate reply, forged SOAP ID, wrong XSD namespace and invalid type/value are denied; returned values are not logged. This is a narrow read-only profile, NOT full CWMP support.
- Extended **existing sealed** `cwmp-admission` synthetic peer/tenant/lease registry: only a prior trusted synthetic Inform and truly empty CPE POST without Content-Type or SOAPAction permit a single lab read RPC. Same peer and opaque internal lease must correlate one response or CWMP fault before closing; wrong tenant/peer, duplicate issue, replay, malformed reply and unfinished session fail. Metadata excludes raw CPE values and keeps operator review, physical verification and tenant write authorization FALSE. Persistent mTLS/session recovery does not exist.
- Created `apps/cwmp-gateway` executable Rust/Axum private HTTP **parser-only** laboratory. It requires a separate opt-in to start, always binds literal 127.0.0.1:3300, never exposes real unauthenticated ACS SOAP ingestion: actual `/cwmp` returns HTTP 503 even with forged tenant/client-cert HTTP headers. `/lab/parse-inform` returns only no-enrollment/no-authentication boolean metadata. Parsing does NOT send InformResponse or invoke the sealed admission registry.
- New `deploy/scripts/lab/r66/cwmp-loopback-http-smoke.sh` executes the real binary in a temporary private listener with real synthetic HTTP calls. It verifies default opt-out, XML acceptance/negative input, HTTP 503 on `/cwmp`, no echoed identifiers and exact loopback bind, then cleans up the temporary process. Added this exact network-smoke run to unit-tests CI, without any real router/ONT traffic.
- Actual disposable Ubuntu 26.04.1 source checkout passed initial 9 new RPC, 7 new session/lease and 5 new gateway Rust tests; direct temporary real loopback HTTP smoke PASS after correcting a case-insensitive HTTP header check in the test script. Full locked-workspace final regression and independent reviewed final GitHub CI are tracked as additional evidence; no live VPS ACS service, external TLS port, PostgreSQL service, K3s, firewall, tenant enrollment or physical device access is activated.
- Commercial/physical AC-03 and full FR-009/010 **NOT COMPLETE**: real server TLS/mTLS CPE trust-anchor/SPKI verification, distinct operator-authorized persistent enrollment, full authenticated HTTP CWMP session semantics/timeout and durable locks/replay, per-device data model, correlated live RPC and tested exact ONT firmware are outstanding. USP remains separate and mandatory per ADR-002. Existing no-provider-firewall decision unchanged.

## R6.7 — asli TLS 1.3 mTLS wajib pada gerbang ACS Rust, lingkungan terbatas

- Baseline resmi dari sumber utama dan keputusan: ACS Rust tetap asli, tidak
  memakai GenieACS; USP/TR-369 tetap proses wajib terpisah. Protokol R6.6
  dan sealed Rust synthetic-peer *belum otomatis* memiliki autentikasi
  jaringan. Ini blocker penting menuju perangkat fisik.
- Kode baru `apps/cwmp-gateway/src/bin/cwmp-mtls-lab.rs`: listener
  TLS1.3 aktual berbasis Rustls, wajib sertifikat klien dengan
  rantai CA yang dipercaya dan tujuan clientAuth tepat. Proses
  hanya dapat diaktifkan khusus untuk laboratorium
  `IPAT_RUN_PRIVATE_CWMP_MTLS_LAB=YES`, melarang root,
  mengikat literal 127.0.0.1:3433, dan menerima berkas
  CA, sertifikat server serta key server di folder privat 0700,
  mode 0600, pemilik sama, no symlink, no hardlink,
  `O_NOFOLLOW`, batas ukuran 32 KiB. Tidak ada trust-bypass,
  autentikasi client anonim, maupun TLS early data.
- Hanya `/lab/mtls/parse-inform` memvalidasi SOAP
  Inform sintetis setelah handshake mTLS. Respons hanya
  menyatakan transport CA terverifikasi, *bukan* device
  enrollment, tenant binding ataupun izin provisioning.
  Semua request `/cwmp` tetap HTTP 503,
  sekalipun sertifikat klien benar dan header palsu
  mengklaim identitas klien atau tenant.
- Skrip `deploy/scripts/lab/r67/mtls-loopback-contract.sh` menggunakan

  OpenSSL untuk membuat CA/server/client EC sintetis
  **sementara**, dan cURL tepercaya untuk memverifikasi
  TLS1.3 sungguhan dengan private listener Rust. Uji
  tanpa sertifikat, CA salah, EKU serverAuth untuk klien,
  hostname server salah, CA server salah,
  private key symlink, mode private key tidak aman,
  DTD SOAP terlarang dan setiap akses real /cwmp
  ditolak. Skrip menghentikan proses **miliknya**
  dan menghapus semua material kriptografis tes.
  Enam test statis tambahan memverifikasi invariant.
- **Belum diuji perangkat asli, belum siap SaaS/produksi:**
  CA tepercaya saja tidak membuktikan kepemilikan
  ONT maupun tenant. Belum ada cert leaf/SPKI pinned
  per device, pemetaan ke enrollment terverifikasi,
  pencabutan cert/rotasi kunci terkelola,
  integrasi kepada sealed AuthenticatedPeer,
  penyimpanan sesi PostgreSQL, request-response
  CWMP sungguhan, atau uji model/firmware ONT fisik.
  No-go operasional tetap berlaku; tidak ada
  perubahan server publik, firewall penyedia,
  K3s atau database live, dan tidak ada
  penggunaan kredensial pelanggan.
- Bukti real VPS+CI+Restic final perlu dicatat
  sebagai komentar PR immutable setelah source

  gabung; hindari commit status baru yang
  terus mengubah SHA sebelumnya.

## R6.8 — tiga workspace dashboard privat, tanpa promosi identitas tenant palsu

- Rancangan UI yang dapat dicoba untuk tiga workspace:
  **Platform Admin**, **Tenant Admin ISP contoh**, dan
  **Operasional/NOC ISP contoh**, dari sumber
  `web/lab/dashboard-preview.{html,css,js}`.
  Link ditempatkan pada halaman lab R5.9;
  dihidangkan hanya melalui toggle preview
  privat nonroot dan **tidak pernah** dari
  router K3s/public bind.
- Selector peran hanya mengubah
  data ilustrasi dan menu sintetis,
  **bukan autentikasi**, bukan impersonasi,
  bukan klaim entitlement yang aktif.
  Pratinjau memakai tiga GET same-origin
  saja dari lab health, status dan
  katalog delapan target rencana;
  hasil fisik terdaftar tetap nol.
  CSP strict, no-store dan tidak
  memakai CDN, secret, form atau
  backend mutasi.
- Endpoint nyata wildcard platform,
  tenant dan operations selalu
  HTTP 401/no-store, seluruh metode,

  meskipun klien menambahkan
  Authorization, X-Tenant-Id,
  X-Verified-Role dan Host
  yang dipalsukan.
  Sejumlah tes Rust pada
  `control-api` memastikan
  preview absen pada mode K3s
  dan tidak membuka API.
- Kebijakan pure
  `crates/authz-core/src/dashboard.rs`
  menyediakan matriks referensi
  berbeda untuk metadata platform
  dan data tenant/POP, termasuk
  admin, NOC, helpdesk dan auditor,
  dengan penolakan silang dan
  bulk PPPoE berisiko tanpa
  persetujuan. **Belum terhubung
  ke endpoint** karena JWT OIDC
  dan membership tenant terverifikasi
  belum diimplementasikan.
  Exact role matrix tetap
  proposal menurut ADR-009.
- Bukti awal real Ubuntu
  terisolasi: sepuluh tes

  `control-api`, sembilan
  tes `authz-core` dan
  enam kontrak Python serta
  satu uji Node DOM/fake-fetch
  untuk switching sintetis
  dan larangan akses real API.
  Bukti total workspace, HTTP
  privat di owner Mac, empat
  CI GitHub independen,
  encrypted exact source backup
  dan final SHA harus dicatat
  setelah review.
- R6.8 **menambah produk
  yang dapat ditinjau**, tidak
  mengubah keputusan produksi
  NO_GO: platform admin, tenant
  dan NOC sungguhan tetap perlu
  login OIDC/MFA, verified tenant
  domains, DB RLS+audit, device
  onboarding ACS/USP dan uji
  interoperabilitas fisik.
  Tidak ada perubahan live
  K3s, PostgreSQL, firewall
  penyedia ataupun perangkat.

## R7.0 — skema kandidat identitas dan keanggotaan dashboard (laboratorium saja)

- Setelah R6.9, dokumentasi autentikasi pratinjau
  melalui PR #68 digabung setelah empat job CI
  sukses pada `36245992222`, ke baseline sumber
  `04c711c956bcc89fb30e83b9ec44713841ca9135`.
  Kode Rust R6.9 sebelumnya hanya memeriksa
  tanda tangan token RS256, **tidak** keanggotaan,
  role, MFA maupun hak bisnis; API bisnis tetap 401.
- Memulai R7.0 `0003_lab_identity_memberships.sql`,
  **kandidat** model identitas PostgreSQL sintetis.
  Menambahkan keanggotaan tenant-issuer-subject-role,
  grant POP FK gabungan tenant+issuer+subject+role
  dan prinsip `platform_owner` yang terpisah.
  Semua tabel `ENABLE+FORCE RLS` dan tidak
  mempunyai grants maupun policies untuk
  `ipat_app_runtime`; tidak ada fungsi
  SECURITY DEFINER maupun endpoint lookup
  yang bisa menerima klaim sub/tenant palsu.
- Menambahkan pengujian PostgreSQL 16 disposable
  `test_identity_memberships_integration.py`
  untuk penolakan runtime, batas tenant+POP,
  validasi role/expiry dan pemisahan platform.
  CI mengeksekusi suite ini sesudah migrasi
  dasar RLS serta outbox sintetis, bukan

  terhadap database VPS nyata.
- Rencana bersyarat untuk ketiga dashboard:
  UI privat sudah ada; login/MFA/permissions
  lab membutuhkan kira-kira 1–2 minggu;
  integrasi admin/tenant/NOC memakai
  PostgreSQL/observability awal 4–6 minggu;
  pilot ISP perangkat fisik 8–12 minggu,
  semuanya bergantung pada sumber daya,
  identitas/akses dan tujuh gate independen.
  Detail [rencana terukur R7.0](DASHBOARD_MEMBERSHIP_R70.md).
- **Belum selesai:** operator-approved DB
  enrollment nyata, OIDC Keycloak login+PKCE+MFA,
  verifikasi domain, runtime role/POP membership,
  frontend server-driven menus, device
  inventory/telemetry/ACS real, real HA/PITR.
  Produksi NO_GO; proyek tidak menggunakan
  firewall penyedia hosting atau mengubah
  K3s/PostgreSQL/firewall live.

## R7.1 — ZTE C320 safety-first, real physical test still blocked

- User requested immediate one-OLT real connection and firmware upgrade.
  Reported expectation does NOT match actual readiness: physical DEV-01
  has no verified card/firmware, approved private management access
  or independently tested device recovery; TC-OLT-01 remains NOT RUN.
  Original FR-016 permits a simulator when physical access is unavailable;
  actual firmware updates are outside initial seven-day guarantee and
  must meet FR-017 exact-tuple proof and high-impact approval requirements.
- Built pure Rust offline crate olt-core with bounded parsers for two
  candidate read-only ZTE C320 CLI outputs (show card and
  show version-running), duplicate/unsafe/malformed input rejection
  and non-executable firmware approval gate. WRITE_ENABLED=false.
- Built a separate offline owner-private Rust importer for two
  fixed output files in a mode-0700 directory with owner-only mode-0600
  files, no symlink/hardlink, no network. Successful output explicitly
  says physical identity UNVERIFIED, compatibility UNTESTED, no firmware.
- Added large bright RED PRD gap alert across three synthetic
  preview dashboards plus PRD_DEVIATIONS_R71.md audit. Even operator
  declarations satisfying all software checks produce HUMAN_REVIEW_ONLY;
  there is no firmware image transport, upload or executable upgrader.
- R7.1 tests are synthetic and isolated. Physical connection remains
  BLOCKED until approved access, exact real card/firmware versions,
  private trusted management route and safe onsite rollback are provided.

  No K3s/PG/nftables live services, shared provider firewall or
  customer device configuration was touched by this feature.

## R7.2 — hard-disabled local C320 firmware SHA-256 check and PRD gap visibility

- Project source-of-truth R7.1 pre-checked on
  clean private GitHub/Mac: `bbcffda3fb13689162561cde2abd872eb1f59985`,
  final independent CI `36250596792` SUCCESS.
  Existing red banner on all Platform Admin,
  Tenant Admin and NOC private previews
  continues to state physical OLT and
  firmware functionality **NOT DONE**.
- Added `deploy/scripts/lab/r72/c320-firmware-check.py`
  for nonroot local integrity only: exact
  owner-private mode0700 directory,
  three fixed 0600 operator-owned no-link
  files, safe `O_NOFOLLOW`, bounded
  syntax and image size, strict DEV-01
  C320 metadata, SHA256 over bytes
  compared to **operator-supplied checksum**
  only after explicit offline opt-in.
  Zero network access, no firmware
  upload/actuator, no device writes.
  Output permanently marks physical
  firmware compatibility, vendor
  authenticity and human approvals
  FALSE, even if hash matches.
- Nine local synthetic Python tests

  exercise positive **fake** image hash,
  corrupt image, fraudulent checksum,
  wrong model, metadata injection,
  lack of consent, unsafe directory/
  file permissions, links and absence
  of firmware remote commands.
  Initial negative traversal test
  failed because release metadata
  permitted `..` inside a field;
  implementation was corrected and
  all nine tests then passed, before
  CI review or merge.
- CI now repeats the new checks on
  disposable runners with no real
  firmware bytes. Extended red PRD
  ledger and R7.2 runbook. Still
  requires physical C320 identification,
  independently approved access
  and firmware release/build/board
  compatibility as well as external
  recovery, maker-checker and
  owner-approved maintenance window.
  Issues #72 and #73 remain OPEN.
- Only feature implementation
  is complete at this initial

  local checkpoint. Independent
  CI, final exact-main backup
  and commit evidence must be
  verified separately. No actual
  C320 packets or firmware
  writes were made. Commercial
  and field acceptance NO_GO.


## R7.2 final main-CI verification — 2026-09-26

- Private GitHub main and clean owner Mac checkout were verified at
  `51763be13a7934f1e2e75a483a5adcf3853fdfcb`, merged PR #75.
- Independent GitHub main Actions run `36251850735` finished SUCCESS:
  `unit-tests`, `postgres-rls-restore`,
  `postgres-physical-recovery-lab` and
  `k3s-ubuntu26-disposable` all SUCCESS in isolated CI.
- This is CI evidence for an **offline** firmware hash guard, NOT
  physical OLT firmware verification, production deployment or a new
  offsite post-merge backup. Firmware execution remains DISABLED.

## R7.3 — offline C320 configured/real board alias correctness

- Historical vendor-manual-shaped **synthetic** text exposed a
  compatibility-of-format problem: version MVR may use CfgType
  while card inventory has a distinct RealType at the same slot.
- Shared Rust parser now preserves both card names; the offline importer
  requires MVR to match EITHER observed name **at the exact slot**.
  Unrelated names, incorrect slots and boot-only records are denied.
- Changes: `crates/olt-core/src/lib.rs`,
  `crates/olt-core/src/bin/c320-offline-review.rs`,
  `crates/olt-core/tests/c320_fixture.rs`,
  `deploy/scripts/lab/r71/test_r71_offline_cli.py`,
  `docs/C320_BOARD_ALIAS_R73.md` and this status/deviation ledger.
- Executed on owner Mac: 5/5 existing R7.1 static Python contracts PASS,
  Python compile PASS, `git diff --check` PASS. Mac has no
  accessible `cargo`; Rust and offline Rust-CLI tests require
  independent CI. Do NOT count them passed before actual run.
- DEV-01 TC-OLT-01 stays NOT RUN/untested. No SSH/OLT network
  connection, firmware upload, firewall, VPS service or database change.
  Missing exact physical boards/build, approved trusted private access,
  independent recovery and verified vendor firmware/rollback remain blockers.


## R7.3 final independent evidence — 2026-09-26 Asia/Jakarta

- Feature PR #76 merged through reviewed squash commit
  `7bc5e15af1a859a5cd9d819f79f9f1bd645dd021`.
  The earlier feature CI run `36252923319` FAILED rustfmt
  before Rust unit tests; that failure was not hidden.
- Applied the pinned 1.98.1-compatible rustfmt formatting
  using a nonprivileged, stdin-only Ubuntu rustfmt invocation.
  Feature PR exact revised source SHA
  `58170b47c470a3f99e7a72278c28d6b81ad1f4da`
  passed independent CI `36253247630`:
  all FOUR isolated jobs PASS (locked workspace tests and
  offline C320 CLI tests; synthetic PostgreSQL RLS/restore;
  separate synthetic physical PG recovery; disposable Ubuntu26 K3s).
- Independent **post-merge main** CI `36253501422` also
  SUCCESS for exact merged code SHA
  `7bc5e15af1a859a5cd9d819f79f9f1bd645dd021`:
  all four independent jobs PASS. These CI runners are
  disposable and do not validate a production deployment.
- The owner FileVault Mac clean `main` checkout matched the
  above GitHub merged SHA; locally the earlier 5/5 R7.1
  static guards and 9/9 R7.2 synthetic firmware hash tests
  passed; read-only Rust formatting was separately checked.
- Encrypted source-only Restic snapshot `7a5aecd9`
  contains the exact merged `main` Git archive. Complete
  repository pack read (108/108), isolated source-tar
  SHA256 restore and independently restored selected
  historical readable-config archive PASS.
  **LIMITATION:** same-Mac local backup, NOT offsite
  full-host restore, PostgreSQL PITR, K3s datastore
  recovery or an approved production disaster-recovery gate.
- **FR-016 physical / TC-OLT-01 remains BLOCKED and
  DEV-01 remains `untested`.** No real OLT credentials,
  SSH/SNMP session, packet exchange, board/firmware
  inventory or vendor-validated management command was
  exercised. New configured/physical board alias behavior
  is synthetic offline *format* validation only.
- **FR-017 OLT firmware updates remain hard-disabled.**
  Hash equality with an operator-provided checksum cannot
  prove vendor authenticity, exact-card suitability,
  recoverability or human approval; no OLT write actuator.
  Real field pilot requires trusted owner-approved
  private management, exact boards/build, independent
  recovery; high-impact firmware needs official matching
  vendor release, onsite rollback, maintenance approval
  and separate maker/checker sign-off.
- This final documentation checkpoint changes evidence only;
  binding architecture ADRs are unchanged. It does NOT
  remove the large RED deviations banner in the three
  private dashboards, nor authorize VPS firewall, database,
  K3s, customer router or physical firmware changes.
- Open tracking: Issue #72 (authorized DEV-01 first physical
  read) and Issue #73 (separate firmware safety approval).
  Next work: independently supply private real C320
  capabilities to lab operator, first read-only test,
  review sanitized observations and update DEVICE_MATRIX
  per exact tuple, with failures/uncertainty included.


## R7.4 — Per-dashboard PRD gap audit + isolated nonproduction C320 preparation

**2026-09-27, developer Mr. iPat. Status implementation:** candidate source
tested locally; independent feature PR and clean main CI must be separately
reported before treating source as merged.

- Owner states physical trial units are DEDICATED LAB HARDWARE, not active ISP
  subscribers. This declaration reduces intended blast radius, not the
  obligation to verify actual isolated topology, ownership, exact build,
  recovery and a restricted management channel.
- Audited three existing private-only browser previews against existing
  v0.1 FR items. Platform Admin misses real OIDC/MFA, verified domains,
  tenant/plan runtime; Tenant Admin misses trusted tenant/POP membership
  and server-driven menus; NOC lacks physical OLT/ACS telemetry, operational
  diagnostics and approved live PPPoE flows. All business APIs stay 401.
  NO PRD scope reduction or new architecture ADR is approved.
- Added large RED, context-specific PRD gap lists for each workspace in
  private dashboard JS/HTML/CSS. The selector remains demonstrative only:
  it never confers role, tenant membership or API access.
- Added a separate Python DEV-01 OFFLINE-ONLY lab packet validator that
  reads exactly private 0700 packet folder stage.json and plan.json,
  both mode0600, rejects duplicates/extra keys, unknown target/production
  declarations/unsafe file permissions and all undeclared or false gates.
  Even when all human statements are true it returns
  HUMAN_REVIEW_REQUIRED and all physical/firmware authorization fields FALSE.
  Existing R6.0 metadata staging and R7.1 offline Rust C320 parser stay
  separate. No network/SSH transport or firmware actuator was added.
- Source paths changed: web/lab/dashboard-preview.html/.js/.css,
  deploy/scripts/lab/r68/test_dashboard_preview.py/.mjs,
  deploy/scripts/lab/r74/lab-readiness.py and test_lab_readiness.py,
  .github/workflows/ci.yml, docs/DASHBOARD_PRD_AUDIT_R74.md,
  docs/C320_ISOLATED_LAB_R74.md, docs/PRD_DEVIATIONS_R71.md,
  docs/DEVICE_MATRIX.md and this PROJECT_STATUS.md.
- Local owner Mac: Python R68 static tests 7/7 PASS,
  JS R68 DOM/role demo no-API synthetic test PASS,
  R74 offline packet 8/8 tests PASS, Python compilation and
  git diff --check PASS at initial source checkpoint.
  CI/post-merge evidence will be recorded separately.
- MUST next: physical operator console identifies real C320 exact boards
  and firmware, confirms isolated lab, trusted private management route,
  dedicated read-only account and backup; peer-reviews packet. Then ONE
  actual read-only test per independently supported vendor command,
  sanitize evidence and update exact tuple in DEVICE_MATRIX.
  TC-OLT-01 currently NOT RUN, firmware hard-disabled, production NO_GO.
- MUST separately for actual dashboards: approved identity provider +
  MFA, approved membership + verified POP mapping, server-driven
  entitlements, API/RLS/cookie/domain negative tests and audit.
  No VPS firewall/K3s/real PostgreSQL deployment in this milestone;
  external seven production safety gates are not bypassed.


## R7.4 final code-release lab evidence — 2026-09-27 Asia/Jakarta

- PR #78 is MERGED on GitHub private main at exact code SHA
  `24bca90e45001aea87440c84b458df025771b8f6`.
  Its reviewed feature run `36281657014` and independent post-feature
  main run `36281835353` both completed SUCCESS in all four jobs:
  Rust workspace/static UI/offline packet, disposable Ubuntu26 K3s,
  synthetic PostgreSQL RLS/restore and synthetic independent PG recovery.
  All runners are disposable, not actual HA/K3s production deployment.
- Owner FileVault Mac created encrypted **SOURCE ONLY** restic snapshot
  `fc51c909` at exact code SHA; restic full pack read 112/112,
  isolated restored archive SHA256 and historical partial readable-config
  restore SHA256 PASS. This remains same-Mac lab backup,
  NOT complete offsite VPS backup, PG PITR or K3s disaster recovery.
- Exact SHA-preserving Git bundle transferred from owner Mac to
  nonprivileged actual Ubuntu 26.04.1 VPS source
  `/home/openai/workspaces/ipat` with independent SHA256, bundle
  verification, clean fast-forward. No remote root action,
  firewall rules, K3s, PostgreSQL or production services changed.
- Actual VPS nonprivileged `cargo fmt --all -- --check` PASS;
  `cargo test --workspace --locked --offline -q` PASS for the whole
  current Rust workspace; R7.4 Python offline validator 8/8 and
  existing R6.8 private dashboard Python 7/7 PASS there.
  Mac independently reran Node DOM synthetic role-switch tests
  and static R7.1 red PRD guard tests.
- Controlled R5.9 stop/start restarted ONLY tracked nonroot private
  control-api, with Mac localhost SSH tunnel (not public service).
  End-to-end R5.9 tunnel and VPS loopback checks PASS.
  Actual Mac private HTTP GET for preview HTML/JS/CSS returned 200,
  `Cache-Control: no-store`, includes large red PRD warning,
  per-dashboard GAP_LEDGER with C320 FR-016/TC-OLT-01, and red CSS.
  Actual forgery attempts against platform/tenant/operations business
  API paths all returned HTTP401. This is NOT OIDC/RBAC end-to-end.
- New Mac owner-private `~/.local/share/ipat/device-intake`
  and `~/.local/share/ipat/c320-private-packet` directories are
  mode0700. Example intake metadata is intentionally incomplete,
  private plan.json mode0600 has every declaration FALSE,
  stage.json does not exist. Actual R74 --check-packet correctly
  DENIED: no invented device facts or approval. No credentials stored.
- First Git push with the host's stale macOS credential helper was
  rejected for workflow scope, despite current authorized gh login
  advertising workflow scope. Explicit Git credential-helper override
  used the verified current gh credential and succeeded, without
  adding tokens to Git, command flags or chat.
- PRD decision: preserve v0.1. Only the simulated preview's
  **visibility of explicit gaps** has improved; all three dashboards
  remain missing runtime trusted identity/tenant entitlements/data.
  User declares DEV-01 a nonproduction lab unit; this assertion is
  NOT independently verified physical evidence. TC-OLT-01 still
  NOT RUN, no remote OLT connection, no vendor release validation
  or firmware upgrade. Real device pilot and all seven production
  external safety gates remain blocked separately.
- Next MUST: owner privately records actual physical C320 boards,
  firmware, trusted isolated path and least-privilege read-only user,
  independent recovery and reviewer; run one separately authorized
  physical read, review sanitized evidence, update DEVICE_MATRIX
  for the exact tuple. Dashboard MUST follows verified OIDC/MFA,
  authoritative tenant/POP membership, server-driven menus and
  integrated backend/DB negatives; never turn on UI access merely
  because the browser switches workspace.


## R7.5 — owner-approved domain-later/private-lab-first implementation

**2026-09-27.** Product owner approves sequencing only: subdomain/custom
domain verification and across-DOMAIN session isolation to M2 after
working integrated private system. Existing FR-004 commercial priority,
AC-09 and SaaS target unchanged. No approval to postpone tenant data
isolation FR-001/002/003, MFA/verified membership, backend/RLS/job
negative tests or expose a public multi-tenant URL. Recorded as
ADR-020 APPROVED_SEQUENCE while ADR-014 DNS/OIDC technical choices
remain OPEN.

Implementation candidate: single existing private SSH loopback URL;
new static control-api private GET /lab/rollout-phase returning
only explicitly deferred domain policy and FALSE for physical,
customer auth, real verified tenant isolation and firmware.
No endpoint mutation or runtime auth bypass. Updated all
three private dashboard previews with prominent domain-deferred
banner plus MUST tenant isolation warning and fail-closed
phase fetch/check. Extended Rust Axum tests for opt-in-only,
no-store, JSON false gate and POST/PUT/DELETE 405; Node DOM
tests reject forged domain-enabled/tenant-isolation-disabled
manifest; disposable ephemeral real HTTP smoke verifies
read-only route and unchanged forged business API HTTP401.
Source: web/lab/rollout-phase.json, apps/control-api/src/main.rs,
web/lab/dashboard-preview.html/.css/.js,
deploy/scripts/lab/r68/test_dashboard_preview.py/.mjs,
deploy/scripts/lab/r68/private-dashboard-http-smoke.sh,
docs/LAB_FIRST_DOMAIN_LATER_R75.md, affected PRD,
ARCHITECTURE, SECURITY, DECISIONS, sprint, and this status.

Initial owner Mac check: JSON syntax PASS, Node JS parse/
synthetic UI+negative phase test PASS, 7/7 R6.8 static Python
tests PASS, git diff --check PASS. Real Rust compilation, CI,
reviewed merge, live private preview refresh and exact source
restore must be verified separately and not inferred here.
No C320 device connection or firmware change;
TC-OLT-01 remains NOT RUN. Existing VPS firewalls/K3s/
PostgreSQL production safety gates unchanged.


## R7.5 independently verified feature/code release — 2026-09-27

- Owner-approved **sequencing** (ADR-020) preserves FR-001/002/003
  true tenant/POP isolation as mandatory while postponing only
  FR-004/AC-09 domain ownership, TLS/cookie/hostname isolation
  until M2 after private integrated lab; ADR-014 design OPEN.
- Feature PR #80 was merged at exact canonical CODE SHA
  `1c46df653557cb7ecc940abe3c3eedb674fe4bf8`
  following all-four-job successful GitHub PR CI `36283055368`.
  This includes independent locked Rust/security/private HTTP
  checks, disposable Ubuntu26 K3s, and both separate
  disposable synthetic PostgreSQL checks.
- Same owner FileVault Mac encrypted SOURCE-only Restic
  snapshot `9c38f61c` at the exact reviewed code SHA
  verified 116/116 pack read, isolated exact SHA256 source
  restore and historical selected partial readable-config
  restore. This does NOT close independent offsite DR,
  full VPS/PG/K3s or public deployment recovery gates.
- SHA256-verified Git bundle delivered code SHA to the existing
  nonprivileged actual Ubuntu 26.04.1 VPS checkout via
  independently known SSH host, clean fast-forward.
  Real VPS `cargo fmt --all -- --check` and complete
  `cargo test --workspace --locked --offline` PASS;
  preexisting synthetic R6.8 Python 7/7 and R7.4
  offline packet 8/8 PASS. No root installation or
  live PostgreSQL/K3s/firewall changes.
- Controlled R5.9 strict stop/start updated ONLY the
  nonroot private loopback dashboard preview, retaining
  localhost SSH Mac tunnel and exact source Git hash.
  Actual Mac HTTP GET rollout-phase was JSON validated:
  custom domains/public hostnames/real customer login,
  physical connections, firmware and E2E isolation
  all DISABLED/UNVERIFIED, while tenant isolation mandatory TRUE.
  Manifest was no-store, POST/PUT/DELETE each HTTP405,
  private UI rendered the MUST-not-defer isolation banner,
  and forged Platform/Tenant/Operations API requests
  all returned HTTP401.
- This release changed scheduling/visibility and
  a private static release-state contract ONLY.
  It did NOT implement OIDC/MFA, real tenant membership,
  live ACS/USP sessions, customer-domain routing or
  authenticated C320 network reads. Owner lab packet still
  all false; actual physical C320 TC-OLT-01 NOT RUN.
  All seven production external safety gates remain open.
- Independent POST-MERGE MAIN CI run `36283225844` at exact
  merged code SHA `1c46df653557cb7ecc940abe3c3eedb674fe4bf8`
  independently completed SUCCESS in all four jobs, including
  real disposable Ubuntu26 K3s smoke, locked Rust/security/UI,
  synthetic PG RLS/restore and separate synthetic PG physical
  recovery. None activated real VPS K3s, PostgreSQL or firewall.
  Final docs-only merge, exact docs main SHA backup and final
  docs-main CI must be verified separately after this checkpoint.


## R7.6 — owner Fadly custom domain intent and verified identity/menu candidate

**2026-09-27. Developer Mr. iPat.** Owner specifies intended
Fadly tenant custom domain ipat.fadly.id; intended IPAT
commercial platform domain ipat.id. DNS/TLS/business ownership
not independently verified; ipat.fadly.id is also the CURRENT
authorized management SSH hostname for lab VPS. The SSH
path was not touched or repointed, and there is no new
public HTTPS/customer routing. ADR-021 records owner naming
intent and the management-host collision; ADR-014 remains
OPEN, ADR-020 domain work remains deferred.

This milestone advances S1-02 identity-bound menu reference
BEFORE optional domain routing: verified pinned token issuer
is preserved in identity-core VerifiedSubject; authz-core
pure candidate-row bridge compares cryptographically
verified issuer+subject, intended exact tenant, single
candidate role/POP, record/token expiry, revocation and
nonempty synthetic approver before returning only existing
read-only DashboardSection policy. Genuine synthetic
generated RSA JWT integration tests cover positive
Fadly lab POP and cross-issuer/subject/tenant/POP,
revocation/expiry/forged platform and bulk write negative.
CandidateMembershipRow is NOT trusted DB provenance or
login: there is still no real MFA, restricted persistent
DB membership adapter, API entitlement enforcement or
operational customer dashboard. No domain membership
inferred from host or JWT claims. No migration/new device
actuator. Existing business HTTP401 policy unchanged.

Changed source: crates/identity-core/src/lib.rs and
tests/oidc_signature.rs, crates/authz-core/Cargo.toml,
src/lib.rs, src/verified_menu.rs and tests/verified_menu.rs;
CI explicit two signed-token tests; docs in
DOMAIN_INTENT_FADLY_R76.md, PRD, ARCHITECTURE,
SECURITY, DECISIONS, SPRINT_BACKLOG and dashboard
audit. At initial status entry tests not yet verified;
record actual local/independent CI after execution.
Next MUST audited membership DB read binding and
real IdP/MFA lab test; separate C320 physical
operator prerequisites not fulfilled, TC-OLT-01 NOT RUN.
Production stays NO_GO and no VPS/K3s/firewall writes.


## R7.6 final verified code release — 2026-09-27

**Identity-first Fadly tenant intent:** Product owner plans
ipat.fadly.id as eventual Fadly company custom domain and
ipat.id as future commercial IPAT platform domain. Neither
DNS ownership nor tenant-domain mapping was independently
verified or deployed. ipat.fadly.id currently remains the
operator's EXISTING lab VPS SSH target; no DNS, external
firewall, VPS management routing or public HTTPS changed.

**Immutable code PR #82 evidence:** first source CI run
36284416805 failed because authz-core added a Rust
identity-core test dependency without updating Cargo.lock.
This was fixed by generating Cargo.lock with *offline*
Cargo in an isolated unprivileged VPS Git worktree,
real generated synthetic RS256-signed token integration
6/6 identity-core and 4/4 issuer-bound authz menu
tests PASS there. Corrected feature commit e9d7479
independent GitHub CI run 36284492902 completed
SUCCESS in all FOUR jobs. PR #82 squash merged to
canonical main code SHA
881607e1a8a1a7bf8a9b93bd012d2a18801eb916.

**Source backup, SSH and independent real VPS proofs:**
Owner FileVault Mac source-only Restic snapshot 2eb84705
at exact code SHA; 120/120 entire encrypted packs full
read PASS and separate isolated restored Git archive
SHA256 and selected historical partial readable-root
config SHA256 PASS. This is NOT independent offsite VPS
whole-host/real PostgreSQL/K3s disaster recovery.

Exact code SHA moved via local Git bundle and SHA256
over preexisting identity-pinned key-only SSH to actual
nonprivileged Ubuntu 26.04.1 VPS main checkout,
clean fast-forward PASS. VPS cargo fmt --all -- --check
and full cargo test --workspace --locked --offline -q
PASS; existing private packet R7.4 Python 8/8 PASS.
Actual owner Mac used the existing R5.9 guarded
stop/start to restart only tracked nonroot private
dashboard and localhost tunnel, verified exact current
main source and strict loopback. Real HTTP requests
with forged Host: ipat.fadly.id, X-Tenant-Id: fadly
and X-Verified-Role: platform_owner all got HTTP401
from Platform, Tenant and NOC business endpoints.
This is a DENIAL check, not an authenticated
Fadly company integration or public-domain test.

The candidate identity-core issuer + authz-core
membership row bridge is OFFLINE/TEST-only; it has
NO approved trusted DB actor, persistent authenticated
tenant/POP lookup, real IdP/MFA login, authenticated
business route, actual customer records or firmware
actuator. DEV-01 actual physical C320 exact cards/
running firmware and independent rescue evidence
remain unavailable; TC-OLT-01 NOT RUN.
No real OLT connection or write, live VPS K3s,
PostgreSQL, host firewall/provider firewall or
public-domain routing happened. Production NO_GO,
all external recovery/security gates remain blocked.
Next MUST: independently trusted provider/DB membership,
MFA and end-to-end UI/API/RLS/jobs two-tenant controls,
and separately authorized physical C320 read-only pilot.

**Independent exact-feature-main CI:** GitHub Actions run
36284679208 on canonical code commit
881607e1a8a1a7bf8a9b93bd012d2a18801eb916
completed SUCCESS in all FOUR jobs independently of
corrected feature PR CI 36284492902. Results were
locked Rust workspace / explicit signed JWT/tenant
negative tests and source safety reviews, synthetic
PostgreSQL RLS logical restore, synthetic separate
PostgreSQL physical recovery, and disposable Ubuntu26
K3s smoke. These CI jobs do NOT prove a real OLT,
full production database/K3s HA or production login.


## R7.7 initial scope — 2026-09-27

Owner states ipat.fadly.id already points to the VPS; a
read-only recursive DNS lookup from owner Mac
observed A 202.162.204.121 and no AAAA answer in that
sample. This is NOT proof of domain ownership, certificate
issuance, safe public customer web service or real tenant
assignment. ipat.fadly.id continues to be current authorized
SSH management hostname, unchanged. Independent current
GitHub docs-main CI run 36285111075 completed all 4/4 SUCCESS
at preceding exact main SHA 1cd2733. R7.7 is a forward
feature branch, not completion of the overall PRD.

Scope: disposable ONLY PostgreSQL migration 0004 for exact
issuer+subject+tenant UUID+fixed read-only role+POP active
membership lookup through nonlogin function-owner and
nonlogin executor; no GRANT to app runtime and no
production endpoint. Includes 7 ephemeral SQL integration
tests and 5 offline static safeguards; security scope
ADR-022 PROPOSED. Next: actual real-user OIDC/MFA,
approved/audited persistent DB lookup adapter, tenant
RLS/POP per request, backend/menu/queue denial and
physical C320 evidence. Any CI, backup and deployment
results for R7.7 must be added AFTER independent
execution, not inferred from scripts.


**R7.7 independent feature-CI evidence:** feature commit
7e131bd52fe714537a21742aa699d13971caad9e
passed all FOUR GitHub Actions jobs in run 36285594713:
locked Rust workspace / offline R7.7 5 static
safety contract tests, disposable PostgreSQL
RLS + logical restore, R7.0 synthetic memberships
followed by R7.7 7 exact identity-lookup
integration tests, independently isolated
PostgreSQL physical restore and disposable
Ubuntu 26 K3s smoke. This is CI SYNTHETIC ONLY.
The owner Mac separately ran the 5 static
contract tests and Python syntax with success.
No real database was migrated, no actual
identity user authenticated, and no real
device has been read.


## R7.8 in-progress private identity→PostgreSQL→menu integration — 2026-09-27

User requested continue and asked for concrete next operator actions.
Required owner inputs remain: independently trusted read-only
ZTE C320 local console board/firmware/route/backup metadata;
tested VPS out-of-band recovery and off-site encrypted
restore; legitimate MFA-enabled lab IdP operator identity.
No passwords, serials, private keys or production routes
are solicited for chat/Git. DNS ipat.fadly.id already
observed pointing, but it still doubles as the trusted
lab SSH hostname; NO DNS, TLS, firewall or public UI
changes are approved before recovery and authentication.

This next slice creates an ACTUAL optional read-only
Rust Axum endpoint that verifies a genuine RS256 token,
looks up exact verified issuer+subject and untrusted
requested tenant UUID/role/POP through the existing
restricted SQL function and applies the previous
fail-closed Rust server-side menu policy. Returned
tenant slug is DB-owned, not extracted from Host.
A second explicit env flag, owner-only private
0600 DB config, fixed dedicated SQL role, absolute
Unix socket and independent private nonroot
127.0.0.1:3001 bind are required. No customer data,
real MFA/session or business API is activated.

New source: apps/control-api/src/tenant_membership_lab.rs;
apps/control-api/src/main.rs, Cargo.toml, Cargo.lock;
deploy/db/migrations/0004_lab_scoped_identity_lookup.sql
adds DB-owned tenant_slug projection; disposable
CI seeded minimal reader fixture; new static safety
tests and explicit signed JWT+real PostgreSQL+real
HTTP handler test. Initial source was formatted
and compiled in an independent nonroot Ubuntu
VPS worktree (NOT the running main service):
16/16 Rust control-api tests PASS (the real SQL
test requires separate CI ephemeral PG env);
Mac 4/4 R7.8 offline safety checks PASS.
Independent real PostgreSQL CI not yet concluded
at first status entry. Default private preview
and actual business HTTP401 are preserved;
physical C320 TC-OLT-01 still NOT RUN,
firmware still HARD DISABLED, public access
and real VPS PostgreSQL/K3s still NO_GO.


## R7.8 independently verified final code milestone — 2026-09-27

**Source and immutable CI evidence.** Source PR #85 feature commit
b8b92b8f73f6e81dc8e8091da73c6f7dc99921b3
passed GitHub run 36287648718 SUCCESS 4/4. Crucially,
its disposable PostgreSQL integration job independently
ran the exact test
tenant_membership_lab::tests::r78_end_to_end_real_signed_jwt_real_restricted_sql_real_axum_router,
with explicit log `test result: ok. 1 passed; 0 failed`.
This was a REAL Rust Axum HTTP handler, genuine ephemeral
RS256-signed JWT and separate low-privilege PostgreSQL
test account on a throwaway PostgreSQL 16 instance,
not a mocked DB integration. Its companies and
reviewer records were nevertheless SYNTHETIC CI fixtures,
with no real human MFA, subscriber, ONT or OLT data.
Feature was merged via PR #85 squash into canonical main
15cd78de46bc08f1563a3eeee229bbcf7e05b3ab.
Separate POST-CODE-MAIN CI run 36287880873 at this
exact code main SHA completed SUCCESS 4/4:
locked workspace/auth/security and private HTTP,
disposable Ubuntu26 K3s, ephemeral PostgreSQL RLS
logical restore and independent disposable PostgreSQL
physical restore. These do NOT establish production
PostgreSQL HA or off-host whole-server recovery.

**Independent actual nonroot Ubuntu26 and private Mac HTTP.**
Owner FileVault Mac backup source-only encrypted Restic
snapshot 588edeb3 of the exact merged code main:
full read of 126/126 encrypted packs PASS;
isolated source archive SHA256 restore PASS and
historical selected PARTIAL readable-root config
SHA256 restore PASS. This is NOT independent
off-site real VPS whole-host/PG/K3s disaster recovery.
A local SHA256-verified Git bundle was safely
fast-forwarded from clean owner GitHub main/Mac to
clean actual NONROOT Ubuntu26.04.1 VPS main checkout
at the exact source SHA. VPS independently ran
cargo fmt --all -- --check and full
cargo test --workspace --locked --offline PASS,
4/4 new R7.8 private HTTP static guards,
5/5 R7.7 SQL static guards, and 8/8 offline
R7.4 C320 readiness tests PASS.

Existing reviewed nonroot R5.9 private preview was
stopped/rebuilt/restarted from exact final code SHA,
over the preexisting trusted SSH localhost tunnel;
actual Mac HTTP GET /lab/rollout-phase confirmed
strict custom domains off, mandatory tenant isolation,
no real authenticated business data or firmware.
Private /lab/auth/sections remains HTTP404 on
the DEFAULT live preview because no actual reviewed
OIDC+PostgreSQL service configuration or account
is installed. Actual forged Host: ipat.fadly.id,
tenant and platform role all still received
HTTP401 across Platform, Tenant and Operations
business APIs. The R7.8 HTTP route is implemented
and PROVEN ONLY in disposable opt-in CI, NOT
an active real customer login on the VPS.

**Still blocked by external prerequisites:**
Operator-sanitized C320 exact boards/firmware,
private physical management identity, read-only
approved account, independent console and
verified local device backup; TC-OLT-01 NOT RUN.
Independent VPS out-of-band rescue console and
off-host full encrypted recovery/restore not
yet proven, so no live K3s, public HTTPS, host
or provider firewall changes. No actual Keycloak
issuer/users/MFA and independently approved
human tenant/POP membership; FR-001/002/003
end-to-end actual customer acceptance PARTIAL.
Future custom domain ipat.fadly.id DNS pointing
alone does NOT prove TLS/domain binding; it
is also current operator SSH hostname. Future
ipat.id commercial platform domain remains
intended and unverified. Firmware update remains
hard-disabled. Production readiness NO_GO.
Next MUST: audited real human IdP+membership
read integration, genuinely restricted server
menus/APIs/RLS/jobs, then separately approved
first physical read of C320 in isolated lab.


## R7.9 remote C320 and provider-neutral K3s — 2026-09-27

Owner clarifies that a ZTE C320 needs supported
REMOTE IP-based management rather than a local
physical L1/serial connection for initial
read-only integration. Historical ZTE product
description documents SSH CLI and SNMPv3
capability at product-family level ONLY;
no real firmware/board support certified.
ADR-025 records approved product direction,
R7.4 conservative strict physical packet
remains an OPTIONAL alternative only, not
a gate on the separately guarded R7.9
read-only remote path. Firmware continues
HARD DISABLED and requires independent
tested out-of-band recovery for high-risk writes.

Owner also requires portable heterogeneous
K3s workers from ANY capable provider,
without mandatory vendor firewall integration.
ADR-026 records direction and PROPOSED
WireGuard-private inter-provider worker
network, one initial control-plane server,
NO unsupported across-cloud etcd HA.
No host firewall installation occurred
or is mandatory software dependency; true
private encrypted ingress and authenticated
node connectivity remain security requirements.
This does NOT waive production deployment
recovery, external complete backup,
approved live network or testing gates.

New source: deploy/scripts/lab/r79/
c320-ssh-readonly.py; k3s-provider-neutral-plan.py
and negative unit and true Rust offline
parser cross-contract tests. Both use
explicit private-only networks and refuse
arbitrary unsafe user strings; K3s output
installation_authorized remains false.
Mac 16/16 synthetic tests PASS, actual
nonroot Ubuntu26 temp clean main worktree
built REAL locked olt-core C320 Rust parser
and ran 17/17 synthetic-only R7.9
Python tests (including fake SSH data
consumed and validated by ACTUAL
compiled Rust binary) PASS. Old actual
Ubuntu VPS nonroot K3s read-only
inventory shows Ubuntu 26.04.1 LTS,
KVM, x86_64, 16 logical CPUs and
cgroup v2. Overlay and related kernel
modules were NOT LOADED/VISIBLE
in that sample; this alone neither
proves unsupported nor readiness.
No physical OLT IP/hostpin/auth or
real second VPS private overlay is
currently provisioned. TC-OLT-01
real device NOT RUN and real
multi-provider cluster NOT TESTED.
No firewall, DNS, public ingress,
root K3s or firmware change.
GitHub feature PR and CI evidence
will be recorded ONLY after run.


## R7.9 independently verified merged-code checkpoint — 2026-09-27

**Actual PR and CI:** GitHub PR #87 original feature
commit 370aa6863bb1f5d6f6de09435ea8e2a2cd580e3b
CI run 36289356652 SUCCESS 4/4, independently
including explicitly wired new R7.9 17 tests
in the locked unit suite (8 guarded mock remote
SSH, 8 multi-provider private K3s offline
network-plan tests, 1 mock SSH output→REAL
compiled Rust C320 offline importer integration).
The separate disposable Ubuntu 26.04 job
tested the provider-independent private-plan
test suite then ran EXISTING checksum-pinned
single-node actual K3s/etcd/CoreDNS smoke;
THIS IS NOT a real cross-provider worker join.
PostgreSQL synthetic RLS/restore and separate
physical recovery jobs passed independently.
Merged code main exact SHA
6ce2e79686caddcaeafa275255b67b49a9befabe.
Independent **post-feature-main** GitHub run
36289553986 also SUCCESS 4/4 on that exact SHA.

**Actual owner environment/recovery evidence:**
Owner FileVault Mac encrypted SOURCE-only
Restic snapshot 0e5eda33 at exact merged
code SHA; all 130/130 encrypted packs
read PASS, isolated Git source SHA256 restore
PASS, selected HISTORICAL PARTIAL readable-root
config SHA256 restored PASS. This is NOT
offsite complete VPS/database/etcd disaster
recovery. Canonical GitHub owner Mac and
actual nonroot Ubuntu 26.04.1 VPS code
were synchronized via SHA256-verified
Git bundle and clean fast-forward to same
exact code SHA; actual VPS reran full locked
offline Rust workspace/fmt, compiled
real C320 offline parser, then reran
17/17 R7.9 synthetic tests PASS.
The original VPS R79 K3s read-only
preflight showed x86_64, KVM, 16 CPUs
and cgroup v2. A further independent
read-only `modinfo` check actually
found kernel files for BOTH VXLAN and
WireGuard on the VPS; neither module
was loaded, no overlay route was
created and no second provider tested.

Original guarded private nonroot web
preview was restarted from EXACT
released code SHA via existing
SSH localhost tunnel on authorized
owner Mac; real local HTTP rollout
phase still denies unverified domain,
physical device/firmware and real
authenticated business data. Three
real Platform, Tenant and NOC
HTTP business namespaces returned
401 when Host: ipat.fadly.id,
tenant and claimed platform role
headers were forged. No new
public HTTPS or SSH/DNS route was
created or altered.

**Outstanding:** neither C320 remote
SSH nor SNMPv3 has contacted a
real owner OLT; physical model/card/
firmware tuples remain unknown;
TC-OLT-01 physical NOT RUN and
firmware HARD DISABLED. K3s
portable static plan plus one
disposable REAL Ubuntu26 single-node
smoke is NOT proof of a LIVE
second heterogeneous VPS cluster
or a permission to install on
the actual single reachable VPS.
Independently verified private
management route/host key and
read-only principal needed for
C320. Independent complete offsite
restore, genuine external rescue
and verified private VPN
reachability needed BEFORE live
K3s root installation. FR-001/
002/003 real user end-to-end
MFA/DB/API/RLS/jobs unfulfilled
and production NO_GO. ADR-025
approved product direction,
ADR-026 technical private overlay
proposal pending real measurements.


## R8.0 software-first milestone — 2026-09-27 (before physical devices online)

Owner requests proceeding with software completion and
virtual integration while the actual OLT/ONT/
MikroTik devices are unavailable until tomorrow.
Continuing real working modules NOW is approved;
no premature claim of real hardware compatibility,
customer readiness or enterprise deployment.

New code under review: actual Rust private
`/lab/auth/devices` route in
`apps/control-api/src/tenant_membership_lab.rs`;
`deploy/db/migrations/0005_lab_verified_device_inventory.sql`
sealed tenant/POP/role/revocation/expiry
function; disposable PostgreSQL negative SQL
tests; real signed JWT→restricted Postgres→
real Axum inventory HTTP integration test
for TWO valid ISP membership grants with
same synthetic signed principal.
Only NOC exact POP permitted for initial
inventory, 100 maximum read-only devices.
This remains loopback opt-in-only and does
not activate platform/tenant/NOC business
HTTP endpoints, real IdP/MFA, database
on real VPS, C320 or firmware.

Initial owner Mac 4/4 static contract checks
PASS; actual independent nonroot Ubuntu26
separate worktree full offline locked Rust
workspace compile/tests and formatting
PASS for new route (18 control API
tests compiled and ran; ephemeral
PostgreSQL-only test body requires CI
environment). Actual live disposable
Postgres migration/integration CI is
PENDING at preliminary status entry.
Existing canonical main remains R7.9
release until all CI jobs pass.
Physical TC-OLT-01 NOT RUN.


## R8.0 verified integrated VIRTUAL inventory — 2026-09-27

Owner requested completion of as much software integration
as possible while physical devices remain OFF until
the next day. Chosen safety scope is a REAL signed
JWT→restricted PostgreSQL→Axum device inventory
vertical slice using the SAME synthetic operator
with independently valid tenant+POP NOC
membership in TWO different synthetic ISP tenants.

**Exact real disposable integration evidence BEFORE merge:**
feature commit `1ef12afd7232c04e49d732427fc1182fb1978bdd`
GitHub Actions run `36291884873` PASS
4/4 independent jobs. In particular the
`postgres-rls-restore` job applied actual
`0005_lab_verified_device_inventory.sql`
after real 0001-0004 migrations and passed
4/4 REAL PostgreSQL 16 negative/positive
device function tests (exact two company
membership, wrong role/POP/identity,
revocation/expiration/suspension and
denial of direct table read), then
passed the preexisting genuine pinned
RS256→PostgreSQL menu test AND the NEW
`r80_real_signed_jwt_to_postgres_tenant_pop_inventory`
REAL Rust Axum HTTP→signed token→separate
PostgreSQL restricted login test with
`1 passed, 0 failed` on the new test.
ALL customer device fixtures were
synthetic and only the CI ephemeral
database held them. The same CI run
passed locked Rust workspace/negative
static contracts, separate disposable
PostgreSQL physical restore, and
actual Ubuntu26 single-node K3s
ephemeral smoke; no real cross-cloud
K3s worker was added.

**Integration defects discovered AND corrected by tests:**
earlier feature snapshots failed before release,
first because the expired test fixture violated
the schema's legitimate CHECK that expires_at
must postdate created_at; the fixture now
sets both dates consistently in the past.
Next, a fragile HTTP assertion assumed only
one device per tenant, but the SAME real
disposable database also retained earlier
synthetic provisioning/outbox test router
records; corrected assertions validate
bounded set size, allowed exact POP and
presence of the known tenant's own
device rather than assuming a fresh
empty database. Final feature SHA CI
above independently passed everything,
not a claimed unexecuted fix.

**Software limits after passing:** real
`/v1/platform/*`, `/v1/tenant/*` and
`/v1/operations/*` still deny HTTP401.
The new `GET /lab/auth/devices` is
restricted to NONROOT optional private
OIDC+PostgreSQL dual opt-in and a
dedicated reviewed reader role;
it is NOT active in the default
private Mac preview because no real
owner-approved OIDC MFA realm,
membership/login or production DB
has been provisioned. NO real ZTE
OLT, ONT, RouterOS, subscriber
data, firmware or user-facing
customer dashboard has been
activated or verified. This is a
real integrated software pilot
with synthetic external identities
and data only. Actual hardware
interop and production release
MUST remain separately gated.


## R8.1 native USP protobuf BEFORE physical agent availability — 2026-09-27

Owner requested continued substantive system development
without waiting for physical devices. New original
Rust genuine BBF v1.4 protobuf subset adds
`crates/usp-core/src/wire14.rs`, pinned
`prost 0.14.3`, independent hand-coded
Python binary fixture generator,
`crates/usp-core/tests/usp14_wire.rs`,
and a test-only real protobuf Get/response
through existing tenant-bound mock
Controller with negative wrong
tenant, wrong mock identity, replay.
`apps/usp-controller/src/main.rs` now
offers only opt-in private loopback
`POST /lab/inspect-usp14`: real
prost GetResp inspection with safe
counts and FALSE agent/tenant/session
authentication indicators; malformed
binary/oneof/oversized bodies rejected.
K3s mode mounts ONLY health, never
the unauthenticated parser; all
real service/agent operations remain
closed. No MQTT, TLS Agent identity,
real enrollment, queue, physical
ONT or CWMP session activated.

Independent source/schema provenance:
BBF v1.4 record raw SHA256
d32810c332c6ad5b7df3953ad0c8bb9928755c486efca4f57be78effef440435;
msg raw SHA256
96f18d5f6912c625126c1f1f917b2fc21f4fd6e3474607b9496e8e3dcdcfd3a8.
Actual separate nonroot Ubuntu26
ephemeral Git worktree full locked
offline Rust workspace tests and
format pass; real protobuf
7/7 integration tests PASS,
controller 6/6 tests PASS,
genuine binary cross-tenant
virtual domain test PASS;
actual loopback live Rust USP
binary `127.0.0.1:3100`
HTTP positive/negative/oversized/
spoofed-header tests PASS and
exclusive listener verified.
Owner Mac independent fixture and
private process contract static
4/4 PASS, no real network
device or privileged host change.
New feature CI not yet recorded
at preliminary docs timestamp:
only append its VERIFIED results
after exact SHA run completes.
ADR-029 records approved
virtual slice only. TC-USP-01
real agent NOT RUN, complete
USP native MTP still OPEN.


## R8.1 VERIFIED original protobuf + private Rust binary checkpoint — 2026-09-27

**Canonical merged code milestone:** original feature
PR #90 at final feature SHA
`82e26a606d2c1411a0f44caa022c26ab8189cc5f`
passed GitHub run `36299321859`
in all FOUR independent jobs: locked
Rust workspace including REAL
BBF v1.4 protobuf independent
goldens and actual private
Rust HTTP process smoke,
separate disposable PostgreSQL16
RLS/restore and physical-recovery
jobs, and real disposable Ubuntu
26 single-node K3s/Helm pod
readiness + deny-default smoke.
It merged as exact code main SHA
`fbed8eaeb93090966e68337ed544ea81c6b0b6c1`.
INDEPENDENT post-code-main
run `36299574222` also
finished SUCCESS 4/4 on that
exact merged code revision.
These disposable runners are
NOT the owner's live K3s,
live PostgreSQL, physical
CPE or cross-provider cluster.

**Regressions caught and repaired BEFORE release:**
first feature CI detected historic
R5 USP static expectations hard-coded
to exactly three former controller
HTTP tests and twelve old domain
tests; updated historical gates now
explicitly require the older named
trust/deny checks AND the newer
strict real-protobuf and virtual
agent correlation tests. An
intermittent R7.7 PostgreSQL
membership-expiry fixture violated
the actual invariant
`expires_at > created_at` at
sub-second timing: the updated
synthetic fixture deterministically
uses created_at two days ago and
expires_at one day ago.
Next feature CI caught a health
string regression on existing
disposable K3s Helm smoke,
which checks the prior
`synthetic-usp-lab-only` sentinel:
the new K3s health-only
route now retains exactly that
historical compatible string,
but DOES NOT mount the private
`/lab/inspect-usp14` parser
or expose `/v1/usp`.
Final feature CI 4/4
demonstrates all three fixes.
NO failure was hidden or
declared passed prematurely.

**Actual owner hardware/code safety
after feature merge:** GitHub
canonical main, clean owner
FileVault Mac and actual
nonroot Ubuntu 26.04.1
VPS source all matched exact
code SHA `fbed8ea...` after
SHA256-verified Git bundle and
guarded clean fast-forward.
Owner Mac source-only
encrypted Restic snapshot
`83505eaa` read ALL
136/136 encrypted packs
without errors, restored
isolated exact current Git
source SHA256 and separately
selected HISTORICAL PARTIAL
root-readable config SHA256
PASS. NOT offsite full-VPS,
live PG PITR or etcd restore.

On the actual nonroot Ubuntu
VPS at exact merged code SHA
full offline locked Rust
workspace and rustfmt PASS,
independent synthetic
USP manual binary fixtures
PASS, historical static USP
3/3 PASS and new R8.1
4/4 independent static PASS,
real compiled `usp-controller`
live `127.0.0.1:3100`
HTTP valid protobuf/reject
invalid/oversized/duplicated
and forged-agent headers
PASS, exclusive loopback
binding PASS and test binary
stopped. Previous private
nonroot web preview was
guardedly restarted from the
same code SHA on owner Mac
over the original localhost
SSH tunnel: /lab/auth/sections
and /lab/auth/devices stayed
404 because real operator IdP/
PG were NOT activated; forged
Host:ipat.fadly.id,
claimed role/tenant/bearer
for all three real business
API namespaces returned
HTTP401. Firmware and
physical device operations
remain HARD DISABLED.

**OPEN / PRD NOT COMPLETE:**
Actual native TR-369 MQTT
MTP, peer certificate
verification and approved
tenant→agent enrollment,
secure Record/session
protocol and standards
interop remain unimplemented;
TC-USP-01 real device NOT RUN.
Real ACS authenticated
Inform→RPC interoperability,
customer MFA onboarding,
production business
dashboard endpoints, physical
OLT/ONT/RouterOS tests,
a live cross-provider
K3s worker, true offsite
whole-host+DB restore
and customer public TLS
remain pending. R8.1 is
a verified original
PROTOBUF SOFTWARE LAB
subset, not end-to-end
USP conformance or
enterprise product GO.


## R8.2 in-progress verified LOCAL ACS SOAP slice — 2026-09-27

Continuing real software while owner hardware is offline,
the latest original Rust CWMP gateway now contains
three EXTRA opt-in-localhost-only virtual ONT
HTTP routes: actual SOAP Inform→InformResponse
using existing original parser and serializer,
a strict immutable SoftwareVersion
GetParameterValues serializer, and a strict
correlated value/fault reply parser returning
safe counts only. No random actual CPE may
pass the fixed fake-only tuple gate.
Production-looking /cwmp remains unconditional
HTTP503. No device session/enrollment,
real TLS/MFA, tenant binding, firmware
or actual C320/ONT access was activated.

In the first independent temporary
nonroot Ubuntu26 candidate worktree,
original Rust cwmp-gateway tests
PASSED 10/10, companion separate
existing integration tests 2/2 PASS.
An ACTUAL loopback Python→compiled
Rust HTTP smoke caught an initial
incorrect header casing assertion in
the Python test script, corrected
without weakening the backend.
Earlier Rust 413 oversize unit test
caught a real Axum body-limit layer
ordering bug; fixed the middleware
ordering to bind ALL new routes.
After both corrections the real
HTTP smoke PASSED: genuine
SOAP InformResponse, actual strict
GetParameterValues request and
response, safe fault treatment,
unsupported/write/correlation/
DTD/oversize negative paths,
fake Host/tenant/mTLS headers
and permanent live /cwmp denial.
Mac static acceptance contract
4/4 PASS. GitHub integrated feature
CI and canonical release verification
must be logged AFTER execution.
No physical test or production
deployment is implied.


## R8.2 verified original Rust ACS SOAP actual HTTP + source release checkpoint — 2026-09-27

**Immutable source and CI:** PR #92 featured original
Rust CWMP 1.0 actual virtual ONT HTTP
`Inform`→`InformResponse`, static read-only
`GetParameterValues` serialization and
strict correlated response/fault parsing.
The feature commit `3911c846f3b0e28ae5d62ff466a95a3d2bbb7680`
CI `36301589491` completed SUCCESS **4/4**.
The merge committed main exact
`d8e75c8a8f2e12dad15797135960e5de9bb30067`,
and separate actual post-code-main CI
`36301824513` also completed SUCCESS **4/4**,
independently including real compiled Rust
local SOAP HTTP tests, preexisting
native genuine BBF USP 1.4 protobuf
virtual-agent tests, signed synthetic
JWT→isolated real PostgreSQL/RLS
two-company inventory tests, real
disposable Ubuntu26 single-node K3s
smoke and separate disposable
Postgres backup/restore jobs.

**Exact real owner environment:** the
FileVault owner Mac encrypted
SOURCE-only Restic snapshot
`a44374c5` of merged code main
was fully verified with **138/138**
encrypted pack reads and isolated
SHA256-exact canonical source
restore. A separately selected
historical PARTIAL readable root
configuration also restored
SHA256-exact, NOT an offsite
whole VPS, production PostgreSQL
or K3s datastore DR drill.
GitHub, owner Mac and actual
nonroot Ubuntu 26.04.1 VPS
were synchronized to identical
exact code SHA via checked Git
main and SHA256-verified
Git bundle clean fast-forward.

On the actual nonroot VPS
canonical SHA, full locked
offline Rust workspace unit
tests and rustfmt passed, new
R8.2 source contract 4/4
passed, and actual Rust
CWMP gateway was compiled,
served on exclusive
`127.0.0.1:3300`, passed
independent Python genuine
HTTP/SOAP positive/fault/
negative/oversize tests,
and was stopped by guarded
smoke cleanup. No real
device identity or external
packet was involved.
The owner Mac original
trusted localhost tunnel
private dashboard was
guardedly restarted at the
same main source SHA.
Actual Mac HTTP verified
the absent unprovisioned
MFA+database membership
routes remain 404 and
all three real Platform/
Tenant/NOC business
namespaces reject forged
Host/tenant/role/bearer
with HTTP401. Firmware
and actual device actions
remain disabled.

**OPEN PRD / external gates:**
The C320/ONT/MikroTik
are not yet live-tested,
CWMP real `/cwmp`
unconditionally HTTP503,
no actual trusted HTTPS
device enrollment or
durable CWMP session,
native USP real MQTT
MTP and agent certificate
enrollment not tested,
actual customer MFA/
admin/tenant operations
and real public SaaS
dashboard not provisioned,
independent complete offsite
VPS/PG/K3s recovery not
verified, and no live
heterogeneous cross-provider
K3s cluster. R8.2 is
verified synthetic real
SOAP software, not a
production readiness GO.


## R8.3 device-management corrective milestone — 2026-09-27 (initial code candidate)

Owner correctly identified the lack of visible
Add Device, Device List and Device Condition
controls as a blocking product issue for
tomorrow's hardware trial. Work is reprioritized:
a genuine working private Device Manager
user interface has been coded, linked
from existing preview and main lab page,
and a separate genuine PostgreSQL
tenant-scoped pending metadata registry
with independent signed Rust read/write
handlers has been prepared. No actual
physical device or credentials are
contacted or stored.

Local UI is explicitly fake-only LAB-/
VIRTUAL-, bounded per-process volatile
fake inventory (24 items) with actual
Rust HTTP Add/List/Delete, POP/kind
filter, fixed PENDING_REVIEW /
UNKNOWN / NOT_MEASURED indicators and
large RED PRD warning. Demo POST/
DELETE requires same-origin Host
and private custom header; public
business endpoints remain denied.
R8.3 new SQL 0006 is a review-only
metadata table and sealed function
pair: one restricted pending-only
INSERT path for verified tenant_admin
via independently provisioned reader/
registrar identities, one list path
for approved own admin / NOC exact
POP. This SQL is NOT installed on
the actual VPS or a customer DB.
Actual trusted customer MFA, approved
device enrollment and live read-only
probe still remain external gates.

Independent isolated nonroot Ubuntu26
Rust worktree earlier compiled 21/21
Control API tests, and with the new
signed-database R8.3 test added,
22/22 offline control API suite PASS
(the disposable-PostgreSQL-only
test body intentionally skips in
ordinary offline run). Owner Mac
R8.3 static dashboard/backend checks
4/4 PASS. FULL genuine CI,
actual PostgreSQL migration/tests
and final live owner Mac web
proof remain PENDING at this
initial in-progress checkpoint.
A later immutable milestone entry
will record real CI/HTTP evidence
only if independently executed.


## R8.3 verified real Device Manager release checkpoint — 2026-09-27

The owner correctly identified visible Add Device, List Devices
and honest Device Condition as missing operational prerequisites.
Feature PR #94 was MERGED at exact canonical main code
`94c6b2f07bd63a4ae0578a5915e232b1bd1fa0b4`.
The final feature HEAD
`5e3d97ec4f1bb2f63618a46dfa25485ad3543284`
GitHub CI run `36305345728` passed all FOUR jobs; separate
independent AFTER-MERGE code-main GitHub CI
`36306977093` passed all FOUR jobs again at the exact
main SHA, including real PostgreSQL disposable signed
JWT→separate EXECUTE-only registrar + member reader
two-company positive/negative integration,
actual compiled Rust localhost HTTP Add/List/Delete,
original native USP protobuf/CWMP virtual SOAP,
independent disposable real PostgreSQL physical recovery
and disposable Ubuntu26 K3s one-node smoke.

**Actual Mac browser-local proof:** the existing
owner private localhost SSH tunnel and ORIGINAL
nonroot Rust preview were guardedly restarted from the
merged exact code SHA. The real Mac `http://127.0.0.1:48765`
served new interactive HTML/JS/CSS Device Manager with HTTP200,
real POST Add returned 201, real GET List returned its new
virtual OLT with exact `PENDING_REVIEW/UNKNOWN/NOT_MEASURED`,
and real DELETE returned 200. Missing demo-origin
CSRF was rejected HTTP403 with correctly typed JSON.
All three actual platform/tenant/NOC business
API namespaces rejected requests HTTP401.
The real signed-identity proposed registry
was NOT provisioned on the actual VPS
and its routes remained HTTP404. Both
rollout flags for physical connection and firmware
were verified FALSE. The preview was
guardedly restarted again afterward to clear
all temporary demo records: 0 fake records,
0 physical devices. No physical management
traffic was generated.

**Exact source and recovery:** private GitHub,
owner FileVault Mac and actual authorized
nonroot Ubuntu26 VPS matched this main SHA,
synchronized to the VPS with independently
SHA256-verified Git bundle and clean
fast-forward; actual VPS full locked
offline Rust workspace tests and
rustfmt PASS, R8.3 4/4 static
acceptance PASS. An attempted second
standalone R8.3 socket smoke directly
on the already-running actual VPS was
correctly REFUSED because the existing
real preview owns port 3000; the
equivalent actual live Mac→VPS SSH
HTTP demo proof above passed,
and independent disposable GitHub CI
executed the standalone smoke successfully.
Never disrupt or silently replace
another existing preview listener.

FileVault owner Mac encrypted source-only
Restic snapshot `3e40cc27` of exact merged
code main full-read **142/142** packs
and SHA256-isolated source restore PASS.
Historical PARTIAL readable root-config
restore PASS, but offsite FULL host,
real PostgreSQL customer data/PITR
and real K3s cluster datastore
restore are STILL UNVERIFIED.

**PRD REMAINING:** the immediately visible
Device Manager is a working private demo
with volatile fake-only data and truthful
no-telemetry states, NOT an authenticated
real ISP operator dashboard.
The separate audited exact signed
JWT/tenant/POP PostgreSQL draft APIs
and schema are real code/real disposable
CI-tested but NOT enabled with real
company MFA on actual VPS.
Production login and membership
enrollment, approval and audit UI,
device credential vault, true SSH/
SNMP/TR-069/USP adapters with exact
model+firmware recognition, live
evidence-to-health freshness,
end-to-end durable jobs, physical
OLT/ONT/MikroTik tests and whole
system disaster recovery remain MUST
before claiming usable real
physical adoption or full PRD.


## R8.4 code candidate — independent security-admin maker-checker, 2026-09-27

**Scope:** The already-verified R8.3 private
interactive fake-only Device Manager and
the independent REAL signed tenant/POP
PostgreSQL draft backend are not equivalent
to an actual human login or physically
authenticated adoption. The product owner
prioritizes safe real enrollment, first
preparing a separate reviewer authorization
boundary before any live OLT/ONT action.
The R8.4 source now contains a strict
pinned RS256 access-token exact signed
`amr:mfa` signal (NOT real human
MFA provisioning), independent
`security_admin` membership, NOLOGIN
PostgreSQL reviewer roles, own-tenant
queue excluding the proposing person,
single verdict under true PostgreSQL
row lock, immutable append-only review
record, idempotent retry and no change
to UNKNOWN/NOT_MEASURED device state.
An explicit extra opt-in localhost
Rust reviewer adapter needs a separately
owned 0600 Unix-socket DB identity and
independently verifies the JWT and
current DB membership on every request.

**Initial software evidence only:**
An isolated nonroot Ubuntu26 Rust
worktree compiled and passed 24/24
`control-api` unit tests with the
new reviewer module included
(disposable PG-only body intentionally
does not execute without ephemeral
CI PostgreSQL). The new R8.4 5/5
static checks PASS on the actual
owner Mac. Genuine disposable
PostgreSQL migration/strict privilege
tests and signed Rust Axum→real
reviewer-DB integration plus all
four independent CI jobs remain
PENDING until actually run.
Actual VPS production DB, hardware,
firmware, running K3s and owner
live web user identity have NOT been
touched by this new candidate.

**MUST STILL COMPLETE:** actual MFA
IdP enrollment and sign-in browser
BFF and audited tenant role/POP
onboarding; true production
restricted reader/registrar/reviewer
identities; independent whole-host
and customer PostgreSQL recovery;
physical OLT/ONT model/firmware
read-only verification, ACS/USP
real device sessions and safe
tenant/POP evidence health. Metadata
approval alone cannot adopt
equipment or authorize any
firmware update.


## R8.4 verified atomic maker-checker metadata review checkpoint — 2026-09-27

**What was materially integrated:** PR #96 merged original
Rust signed MFA-claim boundary, separately restricted PostgreSQL
independent security_admin reviewer, own-tenant queue excluding
the maker, row-locked one-time approval/rejection with exact
idempotent retry and append-only one-decision audit.
A reviewer cannot self-approve, access another tenant, override
an already reviewed draft, bypass a revoked/expired membership
or upgrade a candidate from UNKNOWN/NOT_MEASURED merely because
metadata was reviewed. There is no network device admission
or firmware operation in this milestone.

**Reproducible proof:** Merged canonical source code main exact
`c032d3759103d6b3d28b21df83bd923e85ce60ae`;
independent AFTER-MERGE GitHub run
`36311222833` returned **SUCCESS in all 4/4 jobs**:
genuine disposable PostgreSQL 16 ordered migrations including
0007 and actual reviewer/maker-role and immutable-audit tests,
actual short-lived signed RS256 synthetic MFA claim from
two distinct imaginary humans through original Rust
Axum into separately restricted real PostgreSQL,
full locked Rust and R8.4 static 5/5 checks,
independent disposable PostgreSQL physical restore
and genuine disposable Ubuntu26 single-node K3s.
No actual customer IdP or physical equipment
was involved.

**Actual owner environment:** private GitHub,
owner FileVault Mac and actual nonroot Ubuntu26 VPS
were synchronized to the same exact code SHA via
SHA256-verified Git bundle clean fast-forward.
Actual canonical Ubuntu26 VPS `cargo fmt --check`,
full `cargo test --workspace --locked --offline`
and R8.4 5/5 security static tests PASSED.
Owner Mac encrypted source-only Restic snapshot
`2a2a43d7` verified FULL 146/146 pack
reads and isolated source SHA256-exact restore;
separately selected historical PARTIAL readable
root configuration restored SHA256-exact, which
is NOT complete offsite whole-host/PG/K3s recovery.
Existing real Mac→VPS tunnel private
Device Manager was restarted from exact
source SHA and actual owner Mac HTTP verified
dashboard CSS/JS/HTML remained HTTP200,
unprovisioned exact signed registry and
review endpoints remained HTTP404,
all three platform/tenant/operations real
business API namespaces remained HTTP401,
and the actual physical device count and
live firmware capability remained disabled.

**PRD GAP — HIGH:** An independent real human
MFA enrollment in a production-approved OIDC
issuer, approved role memberships and operator
browser login have NOT been provisioned.
A cryptographically signed synthetic `amr:mfa`
JWT is *NOT* actual physical human MFA proof.
The standalone reviewer API is intentionally
not mounted in the existing default owner preview.
A metadata approval never authorizes actual
C320/ONT/RouterOS SSH/SNMP/CWMP/USP access;
firmware tasks remain hard disabled. Next
MUST: independently provision and verify
actual MFA IdP plus authenticated BFF/browser
session and per-tenant restricted real DB
connection/role mapping, then trusted read-only
device identity and evidence.


## R8.5 initial original real OIDC signed-MFA trust preflight candidate — 2026-09-27

The owner wants actual safe operator identity and
persistent tenant/POP Device Manager BEFORE
contacting physical OLT or ONT. After R8.4
independent reviewer, add original Rust
standalone nonroot operator preflight to
check independently sourced HTTPS OIDC issuer,
exact pinned RS256 key/audience/kid and
a real issuer's short-lived signed
`amr:mfa` bearer through private
bounded stdin FD; reject
terminal-echoed token, missing
explicit opt-in, symlink/loose
public-key PEM files and
unrecognized token. Passing is
ONLY a cryptographic signed
claim, NOT actual human MFA
enrollment, signed-in browser,
tenant access or physical adoption.

Actual new original source:
`crates/identity-core/src/bin/oidc-mfa-preflight.rs`,
`crates/identity-core/tests/oidc_cli.rs`,
`deploy/scripts/lab/r85/test_r85_contract.py`
and matching locked crate metadata
plus CI, ADR, security, PRD and
runbook updates.

**Initial independent nonroot Ubuntu26
candidate proof**: fresh disposable
synthetic 2048-bit RSA generation
and original compiled Rust
subprocess+bounded stdin signed
real JWT test 4/4 PASS.
Existing exact locked original
identity tests 6/6 PASS.
Mac R8.5 4/4 independent
static safety tests PASS.
Full GitHub R8.5 feature/main
CI and exact final source
sync/backup still PENDING.
No real operator IdP account
or actual production service
has been provisioned.

**R8.4 final release evidence:**
PR #96 original code-main
`c032d3759103d6b3d28b21df83bd923e85ce60ae`
independent CI `36311222833`
4/4 SUCCESS; docs proof PR #97
final docs-main
`c97e46c663e5a942a8c6fe1332c56944b276f426`
independent CI `36312134432`
also 4/4 SUCCESS. Owner Mac
encrypted source-only Restic
snapshot `74d16ddf` verified
full read 148/148 packs and
isolated exact-source SHA256
restore, plus selected historical
PARTIAL root-readable config
restore, NOT real whole-host
customer PostgreSQL/K3s DR.
GitHub, owner Mac and actual
clean nonroot Ubuntu26 VPS
synchronized final R8.4 docs
main exact SHA via SHA256-
verified Git bundle fast-forward.
R8.4 production reviewer
and signed endpoints remain
OFF on the owner's default
private preview.


## R8.5 independently verified original pinned OIDC real-provider preflight — 2026-09-27

**Exact code release:** PR #98 merged the original
nonroot Rust `oidc-mfa-preflight` utility,
strict owner-owned 0700 parent/0600 no-symlink
pinned PEM, exact original pinned issuer/audience/kid
and RS256 ephemeral access JWT validation,
bounded secure non-TTY stdin, signed `amr:mfa`
check and safe output explicitly denying
real-human MFA proof and all production rights.
Fresh independent RSA-2048 key generation and
ACTUAL Rust subprocess signed positive/no-MFA/
wrong-kid/attacker-RSA/unsafe-key/symlink/
oversize and default-denied tests PASSED.
R8.5 static security tests 4/4 PASSED.

Feature HEAD
`b36f51e236cbe7027c76eeb07d8ce5ee8b34f94e`
GitHub run `36312705685` passed ALL
FOUR independent CI jobs; exact merged
main code SHA
`addf812177f709fdef61af4bbc8de53b006831bd`
was independently tested AFTER MERGE
in GitHub run `36312938159`,
also **4/4 SUCCESS**. The suite
preserved genuine real disposable
PostgreSQL16 signed synthetic JWT
maker-checker cross-company negative
tests and actual recovery exercises,
original CWMP SOAP/USP protobuf
virtual proofs, and real
disposable single-node Ubuntu26 K3s.

**Actual owner execution:** GitHub
private, owner FileVault Mac and
authorized clean nonroot Ubuntu26
VPS canonical source matched exact
merged code SHA after independent
SHA256-verified Git bundle clean
fast-forward. On actual nonroot
Ubuntu26 canonical source, rustfmt,
full locked OFFLINE Rust workspace,
new signed-RSA external CLI tests
and 4/4 source safety tests passed;
`oidc-mfa-preflight --requirements`
printed only external operator
prerequisites, no subject/token.
Owner Mac FileVault encrypted
SOURCE ONLY Restic snapshot
`7ddfcc9f` was verified
150/150 packs full-read plus
isolated SHA256-exact source
restore, and separately selected
historical PARTIAL readable root
config restored. This is NOT
independent whole-host/customer
PostgreSQL/K3s datastore disaster
recovery.

**PRD NOT READY:** No actual
customer human OIDC issuer/MFA
account, private token or browser
session was supplied or created.
No default device-review routes
or customer login were activated,
no physical OLT/ONT/RouterOS
was connected, no live health
evidence was fabricated, firmware
remains disabled. The next MUST
is independently provisioned
real operator IdP/MFA
plus confidential Authorization
Code+S256 PKCE server-side BFF,
real signed-in tenant dashboard
against the existing separately
restricted PostgreSQL registry,
actual reviewer workbench and
approved private hardware read-only
admission. The fact that a signed
synthetic JWT passed R8.5 is
NOT proof actual human MFA.


## R8.6 real browser PKCE initiation security milestone — work in progress 2026-09-27

Owner priority stays real user authorization and visible
Device Manager before allowing actual OLT/ONT network
operations. The preceding R8.5 docs-main independent
CI run `36314168433` revealed a genuine nondeterministic
early-exit race in the **test fixture**, not a verified
production login failure: intentionally default-denied
standalone identity preflight may exit before the parent
writes to its stdin, producing BrokenPipe. R8.6 corrects
the regression test to allow BrokenPipe only in the
explicit default-denied path; enabled positive path
still requires a full successful write. Independent
feature and main runs are mandatory to verify the fix.

New original identity-core RFC7636 PKCE S256 generator
uses three independent 256-bit OS random values,
SHA256, fixed registered owner Mac-loopback callback
and a reviewed owner-controlled Keycloak-candidate
issuer association. New gated original Rust Axum
private browser START/CALLBACK uses bounded
16-pending/300-second one-use memory state,
cookie correlation, denied malicious Host/query/
cookie/replay, and intentionally returns HTTP503
even on correctly correlated callback:
NO token exchange/secure user session yet.
A real compiled independent synthetic issuer
Python→Rust loopback HTTP socket probe passed
303 state+nonce+S256/cookie, bad Host, invalid
callback, 503 on valid shaped code, replay 403,
and all real business namespaces HTTP401 on an
isolated nonroot Ubuntu26 code worktree.
The genuine broader CI and final exact source
backup/release evidence will be appended
only after it is independently executed.
Physical equipment and live full system stay CLOSED.


## R8.7 in-progress verified isolated signed ID+access binding — 2026-09-27

After merging R8.6 browser PKCE security proof,
real operator MFA issuer and confidential HTTPS
code endpoint remain unprovisioned, so actual
callback continues intentionally discarding
returned authorization code and HTTP 503.
To make concrete software progress without
inventing a fake operator login, a new pure
original Rust OFFLINE `oidc_id_token`
verifier was implemented on a separate
feature branch. It independently checks
two actually signed RSA ID and access JWTs
for exact distinct browser/API audiences,
same issuer+subject, original nonce,
SHA256 at_hash, signed MFA on both,
bounded fresh auth_time and key/typ/lifetime.
It returns no role, tenant, device
or web session capability.

On the actual authorized nonroot Ubuntu26
VPS an isolated disposable worktree
compiled and executed all 5 original real
ephemeral openssl-RSA signed positive/
negative token-pair tests PASS; owner
Mac 4/4 separate static guard tests PASS.
This is a software-only acceptance checkpoint
before independent full feature CI and
postmerge code-main verification.
No live IdP token, actual customer session,
real signed-in Device Manager, OLT/ONT,
network firewall or device firmware
was modified or connected.


## R8.7 verified independent full CI and actual nonroot Ubuntu26 release — 2026-09-27

Feature PR #102 added the original strict
OFFLINE cryptographic OIDC ID JWT and
independently pinned access JWT cross-token
binding. The feature HEAD
`be3df17377ef57076de7931655be8a6d94e26923`
had independent GitHub Actions run
`36317730147` SUCCESS 4/4:
unit-tests (genuine real ephemeral
RSA-signed ID+access fixture tests),
real ephemeral PostgreSQL16
tenant/POP/maker-checker, separate
ephemeral PostgreSQL physical restore,
and real disposable Ubuntu26
single-node checksum-pinned K3s.
It merged on canonical main as EXACT
code SHA
`c45e09bc772ef0eab36c458e87d1123692cd9a59`.
A separately triggered independent
POST-CODE-MERGE main CI run
`36318026887` also passed
SUCCESS ALL FOUR JOBS at this exact
source SHA.

On the actual authorized nonroot
Ubuntu26 VPS, a separate isolated
disposable worktree first passed
5/5 genuinely signed RSA token-pair
positive/negative Rust fixtures,
4/4 R8.7 source contract tests,
cargo fmt and full locked offline
Rust workspace. Then exact merged
code main was SHA256-bundle-verified
fast-forward synchronized to the
real nonroot VPS; real canonical
VPS full locked offline workspace
and fmt and four source contracts
again passed, and both the VPS
and owner Mac had clean matching
GitHub main source SHA.
No actual IdP or customer data
was needed, fetched or sent.

Owner Mac FileVault encrypted
SOURCE-ONLY Restic snapshot
`2461545a` of exact code main was
successfully restored SHA256-exact
in isolated restore and the complete
encrypted repository was read
156/156 packs error-free.
Historical selected PARTIAL
root-readable config was also
restored SHA256 exact. These
are NOT independently proven
complete offsite host, actual
customer PostgreSQL or K3s
datastore recoveries.

The current owner-private Mac
Device Manager lab still provides
only separately guarded fake-only
volatile Add/List/UNKNOWN status
and real business namespaces
remain denied; the original R8.6
login callback deliberately
destroys even a matching
synthetic code and returns 503.
R8.7 ONLY constructs an
OFFLINE verified pair primitive,
does NOT wire an external
confidential token endpoint,
mint a login cookie or claim
real human MFA.

**OPEN PRD MUST:** owner-approved
independently proven actual
human OIDC MFA/issuer, secure
confidential HTTPS code redemption
with original PKCE verifier and
private nonce, actual browser
BFF Secure HttpOnly session and
tenant/POP SQL membership,
signed-in per-tenant
reviewer/operations dashboard,
real vendor firmware-matched
private read-only OLT/ONT
admission and honest evidence
freshness, live HA K3s/DB and
separately tested offsite
whole-host DR. No actual OLT/
ONT/RouterOS connected or
firmware action authorized.


## R8.8 original secure identity-bound browser-session milestone — 2026-09-27 (candidate)

Owner requested continuing the agreed priorities,
not prematurely probing actual OLT/ONT while
the real authenticated dashboard is incomplete.
New actual original Rust identity-core
`browser_session.rs` creates an UNMOUNTED
identity-only 256-bit OS-random SHA256-at-
rest session handle and separate anti-CSRF
token. A real separately UNMOUNTED Control
API bridge binds the previous R8.7
genuinely signed ID/access JWT token pair
to existing genuine strict sealed PostgreSQL
approved active tenant/POP membership BEFORE
issuance and RECHECKS that restricted
function on EVERY requested scoped
read or mutation. No role/tenant is
taken from any browser claim; no session
has device or reviewer privileges by
itself. Cookie string is only a future
HTTPS Secure HttpOnly SameSite Strict
policy fixture and no actual browser
login HTTP endpoint is mounted.

Actual independent nonroot Ubuntu26
isolated worktree compiled 9/9
genuine ephemeral RSA signed
identity/session Rust test cases,
29/29 actual Control API unit tests
(the real-PG-required body is
intentionally skipped outside
disposable CI), and full
locked offline Rust workspace.
Owner Mac R8.8 5/5 static
fail-closed regression tests passed.
The exact R8.8 signed token pair →
genuine disposable PostgreSQL 16
restricted two-company membership →
opaque BFF session/CSRF → per-request
DB re-authorization CI test is
ADDED as a mandatory PostgreSQL
CI gate. Its PASS must be claimed
only AFTER actual disposable CI
executes it. No actual human IdP
MFA, live login, real OLT/ONT,
real network routes or actual
customer records were used.
All real business API namespaces
and prior synthetic browser
callback remain default denied.


## R8.9 — internal original BFF session-to-device candidate bridge (2026-09-28, CI candidate)

Continued the binding user priority before
connecting the actual physical OLT/ONT:
integrate genuine verified user identity
session with server-authorized per-tenant
and exact-POP device candidate inventory.
The new UNMOUNTED original Rust
`pending_devices_for_session` accepts
ONLY a previously cryptographically
signed genuine R8.7 ID/access pair→
R8.8 opaque identity-only session;
EVERY request independently checks
actual current PostgreSQL exact approved
active issuer/subject/tenant/role/POP
membership AND current scoped
candidate rows within ONE SQL
statement/snapshot. It never
accepts role/tenant from a cookie
or spoofed HTTP headers. Results
exclude management IP, credentials
and fabricated network telemetry,
and preserve unknown/not-measured
states where no proof exists.

The independent genuine disposable
PostgreSQL CI gate executes a real
synthetic two-company signed ID+access
pair→separately restricted SQL→
opaque cookie→separately reviewed
draft rows. Negative tests cover
forged session, wrong tenant/POP/
role/origin and expired session.
Owner Mac standalone static R8.9
contracts 4/4 PASS and isolated
actual nonroot Ubuntu26 original
control-api Rust 30/30 tests
and rustfmt PASS. The new genuine
CI and final canonical source
release verification are still
PENDING until actually executed.
No real MFA IdP, browser login,
real service/customer DB, physical
device, firmware, live K3s, or
public server configuration
was changed by this candidate.

**PRD OPEN:** actual human
OIDC/MFA enrollment and issuer,
confidential HTTPS PKCE exchange,
public customer TLS and secure
host/origin/CSRF cookie middleware,
real privileged restricted DB
accounts and human maker-checker
audit, horizontally shared
revocable sessions, server-authorized
authenticated tenant/NOC web
integration, genuine vendor
read-only connectivity and
evidence-backed health/firmware
remain MUST. This is NOT yet
a usable authenticated real-device
dashboard or production launch.


## R8.9 VERIFIED feature-code release checkpoint — 2026-09-28

**Original source:** R8.9 merged
GitHub feature PR #105 at exact
main code SHA
`bd788f43e19d94edf5d8eb518053fc2610370a70`.
The separate feature CI run
`36368344427` completed
SUCCESS all FOUR jobs. A fully
independent post-code-main run
`36368663242` completed
SUCCESS all FOUR jobs again,
including genuine disposable
PostgreSQL 16 separate synthetic
reader/registrar plus genuine
RSA signed ID/access pair→
original opaque browser session→
actual POP-scoped two-ISP
candidate list with negative
wrong company, POP, role,
origin and expiry probes.
The unchanged CI also executed
full Rust workspace, genuine
native USP/CWMP software slices,
a disposable physical PostgreSQL
recovery drill and a genuine
disposable Ubuntu 26.04
one-node K3s isolation smoke.
These are SOFTWARE/CI results,
not actual customer/ONT/OLT
or independent production
HA/disaster-recovery results.

**Actual owner deployment and
negative controls:** private
GitHub, clean FileVault
owner Mac and real nonroot
Ubuntu26 canonical VPS code
synchronized to the exact
R8.9 main source SHA by
SHA256-verified Git bundle,
clean fast-forward verified.
Actual final VPS full locked
OFFLINE Rust workspace,
rustfmt and R8.9
static contract 4/4 PASS.
The prior private web
preview initially had old
stale tracked PID state
and correctly REFUSED
unsafe silent restart.
After reviewing its exact
safety-checking stop/start
scripts, the existing
guarded mechanism stopped
the tracked original
nonroot process and rebuilt
the original control-api
from exact canonical code.
Live owner Mac localhost
HTTP `/lab/device-workbench`
returned 200; absent
unprovisioned actual signed
identity candidate path
returned 404, and the
three real platform/tenant/
operations business API
namespaces all returned
401. The R8.9 internal
session-list function
deliberately remains
UNMOUNTED. No physical
device or firewall,
live K3s, real customer
DB, real MFA provider
or firmware action was
activated.

**Recovery:** FileVault
owner Mac encrypted
SOURCE-ONLY Restic
snapshot `65b40fa7`
of merged code main
fully verified 158/158
encrypted pack reads
and independently
isolated SHA256-exact
source restore. Selected
historical PARTIAL
readable root-config
restore independently
SHA256 PASS. These DO
NOT prove complete
offsite whole-host,
customer PostgreSQL
or live K3s datastore
recoverability.

**MUST next:** approved
independent real human
OIDC IdP/MFA and
issuer+MFA semantics,
confidential private HTTPS
authorization code/PKCE
token exchange, actual
safe Secure HttpOnly
revocable session and
exact tenant/POP backend
data-list with only
permitted visible menus;
maker-checker adoption
approval with independent
real human actors and
audit, THEN approved
fingerprint/identity
based actual OLT/ONT
read-only physical
interoperability. R8.9
is an essential genuine
backend security and
data-integration slice;
the full commercial
PRD is NOT COMPLETE.


## R9.0 first genuine operator-authorized network contact — 2026-09-28

**Source/authority:** product owner newly authorized attempting
one exact public TCP321 ZTE C320-candidate endpoint.
A physical TCP connection from the actual authorized owner Mac
succeeded with ~87 ms first observation and 15 initial Telnet
IAC bytes. A separate minimal protocol-only refusal exchange
elicited an UNAUTHENTICATED server-side "ZTE" marker, which
does NOT independently verify C320 chassis, service identity,
serial, firmware, owner POP or compatibility. No real or fake
account credentials, printable login, OLT CLI, PON read, or
firmware command was transmitted.

**First true IPAT worker network blocker:** the same exact
endpoint from actual nonroot Ubuntu26 VPS timed out.
No diagnosis of specific firewall, provider, source allowlist
or NAT is claimed; a Mac-accessible port does NOT make the
site reachable by IPAT production workers.

**Original newly implemented protection:** source
`deploy/scripts/lab/r90/olt-telnet-network-preflight.py`
allows one exact owner-provided global IPv4 on only TCP321,
one explicit nonroot opt-in, one TCP connect, maximum
512 bytes read and ZERO application bytes sent.
It has no login or raw banner output, no retries,
no IPv4/range scans, no target/secret in Git or public
frontend, no device mutations and no automatic health changes.
Optional report is redacted JSON outside Git with owner 0700
directory and O_EXCL mode-0600 new file. The actual new code
was executed on owner Mac 2026-09-28T02:51:40Z:
TCP reachable, Telnet IAC true, 15 inbound bytes, 0 outgoing;
encrypted owner Mac FileVault-protected private evidence
0700/0600 independently checked and sanitized. The same
single-shot script ran as nonroot from the actual VPS at
2026-09-28T02:52:40Z, returning one TIMEOUT and NO sends.
The temporary candidate preflight script on VPS was cleaned up.

**UI:** private Device Manager adds an explicitly timestamped
HISTORICAL network-only panel that does not disclose the
public endpoint and does NOT claim ongoing live telemetry.
Real enrolled device count remains 0, connectivity UNKNOWN,
health NOT_MEASURED and exact OLT physical interoperability
NOT RUN. Existing public business APIs stay DENIED.
**Local QA:** original offline network preflight unit cases
7/7 PASS (one connection, zero sends, exact IPv4/port/opt-in,
silence/timeout, private evidence, redacted dashboard and
prohibition of outbound command methods). The new
GitHub CI job runs ONLY local mocks, never the real site.
**OPEN real physical/access gates:** public Telnet is plaintext
and MUST NOT carry actual or "temporary" credentials. First
establish an independently verified encrypted private
management route or a proven secure management protocol,
check that the actual IPAT worker can reach the site,
independently verify exact peer model and firmware and
its dedicated read-only authority, and retain real human
MFA/POP/maker-checker approval before TC-OLT-01.
The original R7.9 SSH candidate is NOT known supported
on this exact firmware. No vendor support, live health,
OLT/ONT adoption, firmware update, real production login,
K3s on the live VPS or independent production disaster
recovery is claimed. Release source SHA, independent
GitHub CI and backup evidence must be appended
only after independently observed success.


## R9.0 verified original first-site TCP/Telnet no-auth milestone — 2026-09-28

Feature PR #107 merged at canonical exact source SHA
`d516113e9ac1bce29383adbe65e743877b0671c8`.
Feature GitHub CI run `36372210621` finished SUCCESS all FOUR
jobs. Independently, AFTER real feature merge the canonical
main run `36372564123` finished SUCCESS all FOUR
jobs: new R9.0 strict mock-socket seven negative/static
tests, full locked Rust workspace and existing
native genuine USP/CWMP domain software tests,
real disposable PostgreSQL data-integrity/recovery
jobs and real disposable Ubuntu26 checksum-pinned
single-node K3s isolation. CI never contacted
the real owner OLT endpoint, installed production
services or changed network ACLs.

The exact SHA was verified on private GitHub
and clean FileVault owner Mac. By SHA256-verified
Git bundle, the actual authorized nonroot
Ubuntu26 VPS main fast-forwarded cleanly
to that same exact source SHA. On that
actual VPS the FULL locked OFFLINE Rust
workspace, rustfmt and R9.0 mocked seven
zero-send security tests PASSED.
On the owner Mac, the original privately
tunnelled nonroot Rust web preview
was guardedly rebuilt and restarted
from this exact R9.0 code SHA.
Real Mac HTTP verified
`/lab/device-workbench` HTTP200,
the explicitly time-labelled
R9.0 network-only historical
panel and no raw management IP;
0 simulated candidates and
0 physically enrolled devices.
Unprovisioned true signed/SQL
candidate routes remained HTTP404,
and actual Platform/Tenant/Operations
business API namespaces returned
HTTP401 even without real login.
Real physical probes, status
telemetry and firmware remained
HARD DISABLED.

FileVault owner Mac encrypted
SOURCE-ONLY Restic snapshot
`b8684d06` at merged R9.0
code SHA passed full 162/162
encrypted pack reads and isolated
exact source SHA256 restoration;
selected historical PARTIAL
readable root configuration
also independently restored.
The separate 0700/0600
sanitized live network probe
evidence on Mac is local
operator-controlled field evidence,
NOT a separately tested offsite
backup nor authenticated
hardware compatibility evidence.

**INCOMPLETE / PRD blocker:**
one visible TCP/Telnet handshake
and an unverified ZTE text
banner are NOT successful
authenticated physical OLT
integration. Actual nonroot
VPS cannot currently reach
the authorized public source
endpoint; no identity pin,
actual C320 model/card/firmware,
physical PON read, trusted
private route or physical
tenant enrollment exists.
Telnet cleartext MUST NOT
receive credentials, even
temporary trial credentials.
Actual approved human MFA,
site-to-IPAT private secure
management, independent
OLT identity proof, restricted
read-only pilot, durable
device approvals, production
PostgreSQL+K3s recovery
and commercial dashboards
remain OPEN. Docs-only
R9.0 checkpoint CI and
exact final docs-main SHA
will be recorded separately
after the docs merge.


## R9.1 candidate — persistent adoption readiness gates, 2026-09-28

Development continued from the actual R9.0 canonical source without
repeating earlier work. New migration 0008 introduces append-only,
forced-RLS readiness evidence for four explicit prerequisites:
secure_management_path, device_identity, readonly_account and
recovery_plan. Approved metadata remains ineligible until all four
latest unexpired gates are verified; later blocked/expired evidence
fails closed. The evidence writer is a separate NOLOGIN/EXECUTE-only
boundary and requires the same active security_admin who performed
the approved metadata review. The existing restricted identity reader
may call only the safe readiness projection.

The unmounted opaque-session bridge now reads these readiness booleans
under the exact current tenant/POP scope and returns no management IP,
evidence hash/note, reviewer identity or secret. read_probe_eligible
does NOT schedule or execute device traffic and does not modify
connectivity, health or last_verified_at.

On the actual nonroot Ubuntu26 candidate worktree, control-api Rust
tests passed 31/31 after rustfmt. Owner-Mac static R9.1 source contract
passed 5/5. Real disposable PostgreSQL migration, signed-session
integration and full GitHub CI are still PENDING at this candidate
checkpoint and must pass before merge. No real OLT/ONT/router packet,
credential or firmware operation was generated by R9.1 development.

RED PRD OPEN: actual human IdP/MFA/BFF, secure private device path,
verified physical model/firmware identity, dedicated real read-only
account, durable read-probe intent/worker, live vendor interoperability
and full production DR remain mandatory.


## R9.2 development checkpoint — 2026-09-28

Prior code main R9.1 was independently
verified in GitHub Actions run 36377736411:
all four jobs SUCCESS at SHA
b7a4812b3751f33e911b668fc6cafa655208b1de.
Its migration 0008 stores four immutable
separately attested physical readiness
prerequisites and exposes an UNMOUNTED
signed-opaque session read projection only.

Following the requested priority sequence,
R9.2 adds genuinely durable but explicitly
nonexecutable per-candidate first-read intent
in PostgreSQL migration 0009, unique
idempotency and same-transaction audit,
a separate NOLOGIN SECDEF+EXECUTE
writer, restricted own POP read projection,
and an UNMOUNTED original Rust
opaque signed-identity+fresh SQL+
trusted-origin+CSRF mutation bridge.
No public HTTP mount, execution broker,
device worker, transport, actual OLT/ONT
connection or firmware write is added.
Actual disposable PostgreSQL and CI
integration must pass before merging;
no claim of successful physical device
adoption is made. See the detailed
R9.2 runbook, ADR-040 and newest CI
checkpoint at milestone close.

## R9.2 independently verified immutable nonexecutable read-intent release — 2026-09-28

**Real software acceptance:** feature PR #110 commit
`b9e3edd264bb7336b68fe071603ea5c7af69f9cc`
passed mandatory GitHub run `36379223832`
ALL **4/4 SUCCESS**, including real disposable
PostgreSQL 16 0009 migration/role isolation/
two-company maker-checker readiness/intent/audit
checks and the real RSA signed ID+access-bound
opaque identity session with exact CSRF, fresh
separate restricted SQL role and a successful
actual PostgreSQL immutable request+safe list.
Independent AFTER CODE MAIN merge CI
`36379530246` passed ALL **4/4 SUCCESS**
at exact canonical code main SHA
`9e02ded020d697b8c5b82bc6ad3e3357c2eda2bb`.
All existing real disposable 1-node Ubuntu26
K3s, PostgreSQL physical/logical restore,
CWMP original SOAP/USP actual protobuf
software tests were retained and passed;
NO test contacted owner's real OLT.

Private GitHub, owner's clean FileVault Mac
and actual authorized clean NONROOT
Ubuntu 26.04 VPS were independently
synchronized to this exact code SHA by
SHA256-verified Git bundle and clean
fast-forward. Actual nonroot canonical VPS
full locked OFFLINE Rust workspace/fmt,
R9.1 static 5/5 and new R9.2 static
4/4 PASS. The existing actual nonroot
VPS Rust dashboard preview was guardedly
restarted and owner Mac's actual private
SSH tunnel reestablished using exact SHA.
Real owner Mac HTTP returned HTTP200
for current /lab and /lab/device-workbench,
physical device count remained ZERO;
forged Host/tenant/role/bearer were refused
HTTP401 by all three business
namespaces; unprovisioned true signed
SQL private routes remained HTTP404.
No intent mutation or live IdP
route was publicly or privately mounted.

FileVault owner Mac encrypted
SOURCE-only Restic snapshot
`f7aa9126` passed full 168/168
pack reads, isolated exact
SHA256 source restoration,
and separately selected historical
PARTIAL root-readable restore.
This is NOT complete offsite
VPS/actual customer PostgreSQL
or actual K3s datastore DR.

**RED MUST REMAIN UNIMPLEMENTED:** customer
confidential true IdP authorization-code
redemption, production human MFA and
multinode HTTPS session, physical
device-specific verified secure
private management, exact real C320
identity/model/firmware, independently
approved restricted real device
read-only account, actual
hardware interop/health, separately
audited per-device durable worker
execution with atomic fresh gate
checks, and whole-host+PG+K3s
offsite restore. R9.0 owner Mac
observed untrusted public Telnet IAC
but actual VPS timeout; NEVER
send even test passwords on this
public plaintext Telnet path.
R9.2 is an audited stored INTENT
only, NOT physical authorization.

## R9.3 secure site-access gate draft — 2026-09-28

Following the owner-provided exact DEV-01 C320 public Telnet NAT,
created `docs/R93_C320_SECURE_SITE_ACCESS_GATE.md` and aligned PRD
and device ledger. This milestone is DOCUMENTATION ONLY and requires
owner-side secure management access before any further live attempt.
No real device login, secret handling, secure route, VPS retest, worker,
firmware change, deployment, live identity or fresh physical test
was performed in R9.3. R9.2 stored requests remain NONEXECUTABLE.
Current blockers: safe management protocol/version and secure owner
site router/tunnel selection, verification of actual restricted worker
route, independent exact device identity and account, live IdP/MFA,
independent review and actual read-only TC-OLT-01 interoperability.
Next priority: owner confirms SSH/SNMPv3 or site-isolated Telnet over
verified VPN; validate the exact secure path and identity first, then
implement narrowly audited leased read-only dispatch with disposable
tests before separate opt-in physical read. No architectural decision
changed; existing ADR-038 through ADR-040 remain binding.

## R9.3 owner-site transport clarification — 2026-09-28

Owner now confirms DEV-01 ZTE C320 has ONLY Telnet available and
no existing site VPN. Proposed solution is a separate scoped WireGuard
site gateway plus truly isolated final local Telnet management hop.
The site router, VLAN isolation, exact private OLT IP and rollback
route have NOT been confirmed; no live network modification,
connection with credentials or physical device test occurred.
Documented staged requirements and no-bypass risks in
`docs/R93_C320_SECURE_SITE_ACCESS_GATE.md`. Continue offline
software work independently; live physical TC-OLT-01 BLOCKED until
independently reviewed gateway, secure route, account and identity.

## R9.4 RouterOS 7 owner confirmation and offline planning

Owner confirms MikroTik RouterOS 7 is available at the C320 site.
Added offline-only WireGuard site-gateway predeployment guidance and
address overlap preflight/tests. No actual MikroTik, C320 or Ubuntu
VPS changes or credentials were used. Next: review redacted gateway
model/RouterOS minor version, safe OOB recovery and exact isolated
OLT management path; only then propose an attended staged rollout.

## R9.4 gateway information checkpoint — 2026-09-28

Owner reports an x86 RouterOS 7 gateway directly connected to the
C320 local network and physical or console access. R9.4 plan and
DEVICE_MATRIX updated. Addressing and local isolation remain unknown.
Five offline address-planning tests passed. No physical test or
network configuration changes were made in this checkpoint.

## R9.5 optional VPN and lab connection selector — 2026-09-28

Owner clarified VPN should be optional and manageable by Tenant Admin
Dashboard, not an ad hoc CLI dependency. Added LAB-only connection
method selector to existing Device Manager HTML/JS with public Telnet
and RouterOS6+WireGuard rejection, plus static offline contract tests.
Owner-authorized one-shot no-auth Mac network recheck to reported
public candidate returned TCP reachable/Telnet IAC/15 bytes, ZERO
outbound credential or Telnet bytes. This is NOT authentication,
physical adoption, verified C320 identity or actual VPS route health.
No actual tenant backend, configuration database, gateway deployment,
OLT CLI, firmware changes or customer network changes performed.
Next: signed MFA Tenant Admin network connection inventory persisted
with exact tenant/POP isolation, reviewed transport-plan workflow,
and an explicitly nonexecuting dry-run; separately verify owner-site
secure last hop and dedicated actual worker reachability before any
physical read-only C320 acceptance.

R9.5 executed local verification: 3/3 new preview contracts, 5/5
R9.4 network-plan unit tests, 7/7 R9.0 no-auth preflight tests,
8/8 isolated R7.9 C320 SSH adapter mock tests and Node JS syntax
passed. The broader R7.9 test discovery FAILED one integration
setUpClass because the required compiled Rust parser binary was
absent on the owner Mac; do not report the full R7.9 suite passed.
Actual DEV-01 test was bounded Mac TCP/Telnet transport only.

## R9.6 safe-as-live device protection and backend-only lab planner

Implemented Rust Axum private LAB-only enum `/lab/demo/connection-plan`
POST and deliberate dashboard validation button, no saved config,
credentials, network actuation, adoption or worker dispatch. Added
Rust Axum tests for same-origin denial, plan-only response, public
Telnet denial, RouterOS6 mismatch, unsupported secure protocol and
unknown real-secret fields. R9.6 static Python UI 3/3 and JS syntax
checks executed successfully; Rust code compile/actual Axum tests
must run on toolchain-equipped Ubuntu or GitHub CI before merge.
No new physical OLT command, Telnet login or network mutation.
Prior Mac passive TCP/Telnet 15-byte result is historical only.
Next: independently run locked Rust tests/CI, merge only after
acceptance, then build genuine DB-backed Tenant Admin scoped draft
and approved tunnel workflow with safe staged rollback.

## R9.7 staged durable network-method draft (development only)

Proposed `deploy/db/migrations/0010_lab_connection_drafts.sql` and
new actual disposable PostgreSQL integration test plus ordered CI
step; no live customer migration, HTTP mount or tunnel executor.
Client can select supported methods via R9.5 and invoke nonexecuting
R9.6 LAB backend plan; R9.7 intends server-side durable actual-tenant
method-only drafts once real IdP/MFA prerequisite is proven. Until
fresh complete CI after this change, SQL integration is UNVERIFIED.
No actual C320 provisioning or CLI command performed; treat it as
live distribution and require independent safe path + evidence.

## R9.8 unmounted typed tenant connection BFF (development only)

Added Rust signed opaque-session plus exact CSRF and fresh scoped SQL
seam to propose only immutable connection-choice metadata via separate
role. Not mounted in actual HTTP. R9.8 static boundary test 1/1 PASS
and pinned Linux rustfmt stream comparison PASS. Rust compile and
real signed-session/PG R9.8 integration NOT YET independently proven.
No keys, credentials, device traffic, gateway configuration or
real C320 operation introduced by R9.8.

## R9.8 actual IPAT VPS source-route recheck — 2026-09-28

One explicitly bounded nonroot no-auth check from the real Ubuntu
IPAT VPS to the exact owner-provided public DEV-01 TCP321 endpoint
returned TIMEOUT (observed 2026-09-28T06:19:51Z), despite historical
owner-Mac TCP/Telnet receipt. Zero client credential bytes,
no OLT command, authentication, firewall/NAT/router modification,
firmware or physical-adoption status changes. This is a source-
dependent reachability blocker of UNDETERMINED cause. Public Telnet
would STILL be unsuitable for credentials even if route were fixed.
No provider firewall mutations are approved without independent
recovery and per-VPS isolation; obtain controlled private site path.

## R9.6–R9.8 independent GitHub acceptance — 2026-09-28

Feature SHA `434e6b9144b05ef6a54dc46c4439c40ad0189d14`
passed ALL 4/4 independent GitHub Actions jobs in run
`36385828544` on PR #112: locked Rust/unit/static tests,
real disposable PostgreSQL two-tenant R9.7 draft migration
and exact own-POP policy tests, separate PostgreSQL physical
recovery lab, and isolated Ubuntu26 K3s. CI does NOT imply
real private OLT connectivity, real IdP/MFA or secure management.
R9.8 original Rust session seam is UNMOUNTED and only static
boundary plus compilation/unit code was exercised, not genuine
joined live human IdP and separate real writer account.
Owner-reported C320 must remain UNKNOWN/NOT_MEASURED. Current
actual IPAT VPS recheck TIMEOUT is recorded above; do not
attempt public Telnet credential-based adoption. Feature PR
remains a development branch pending separate integration and
release review; no customer production deploy performed.

## R9.9 owner default OLT credential disclosure and VPS root-access diagnosis — 2026-09-28

Owner offered temporary factory/default DEV-01 C320 username and
password in chat. Treat as disclosed/high-risk credentials; DO NOT
persist exact values in repository, tests, terminal history or
logs. NO Telnet authentication was attempted: public TCP321 is
plaintext and real VPS source path most recently TIMEOUT. Public
banner is not validated hardware identity; no device provisioning,
OLT read, ONT operation or firmware activity occurred.

Independently verified actual owner-Mac `ssh ipat-lab` succeeds as
nonroot `openai`, which belongs to Linux `sudo` group. VPS has
`/etc/ssh/sshd_config.d/00-ipat-lab-hardening.conf` with
`PermitRootLogin no`. Explicit root key attempt is offered and
accepted at SSH key-selection stage but direct root login is denied;
there is no SSH authentication-agent session from tool context.
Advised owner to use `ssh ipat-lab` followed by local interactive
`sudo -i` without uploading any passphrase/password. User-side
sudo root shell NOT independently confirmed. No SSH policy, keys,
root filesystem or perimeter firewall were modified. Direct root
SSH re-enable remains blocked pending tested independent console
recovery and approved rollback.

Newest PR #112 docs-only HEAD `e2347171229ee73e5e5d3a78ea0b221124cf866e`
passed 4/4 GitHub jobs in run `36386319167`. This confirms lab
software CI, NOT production physical OLT interoperability or
human MFA. Next gated priority remains owner-controlled secure
site gateway with dashboard-backed staged review and no-impact
read-only C320 validation after verifying actual local isolation.

## R9.10 DEV-01 private SSH: actual Mac handshake only (2026-09-28)

Owner reported private SSH candidate `10.10.13.233:321` and
OpenSSH failure with legacy host key offers. Per-owner-authorized
Mac diagnostics established TCP and received `ZTE_SSH.1.0` SSHv2
banner. Default client negotiation failed because offered host key
algorithms were ssh-rsa and ssh-dss. Explicit per-process
HostKeyAlgorithms=+ssh-rsa negotiated group16-sha512, then failed
because server offered only older CBC cipher suites. A SECOND
bounded diagnostic with per-process Ciphers=+aes128-cbc
negotiated AES128-CBC / HMAC-SHA1, obtained an RSA host-key
fingerprint (reported to owner for independent trusted-site
verification), and STOPPED at `Host key verification failed`.
No SSH login, passwords, stored known_host pin, shell commands,
production gateway change, OLT/ONT config read or write occurred.
The host banner and network reachability do NOT independently prove
actual chassis serial/model/firmware or nonshared trusted last hop.
No host-key verification bypass or global weak SSH config change
was performed. Next gate: owner compares host-key fingerprint
via independently trusted local OLT console or controlled site
inventory, validates dedicated non-disruptive read-only account
and local path isolation, then controls one session with audited
bounded commands and measured operational impact. Newer SSH
firmware/options should be evaluated vendor-specifically but
firmware change remains separately gated on this LIVE OLT.

## R9.11 live-distribution physical no-auth pre-adoption — 2026-09-28

Built fixed-target private legacy SSH no-credential transport checker
`deploy/scripts/lab/r911/private_ssh_identity_probe.py` with exact
RFC1918 validation and per-run nonroot opt-in. Actual Mac R9.11 first
9s check TIMEOUT, bounded 20s repeat reached `ZTE_SSH.1.0`, same
R9.10 RSA SHA256 fingerprint, and STOPPED at unverified host key.
ZERO SSH credentials and OLT commands; no router/ONT/OLT mutation.
Added private-only static historical evidence endpoint and Device
Manager physical-gates panel, including explicit no online/health
claims. Five new mock/static unit tests PASS; existing R9.0 plus
new R9.11 12/12 on Mac; JS syntax and git diff check PASS.
Rust HTTP test/whole workspace should pass pinned CI before merge.
Physical C320 adoption remains BLOCKED by independent key proof,
local isolation, restricted read account, approved baseline and
actual private VPS worker path. No absolute zero-impact guarantee.

R9.11 added an explicit Rust public-API regression test that rejects
the physical-evidence route outside private non-K3s lab mode.
R9.11 source also documents offline repro plus an optional bounded
nonroot one-shot NO-AUTH private SSH probe; it is NOT a polling worker.
This final code must clear independent CI before merging. The owner
has not yet independently verified the observed RSA fingerprint,
so physical login remains prohibited in the live-distribution gate.

R9.11 parallel-lab canary code: hardcoded opt-in isolated
`127.0.0.1:3002`, never interferes with existing :3000 and refuses
public K3s and all identity/membership/registrar/reviewer options.
New Rust bind-control tests added; actual Ubuntu nonroot canary
runtime/network isolation remains UNVERIFIED until build, listener
inspection and HTTP negative checks are explicitly executed.
No live OLT login or router operations authorized by this setting.

## R9.11 independently tested owner VPS temporary canary

Owner nonroot VPS build from verified Git bundle SHA
`cea7692fcd4ab73be7aa62bb0ba73a68837fa4b023e4ea560e7de29d69185f4c`
checked out standalone code SHA `621ad09` and compiled pinned locked
Rust OFFLINE single job at low priority; binary SHA
`39d3bb51d6b5359c48af00732d185cd0c46aab65b02ab207ec26a206b0cec233`.
Independent GitHub Actions on SAME code SHA: run `36391439831`
ALL 4/4 jobs PASS (unit/static Rust, disposable PostgreSQL RLS,
separate PG physical recovery, isolated K3s Ubuntu26).
Temporary private VPS HTTP canary after two corrected smoke-harness
failures (TIME_WAIT bind check and header casing) PASSED actual
127.0.0.1:3002 evidence HTTP200/no-store, denied POST405 and forged
real APIs401, loopback-only bind, original :3000 HTTP200 before/after.
No account/tenant data or OLT credentials read; canary intentionally
STOPPED at end and no persistent public service installed.
Added versioned Python smoke script and static safety contract;
local combined R9.0+R9.11 tests 13/13 PASS.
This is LAB preview validation, NOT verified actual OLT adoption.

## R9.11 current owner VPS on-login private preview operational evidence

After successful temporary nonroot canary smoke and previously verified
4/4 CI on exact app code SHA `621ad09`, staged reviewed source-only
user service SHA256:
`7dffe3e79e492a830cd69ba9d90fa2fe0b878455c860e7de29d69185f4c`.
Installed at `~/.config/systemd/user/ipat-r911-preview.service`.
Installed/running/enabled under `openai` WITHOUT `sudo`; existing
127.0.0.1:3000 remained bound and HTTP200 throughout. New
127.0.0.1:3002 responds HTTP200 on device-workbench and physical
evidence GET with no-store; unsupported POST HTTP405 and fake real
business API GET HTTP401. A separate actual Mac SSH local-forward to
VPS :3002 successfully fetched both and confirmed adoption false.
Verified systemd runtime: ActiveState=active, SubState=running,
NoNewPrivileges=yes, ProtectSystem=strict, ProtectHome=read-only,
MemoryMax=268435456 bytes and CPU quota 200ms per second.
`Linger=no`: service is enabled to start with user session, not
promised HA after all sessions end; root-level linger intentionally
unchanged. Safe nonroot rollback:
`systemctl --user disable --now ipat-r911-preview.service`.
No change to old :3000.
This preview DOES NOT accept real VPN keys, OLT logins, provisioning
or customer tenant data; physical DEV-01 remains NOT ADOPTED.

## R9.12 synthetic dashboard WireGuard review implementation

PR #112 R9.3–R9.11 MERGED into GitHub main SHA e4ab353 after
independent feature CI 4/4 PASS; newest GitHub main CI should be
separately verified before claiming main acceptance. Branch
`feat/r912-wg-reviewed-dashboard-preflight` introduces one synthetic
WireGuard wizard in existing lab Device Manager and private Rust
fixed-enum zero-action review POST, with real Axum tests and
static tests included in pre-existing CI R9.0 test discovery.
Mac local combined lab 17/17 PASS, Node JS syntax PASS, diff clean.
Real OLT authentication, tunnel configuration, live tenant MFA,
worker private routing and physical adoption remain NOT DONE.
