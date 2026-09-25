# IPAT R4.7 — streamed encrypted root configuration backup preparation

**Date:** 2026-09-25 (Asia/Jakarta). **Laboratory security milestone, partial completion.** Continue from [R4.6](EDGE_SECURITY_GROUP_R46.md), the source PRD's S1 [AC-07](PRD.md), and the latest [actual status](PROJECT_STATUS.md).

## 1. Facts and current hard-stop gates

- **Owner-provided:** external provider VNC Console login has **NOT succeeded**. Their external provider `allow-all` Security Group is **SHARED BY MULTIPLE VPSs**. Do not edit or remove that group: doing so could affect unrelated services. No provider or guest firewall changes are authorized while the independent rescue path is untested.
- **Owner-provided:** the Mac Restic repository password has been stored securely outside the Mac and is retrievable from another device. This is an owner **ATTESTATION**, not an independent recovery exercise; no password or recovery key should ever appear in chat/Git.
- **Assistant verified read-only:** the real Ubuntu `getty@tty1.service` was **active**; the `openai` account had password state `P` and login shell `/bin/bash` when inspected. This does NOT establish that the user can type or authenticate successfully in external provider VNC. The first console check should use `openai` and the real **Ubuntu account/sudo password**, not an SSH private-key passphrase. Do not alter SSH, root passwords, or reboot merely to debug the console.
- Existing independent Mac/FileVault/Keychain encrypted Restic repository and older successful partial config + Git-source restore remain valid. The Mac backup is **temporary and on one device**; the Restic passphrase's second custody has not been independently tested. The target Ubuntu still has no provider snapshot.

## 2. Streamed privileged-root backup design (PREPARED, NOT YET EXECUTED)

Two code files in [the lab scripts directory](../deploy/scripts/lab/README.md):

- `deploy/scripts/lab/root-config-stream.py` is a fixed-purpose Mac binary-safe SSH producer. It requires SSH host-key checking, disables SSH multiplexing/password auth, and uses only the existing public-key `ipat-lab` account. For `--root`, it reads the Ubuntu `openai` sudo password via a local *interactive Mac Terminal* (`getpass`), forwards the password only through encrypted SSH **stdin** to a fixed `sudo -S -k` read-only `bash -c` `tar` command and outputs only gzip archive bytes through Python **stdout**. Neither private SSH key nor Linux sudo password is written to Git, Mac temp files, Restic, command-line arguments or environment variables.
- `deploy/scripts/lab/mac-root-config-backup.sh` requires actual `FileVault is On`, Mac mode-`0700` Restic repo with Keychain-held password, clean `main` matching current private GitHub `main`, fresh SSH key connectivity and an explicit operator confirmation of off-Mac escrow. It invokes `restic backup --stdin-from-command` to **directly encrypt** the remote root tar stream on the Mac, propagating producer errors. **No unencrypted root archive file is created on the VPS or Mac by the backup step.** The root action reads only selected configuration and does not install anything or rewrite any system settings.

The root stream is a **SELECTED CONFIGURATION ARCHIVE**, not a block-volume, disaster-recovery, or full host backup. It covers `/etc/ssh/sshd_config`, root-owned SSH config drop-ins, `/etc/sudoers`, optional `/etc/sudoers.d`, network/service config directories when present, selected Ubuntu apt/cloud-init config, `/etc/fstab`, and public `openai/authorized_keys`. It **intentionally does not include private SSH host keys, the Linux password/shadow database, K3s token/datastore, application secrets outside these paths, or PostgreSQL**. Selected cloud-init/network settings can still be sensitive, hence encryption and temporary FileVault protection are mandatory.

After a successful privileged snapshot, the runner performs `restic check --read-data` and **restores the encrypted snapshot** into a private temporary Mac directory for gzip/tar validation and representative required path/content checks (including the managed key-only SSH policy and the root-only `sudoers` archive member). The temporary decrypted archive and member list are removed on success or failure through a cleanup trap. A separate `--verify-root` command repeats the test later without sudo. This is isolated **configuration-archive** restoration, not a rehearsal of restarting a recovered VPS.

### Run after reviewed PR is merged (one owner-controlled Mac action)

Use the authorized Mac's **own interactive Terminal**; do not send any password or key in ChatGPT:

```bash
bash ~/Projects/ipat-current/deploy/scripts/lab/mac-root-config-backup.sh --backup-root
```

The runner requests the literal operator acknowledgement `ESCROW_CONFIRMED_AND_BACKUP_ROOT`, then prompts locally for the *Ubuntu openai sudo password*. It never requires external provider Console login or provider SG changes for this read-only remote backup; if anything fails, **do not retry by weakening SSH**. Report only the final success/failure lines, not secrets or archive content.

**If still unable to log in to external provider VNC:** verify you are looking at the *correct VPS's normal browser Console* (not Rescue), click into the console and wake its login prompt with Enter, enter Linux user `openai` and the account password. The Ubuntu getty/password state was observed read-only, not proven as a browser login. If login still fails or VNC is blank/unresponsive, capture a **redacted** screenshot/error state and open a external provider Support ticket requesting a *non-rebooting* VNC/login-path diagnosis. The provider's Rescue Mode does reboot into a separate temporary environment: **do not activate it as a casual test**.

