# R9.30 — First REAL owner-authorized ZTE C320 lab login/read, 29 September 2026

## Explicit owner scope and change controls

The owner clarified that this exact C320 is an OFFLINE TEST LAB with
NO attached live subscribers, but requires treating ALL adoption
changes as if this were a customer-serving live distribution OLT.
This **supersedes the earlier site-impact assumption for this device
only**. It does NOT loosen IPAT's future production adoption policy
or authorize blanket server-side SSH, ONT, PON, firmware, network or
credential configuration writes.

Temporary factory-like account credentials were explicitly furnished
in the active chat solely for lab first-access. NEVER store them in
source, scripts, CLI arguments, log files, ChatGPT artifacts or the
public synthetic mirror. Rotate/revoke the weak default test
credential after a validated restricted replacement account, secure
configuration backup and working break-glass console are confirmed.

## ACTUALLY performed on real private ZTE lab hardware

Owner Mac Remote Desktop to approved nonroot owner VPS, then **one
owner-approved interactive private TCP323 Telnet login succeeded**.
Real OLT emitted a `Welcome to ZXAN product C320` banner, accepted
the owner-supplied temporary test login, reported weak-password
warning, presented a privileged CLI prompt. Three successful
read-only CLI commands (`show card`, `show version-running`, `show ssh`)
returned REAL output. One proposed alternate read-only command
`show alarm active` returned a vendor syntax error; do NOT consider
active alarms measured or retry guessed vendor commands in batch.
No OLT configuration command or firmware operation was executed.
The Telnet session terminated before switching transport.

A fresh owner-VPS SSH **network-only TOFU** RSA public key was then
observed in a protected 0700 private directory and its fingerprint
matched the previously observed same management endpoint from Mac
and VPS; **there is still NO independent out-of-band physical RSA
attestation**. With exact session-scoped SSH compatibility
(`ssh-rsa` host key, `aes128-cbc`, `diffie-hellman-group14-sha256`),
`StrictHostKeyChecking=yes`, exact protected network-observed 0600
known_hosts and user-approved temporary login, the ACTUAL private
OLT SSH session successfully authenticated to ZXAN C320. Two
independent *encrypted-session* read-only commands (`show card` and
`show version-running`) returned the same three slots and five
running-version records as the earlier Telnet session. The SSH
session explicitly exited; no unattended default-credential session
or persisted test password was left running.

**Note**: matching network-only SSH fingerprints and the matching
SSH/Telnet readouts give repeatable LAB endpoint evidence, NOT
cryptographically independently verified physical chassis ownership.
The owner asserts a disconnected test OLT; POP isolation and the
full true production trust chain remain unchecked.

## REAL observed board/firmware metadata (redacted, no credentials)

| Slot | Configured | Physical card RealType | Card status | Firmware report status |
|---|---|---|---|---|
| 1/1/1 | GTGH | GTGHK | INSERVICE | `show version-running` reports `GTXK` MVR V2.1.0, BT V4.0.16 on SAME SLOT. `GTXK` versus `GTGHK` vendor mapping **UNVERIFIED**. |
| 1/1/3 | PRAM | PRAM | INSERVICE | NO MVR version row in observed `show version-running`. Card `show card` SoftVer V1.01; do NOT claim its running firmware verified. |
| 1/1/4 | SMXA | SMXA | INSERVICE | Matching `SMXA` MVR V2.1.0, BT V4.0.13, FW V2.1.0. |

Real `show ssh` returned server enabled, ver2.0, local auth, CHAP,
with historical `SSH init server key : not initialized` wording.
Because actual SSH RSA packet was observed AND encrypted SSH login
succeeded, **do not regenerate server keys, change SSH daemon crypto,
reboot or alter PON configuration based on that vendor field**.
The first offered SSH client account auth method `password` is now
actually shown to work in this explicitly owner-authorized lab.

## Owner-private provenance and normalizer

The first outputs were visible live through the authorized interactive
remote terminal, and manually copied WITHOUT secrets/PII into owner
Mac FileVault folder:

`~/.local/share/ipat/c320-real-read-20260929/`

with `cards.txt`, `versions.txt`, `ssh-status.txt` and
`capture-provenance.json`, each mode 0600 under owner-only folder
0700. **Manual transcription != byte-exact device raw capture**.
They were then transferred over the authenticated owner Mac↔VPS SSH
channel into a separate nonroot 0700 owner VPS folder
`/home/openai/.local/share/ipat/r930-real-read-20260929`, files mode
0600. No actual device password/SSH private key/transcript is in Git.

The already-compiled protected Rust `olt-evidence --cards` ran
ACTUALLY against this user-captured first real card evidence on the
owner VPS, emitted a separate owner-private 0600 structured
`normalized/actual-c320-first-cards.json` and verified THREE distinct
`INSERVICE` cards. Its output **correctly** leaves independent
chassis identity, fully matched firmware, authorizations and
production adoption FALSE. The existing strict paired card+version
parser MUST NOT be loosened to silently call GTGHK==GTXK or
fabricate a missing PRAM MVR. R9.30 adds separately tested
slot-based partial version reconciliation with explicit unresolved
alias/missing MVR lists; it never generates an OLT command or
production adoption status.

## Product stage and next real controls

Private owner-only LAB dashboard now reports:
`AUTHENTICATED_LAB_READ_OBSERVED_ADOPTION_PENDING`, the three
actual INSERVICE cards, five version rows and both successfully
tried manual transports. Browser remains read-only; its eight
physical action POST routes MUST continue returning 403 and its
production worker MUST stay disabled. Original production
application port :3000 remains unaffected by any R9.30 preview.

Before unattended IPAT tenant adoption, owner needs an actual
firmware-supported *nondefault least-privilege* device account,
secret manager external to Git, verified scoped SSH host key identity
or explicitly approved lab-only equivalent, a bounded process
per-device read worker with retry/rate limits, actual signed tenant
MFA and independent approval where required, audit, baseline and
recoverable secure configuration backup. A privileged default test
session is NOT a production worker identity. Telnet may be retired
after restricted protected SSH is proven. No actual operator wrote
OLT configurations, changed SSH settings, created a user, rotated a
password or transferred firmware in this milestone.

## Strict private CLI for the actual PARTIAL first firmware capture

The original strict complete-firmware `olt-evidence --cards --versions`
MUST continue rejecting this observed firmware tuple until exact
firmware-specific vendor mapping has been proven. R9.30 therefore
adds a deliberately separate `--first-observation-partial-versions`
**OFFLINE OWNER-PRIVATE** mode to the same strict Rust parser. It
reads separately captured owner-only 0600 card/version files from
0700 non-Git folders, validates vendor clock format including real
single-digit hours, validates that ALL version records belong to
actual card slots and have one consistent per-slot FileType, then
reports an `exact_mvr_slots`, `unresolved_mvr_filetype` and
`no_mvr_reported_slots` structured summary. This does not whitelist
`GTGHK -> GTXK` globally, invent PRAM firmware, log raw CLI, send
packets or confer adoption/firmware-write rights.

On the approved nonroot owner VPS, after independently reviewing the
private transcribed capture provenance:

```sh
p="$HOME/.local/share/ipat/r930-real-read-20260929"
b="${HOME}/.cache/ipat/r930-release/target/debug/olt-evidence"
"$b" --first-observation-partial-versions \
  --cards "$p/cards.txt" --versions "$p/versions.txt" \
  --out "$p/normalized/actual-c320-partial-versions.json"
```

That private normalized output is a first-read acceptance input,
NOT a signed asset/tenant verification. Never copy it into a public
mirror or claim complete firmware equivalence from slot correlation.
