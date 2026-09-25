# IPAT — Project Status

**As-of:** 2026-09-25 Asia/Jakarta  
**Milestone:** R4.7 — non-root binary SSH→encrypted Restic streaming + isolated restore verified; privileged root backup queued for owner sudo  
**Milestone state:** R4.6 live SSH/FileVault and existing encrypted partial/source Restic restores are verified; owner now states the Nusa allow-all security group is SHARED across multiple VPSs, actual VNC Linux login is UNSUCCESSFUL, and Restic secret escrow outside Mac is done (OWNER-ATTESTED, not independently recovered). New binary-safe root-config streaming/Restic code is prepared and its real unprivileged SSH→encrypted Restic→isolated-restore smoke test and intentional failed SSH producer/no-snapshot test PASS. A privileged root-owned configuration backup + restored-content verification require one owner-entered Ubuntu sudo password locally and are NOT YET COMPLETE. Shared provider SG, guest firewall and K3s are UNCHANGED.

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

## 17. R4.5 — verified temporary Mac restic backup, isolated restores and Nusa network preflight (2026-09-25)

**Actual work completed by assistant using authorized Mac (no Ubuntu root/provider rule changes):**
- Installed `restic 0.19.1` on the user's Mac with Homebrew. Generated a high-entropy password using local cryptographic randomness; stored it only in the user's macOS login Keychain (service `id.ipat.lab.restic.backup.v1`, account `ipat-lab-backup`), never printed the password or committed it to Git. Restic reads it via `RESTIC_PASSWORD_COMMAND`. Created mode-0700 encrypted repository `~/IPAT-secure-backups/restic-lab-v1` (ID prefix `8b539c4534bf`).
- Snapshot `c5447733`: encrypted prior **PARTIAL** Mac-local VPS configuration archive. Ran a real, *separately restored* test under a private disposable Mac directory, compared actual SHA-256 against the earlier known verified archive checksum `2d74782d0e418262701107be01e91c0ea87d3b7406803ca58acdb91a57338a77`, and checked the tar could be listed. **PASS**.
- Snapshot `f38403d0`: streamed Git `main` revision `d5ac374eec5fff27a2f58bbc1f8887768050e2f4` using `restic backup --stdin-from-command -- git archive`, which propagates command failures. Restored into a separate private disposable directory; independently computed the original Git archive SHA-256 and verified it matches the decrypted/restored archive byte-for-byte; brief/decisions/status files present. **PASS**. Re-backup the newly merged source after the current R4.5 PR lands (snapshot above only covers its stated revision).
- Ran real `restic check --read-data` across **both snapshots and four packs**: **PASS, no errors**. Removed temporary restored plaintext test directories; retained the old source PARTIAL plaintext archive from prior milestones pending explicit safe retirement.
- **Security limitations:** `fdesetup status` on Mac returned **FileVault Off**. The Keychain-held passphrase and encrypted restic repository reside on the **SAME MAC**, with no independently protected passphrase escrow or separate backup copy. The snapshots cover only the earlier partial readable VPS config and committed Git source; VPS root-only files, private SSH host keys, PostgreSQL and future K3s/datastore/server token are **NOT BACKED UP** in this milestone. This is useful temporary, separate-host laboratory redundancy; NOT production DR and NOT PRD AC-07 database restore.
- Public external network check from authorized Mac: DNS `ipat.fadly.id` returned IPv4 `202.162.204.121`, no DNS AAAA in sampled query; **TCP/22 accepted**. TCP 80, 443, 6443, 10250, 2379 and 2380 did not establish connections. Without listeners, this **cannot prove Nusa Managed Firewall filtering**. UDP/8472 was not tested; IPv6 provider rules uninspected; host firewall, Nusa panel rules and NAT details UNKNOWN.
- Official Nusa documentation confirms the Cloud VPS management UI exposes IPv4/IPv6 inbound/outbound custom rules, VNC Console and Rescue Mode. The official `Reset Rules` feature restores **allow-all IPv4 and IPv6** and must NOT be used as a security hardening shortcut. **No Nusa panel/provider firewall rules or Ubuntu firewall were changed** during these tests.
- Changed code/docs: `deploy/scripts/lab/backup-mac-restic.sh`, `test_backup_mac_review.py`, `nusa-readonly-network-check.sh`, `test_nusa_network_review.py`, `docs/LAB_BACKUP_NUSA_R45.md`, `docs/PROJECT_STATUS.md`, plus `docs/SECURITY.md`, `docs/DEPLOYMENT.md`, `deploy/scripts/lab/README.md`, `README.md` and CI. Mac initial static checks **PASS**: five backup-contract and four Nusa source-inspection tests; the read-only Nusa probe was actually run. Newly generated backup script `--verify` requires a clean `main` and must run again after the R4.5 PR merge.
- **No architecture ADR automatically approved:** user selected a temporary Mac backup destination for this restricted lab, not a final ADR-008 secret/backend storage strategy; ADR-010 production PostgreSQL PITR, and ADR-017 cluster network/provider/datastore remain OPEN.

