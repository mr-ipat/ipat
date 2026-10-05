# R9.78 — Tenant NOC real-POP grant workflow (not public hardware provisioning)

R9.76 established independent POP and Site masters, and R9.77 established typed real-POP NOC scopes that cannot inherit legacy exact-Site grants. R9.78 adds tenant-admin-maker/checker issuance and immediate revocation for **read-only real POP inventory only**.

## Standard operator sequence

1. An independently authenticated Tenant Admin opens NOC access. Current real POP and eligible NOC users come from server-controlled tenant-scoped lists. Platform Owner and legacy exact-Site scopes cannot issue the typed grant.
2. Admin selects exact NOC identity, real POP and a bounded duration of 1–2160 hours. Browser generates one request UUID and UTC absolute expiry, retaining both across an uncertain network retry. The PostgreSQL transaction compares *all* actor/target/POP/expiry fields for exact idempotency.
3. The request starts PENDING. The maker sees that a different Tenant Admin must review it; the maker has no approval buttons and direct self-approval is rejected independently by SQL.
4. A different, independently current Tenant Admin can approve or reject once under a Host-bound session plus same-origin CSRF. Approval rechecks exact current target NOC membership, tenant status, current POP and grant expiry. Existing active duplicates and cross-tenant inputs are rejected.
5. Approved grants reveal **read-only** inventory from only Sites presently associated with that real POP. Reassigning Sites or revoking NOC membership dynamically removes visibility. Revocation by current Tenant Admin takes effect on the next authorization call. The independent legacy exact-Site permissions remain unchanged and are never silently upgraded.
6. An append-only service-scoped event trail records REQUESTED, APPROVED/REJECTED and REVOKED with actor and target. SQL role separation excludes OIDC issuer and direct raw-table access. No operator controls device credentials, physical operations, firmware or other tenant POPs through the grant.

## Deployment/testing boundary

- Migration: deploy/db/migrations/0031_noc_real_pop_grant_lifecycle.sql, after 0030.
- SQL acceptance: IPAT_R978_SYNTHETIC_DOCKER=YES bash deploy/db/tests/r978_docker_repro.sh, isolated disposable no-published-port PostgreSQL16. Regression coverage includes cross-tenant, replay-with-different-expiry, maker==checker, revoked/expired member, duplicate, approval/rejection/regrant/revocation and legacy-site non-escalation.
- Commercial HTTP: apps/control-api/src/commercial_tenant_api.rs; UI: web/console/tenant/dashboard.html and app.js. Extend existing R9.70 Rust+disposable PostgreSQL integration with two distinct signed synthetic durable sessions to prove current CSRF, Host isolation, role denial, maker/checker and immediate typed POP access revocation.
- Genuine customer MFA/IdP, provider rescue console, off-host host and PostgreSQL PITR recovery, public trusted TLS and physical interoperability are independently required. The SQL and HTTP functionality cannot be described as live public SaaS until deployed and independently verified.
