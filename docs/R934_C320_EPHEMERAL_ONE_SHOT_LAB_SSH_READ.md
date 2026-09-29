# R9.34 — First explicitly coded owner-lab one-shot SSH C320 read adapter

## Purpose and STRICT non-production trust scope

The owner repeatedly requires the physically attached TEST LAB ZTE
C320 to be actually integrated while every management change is
controlled as if the unit is LIVE. R9.30 established genuine
interactive real Telnet and SSH reads, and R9.33 establishes
independently checked encrypted off-VPS recovery of the full
OWNER-PRIVATE running-config CLI reference. This is not vendor-native
import or sufficient commercial production disaster recovery.
R9.33 private :3002 Rust LAB now displays three true historical
card slots with real-source provenance; it does NOT yet poll.

R9.34 introduces the FIRST bounded standalone LAB transport adapter
that actually runs one explicit known-supported `show card` over
encrypted SSH (not a pasted transcript) only from the existing
nonroot owner VPS using its exact prior owner-network-observed
0600 RSA pin and one ephemeral password supplied interactively at
TTY. It is a LAB proof-of-transport only, NOT a K3s production
worker, NOT a POP-isolation assertion, NOT independent chassis RSA
verification or a privileged default credential daemon.
The executable absolutely refuses cron, noninteractive runs,
no explicit one-time owner opt-in, disabled strict host checking,
untrusted/missing pin, root, missing offline Rust normalizer, or
unknown card layout. It does not import a private SSH key,
store or print the human-entered temporary TEST password, create
local device accounts, send a firmware/config write or run a loop.

Source path:
`deploy/scripts/lab/r934/one_manual_c320_ssh_read.py`

- `--requirements`: NO NETWORK, safe offline machine-readable status.
- `--one-manual-lab-read`: one nonroot human owner-VPS terminal action
  with explicit prompt phrase, protected interactive getpass and
  process-scoped strict RSA/AES128-CBC/group14-SHA256 compatibility.
- ONE read-only CLI `show card`, bounded to <=65536 bytes. On accepted
  prompt/result, saves ONLY command response in a new 0700 private
  folder, source 0600. Passes it to the already-compiled protected
  Rust `olt-evidence --cards` for three exact expected physical
  slots and status, and creates a 0600 redacted JSON audit receipt
  with actual time/SHA256, no raw output printed.
- Exact vendor slot mismatch/timeout/auth failure fails closed,
  NO promotion to `device_adopted`, NO worker enabled. A final
  `exit` and forced process cleanup in `finally` close the SSH
  connection, even when validation fails.

ONLY after reviewing code and owner-specific LAB policy, the
owner-VPS operator can run from their interactive terminal:

```sh
cd /home/openai/.cache/ipat/r934-release/src
python3 deploy/scripts/lab/r934/one_manual_c320_ssh_read.py --requirements
IPAT_R934_APPROVE_EPHEMERAL_OWNER_LAB_READ=YES \
  python3 deploy/scripts/lab/r934/one_manual_c320_ssh_read.py \
  --one-manual-lab-read
```

Unlike an actual audited production tenant service, this uses the
owner's already-allowed TEMPORARY privilege15 testing login. The
human test password is entered once interactively and never
persisted in code or the private audit. Rotate it once a scoped
replacement and native recovery are independently proved. Never
schedule this adapter, expose it as production API POST, or add
fallback Telnet. Production must use a new verified limited-role
identity, external vault, independent OIDC MFA reviewer, per-device
lease, immutable audit and physical POP/host provenance.

**Evidence:** all synthetic test cases, owner-VPS actual hardware
run (if approved), SHA and read results are documented in
PROJECT_STATUS.md only after the tools actually verify them. No
claim of hardware success based on synthetic tests alone.

## ACTUALLY RUN successful owner-VPS REAL hardware adapter

The complete exact R9.34 first transport code was committed as
`3afe20bd7fd2edf86637cf147f29df877110f0f8`, securely moved by
SHA256-checked source-only Git delta
`cd230d0fc5cf487e422c0000c6a7b4f8aafc9c3f53f3309565a8c4c1608f58cb`
into a SEPARATE nonroot owner-VPS checkout. Exact source static
synthetic dedicated test suite **5/5 PASS** and combined 96/96
security suite PASS on owner Mac + VPS before any hardware use.

The OWNER-approved one-shot code was then ACTUALLY RUN from an
interactive owner-VPS session, with the explicit LAB-only phrase
and temporary owner-authorized password entered through getpass
into short-lived process memory ONLY. The exact strict pinned
private AES128-CBC/group14-SHA256/RSA SSH session SUCCEEDED against
the existing approved physical C320. The Python adapter sent
ONLY one `show card` command and bounded/reviewed the complete
CLI prompt/table return. It then passed the REAL fresh board output
to the locally built protected Rust `olt-evidence` OFFLINE parser;
exactly **3/3 hardware slots matched the prior independent manual
snapshot**, and all three returned INSERVICE at the moment of
this scripted read. No production SaaS automatic polling occurred.

ACTUAL separate owner-VPS private `0700` folder created:
`/home/openai/.local/share/ipat/r934-one-read-*`.
It holds `cards.txt`, `normalized.json` and
`manual-one-read-result.json`, each owner 0600, *not* in Git, CI,
public mirror, browser nor ChatGPT. Independent nonroot reread of
all private files verified real card SHA256
`5dc6aebaa162de7899fdec974377e1b9631647bdbab3b2b1502e20ce6810a722`,
audit timestamp `2026-09-29T06:10:34+00:00` and exact normalized
3-slot output in order 1/1/1 GTGHK, 1/1/3 PRAM, 1/1/4 SMXA, all
INSERVICE. Explicit proof: zero config commands, no persisted
password, owner network-observed RSA (still NOT independently OOB),
`production_worker_enabled=false` and `device_auto_adopted=false`.
Independent actual owner-VPS socket inspection immediately after
found **ZERO established management connections** to actual C320.

PRIVATE R9.34 Rust :3002 LAB static historical readiness and
separate historic inventory JSON now additionally contain the TRUE
actual latest operator-interactive one-shot adapter proof, without
claiming background/production worker or exposing private audit.
Still no true dedicated restricted service identity, no tested
vendor-native startup import/recovery drill, no OOB chassis key or
independently audited actual production tenant MFA+reviewer.

**Do NOT label this `ADOPTED_READ_ONLY` for commercial SaaS:**
that status requires a real signed tenant+POP-scoped independently
reviewed bounded worker with actual external secret manager and
negative device role permission tests, none of which are made true
by one manually approved script execution.
