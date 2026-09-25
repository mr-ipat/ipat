# IPAT Ubuntu laboratory — staged bootstrap

Status: **Stage 1 root operation prepared, NOT executed.** Owner authorizes the staged toolchain only by entering their Linux sudo password in their own Mac Terminal. The assistant never receives the password. Server is 16 KVM vCPU / 30 GiB guest RAM / 250G disk, below the provisional 16 / 64 GiB / ~1 TB pilot baseline. Keep this as a bounded lab, not a commercial deployment.

## Changes already executed without root

- Dedicated Ed25519 Mac SSH alias authenticates to the existing non-root `openai` account with pinned host-key verification.
- User-only official Rust 1.98.1 toolchain and rustfmt were installed from a TLS-downloaded `rustup-init` checked against its published SHA-256. No APT packages or global environment files were changed by this installer.
- Canonical GitHub private `main` was transferred as a verified Git bundle to `/home/openai/workspaces/ipat`. No GitHub credential, PAT, GitHub deploy token or private SSH key was copied to the VPS. Git bundle remote was removed; future updates require a new verified transfer.
- A **PARTIAL** pre-change backup of user-readable SSH/apt config and `authorized_keys` was saved to the Mac at `~/IPAT-secure-backups/` with private filesystem modes and archive integrity checked. Root-only `50-cloud-init.conf`, host SSH private keys, VM block storage, and application/database data are NOT in this backup.
- Local `cargo fmt --all -- --check` on actual Ubuntu passed, but initial `cargo test --workspace --locked` **failed** because Ubuntu image lacks the system linker `cc`. GitHub runner tests are separate evidence, not an Ubuntu-local pass.

## Stage 1: one-time, narrowly approved privileged action

Prerequisites: owner access to the provider's *functional* out-of-band console, intact Mac private key and partial backup, and approved restricted lab scope. **No provider snapshot exists.** Keep a separate provider console tab open while running the script.

From the authorized Mac's **interactive Terminal** (not inside ChatGPT):

```bash
bash ~/Projects/ipat-current/deploy/scripts/lab/apply-stage1-from-mac.sh
```

The helper checks the off-host partial archive, fast-forwards the Mac Git checkout from private GitHub, transfers the verified main Git bundle to the non-root account, and runs a read-only Ubuntu `--check`. It then explains the exact change and prompts you to type **APPLY**. Only then it creates a root-owned copy of the SHA-256-verified root script and requests the Linux `openai` user's sudo password **locally in that Terminal**.

The root script checks Ubuntu 26.04, disk headroom, existing SSH access and `sshd -t`; makes a 0700 **local root-only** config/package baseline in `/var/backups/ipat-lab/stage1-*/`; refreshes existing signed Ubuntu APT sources; simulates `build-essential` package resolution, aborts if removals or upgrades of existing packages are planned; installs only `build-essential`; writes a sanitized, clearly synthetic-context `sshd -T` report to `~/.cache/ipat/stage1-sshd-policy.txt` for the follow-up review; verifies `cc` and SSH health. It deliberately does **NOT** modify SSH policy, root login, password authentication, firewall, routing, K3s, DNS, systemd services, database, secrets or user permissions, and does not request a reboot.

After successful installation, the helper runs `cargo fmt --all -- --check` and `cargo test --workspace --locked` as the non-root user on the actual Ubuntu VPS, printing the results. It can be rerun; it will create another dated root backup and repeat an idempotent APT install if invoked again, so avoid unnecessary repeats.

## Stage 2: security hardening requires its own checked change plan

- With approved elevated read-only access, record real effective `sshd -T` including all root-only Ubuntu cloud-init drop-ins and root/openai match context. The *readable main file* currently contains `PermitRootLogin yes` and `PasswordAuthentication yes`, but a root-only early include may override them. Do **not** claim these are effective until verified.
- Establish and **test** the provider console recovery path. Export a complete **encrypted off-host** backup appropriate for the sensitivity of privileged config, and verify that it can be restored; the current partial Mac backup and same-disk root copy are not full disaster recovery.
- Plan key-only SSH with an automatic timed rollback and an independent second session test. Do not change the port, deny password login or disable root SSH in the same transaction as untested firewall changes; preserve known-good access throughout.
- Verify provider ingress controls. K3s networking/CNI and default UFW rules require joint design; avoid exposing control-plane, VXLAN or metrics ports to the public internet. No externally accessible ACS/USP/API listener until tenant/agent auth, TLS, rate limits and server network boundaries are tested.
- PostgreSQL HA/PITR, external backup destination, restore drills, telemetry retention and K3s production configuration remain separate gated milestones. No snapshot means **no stateful real subscriber or device data yet**.

## File paths and verification

- `stage1-root-preflight-and-toolchain.sh --check`: safe rootless read-only preflight.
- `stage1-root-preflight-and-toolchain.sh --apply`: requires root, two independent explicit owner/backup gate variables and is invoked only by the Mac helper.
- `apply-stage1-from-mac.sh`: interactive, prevents dirty repo overwrite, transfers a verified source bundle, validates root-script SHA-256, then requests sudo interactively.
- Actual run evidence and blockers belong in `docs/PROJECT_STATUS.md` and `docs/LAB_SERVER_READ_ONLY_2026-09-25.md`. These shell scripts do not claim production hardening or network-device interoperability.
