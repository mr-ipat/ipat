# IPAT R7.8 — Signed OIDC → exact restricted PostgreSQL → private menu

**Status:** PRIVATE LAB vertical slice and *separately* synthetic disposable
database proof; NOT real company login, MFA, customer data access or
production-ready database deployment. Date: 2026-09-27.

## Why this is different from previous synthetic policy tests

The existing R6.9 pinned-issuer RS256 verifier, R7.6 scoped
dashboard policy and R7.7 restricted nonlogin PostgreSQL function are
now connected in the ACTUAL Rust Axum application, rather than three
unconnected individual test fixtures. The end-to-end CI test signs
a real synthetic JWT with a newly generated RSA key and calls the
actual private HTTP route, which queries a disposable PostgreSQL 16
over a separately created least-privileged LOGIN test role.

No real Keycloak issuer, human MFA, approved company membership or
real data has yet been provisioned. A successful synthetic test does
NOT satisfy the real-user portion of FR-002, AC-01 or AC-02.

## Implementation

- `apps/control-api/src/tenant_membership_lab.rs`: read-only
  `GET /lab/auth/sections`, available ONLY when both
  `IPAT_LAB_WEB=1` and `IPAT_LAB_OIDC_VERIFY=YES` and independently
  `IPAT_LAB_SCOPED_MEMBERSHIP=YES`, on separate NONROOT loopback
  listener `127.0.0.1:3001`. No route on regular app/K3s bind.
- The JWT is verified against an independently owner-pinned issuer,
  audience, key ID and public RSA key. The issuer+subject from that
  VERIFIED token are the **only** identity inputs to the DB. Host,
  X-Tenant, X-Role and token custom claims have NO influence.
- The requested tenant UUID, fixed role and optional POP are treated
  as untrusted resource selectors: exact server-side PostgreSQL
  lookup permits them only if a matching approved, active,
  unrevoked, unexpired membership and role-specific POP grant
  exists. Tenant slug is returned directly from that trusted DB
  function, not inferred from domain.
- The existing R7.6 fail-closed Rust dashboard policy generates
  allowed read-only menu names. Platform admin, bulk PPPoE and
  firmware remain inaccessible. Response explicitly includes
  `lab_only=true`, `mfa_verified=false`,
  `real_business_access_enabled=false`; no database approver
  identity, signed token, customer data or device credentials
  are returned. Wrong token 401, wrong scope 403, DB outage 503.
- The runtime refuses enabling the lab without an OIDC verifier or
  when it is configured on K3s. Its DB connection string must
  be a non-symlink single-link owner 0600 file in an owner 0700
  folder, with exactly one ABSOLUTE Unix-socket host and exact
  dedicated user `ipat_lab_identity_reader`; PostgreSQL login
  role is NOT created on real VPS. Only the disposable CI seed
  creates this account and grants the existing NOLOGIN query role.
- All existing business routes `/v1/platform/*`,
  `/v1/tenant/*`, `/v1/operations/*`,
  `/v1/devices/{device_id}` still reject all users HTTP401;
  the existing owner private dashboard continues to show
  synthetic data. No live PostgreSQL, host firewall, DNS,
  certificate, root, K3s, OLT or firmware change was made.

## CI and laboratory verification

CI first runs R7.0/R7.7 existing synthetic PostgreSQL migrations
and negative RLS checks, then creates a SEPARATE disposable
`ipat_lab_identity_reader` non-BYPASSRLS login account,
approved synthetic lab records for two UUID tenants and
their exact roles+POPs, with a known synthetic CI-only
password. The integration test invokes the actual Axum
handler using generated short-lived RS256 tokens and the real
dedicated PostgreSQL connection. Positive and negative
cases include forged Host/tenant/role/JWT claims, wrong
role/tenant/POP, missing token, POST denial and all business
endpoints remaining HTTP401. Private source contract tests
add double-opt-in, no untrusted SQL or host-role inheritance.

Commands (not for live VPS database):

    cargo fmt --all -- --check
    cargo test -p control-api --locked --offline
    python3 -m unittest discover deploy/db/tests \
      -p test_private_menu_contract.py -v
    # CI disposable database ONLY, after R7.7 tests:
    python3 deploy/db/tests/seed_private_http_identity_ci.py
    cargo test --locked -p control-api \
      tenant_membership_lab::tests::r78_end_to_end_real_signed_jwt_real_restricted_sql_real_axum_router -- --exact

Do NOT execute the synthetic CI seed against a real database,
copy sample credentials into live deployment, or activate the
private lab reader before operator-reviewed RLS, server
identity and independent full recovery. A later provider
must enforce MFA and accountable approval before real access.

## Owner's current action / blocking dependencies

1. Actual isolated ZTE C320 console evidence: running exact
   firmware/build, board/controller/PON card types and revision,
   reliable private read-only device route, independent console,
   offline backup and reviewer. Do not share device passwords,
   IPs, unredacted serials or SSH host private keys in chat/Git.
   `TC-OLT-01` remains NOT RUN.
2. Independently accessible VPS emergency console and a
   different-location encrypted full recovery copy/test:
   no new production firewall, public domain app or live K3s
   until independent external gates close.
3. Identify or approve private OIDC IdP accounts with MFA
   for one tenant admin and one NOC tester. Issuer HTTPS,
   audience, JWKS and independently trusted public-key
   pin may be given without any private keys/passwords.
   The product will not claim real company login until
   actual scoped membership is enrolled, audited and tested.

`ipat.fadly.id` is already DNS-pointed according to the
owner and was read-only resolved earlier; the same hostname
is currently also used for SSH management. Its future customer
custom-domain TLS/ownership and management separation are
still deferred per ADR-020/021, with NO backend tenant
selection from DNS/Host before verified commercial rollout.


## Actual immutable code validation

Feature PR #85 `b8b92b8` passed CI run
`36287648718` with all four jobs SUCCESS; GitHub
PostgreSQL integration logs explicitly show the
real token+Axum+dedicated role+PostgreSQL test
executed `1 passed, 0 failed` (not skipped).
PR #85 squash-merged main code SHA is
`15cd78de46bc08f1563a3eeee229bbcf7e05b3ab`;
independent post-code-main run `36287880873`
passed all four jobs. The authorized nonroot
Ubuntu 26.04.1 lab ran full locked offline Rust
workspace, four R7.8 static and five R7.7 static
tests; existing R7.4 offline C320 readiness
eight tests passed. Owner Mac source-only Restic
snapshot `588edeb3` verified all 126 encrypted
packs and isolated exact source SHA256 restore
plus historical PARTIAL selected config archive.
This is not whole-host/real DB DR.

The reviewed private `127.0.0.1:48765` dashboard
was rebuilt on the actual VPS but does NOT enable
the optional restricted membership route: its
default HTTP response remains 404, with actual
Platform/Tenant/NOC business endpoints denying
forged tenant/domain/role requests HTTP401.
Use this milestone only as a VERIFIED integrated
DISPOSABLE identity test and software foundation,
not proof that IPAT is ready to accept customer
data or perform physical firmware operations.
