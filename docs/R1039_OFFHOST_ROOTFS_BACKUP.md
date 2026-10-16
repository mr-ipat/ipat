# R10.39 — Encrypted host filesystem backup and independent restore gate

**Date:** 16 October 2026 (WIB). This is a production-recovery milestone, not proof of commercial readiness.

## Real environment

The owner VPS is Ubuntu Server 26.04.1. `/` is ext4 on `/dev/sda3`, and `/boot/efi` is a separate vfat filesystem on `/dev/sda2`. PostgreSQL production remains inactive. Root-owned release `a01811457c45d1be4bef959d34f691f1636ea87b` and binary SHA256 `7e041f843668e77d46c89c2fb7cc61972b794af12f3b674beb284cba3ecf91cd` were independently inspected in R10.37.

Mac backup repository `~/Backups/IPAT/R1039-host-recovery-current` was created with local FileVault-backed storage and an existing Mac Keychain-managed Restic password command. Encrypted repository initialization, empty-repository integrity check and synthetic stdin encrypted snapshot writing passed. The older `R1037-host-20261016` encrypted repository is unchanged; the documented Keychain password does not decrypt that earlier repository, and the selected files restored previously are not evidence of a whole-VPS boot restore.

## Live backup procedure

`python3 deploy/scripts/production/r1039/backup_host_to_mac_restic.py --repo "$HOME/Backups/IPAT/R1039-host-recovery-current" --execute`

The Mac process reads the temporary VPS sudo credential through a no-echo stdin prompt, never persists it, and uses known-host SSH public-key verification with `sudo -S -- tar`. A binary stdout tar stream is written directly to encrypted Restic on the Mac, with a running SHA256 and zero plaintext tar on disk. The rootfs archive is followed by a separate EFI archive. On the VPS this is strictly filesystem READ; no service, network, PostgreSQL, firmware or firewall changes are made.

Archives include persistent root data, `/etc`, `/root`, `/var/lib`, application data and source/workspaces; regenerate ephemeral mounts, caches, build outputs, swap, volatile logs and empty mountpoints using documented OS provisioning. Explicit exclusions are in `ROOT_EXCLUDES` in the source. This is a filesystem archive, **not** a bootable raw disk image.

## Actual implementation and security tests

`deploy/scripts/production/r1039/backup_host_to_mac_restic.py`: fixed target SSH alias, checksum and streamed encrypted transfer; verifies both tar and Restic status before reporting success. The initial Mac transport failed without producing a snapshot because the remote SSH command discarded the empty `sudo -p` argument. The corrected `sudo -S --` invocation was independently tested to return UID 0 over non-PTY SSH. Five source security tests cover no secrets/mutation commands, fixed excludes and separate EFI, Keychain/local repository restrictions, and no false claims of boot recovery or public GO.

## Still required to authorize R9.96

A completed encrypted rootfs+EFI snapshot set, Restic read-data integrity and an actual **independent booted VPS/VM** restore are separate acceptance criteria. Verify bootloader/EFI, root ownership/xattrs/ACLs, root-access recovery, network, SSH, expected systemd state and representative file hashes. Attach safe receipts to the privileged R9.96 attestation bound to the immutable root-owned release. Mac storage has ~32 GiB free, so do not assume a full additional VM restore will fit locally; use an independently provisioned VM/VPS with adequate disk. Do not substitute successful tar upload, root-only source custody, a PostgreSQL disposable lab, or a selected-file restore for an independent boot recovery.

**PUBLIC_GO remains FALSE until actual restore/PITR/HA/MFA/TLS tenant acceptance.**
## Verified encrypted host snapshot pair — actual evidence (16 October 2026)

The separately hosted owner-Mac Restic repository `~/Backups/IPAT/R1039-host-recovery-current` was opened using its previously configured Mac Keychain command without exposing the key or the password. The latest completed rootfs snapshot ID `2d767d09` contains the single streamed archive `ipat-r1039-rootfs.tar` (31,759,953,920 logical bytes). The completed EFI snapshot `af1a0129` contains the separate `ipat-r1039-efi.tar` (9,000,960 logical bytes). Both have the `ipat-r1039-host-backup` tag.

**Actually executed** Restic `check --read-data` over the complete encrypted repository; exit code 0 — **PASS**. This verifies Restic pack/decryption/integrity, not the recoverability of the tar contents, metadata completeness, or bootability. The rootfs is a streamed tar with explicit ephemeral/cache exclusions, not a bit-for-bit disk image. Existing Mac has approximately 19 GiB remaining; do not try to restore the ~31.76 GiB rootfs tar plus a bootable VM into this filesystem without additional dedicated capacity. **Independent booted VPS/VM restore NOT VERIFIED.**

The R9.96 recovery attestation requires successful independent boot/root login, reconstructed EFI/bootloader, file ownership/xattrs/ACLs, service checks and independent backup-storage review before it can honestly be issued. These results are a real backup milestone only, **NOT public production GO**.

### Additional no-plaintext archive validation (2026-10-16 WIB)

Using the already authenticated Mac Keychain-backed Restic repository, independently streamed both encrypted snapshots through `restic dump` into `tar -tf -`, with archive path output discarded rather than copied to files or logs. Both the EFI tar `af1a0129` and full rootfs tar `2d767d09` returned status 0 at **both** the Restic decryption and tar-structure validation processes. `R1039_ENCRYPTED_RESTIC_DUMP_AND_TAR_STRUCTURE=PASS` (actual elapsed 39.57 seconds). This is stronger evidence of encrypted archive readability and tar format completeness, but still **not** a booted independent VM/VPS restore or acceptance of root access/EFI bootloader/device-specific networking on recovered hardware.
