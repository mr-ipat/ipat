# R9.12 — WireGuard dashboard safety preflight (synthetic LAB)

Date: 2026-09-28. The previous lab and all its evidence are now merged
into main (`e4ab353`). Owner requests dashboard-managed tunneling, not
CLI-dependent tenant operations, and zero avoidable impact on a LIVE
ZTE C320 distribution OLT. WireGuard is OPTIONAL in IPAT globally;
MikroTik native WireGuard requires RouterOS 7, not every ISP gateway.

## Delivered

- Device Manager shows a dedicated synthetic WireGuard review wizard,
  offering gateway, site LAN isolation, console recovery rehearsal and
  operational baseline scenario choices. These are EXAMPLES; selecting
  a verified option does NOT attest that it was verified on site.
- New original Rust `POST /lab/demo/tunnel-review` allows only four
  fixed-choice fields and denies unknown keys, passwords and endpoints
  via serde deny-unknown-fields plus the existing private lab loopback
  Host/Origin/explicit demo CSRF guard. Endpoint produces only a list
  of missing conditions and ALWAYS returns BLOCKED pending real review.
- Even when ALL synthetic conditions are 'met', mandatory real MFA,
  reviewed site addressing, independent maker/checker and actual
  vendor SSH fingerprint checks remain unsatisfied. No secrets, keys,
  RouterOS scripts, tunnels, network actions or worker dispatch occur.
- Real Axum negative tests exercise missing origin, synthetic best
  case, unknown secret/endpoint fields and invalid enum. Python tests
  cover HTML/JS/backend fail-closed contracts. Existing physical C320
  monitoring and service health remain UNKNOWN/NOT_MEASURED.

## Remaining mandatory phases (no false completion)

1. Genuine signed OIDC MFA Tenant Admin and exact tenant/site/POP
   authority; DB draft-only R9.7 seam remains deliberately unmounted.
2. Verified live address-plan collection into an approved secret-safe
   encrypted draft store, independently measured topology and return
   path, management VLAN isolation and off-path recovery proof.
3. Separately reviewed two-person activation of a narrow gateway
   tunnel, with readiness checks and rollback that cannot affect
   PPPoE, bridges or customer routes.
4. Independently identify/pin actual legacy C320 RSA SSH key through
   trusted owner console and provision a dedicated restricted account.
5. Start one bounded audited read-only CLI operation ONLY after the
   owner records operational baseline/abort thresholds and the worker
   has a verified isolated private route. Record actual chassis,
   firmware and no-impact observations without overclaiming safety.

R9.12 must not auto-enable or silently deploy any live configuration.
