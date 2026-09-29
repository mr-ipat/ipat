# R9.32 — Separate owner-executed C320 encrypted backup and restore handoff

## ACTUAL work performed, 29 September 2026

The REAL owner-VPS has a bounded private mode-0600 119980-byte
complete C320 `show running-config` CLI **transcript reference** plus
its matching SHA-256 receipt. No raw config, password, user names,
subscriber/PII metadata, physical source IP or network RSA key has
been copied into source or public build. The exact source receipt
SHA-256 is already pinned privately in the owner repository's
`deploy/scripts/lab/r932/owner_mac_restic_c320_snapshot.py`:
`5b21f96b7b81dc9a771cc24e6369bc55433748b637bf0e98a3fb0a03e1989ba3`.
This is a digest of a sensitive reference, not a device-native import.

Actual owner Mac preflight, nonsecret metadata only, PASSED:
FileVault is ON, previously verified 0700 encrypted Restic repo
`~/IPAT-secure-backups/restic-lab-v1` is present, its `config` is
read-only mode0400, the Keychain record exists, and the existing
private first-read folder is owner-only 0700. Historical owner
reported independent external escrow for the Restic recovery key;
not independently audited here. A full raw configuration transfer
from VPS to Mac attempted in R9.31 was BLOCKED BY REMOTE SECURITY,
so do NOT retry through remote access or hide the transfer.

Created an **EXPLICIT OWNER-OPERATED, human-present Mac Terminal**
script with three modes:

```sh
cd "$HOME/Projects/ipat-current"
python3 deploy/scripts/lab/r932/owner_mac_restic_c320_snapshot.py --requirements
python3 deploy/scripts/lab/r932/owner_mac_restic_c320_snapshot.py --local-readiness
```

Both above modes were ACTUALLY exercised safely from remote access;
`--requirements` is fully offline and `--local-readiness` verifies
only safe existing Mac prerequisites; neither transmits secrets,
reads full device config or creates a backup.

## One remaining OWNER Terminal operation — not run by ChatGPT

The original guard explicitly REJECTED tool-mediated transfer of
the sensitive complete running configuration. Therefore the OWNER
must execute this exact new reviewed and transparent command in
their own Mac Terminal, with unlocked login Keychain and active
trusted owner SSH connection. DO NOT run it through Remote Desktop
Commander, ChatGPT remote tools, third-party browser or CI.

```sh
cd "$HOME/Projects/ipat-current"
python3 deploy/scripts/lab/r932/owner_mac_restic_c320_snapshot.py \
  --backup-and-restore
```

The script deliberately requires a REAL interactive terminal and
exact confirmation phrase, FileVault ON, 0700 owner-only directories,
0400 pre-existing encrypted Restic repo config and existing trusted
Keychain metadata. `ssh -T` uses BatchMode, actual existing owner-VPS
host fingerprint checked with `StrictHostKeyChecking=yes`, no
interactive privileged OLT login and no remote OLT commands. An
on-VPS Python source uses EXACT expected private path and SHA-256,
owner-only mode checks, complete vendor start/end markers and the
actual original restricted capture receipt; its raw stdout is sent
DIRECTLY into the local OWNER Mac encrypted Restic pipeline as
`--stdin-from-command`, not stored as a plaintext staging file and
not printed to Terminal. No secrets are supplied as arguments or
written to repo: Restic reads its password from the existing
macOS Keychain through RESTIC_PASSWORD_COMMAND. Its one-time
encrypted snapshot uses the exact static `c320-r931-private-complete-cli`
tag and filename. The script aborts on any unexpected source hash,
remote route trust, missing repository, missing Keychain item or
preexisting immutable result receipt.

After encrypted backup, it explicitly runs `restic check --read-data`
on the Mac repository and performs an actual isolated byte-for-byte
snapshot restore inside a new temporary `0700` directory on the
FileVault-encrypted Mac. It computes restored SHA-256 and length;
ONLY if they match the independent on-VPS source digest does it
remove the plaintext temporary restored file and write the final
mode-0600 operator-local nonsecret immutable receipt:

`~/.local/share/ipat/c320-real-read-20260929/r932-verified-restic-snapshot-receipt.json`

The receipt explicitly marks exact restored readable-reference
bytes true, BUT vendor-native import test false, second geographic
backup replica false, weak account rotation false, scoped worker
false and production auto-adoption false. It MUST NOT automatically
unlock device operations. If operator assistance is needed, share
ONLY nonsecret script completion status and accepted receipt metadata;
never send the full config, password, Restic Keychain value, raw
terminal transcript or actual SSH private key to ChatGPT.

## ACTUAL tests performed, distinct from unperformed real backup

- Mac `--local-readiness` actually PASSED, explicitly reports real
  encrypted off-VPS snapshot and real isolated restore NOT RUN.
- The exact Restic stdin-from-command invocation syntax was tested
  independently in a NEW disposable local SYNTHETIC ONLY Restic
  repository with a random process-local test secret, synthetic
  fixture content, and isolated restore. Snapshot path, tag and
  round-trip bytes matched. This did NOT read a byte of the real OLT.
- R9.32 7 synthetic local guard/negative tests PASSED, including
  non-TTY real backup refused, symlink/weak owner folder refusal,
  digest-before-output rule and guaranteed deletion of failed
  synthetic restore; combined full software regression results
  separately reported in PROJECT_STATUS.md after release.

## Adoption policy gate remains distinct from backup success

Even AFTER the owner completes real encrypted off-VPS readable
reference backup, the captured CLI output includes the ZTE CLI
prompt and has **NOT** been proven to be a native device-import
configuration. To change even a no-customer test-lab chassis under
LIVE SOP: verify firmware-specific genuine backup/export and recovery
procedure, prepare a separate least-privilege user and proof of
privilege denial, rotate the weak temporary shared test password,
prove independent owner/source identity and full isolated POP route,
create a scoped non-demo production SSH read worker and verify
signed real tenant MFA+independent reviewer and immutable audit.
No blind firmware flashing, AAA/SSH server key regeneration,
Telnet disable or privileged account invalidation before safe
recovery. Existing device management sessions are CLOSED and current
working login remains available as temporary break-glass.

The ACTUAL confidential running-config was classified offline,
without exposing content: 16 user declaration lines, of which
2 included a parsed explicit level15 attribute; 14 lines did not
express a numeric level in the bounded parser. This DOES NOT prove
that any safe dedicated read-only account exists. The snapshot also
contained 4 SSH-related configuration declaration lines and 196
interface declarations; counts do not establish exact firmware
role capability. Usernames/hashed passwords/ACL contents never
entered documentation, CI or assistant response. Verify the exact
firmware role syntax before attempting an actual privileged change.
