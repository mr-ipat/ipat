# R7.7 — Restricted PostgreSQL identity lookup; domain remains unrouted

**Date:** 2026-09-27. **Classification:** SYNTHETIC disposable PostgreSQL
test migration, not live tenant login, production configuration or
a production-approved ADR-005 database architecture.

## Verified current context

- At the time of this milestone, a read-only DNS `A` lookup from
  the authorized owner Mac returned `ipat.fadly.id → 202.162.204.121`;
  the owner states they pointed this customer-domain illustration.
  This is DNS observation only: NOT proof of DNS ownership, TLS readiness,
  dedicated company web ingress or commercial tenant authorization.
- `ipat.fadly.id` also remains the previously verified operator
  SSH destination for the lab VPS. We did NOT modify its DNS,
  SSH ingress, host/provider firewall, certificate or public listener.
  `ipat.id` remains proposed future platform branding, not a live
  independently proven product-owned domain.

## Product acceptance and threat-model slice

**MUST (delivered as candidate with disposable test, not live deploy)**

- `0004_lab_scoped_identity_lookup.sql` defines an explicitly
  restricted `NOLOGIN`, `NOBYPASSRLS` lookup owner role, an independent
  `NOLOGIN` query role, and a fixed `SECURITY DEFINER` SQL function.
  Neither role has login credentials or independent customer access.
  The existing `ipat_app_runtime` role does not gain platform-table
  privileges, platform schema usage or function EXECUTE.
- The function accepts EXACT issuer, subject, tenant UUID, fixed
  read-only role, and optional explicit POP (non-admin roles MUST have
  matching POP grant). It returns only approved_by and expiration
  for an active, nonrevoked, nonexpired membership of an active tenant.
  Matching issuer+subject MUST originate from pinned verified access
  token server-side and a separately trusted membership store;
  NEVER from Host, forwarded headers, URL, JWT extra claims or browser.
- Two targeted RLS `FOR SELECT` policies permit only the sealed
  function owner to inspect the required platform rows. It can
  query `tenants` + `identity_memberships` + `identity_pop_grants`;
  it CANNOT SELECT `platform_principals`, mutate tables, create schema
  objects, or grant client credentials. Function uses fixed
  `search_path`, schema-qualified table references and no dynamic SQL.
- Disposable synthetic PostgreSQL verifies positive two-tenant
  fixture roles/POPs and denies wrong issuer, subject, tenant, role,
  missing POP, wrong POP, revoked or expired membership, suspended
  tenant, direct table read, runtime invocation and platform-owner
  privilege escalation. Synthetic data contains no subscriber/device
  credentials. Static checks prevent unsafe GRANT/code drift.

**SHOULD next, not delivered**

- Authenticated, MFA-backed IdP; trusted verifier-to-query service
  integration via restricted credentials outside Git; operator-reviewed
  grant creation; audited revocation and token/session invalidation.
- Complete trusted server-side menu/API/queue/RLS role/tenant/POP
  lifecycle with two-tenant negative tests through REAL backend
  and recoverable production configuration.

**LATER, only after system and recovery pass**

- Independently verify ipat.fadly.id customer-domain ownership,
  DNS challenges and TLS/callback/cookie isolation. Separate or
  explicitly recovery-test management SSH routing BEFORE repointing
  this hostname to a public company dashboard. Reserve the future
  ipat.id mapping until independently proven and commercially ready.

## Security constraints and residual hazards

A SECURITY DEFINER function is powerful: execution is restricted
to a dedicated NOLOGIN role with no runtime assignment or login,
specific table RLS SELECT policies, a hardened fixed search_path and
bounded exact-identity checks. Future connection provisioning must
not accidentally make ipat_app_runtime a member of the query role
or allow user-controlled issuer/subject/tenant/role arguments;
do not activate this SQL as an identity source without independent
security review. SQL approved_by is a syntactic record field:
it does NOT cryptographically prove approver identity or MFA.
The function returns a candidate membership, not a final principal.
All actual business HTTP endpoints remain HTTP401, physical
DEV-01 TC-OLT-01 is NOT RUN and firmware operations stay disabled.

## Exact tests and rollback

The disposable PostgreSQL CI job MUST execute migrations 0001-0004
in that order, then the R7.7 query-role tests. It must not point
at the real VPS PostgreSQL or import actual operator credentials.

    python3 -m unittest discover deploy/db/tests \
      -p 'test_scoped_identity_lookup_contract.py' -v
    # Only in explicit disposable PostgreSQL test env AFTER 0001-0003:
    python3 -m unittest discover deploy/db/tests \
      -p 'test_scoped_identity_lookup_integration.py' -v
    cargo fmt --all -- --check
    cargo test --workspace --locked --offline -q

Rollback of the unapproved LAB-ONLY candidate: discard its entire
disposable PostgreSQL database/container; do not run a drop/role
migration against live systems. Before a REAL migration, take
independent transactionally consistent backups, verify restore,
assign identity/reviewers and write a separate audited rollback plan.