**Next change gate:** obtain the owner's redacted **Nusa Managed Firewall rule inventory** for both IPv4 and IPv6 and evidence of working VNC Console rescue login; the owner should enable Mac FileVault and safely escrow the restic recovery password outside this Mac *without sharing it*. Build a separately reviewed owner-interactive root-only configuration backup that streams sensitive data directly into encrypted restic with a successful isolated restore. Only after these gates can an approved provider ACL transaction and isolated/disposable K3s laboratory installation be considered. Until then, do not activate public Kubernetes/CWMP/USP ingress or stateful subscriber/tenant data.


### R4.5 post-merge independent backup/CI verification (2026-09-25)

- PR #22 merged into GitHub private `main` `deecfed13bfa6d453816e89af0b3592c8a541067`; its PR and post-merge GitHub main workflow `36115936092` **SUCCESS**, including nine newly added static Mac-restic/Nusa source checks, previous nine static SSH/K3s checks, formatting and 23 locked Rust unit/simulator tests on GitHub-hosted Ubuntu.
- The freshly merged clean `main` was re-backed-up using the committed actual Mac helper, producing latest-encrypted-source snapshot `2ee0cc82` (the older initial source snapshot `f38403d0` remains as history). Actual helper `--verify` succeeded on **3 snapshots/6 packs**, decrypting and isolating both the latest source and earlier partial configuration, checking independent bytewise SHA-256 and tar entry integrity. Temporary test plaintext was deleted.
- Verified private GitHub, Mac and actual Ubuntu 26.04.1 `main` commit checksums match the merge commit via short-lived Git bundle; SSH service active and no Stage-2 rollback marker. **Mac FileVault still OFF**. Local Keychain password and repository lack independent escrow/second copy; ROOT-ONLY VPS config, K3s token/datastore and any future PostgreSQL database remain UNSAVED. Nusa provider panel firewall rules remain UNKNOWN; no firewall or K3s installed/modified.
- A later docs-only merge will advance Git `main`; re-run the Mac backup helper against that new HEAD and log its snapshot in GitHub issue #18 without recursively amending this evidence. Full restore and AC-07 acceptance remain **BLOCKED**.

## 18. R4.6 — owner screenshots and independent FileVault/Nusa dual-stack check (2026-09-25)

- **Owner evidence:** Nusa panel screenshot displays global IPv6 address and `allow-all` security group permitting **all inbound IPv4 `0.0.0.0/0` and inbound IPv6 `::/0`**, as well as all outbound for both families. The image does NOT show whether `allow-all` is shared by other VMs/interfaces, whether additional security groups exist, or the full per-VPS Managed Firewall/NAT policy. The owner also provided a Mac FileVault screen reporting **encryption finished** and a recovery key set, but separate recovery-key custody was not verified.
- **Independently verified on authorized Mac:** `fdesetup status` now reports **FileVault is On** (supersedes older R4.5 FileVault OFF measurement); the Keychain password still decrypts the existing Restic repo. After enabling FileVault, the assistant re-ran the actual committed `backup-mac-restic.sh --verify` on current reviewed source `main`: `restic check --read-data` **PASS** over **4 encrypted snapshots/8 packs**, isolated temporary restore of latest canonical source and historic partial config matched independently computed SHA-256, and temporary test directories were removed. Same-Mac Keychain + backup location remain a correlated failure risk; independent password escrow and second encrypted copy are **NOT VERIFIED**.
- **Independently verified on Ubuntu:** actual guest `eth0` holds global IPv6 `2401:2900:2fc::114/128` and a default IPv6 route; SSH listens on `0.0.0.0:22` and `[::]:22` with previously verified root-owned key-only directives, even though earlier sampled DNS had no AAAA. Root login/password authentication is disabled in the installed host drop-in, but open public dual-stack provider ingress is still a significant risk for future listeners and source-agnostic SSH brute-force attempts.
- **Mac management-source observation:** two separate direct IPv4 echo checks agreed at inspection time; this **does not prove stable assignment**. Strict external Mac IPv6 source check did NOT establish a usable IPv6 management address; do not guess a /128. No externally reachable IPv6 probe from an independent source was performed. The tested Mac IPv4 /32 is only a candidate to confirm immediately before a staged firewall transaction.
- **R4.6 source prepared:** `docs/NUSA_SECURITY_GROUP_R46.md` includes exact current allow-all exposure, family-specific proposed default-deny inbound, documented rule/VM attachment inventory, fresh Mac IPv4 source + tested VNC console gate, and provider-specific rollback. New non-mutating `deploy/scripts/lab/nusa-r46-gate-check.sh --report` and four accompanying static tests passed syntax and Mac run. No Nusa panel access, host firewall modification, security-group change, sudo action, root-only backup or K3s install was executed. Existing Open ADR-008/010/017 remain unchanged.
- **Required follow-up before live network change:** (1) owner independently confirms **actual VNC console login**, not just that a console screen opens; (2) owner confirms the Keychain-held Restic password has been securely escrowed outside this Mac without sharing it; (3) review affected `allow-all` group attachment and all IPv4/IPv6 per-VPS rules to avoid disrupting another VM; (4) prepare and separately verify a privileged root config backup streamed directly into encrypted Restic without persisting plaintext on Mac; (5) only then perform one monitored provider rule change and fresh dual-stack testing with documented manual rollback. Avoid Nusa `Reset Rules`, which official docs show restores global allow-all. K3s/PostgreSQL remain BLOCKED.

