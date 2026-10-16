# R10.36 — Privileged recovery evidence and root release custody

Date: 16 October 2026 (Asia/Jakarta). Reviewer: authorized owner interactive VPS session. This report is **public-safe** and contains no credentials, device raw CLI, or host backup payload.

## Actually observed as root

- Interactive sudo returned UID 0. Exact reviewed commit and binary checksum on owner VPS passed the existing R10.35 read-only root checks.
- No current commercial database cluster/service or listeners on production ports; owner-private ZTE C320 remains running and physically read-only.
- Missing at the time of root check: `/root/ipat-evidence`, `/root/ipat-evidence/r996-recovery-attestation.json`, `/var/backups/ipat`, `/root/backups`, `/root/.config/restic`. Restricted filename-only search under /root, /var/backups, /mnt and /media found no matching root host restore reports. This is evidence of the **production host's inspected locations only**; backup or restore evidence might exist in a different system.

## Exact release custody

- Review commit: `faf0b25e6f2456820c6a088719b26d0807fc0154`.
- Executable SHA256: `7e041f843668e77d46c89c2fb7cc61972b794af12f3b674beb284cba3ecf91cd`.
- Initial root-only clone from the long-lived local Git repository failed closed because that source advertised no reachable detached commit. No PostgreSQL install or commercial deployment occurred.
- A nonroot clone from the **exact clean detached** `/home/openai/ipat-r1034-merged` source was separately tested to contain the commit and allowed successful checkout. The root-only v2 handoff script uses `git clone --no-local --no-checkout`, `git fsck --full --no-reflogs`, exact HEAD and clean working-tree checks including ignored files, all-inode root ownership/permissions, and SHA256 before marking the candidate accepted.
- Test suite `python3 -m unittest deploy/scripts/production/test_r1036_root_release_handoff_source.py -v`: 4/4 PASS on the Mac, including nonroot refusal and no installed-service side effects.
- **Actual root-owned v2 handoff is not verified complete yet.** It has been staged as `/home/openai/.local/share/ipat/release-operations/r1036-root-source-handoff.sh` on VPS and the interactive authorized Mac Terminal is awaiting local sudo entry. Only once it prints `R1036_ROOT_OWNED_RELEASE_HANDOFF=PASS` and is independently inspected may the root source/binary be called staged.

## R9.96 production activation hard gate

The root-owned release handoff is *not* permission to apply R9.96. The existing recovery validator demands an actual recent, independently reviewed `ipat.recovery-readiness.v1` attestation whose source commit matches the intended production release, with verified provider rescue login, independent backup destination, and full-host restore on a distinct VPS/VM. Do **not** set these flags to true based on a backup command, source test, VM simulator, old C320 configuration receipt, or an unsupported statement. Inspect independently verifiable recovery artifacts and only then authorize and execute the R9.96 production first-install procedure.

**Production status:** PostgreSQL18 foundation NOT INSTALLED; production PITR/HA, tenant human MFA, custom-domain TLS/Host isolation, public dashboard, and physical provisioning not verified. PUBLIC_GO=FALSE. The architecture, security gates and PRD remain unchanged.
