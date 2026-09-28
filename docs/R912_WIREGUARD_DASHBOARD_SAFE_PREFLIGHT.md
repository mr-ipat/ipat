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

## Actual owner VPS private LAB deployment (bounded, reversible)

Original code SHA `42f7a2051554e8e6ed2d5f8010515d669d7d53f3`
passed GitHub R9.12 PR #113 run `36396487249` 4/4 independent
jobs. An exact source-only Mac Git bundle was SHA256 verified on
the nonroot owner VPS and checked out separately, leaving the existing
canonical VPS `main` workspace unchanged. Reproducible low-priority,
locked, OFFLINE build produced binary SHA256:
`cef14131bd67cfd0b57efc6228353fef83dd54d0370bb0fbcab1d027aef23147`.

Reviewable replacement user unit:
`deploy/scripts/lab/r912/ipat-r911-preview.service`. Its checksum
matched on Mac and VPS before the existing user unit was replaced.
The old user unit is saved OWNER-ONLY at
`/home/openai/.cache/ipat/r912-preview/rollback-unit.service`.
After constrained nonroot user-unit restart, actual VPS HTTP proof
confirmed: new synthetic review GET page/POST API200, both no-real-
key conditions and even best-case synthetic scenario blocked, fake
secret/endpoint injected JSON rejected 4xx, missing origin HTTP403,
real business API HTTP401, physical adoption false, old :3000 HTTP200.
Test harness: `deploy/scripts/lab/r912/actual_lab_wizard_http_smoke.py`.
No live tunnel or customer infrastructure interaction performed.

The restricted R9.12 preview remains loopback-only :3002. It is NOT a
real signed Tenant Admin wizard; owner Mac requires its private SSH
port-forward to view. Existing user manager has `Linger=no`, so
service availability after all SSH user sessions end is NOT promised.
No system/root service, host SSH/firewall policy or K3s changed.

Owner-only rollback, without affecting original :3000 demo:

```sh
ssh ipat-lab
cp -p /home/openai/.cache/ipat/r912-preview/rollback-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user restart ipat-r911-preview.service
```

A subsequent real tunnel activation feature MUST use authenticated
Tenant Admin, encrypted key custody, explicit owner-approved firewall
and routing changes, independent maker/checker, and console-backed
rollback. This prototype does NOT authorize physical C320 login.