## 19. R4.7 — owner restrictions + tested failure-aware encrypted root-stream preparation (2026-09-25)

**Owner-confirmed input, not independently verified beyond cited probes:** real Nusa VNC Console Linux login **UNSUCCESSFUL**; the permissive `allow-all` security group is **SHARED among multiple VPSs**; Restic password is safely escrowed outside Mac and available on another device (**OWNER ATTESTATION** only). Never modify shared `allow-all` rules. The live source-level facts from R4.6 (FileVault ON, global guest IPv6/default route, dual-stack SSH binding) still apply. No provider or guest firewall rule, SSH setting, package or K3s/DB service was changed by R4.7.

**Independent read-only VNC prerequisite inspection:** actual Ubuntu 26.04.1 `getty@tty1.service` returned `active`, `openai` reported password status `P` and interactive shell `/bin/bash`. This does not prove browser VNC input/authentication works; owner must retest the correct VPS browser Console with Ubuntu `openai` account credentials, not an SSH key passphrase, and seek Nusa Support for VNC access if still unsuccessful. Do NOT trigger Rescue Mode (it reboots) or weaken the existing key-only SSH to debug console.

**Code and scope prepared:** `deploy/scripts/lab/root-config-stream.py` and `mac-root-config-backup.sh` generate a fixed-purpose Linux `sudo -S` read-only tar stream from selected root-accessible SSH/sudoers/network/cloud/apt config directly over strict authenticated SSH into the existing encrypted Restic Mac repo, without writing an unencrypted root archive during backup or putting the sudo password into CLI arguments/environment/Git. The sudo password must be typed *locally in the owner's interactive Mac Terminal*. The sensitive root archive intentionally EXCLUDES SSH host private keys and ALL real K3s/PostgreSQL/app state; thus it is a selected-root-config backup, not full host/PRD AC-07 DR. Its runner decrypts snapshots only into a private throwaway Mac directory to verify a real isolated restored tar, then removes plaintext.

**Actually executed test evidence BEFORE privileged backup:**
- `python3 -m py_compile` + `bash -n` **PASS**; actual target Ubuntu `bash -n` parsing of the fixed root tar payload **PASS** without running as root; five Python static source-contract tests **PASS**.
- Actual strict `ssh -T` `--smoke` from the Mac streamed a user-readable Ubuntu tar as **binary** via the same Python and Restic `--stdin-from-command` path; saved encrypted test snapshot `81da8cc6`, and restored it to a separate private Mac test directory. Valid gzip/tar archive entries and full repository data read **PASS**.
- A Mac cleanup trap scope issue appeared after the first smoke's successful restore and was corrected. The first unprivileged test's abandoned `restore-root-test.*` directory was inspected and explicitly deleted. Rerun with the repaired cleanup generated snapshot `ce1c8807`, successfully restored it, validated its gzip/tar listing, checked all Restic packs and **removed** the throwaway plaintext test directory; **PASS**.
- Actual forced `--fail-smoke`: intentionally incomplete remote stream, SSH producer exit 42 propagated as nonzero from the Python wrapper. Restic correctly **FAILED** the backup and a subsequent snapshot query verified **NO snapshot with the negative-test tag** was created. The documented `--stdin-from-command` failure check is materially safer than a blind pipe to Restic `--stdin`.
- **Not yet run:** real root `sudo` stream; encrypted privileged/root-config snapshot; independent actual root-config restore; any full-root SSH host-key/disk snapshot or PostgreSQL/K3s recovery. These require the reviewed helper merged into GitHub, Mac/VPS synchronized, and the owner's one-time LOCAL Ubuntu sudo prompt.

**Repository changes in this milestone:** `root-config-stream.py`, `mac-root-config-backup.sh`, `test_root_config_stream_review.py`, `docs/ROOT_CONFIG_STREAM_R47.md`, updates to CI, `README.md`, `docs/SECURITY.md`, `docs/DEPLOYMENT.md`, `docs/NUSA_SECURITY_GROUP_R46.md`, lab runbook and status. No ADR was auto-approved: ADR-008 encrypted backup/secret design, ADR-010 PostgreSQL HA/PITR and ADR-017 CNI/provider private network selection remain OPEN where recorded.

**Next operator action after PR/CI merge:** from Mac interactive Terminal only, run `bash ~/Projects/ipat-current/deploy/scripts/lab/mac-root-config-backup.sh --backup-root`, confirm independently escrowed Restic password and enter Ubuntu account `openai` sudo password when prompted. The script then directly encrypts the root config stream and independently restores/validates the selected root archive, without touching Nusa rules. Ask for final nonsecret test lines; reverify snapshot and update status/issue #18 in a separate milestone. Keep K3s/firewall/production services blocked until actual VNC/equivalent rescue access and **dedicated IPAT-only SG** plus provider IPv4+IPv6/network restore review.
