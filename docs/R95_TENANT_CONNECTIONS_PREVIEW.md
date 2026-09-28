# R9.5 — Tenant network connection selector, offline-only preview

Date: 2026-09-28. Owner decision: optional multi-protocol management;
WireGuard is NOT mandatory; built-in RouterOS WireGuard requires v7,
but RouterOS 6 can use compatible IPsec or a dedicated site gateway.
For the reported ZTE C320 with ONLY Telnet and public NAT TCP321,
never send credentials over plaintext WAN. Permit controlled public
credential-free receive-only testing solely as transport evidence.

Actual owner-Mac R9.5 one-shot receive-only check: TCP reachable,
Telnet IAC observed, 15 inbound bytes, ZERO credential/client bytes,
physical identity NOT VERIFIED, adoption NOT DONE, health NOT MEASURED.
Historical actual VPS R9.0 path timed out; not revalidated in R9.5.
No user-facing real Telnet management integration was enabled.

`web/lab/device-workbench.html` and `.js` now display a same-origin
LAB ONLY Network Connections method/gateway selection in Device Manager.
Direct secure protocols, WireGuard, IPsec, future site gateway and
ineligible public Telnet are distinguished; RouterOS 6 plus WireGuard
is explicitly rejected. This preview collects NO actual addresses,
credentials, keys, or tunnel configuration. It has NO save/activate
button or backend mutation. It is NOT an authenticated tenant panel.

Acceptance: static R9.5 tests, JS syntax, existing private demo smoke;
future real implementation requires independently verified live OIDC
MFA, exact tenant/POP role, backend deny-by-default, encrypted secret
storage, isolated management worker, maker/checker approval, fail-closed
provisioning orchestration, idempotent rollback and audited test report.
Real device adoption and firmware are still RED / BLOCKED.