## 3. Actual tests already performed before privileged execution

- Mac `python3 -m py_compile` and `bash -n` **PASS** for the new scripts, plus five source-only safety-contract unit tests. The fixed root `bash` payload passed a **real Ubuntu** `bash -n` parser check without execution.
- **Real end-to-end unprivileged SSH producer test:** `--smoke` streamed a small approved, user-readable VPS tar file to the Mac encrypted Restic repository using the **same Python binary-safe producer and Restic `--stdin-from-command`** path. Actual snapshot `81da8cc6` was separately restored, checked gzip/tar member paths, and the full Restic data check passed. A corrected trap cleanup was tested again with snapshot `ce1c8807`; separate restored plaintext was removed.
- **Real negative network/command failure test:** `--fail-smoke` emitted a deliberately incomplete stream then returned nonzero over SSH. The Mac Python producer propagated the failure; **Restic rejected the backup** and the follow-up snapshot query verified that the deliberately failed test had **NO SAVED SNAPSHOT**. This validates the failure-aware `--stdin-from-command` behavior; a naïve `ssh | restic --stdin` would not offer the same safety guarantee.
- **NOT YET RUN:** any real `sudo` root tar capture, encrypted privileged snapshot, actual root-config isolated restore or an independent recovered host login. Those require the owner's one-time sudo prompt on their own Mac and evidence collection afterward.

Official method: [Restic documented command-stream backup and failure behavior](https://github.com/restic/restic/blob/master/doc/040_backup.rst).

## 4. Shared external provider `allow-all`: rollback-safe preparation ONLY

The `allow-all` Security Group is **shared across multiple VPSs**, so **do not change its rules**. An attached, permissive security group may override a new restrictive group depending on the external provider/OpenStack attachment semantics; do not assume simply attaching a second group restricts anything.

Before any change, the owner must obtain actual external provider VNC Linux login (or an independently verified equivalent out-of-band provider recovery pathway) and confirm external provider supports **a separate dedicated security group applied only to IPAT** with the old `allow-all` assignment safely removed from IPAT *without mutating rules used by other VMs*. Export the exact current group attachments, per-VPS Managed Firewall configuration and IPv4/IPv6 rules. Prepare a narrowly scoped SSH management CIDR only if its real external source and stability have been verified. Keep public Kubernetes control-plane TCP 6443, kubelet TCP 10250, datastore TCP 2379-2380 and overlay UDP 8472/51820+ **closed**; select private networking as an explicit ADR-017 review.

**Rollback plan:** record IPAT's current group assignment and provider firewall state before change. Keep an independently working external provider recovery session. When a dedicated group is eventually tested, verify a new SSH session and external IPv4/IPv6 behavior; if validation fails, restore only IPAT's recorded original assignment, not rules of the shared group. Do not use external provider Reset Rules, which returns to global allow-all.

**Current gate:** VNC login is not working, so all provider rule changes are BLOCKED. New root-backup code is prepared, but a real privileged encrypted backup and recovery test have not run until the user provides sudo authentication locally. This is not PostgreSQL AC-07 or whole-host disaster recovery.


## R4.7.1 terminal-prompt UX correction (source-only follow-up)

An owner's Mac screenshot during the first *real* root-capture attempt showed Restic progress **0 files / 0 B** without a visible sudo prompt. Assistant inspected process metadata read-only (no secrets/TTY content): Restic had spawned a Python producer, which had not yet launched SSH, and there was no completed root-config snapshot. This is consistent with Python waiting for the local `getpass` prompt while Restic progress redraw makes the prompt difficult to see. It is **NOT evidence of root-authentication failure or successful backup**. Direct `root` SSH is intentionally disabled (`PermitRootLogin no`); the backup uses `openai` over public-key SSH and requires the *Linux sudo password for openai*, not root SSH access or any Mac key passphrase.

**First action for that still-pending Terminal:** user presses `Ctrl+C` once in the *same* Mac Terminal if the prompt is not visible, then waits for the script to return. Do not start parallel Restic jobs or send passwords to ChatGPT. This does not modify SSH, firewall or K3s; an incomplete encrypted snapshot must not be reported as a pass.

The follow-up code makes the correct password type explicit, writes a visible nonsecret instruction directly to the controlling Mac TTY immediately before `getpass`, and sets `restic --quiet backup` for the privileged backup path to avoid overwriting the local prompt with a progress indicator. This is a usability/source change only; actual sudo-protected root capture and isolated root restore remain **PENDING** until user executes the updated merged helper and provides the local sudo password. Never enable direct root SSH to work around a local sudo prompt.


## R4.8 successful privileged completion (supersedes earlier pending instructions)

The owner completed the **actual privileged** streamed selected-root-config archive. An independent follow-up used `--verify-root` on Restic snapshot `abaa9827`, re-read all 11 snapshots/20 packs, restored the selected config tar to a private temporary Mac directory and verified the expected `sudoers` member and managed key-only SSH directives. Temporary plaintext was removed. No host SSH/network/firewall change occurred. It is still not a complete VM snapshot, host private-key backup, database restore or K3s datastore recovery. Do not re-run solely because earlier chronological instructions said the root backup was pending.
