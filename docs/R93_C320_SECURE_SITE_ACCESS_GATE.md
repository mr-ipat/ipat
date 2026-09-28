# R9.3 — DEV-01 secure site-access gate (pending owner-side setup)

Date: 2026-09-28 Asia/Jakarta. Status: PLANNED / BLOCKED ON SECURE ROUTE.
Owner-reported candidate DEV-01: ZTE C320, public IPv4 translated TCP/321
into Telnet/TCP/23. The owner reports access, but authenticated hardware
identity, exact model, board, firmware, read-only account and safe route
are NOT independently verified. Never commit the numeric address or secret.
R9.0 verified only an unauthenticated public Telnet handshake from the
owner Mac; the Ubuntu 26.04 VPS timed out. R9.2 stores only inert intents.

## MUST before physical authentication or device read

1. Owner selects one independently secured path: (preferred) verified
   SSH on the OLT through site-controlled private management VPN;
   alternatively vendor-documented SNMPv3 authPriv for eligible read
   operations; if only Telnet is supported, Telnet MUST stay exclusively
   on a truly private, tightly scoped site-side segment inside the
   authenticated encrypted VPN, with no publicly exposed plaintext
   listener. Do not forward the present public Telnet NAT into IPAT.
2. The owner authorizes a private management address, VPN peer identity,
   ingress/source allowlist, exact destination port and approved device.
   Save route material and credentials only in an approved secret store,
   NEVER in repository, chat or browser preview.
3. Independent NOC/security reviewer checks endpoint identity, tunnel
   termination and last-hop confidentiality, least-privilege account,
   device-side ACL, audit policy, access expiry and documented revocation.
4. Using a restricted nonroot management worker, prove exact private
   route reachability without logging in. Independently verify device
   identity before any command: real serial, chassis/board, firmware,
   management protocol and cryptographic server identity when available.
5. Add immutable time-limited evidence and two-person review to the
   existing R9.1 four readiness gates. Revalidate fresh gates again
   on worker claim, with a per-device lease and hard one-command
   read-only allowlist; keep R9.2 intents permanently nonexecutable.

## Acceptance evidence (all initially NOT RUN)

- SITE-01: signed owner endpoint and exact tenant/POP binding;
  public Telnet remains explicitly ineligible for credential transport.
- SITE-02: independently checked site VPN/SSH or SNMPv3 authPriv
  with isolated last hop, restricted source and stored revocation test.
- SITE-03: nonroot actual IPAT worker reaches only approved private
  host/port; failure path and network timeout safely deny execution.
- SITE-04: authenticated exact physical identity and recorded
  serial, boards, firmware and supported read-only command/protocol.
- SITE-05: live independent identity/MFA + approver proof, audit,
  exact POP, immutable grants, fresh SQL revalidation, durable device
  lease, bounded one-read worker, rollback-safe timeouts.
- SITE-06: actual TC-OLT-01 read-only result and explicitly redacted
  evidence. UNKNOWN/NOT_MEASURED until all applicable gates pass.

## Safe owner input needed, without sharing passwords here

Confirm either **SSH supported by this C320 and its actual TCP port**,
**SNMPv3 authPriv supported**, or **only Telnet available**. State whether
an owner-controlled site router can terminate WireGuard/IPsec and route
to the OLT private management VLAN. Provide its intended public VPN
endpoint only after site approval; place credentials and keys in the
approved encrypted secret store rather than a chat message. Exact
configuration will be vendor/version-specific after read-only inventory.

## Separate firmware milestone (NOT authorized)

No physical firmware push until independently verified official image
and matching hardware revision, backup plus tested recovery, power and
maintenance window, dual approval, pre/post checks and explicit written
change authorization. A read-only test NEVER implies upgrade approval.

**LARGE RED PRD GAP:** physical device admission, secure site route,
real account and identity, genuine live UI health and firmware change
are all currently UNIMPLEMENTED or UNVERIFIED; this document is a
safety and acceptance gate, not an executed device integration.

## Owner confirmation: Telnet only; no existing VPN (2026-09-28)

Owner confirms no currently available secure management protocol other
than Telnet and no site VPN yet. Preferred next proposed change:
IPAT management worker → independently authenticated WireGuard VPN →
owner-controlled site gateway → isolated local management VLAN →
C320 TCP/23. WireGuard alone DOES NOT encrypt site-gateway-to-OLT
Telnet; verify that final hop is a genuinely private, trusted,
restricted segment with no unauthorized sniffing or shared transit.
Gateway vendor/capabilities, true OLT private management interface,
actual site topology, addresses, rollback access and VPS permitted
VPN egress remain UNKNOWN. No provider firewall changes authorized.

Before applying ANY setup: owner identifies approved gateway that can
run WireGuard (e.g. supported MikroTik RouterOS 7), confirms out-of-band
router recovery, protects existing PPPoE/NAT routes, picks a nonoverlap
VPN subnet and authorizes scoped UDP ingress/egress. Keep all live VPN
keys solely on approved endpoints/vault; pin public peer keys out of band.
Test proposed rules OFFLINE, explicitly exclude tunnel->other tenants,
VPN->customer data, and unrestricted WAN->OLT. Implement staged apply
with automatic recovery/rollback and verified alternate session only
after the owner approves exact live configuration and physical scope.
A reachable VPN alone is NOT an authenticated device or permission
to send Telnet credentials. Public NAT TCP/321 retirement is mandatory
before physical authenticated Telnet test.
