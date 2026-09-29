# R9.33 — Actual off-VPS C320 CLI recovery receipt accepted, historic real hardware inventory exposed safely

Date: 2026-09-29. Owner attests exact DEV-01 is a disconnected LAB
OLT, but any config/change remains subject to LIVE-equivalent controls.

## EXACT independently checked REAL owner Mac backup outcome

The human owner personally executed the R9.32 operator-only command
in Mac Terminal and provided its success result. Independent
non-secret reinspection via owner Mac Remote Desktop additionally
ACTUALLY verified exact non-symlink owner-only mode-0600 local receipt
`~/.local/share/ipat/c320-real-read-20260929/r932-verified-restic-snapshot-receipt.json`.
Receipt original UTC timestamp `2026-09-29T05:49:58+00:00`.
Actual 119980-byte source SHA256
`5b21f96b7b81dc9a771cc24e6369bc55433748b637bf0e98a3fb0a03e1989ba3`
EXACTLY matches the untouched protected owner-VPS source SHA256.
Receipt explicitly confirms actual encrypted off-VPS Mac Restic backup,
actual isolated byte-identical snapshot restore and removal of the
temporary restored plaintext. INDEPENDENTLY re-queried the REAL
Mac Restic encrypted repository using existing local Keychain
password-command and located the EXACT receipt snapshot ID with
the exact `c320-r931-private-complete-cli` tag and expected capture
filename. This is no longer a merely owner-stated backup outcome.

**Scope qualification:** These checks prove that a sensitive complete
raw CLI REFERENCE can be restored byte-identically from a separate
host's encrypted Restic repository; this CLI transcript contains
an expected final prompt. This does NOT prove that it is a
vendor-native import file or that the exact firmware can safely
perform a config restore. A second geographically isolated independent
backup replica is also NOT proven. Store the snapshot ID in the
existing OWNER-only 0600 receipt and NEVER reveal backups, keys,
exact user lists, shared ISP credential contents or privileged CLI
output in Git/PR/public mirror/browser. Existing internal digest may
appear in controlled technical documentation but not the raw bytes.

## Additional actual source-device read-only controls

One new limited actual C320 SSH session with the original owner-
authorized temporary test account succeeded using the existing
exact protected VPS network-observed RSA pin, process-scoped legacy
SSH compatibility and STRICT known-host checking. The actual
`show username` table returned TWO entries with explicit privilege
15. This result does NOT guarantee there are only two internal
accounts in the configuration; a bounded review of 16 declarative
`username` lines showed mixed syntax, and NO separate restricted
read-only service identity has been demonstrated.

Actual `show file ?` showed read-only file-directory subcommands,
not a proven vendor-native export/copy or restorable transfer. The
firmware's `show alarm ?` advertised `crtv-active`; real read-only
`show alarm crtv-active` was accepted, BUT its output has not been
parsed/corroborated against device alarm semantics. Do NOT mark
actual alarm health clean or turn on alarm polling from this alone.
No actual account/user/config changes, flash, restart, key generation
or vendor-native import/export were executed in R9.33. A left-behind
interactive SSH process was safely terminated by exact approved
owner-VPS PID, and subsequent `ss` showed ZERO remaining established
connections to the owner-target OLT.

**Important firmware-specific user-role nuance:** public vendor
references for C300/C350/C320 V2.1.0 describe 16 command privilege
levels but do NOT independently establish that a newly created
low-privilege operator is incapable of entering higher-privilege mode
or that `show card` works at a specific low privilege for this exact
unit. Before a new IPAT account or an unattended read worker,
verify its role/CLI deny properties on the owner-approved firmware,
use strong external secrets, real independent approvals and retain
verified rollback and current recovery access. Never auto-persist
the temporary default level15 password as service credential.

## Product implementation

- The PRIVATE local :3002 Axum LAB catalog now correctly includes the
  proven encrypted off-VPS reference backup and byte-identical
  Restic restore with exact UTC historical timestamp, plus explicit
  FALSE native-vendor restore, FALSE restricted identity,
  FALSE qualified future production automatic adoption.
- Created NEW private `GET /lab/c320-first-real-inventory` which
  displays only sanitized *ACTUAL HISTORICAL* board slot/type/status
  and firmware correlation drawn from the real first authenticated
  physical C320 SSH and Telnet read. It explicitly preserves
  1/1/1 `GTGHK` vs `GTXK` unresolved, 1/1/3 `PRAM` missing running
  MVR, 1/1/4 `SMXA` exact MVR match. Every response is no-store;
  never serial/IP, user passwords, running config, customer data,
  network operations or live health claims.
