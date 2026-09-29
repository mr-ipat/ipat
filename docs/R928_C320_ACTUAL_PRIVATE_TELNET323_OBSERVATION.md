# R9.28 — Actual owner VPS private C320 TCP323 Telnet transport, 29 September 2026

User expressly requested a TEMPORARY alternate Telnet path to the
specified PRIVATE ZTE C320 management device. One single actual
owner-authorized nonroot VPS direct private TCP session using the
pre-existing strictly passive R9.0 `probe()` function succeeded:

- Actual private TCP323 connection: TRUE.
- Actual first received protocol sample: 15 inbound bytes.
- Telnet IAC negotiation marker (`0xff`) observed: TRUE.
- Client Telnet option responses sent: ZERO.
- Login username/password, terminal commands and device config writes
  sent: ZERO.
- Server physical identity: NOT independently verified.
- Actual C320 authenticated session, firmware/card capture: NOT RUN.

The observed bytes were used only in volatile memory to classify
protocol negotiation. No raw Telnet banner or user data was logged,
saved, exposed in the dashboard or pushed to CI. This validates a
REAL additional transport route, NOT a trusted terminal or a
functioning administrator account. TCP reachability and IAC bytes
alone do not prove that the far end is the approved physical chassis.

Telnet transports password and CLI data in cleartext and has no SSH
host-key cryptographic authentication. Consequently IPAT MUST NOT
send privileged or factory passwords over this alternate path on
subscriber-serving OLT without independently verified trusted POP
management isolation, a dedicated minimal-role credential, an
approved temporary risk exception/maintenance, measured baseline
and console recovery. Prefer previously demonstrated working SSH
RSA/aes128-CBC/group14-SHA256 host-key transport once actual RSA
source provenance is verified. No plaintext password is accepted
in automation args, repository, logs, Python files or CI.

New `deploy/scripts/lab/r928/private_telnet_323.py` is a one-target
nonroot owner-VPS noauth passive capability check, explicitly opt-in
only with `IPAT_R928_APPROVE_ONE_NOAUTH_PRIVATE_TELNET323=YES`.
Its `--requirements` mode is offline. Its `--one-passive-check`
mode opens exactly ONE direct TCP socket, reads maximum 512 bytes,
never writes even Telnet negotiation responses, prints ONLY sanitized
Boolean/protocol counts and exits. No login mode or OLT command mode
exists. The private Rust LAB readiness endpoint now reports the
actual observed alternate transport and retains all EIGHT hardware
actions locked (real POST rejects 403), health NOT_MEASURED and
`OBSERVED_NOT_ADOPTED`. Port 323 must not be globally opened or
exposed to public networks because of this discovery.

## Actual remaining adoption dependencies

The owner Mac's existing private physical packet still contains only
`plan.json`, whose independently verified chassis/limited account/
POP isolation/backup gate booleans remain false in the local
serialized plan. No authenticated owner supplied physical-console
proof is currently mounted. Complete the genuine on-site host RSA
(or a distinct, independently authenticated management-console
chassis identity for an approved isolated Telnet maintenance
session), verify the actual firmware privilege and non-factory
restricted account, record baseline and reviewer, then one bounded
first REAL `show card` read under the appropriate approved
firmware-specific console protocol. A Telnet server IAC observation
cannot replace those distinct identity and authorization proofs.

**Actual physical OLT configuration commands executed in R9.28: 0.**
**Authenticated physical CLI commands executed in R9.28: 0.**
**Credentials sent in R9.28: 0.**
