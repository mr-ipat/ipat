# R9.0 — First owner-authorized physical OLT network contact (NO LOGIN)

**Date:** 2026-09-28 Asia/Jakarta.
**Status:** actual public transport checked only; **NO physical OLT adoption**.
**Scope:** candidate DEV-01 reported by the owner to be ZTE C320.
Neither the public TCP listener nor its unauthenticated text independently
proves device identity, exact model, board or firmware. This does NOT pass
TC-OLT-01 or authorize any OLT management action.

## Observations actually measured, not a compatibility claim

The owner authorized one exact numeric public destination on an alternative
Telnet TCP port. The exact public address is intentionally NOT committed,
rendered in customer dashboards, or recorded in the sanitized report.

- **Owner Mac, first limited TCP probe:** connected to the exact approved
  address and port; approximate first sample 87 ms; received **15 bytes**
  beginning with actual Telnet IAC negotiations. No credentials or CLI.
- **Owner Mac, separately limited Telnet IAC reply test:** only protocol
  refusals (no printable text/login/commands) were sent in response to
  five server negotiations. A further server-supplied response contained
  the string **ZTE**, but it was UNAUTHENTICATED and might represent a
  forwarded service/banner, so exact ZTE hardware/C320 is **UNVERIFIED**.
- **Owner Mac, repeatable checked R9.0 strict no-auth script at
  2026-09-28T02:51:40Z:** TCP=reachable, initial Telnet IAC=true,
  15 inbound bytes, **ZERO transmitted bytes**, no authentication.
  Sanitized JSON evidence stored outside Git in a FileVault-backed
  owner 0700 folder as a 0600, single-link JSON file; raw peer
  banner, target address and credentials are not in the file.
- **Actual nonroot Ubuntu 26.04 VPS, checked R9.0 script at
  2026-09-28T02:52:40Z:** identical approved destination TCP=TIMEOUT
  after one connection attempt. No transmitted application bytes.
  This is a source-route/ACL reachability blocker; no conclusion
  about which ISP, provider filter, firewall, NAT or device caused it.

**Do not reinterpret the dashboard:** static R9.0 notice includes a
date and says *historical network evidence, not live telemetry*. The
demo shows **zero** real enrolled devices. The physical target's
connectivity stays UNKNOWN, health NOT_MEASURED, model and firmware
UNVERIFIED, and physical device interoperability remains NOT RUN.
Do NOT treat TCP connect or an untrusted ZTE string as "OLT online".

## Implemented reviewed operator-only preflight

Source: `deploy/scripts/lab/r90/olt-telnet-network-preflight.py`.
Checks exact operator-provided **numeric global IPv4 + TCP321 only**,
explicit once-per-execution nonroot opt-in, one TCP connection and a
maximum 512-byte passive receive. It sends **ZERO application or
Telnet bytes**, never asks for, transmits or stores credentials,
never executes device commands, never logs raw banner, and never
updates inventory/approval/firmware or a real customer database.
No DNS lookup, subnets, discovery, multi-port probe or retry loop.

It returns JSON with only the observed TCP/IAC status, timestamp,
declared source label and explicit negative trust/health fields.
An optional sanitised evidence export requires a pre-existing
owner-controlled 0700 directory outside Git and creates an exclusive
0600 report; it intentionally excludes raw public IP and text.
A detected IAC header can identify Telnet negotiation, **not**
an authenticated ZTE/C320 device or a safe login channel.
The manual untrusted ZTE banner observation is tracked only as an
operator-run historical note, not automatically treated as identity.

Inspect its requirements without making any connection:

```sh
python3 deploy/scripts/lab/r90/olt-telnet-network-preflight.py --requirements
python3 -m unittest discover deploy/scripts/lab/r90 -p test_r90_preflight.py -v
```

After independently authorizing the *exact owned endpoint* and before
touching the physical OLT, the owner MAY repeat ONE credential-free
check on a nonroot authorized Mac. Supply the actual public IP via
the local terminal only; never commit it as application config:

```sh
umask 077
mkdir -p "$HOME/IPAT-secure-backups/olt-network-noauth-evidence"
chmod 0700 "$HOME/IPAT-secure-backups/olt-network-noauth-evidence"
IPAT_R90_APPROVE_SINGLE_NOAUTH_TELNET_PREFLIGHT=YES \
  python3 deploy/scripts/lab/r90/olt-telnet-network-preflight.py \
  --preflight --target-ipv4 "$OPERATOR_APPROVED_OLT_IP" \
  --port 321 --source-label owner-mac \
  --report-dir "$HOME/IPAT-secure-backups/olt-network-noauth-evidence"
```

The exact IP is NOT a secret credential, but must stay out of public
front-end code and sanitized evidence. The command does NOT ask for
username, password, firmware image or operator elevated rights.
Never run it in GitHub CI against the actual owner address.
CI uses only local mock sockets/no-outbound fake connections:
seven independent input, trust, file-mode, one-target/zero-send
and dashboard wording tests; full repository CI retains existing
actual disposable Postgres/K3s and Rust workspace gates.

## Next physical-management acceptance gates, in priority order

1. **Secure the path:** prefer independently verified vendor-supported
   SSH or SNMPv3 through a site-controlled private management address.
   If only Telnet exists, isolate it behind an OWNER-APPROVED,
   independently verified encrypted site-to-IPAT tunnel and a tightly
   scoped site-side management gateway. NEVER send usernames or
   passwords over the presently observed plaintext public Telnet
   route, including "temporary" lab credentials. The team must
   separately verify the tunnel endpoint and the final private segment.
2. **Fix source reachability:** from a dedicated restricted nonroot
   IPAT management worker prove the authorized private route and
   site-controlled source allowlist, without changing the existing
   Nusa provider firewall or assuming the current VPS timeout cause.
   Recheck source and destination separately; a Mac-only reachable
   public port does not prove the VPS worker can monitor the OLT.
3. **Verify the actual peer:** independently pin the secure host key
   or managed-device identity; verify the precise chassis, card,
   model, firmware and supported read-only CLI/SNMP capabilities
   using a dedicated least-privilege operator account ONLY after
   steps 1–2. The existing R7.9 SSH-only C320 adapter is merely
   a candidate; never automatically run its fixed commands
   against unknown firmware or switch it to unsafe Telnet.
4. **Bind exact tenant/POP and audit:** finish real human IdP+MFA,
   true browser BFF, verified tenant/POP membership, separate
   maker-checker approval, encrypted managed credentials and an
   auditable immutable device-enrollment workflow. Move status
   beyond UNKNOWN only with timestamped authenticated evidence.
5. **Physical test before writes:** only then execute narrowly scoped
   TC-OLT-01 read-only board/firmware/PON tests, record actual
   exact tuple and reviewer. Firmware upgrade is a **separate**
   high-risk milestone requiring official image verification,
   real full backup, independent recovery/rollback, change
   window and dual approval. NO firmware write authorized here.

**LARGE RED PRD DEVIATION — NO PHYSICAL ADOPTION:**
Successful TCP/Telnet handshake is NOT authenticated OLT access.
Actual commercial device enrollment/telemetry,
real MFA dashboard, live heterogeneous K3s and independent
whole-host+PostgreSQL recovery are NOT COMPLETE.
