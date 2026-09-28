# R9.6 — Safe ZTE C320 production-like adoption staging

Date: 2026-09-28 Asia/Jakarta. Production impact assumption:
DEV-01 is a LIVE distribution OLT with active ONTs/subscribers.
Owner grants a bounded passive TCP/Telnet network check, NOT
permission for arbitrary configuration, read-only login over public
Telnet or firmware modification. No active OLT commands executed.

## Implemented in this milestone (private lab only)

Original Rust Axum `POST /lab/demo/connection-plan` evaluates a strictly
enumerated synthetic connection choice; the existing dashboard now
has an explicit 'Validasi rencana di backend lab' button. Backend
accepts only method, gateway and fake device capability enum, requires
existing exact loopback Host+Origin demo CSRF header and denies other
fields including real addresses, usernames, passwords, keys and CIDRs.
It explicitly rejects public Telnet, RouterOS6 built-in WireGuard,
nonexistent site gateway and native secure protocol for Telnet-only
hardware. Response is plan-only with network_actions=0, no dispatch,
no tenant verification, no device adoption and unmeasured health.
This is NOT the signed Tenant Admin workflow or tunnel installer.

## Mandatory safe live distribution test ladder (NOT RUN)

1. Nonintrusive baseline: site-owned traffic/ONU counters and alerts,
   approved maintenance owner, rollback contact and exact blast radius.
2. Isolate OLT management hop and independently prove site gateway,
   trust path, source and private device host; withdraw public Telnet
   forward only with attended console rollback and explicit approval.
3. Independently validate dedicated restricted device credentials and
   actual C320 CLI read-only allowlist for exact firmware; redact logs.
4. Real MFA Tenant Admin plus maker/checker, exact tenant/POP custody,
   fresh authorization and immutable audit; verify no cross-tenant view.
5. Start only one bounded approved read-only session on isolated path,
   no discovery sweeps, writes, file transfers, polling storm or retry
   amplification. Measure latency/load/alarms and abort on regression.
6. Verify exact chassis/serial/board/firmware, status observations,
   immutable recorded results and operator review before displaying
   online/healthy. Firmware upgrade is separately gated and NOT IN SCOPE.

The historical MAC receive-only check received 15 IAC bytes without
sending credentials; this is network evidence, not C320 compatibility.
The previous actual VPS public path timed out. No path has yet passed
safe authenticated management requirements. STOP any physical login
until gating evidence is complete. For the owner-reported x86 RouterOS7
site, WireGuard is ONE selectable transport, not mandatory generally.
