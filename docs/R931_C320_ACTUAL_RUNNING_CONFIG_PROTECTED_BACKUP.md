# R9.31 — Genuine C320 private full running configuration snapshot

Date: 2026-09-29. Owner explicitly reports a disconnected test-lab ZTE
C320, but requires ALL management adoption changes to follow the
live-device SOP. Evidence was obtained on the authorized nonroot owner
VPS from the actual private owner-directed ZTE device. No customer
service, device config, SSH-server user, ONT/firmware or ACL changes
were authorized by R9.31.

## Real physical read-only pre-change inventory/baseline

- Bound, owner-approved interactive SSH LOGIN using the prior temporary
  account and exact process-scoped RSA/aes128-CBC/group14-SHA256 profile
  SUCCESS. Exact owner-private *network-observed* known_hosts pinned.
  It is not independent physical-console host-key attestation.
- Actual `show privilege`: privilege 15. THIS IS NOT an acceptable
  permanent IPAT least-privilege service identity.
- Actual `show file cfg`: existing `startrun.dat` and `startrun.sav`
  each report size 112902 bytes; an additional startup patch file
  exists and reported free flash space. These are ON-DEVICE files;
  none establish independently recoverable off-device backup.
- Actual `show system-group` reports C320 software V2.1.0 and
  uptime about three days. Sensitive contact/location values are
  deliberately excluded from Git and all public evidence.
- Actual `show alarm counter` reports historical counters, including
  alarmReport 68 and alarmRecv 15 during this observed session.
  THESE ARE NOT a current active-alarm measurement, nor a verified
  before-and-after service-health baseline. `show alarm active` had
  previously failed on this exact firmware; do not guess alarm CLI.
- Actual `show startup-config`: syntax rejected by the real firmware
  in current privileged mode. Do NOT assume historical manual command
  support or treat a failure as missing startup storage.
- Owner-approved session-only `terminal length 0` accepted, no
  persistent config mode entered. Actual `show running-config`
  SUCCEEDED and streamed a large configuration without paging.

## Real PRIVATE actual complete running snapshot

An actual owner-only VPS session output capture was created directly
under the CURRENT USER's nonroot `0700` directory:

`/home/openai/.local/share/ipat/r931-private-olt-backup`

The output-only session transcript
`first-config-terminal.capture` and separated full snapshot
`running-config-full-owner-private.capture` are mode `0600` and
are NEVER allowed in GitHub, ChatGPT messages, PUBLIC mirror, web
LAB preview or regular logs. A separate mode-0600 receipt
`running-config-private-receipt.json` records exact byte count,
SHA-256, actual read origin and explicit UNVERIFIED recovery status.
The one-time temporary password was entered interactively and not
visible as an echoed line in the captured output. Sensitive
configuration sections have NOT been printed into documentation.

The snapshot extraction independently required the real vendor's
`Building configuration` start, a complete standalone `end` line,
the actual ZTE CLI prompt RETURNING after output and absence of
paging/error, escape bytes or a standalone echoed test password.
Actual full owner-only snapshot length: **119980 bytes**, digest:

`5b21f96b7b81dc9a771cc24e6369bc55433748b637bf0e98a3fb0a03e1989ba3`

R9.31 standalone STRICT nonroot local receipt checker:

```sh
python3 deploy/scripts/lab/r931/verify_private_backup.py \
  --owner-private-folder \
  /home/openai/.local/share/ipat/r931-private-olt-backup
```

It emits only safe digests and an explicit `NOT RESTORED` state,
never raw backup bytes. R9.31 synthetic tests confirm reject on
truncation/pager, tamper, weak file permissions/symlink and any
attempt to falsely mark a source capture device-adopted or
successfully restored. It NEVER transfers, decrypts, authenticates,
configures a device or emits restorable CLI instructions.

## Off-host encrypted backup and rollback: still OPEN

The existing owner Mac has FileVault ON, existing restic 0.19.1
encrypted local repo `~/IPAT-secure-backups/restic-lab-v1` mode 0700
and a Keychain-held non-Git backup secret; the owner previously
confirmed out-of-Mac recovery-secret escrow. The direct owner-VPS
PRIVATE full running-config snapshot transfer attempt via Remote
Desktop Commander's guarded `scp` was explicitly DENIED BY TOOL
SECURITY. This was **not** retried through an alternative covert
transfer mechanism. Thus no Mac Restic snapshot or independent
restore of THIS 119980-byte OLT configuration can be claimed.

The authorized owner must perform a separate reviewed transfer into
an independently secured backup destination through their normal
operator access, then verify SHA-256 matches the above source and
perform a protected encrypted repository full-read AND isolated
byte-identical restore test. The on-device startup files must NOT
be mistaken for an off-device disaster-recovery copy. The real
running configuration output is a READABLE MANUAL RECOVERY REFERENCE,
not proof that uploading it would correctly restore this firmware;
actual device-side restoration remains untested and should only be
rehearsed with a disposable lab control board, verified firmware
and independent approval.

Do not disable the known working SSH/Telnet break-glass paths, rotate
the only functional test account, create privileged device users,
change the management ACL, run `write`, reboot or flash firmware
UNTIL this independent encrypted backup and safe return path are
confirmed. Current private full config snapshot resides ONLY on
the current VPS, outside Git; an isolated off-host restore is OPEN.

## Closure status

- Actual authenticated read on the approved physical LAB C320: PASS.
- Real full 119980-byte owner-private running-config snapshot: PASS.
- Device privilege/storage/system/historical alarm counters: READ.
- Device-side writes or credential/account changes: ZERO.
- Encrypted separately recoverable OLT snapshot: BLOCKED.
- Exact firmware minimum-privilege account/production worker: OPEN.
- Independent physical host attestation/tenant MFA/reviewer: OPEN.
- Automatic safe SaaS `ADOPTED`: FALSE.