- Dedicated `web/lab/c320-first-real-inventory.js` validates exact
  historical server response fail-closed and renders the three
  actual historical cards in their own LAB Device Manager section.
  Old misleading September 28 no-auth/timeout-only dashboard prose
  has been corrected to the subsequent authenticated events and
  independent Mac Restic recovery, without implying the automatic
  SaaS adoption worker is active. Eight existing physical POST
  device actions MUST remain HTTP403. Non-LAB routes still deny.
- R9.33 dedicated synthetic metadata/UI tests and full regression,
  pinned Rust private owner-VPS complete workspace tests and real
  localhost :3002 redeployment to be recorded only after execution.

## Remaining adoption closeout (independent blockers, not the earlier resolved backup)

1. Genuine firmware-aware vendor-native export/import validation and
   a safe control-plane recovery drill (ideally spare verified SMXA),
   without restarting or disturbing the LAB device just for a checkbox.
2. Genuine independent physical host identity assurance or an
   approved narrowly scoped owner-attested lab trust exception,
   actual management last-hop isolation and actual role denial tests.
3. Newly generated dedicated device identity with proven least
   privilege, secret in external vault, verify read success and
   disallowed config writes; rotate temporary widely shared test
   password only after replacement + owner break-glass are proven.
4. An actual tenant-bound independent owner/reviewer MFA decision,
   per-device audited bounded SSH read worker with immutable scope,
   structured real observations and stop/abort/rollback.
5. Only then transition to production `ADOPTED_READ_ONLY`; do not
   let successful historic `show card` or the encrypted CLI backup
   falsely claim continuous current telemetry or firmware support.

## ACTUAL R9.33 software verification and LIVE PRIVATE LAB preview upgrade

Exact core source `10061dff2d9d48656ca7419b6fb774a76437b2a3`
was transferred as checksum-verified private Git bundle
`9817b47131f07c0c8c7a8456e94addfc65765a061483dd3da6ef4e4074faa67b`
into a DISTINCT approved nonroot owner-VPS checkout. Actual
owner VPS combined Python security suite **91/91 PASS**; Mac
same 91/91 PASS and Node.js syntax PASS for both original
workbench and NEW historic actual inventory script. Rust pinned
1.98.1 workspace formatting PASS; ALL **221/221 locked offline
Rust workspace tests passed across 40 groups**, with zero failures;
actual protected Rust `control-api` binary SHA256
`13380ec652c2259bd721a497205e907b7e4a89f256479ab8e811c4c1a85cc12e`.

ACTUALLY checksum-pinned replaced ONLY nonroot owner localhost
PRIVATE LAB :3002 `ipat-r911-preview.service` with R9.33 Rust binary
and validated source. New private user service unit SHA256
`5ab0d992328c68ad6c339f860fa03db325294b22c18489d06cdf5523e4456ede`,
actual HTTP smoke SHA256
`8e53d1cd7b25d4b88d984d4717433797602ba889b03425a02015dda03c6174d3`,
checksum-pinned deploy script SHA256
`11b881764ae8f52b7ce9c43559a68ea0c7a13220fb91b5e86e252f1270eaf2c2`.
R9.33 ROLLBACK prior private unit saved before replacement at:
`/home/openai/.cache/ipat/r933-release/rollback-user-unit.service`.
The script explicitly rolls back ONLY the private LAB unit on any
failure; it performs ZERO OLT commands, ZERO config writes and does
not alter public firewall, K3s, original :3000 service or production
PostgreSQL.

ACTUAL independent after-deploy LOCALHOST :3002 HTTP smoke PASS,
including new restricted historical GET and JavaScript asset,
actual card/version metadata, actual user/backup flags, explicit
FALSE independent physical identity/native recovery/worker adoption,
all tested physical POST action routes HTTP403, original :3000
HTTP200. The new frontend displays the three actual historical
card slots; it does NOT perform live OLT polling. Actual owner-VPS
socket inspection after all manual CLI work showed ZERO open
established connections to target OLT.

To revert ONLY the PRIVATE LAB preview user service:

```sh
ssh ipat-lab
install -m 0600 /home/openai/.cache/ipat/r933-release/rollback-user-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
```
