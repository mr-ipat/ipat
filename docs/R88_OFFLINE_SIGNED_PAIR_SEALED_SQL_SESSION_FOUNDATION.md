# R8.8 — Signed browser token pair → restricted PostgreSQL → opaque BFF session (OFFLINE)

Date: 2026-09-27 Asia/Jakarta. Implementation status: original real
Rust software and real disposable SQL test, **not yet a real customer login**.
Owner priority: before permitting add/adopt/health access to real ZTE
C320, C-DATA, VSOL/ZTE ONTs and MikroTik, the browser must have
trustworthy real human MFA and exact active tenant/POP approval.

## Implemented actual code

- `crates/identity-core/src/browser_session.rs` introduces an
  original bounded memory-only `BrowserSessionVault`. Only the
  `VerifiedBrowserIdentity` returned from R8.7 cryptographically
  checked matching **separately signed** ID+access JWTs can request
  an identity-only session. It never mints a role, company, POP,
  device privilege or proof that a real human MFA IdP was used.
- Both the opaque 256-bit browser session handle and independent
  256-bit CSRF nonce use operating-system cryptographic randomness.
  Only SHA256 digests, issuer, subject, bounded expiry and
  CSRF hash remain in memory. No raw access JWT, serial, credentials,
  role or tenant data is saved in the store. The maximum is 64
  sessions per isolated private process. Total duration is bounded
  to min(15 minutes, actual signed pair expiry) and max idle is
  five minutes. Expired sessions are deleted, exact revocation
  and fresh handle/CSRF rotation are supported.
- For a future independently trusted HTTPS BFF, the helper formats
  `__Host-ipat_session` with `Secure; HttpOnly; SameSite=Strict;
  Path=/` and the exact bounded cryptographic expiry as Max-Age.
  It is a **policy fixture only**: no actual cookie, login route,
  token redemption or privilege is issued in today's public/private
  HTTP server. READ requires a valid random cookie and separately
  validated host/origin. MUTATION additionally requires exact
  constant-time CSRF and independently verified origin.
- `apps/control-api/src/browser_session_lab.rs` is an UNMOUNTED,
  internal-only BFF review bridge. It only issues a session after
  independent pinned RS256 ID/access verification against the
  original private server nonce, then a genuine parameterized
  PostgreSQL `lookup_active_membership()` call with EXACT
  issuer+subject, active tenant UUID, approved role and POP.
  It never trusts Host, JWT-supplied role/tenant or untrusted
  user-scoped PostgreSQL role changes. The private membership
  reader is EXECUTE-only and has no direct table privileges.
  Each read or mutation against a chosen tenant and POP MUST
  independently recheck the current sealed database membership
  after session and CSRF checks; a stored cookie never
  becomes permanent tenant entitlement.
- Both functions remain unmounted on any Axum HTTP router,
  including the private lab preview and K3s. The preexisting
  R8.6 real browser callback still consumes a proof once
  and responds HTTP503 without signing in, and all real
  `/v1/platform`, `/v1/tenant` and `/v1/operations`
  business requests remain HTTP401 by default.

## Genuine repeatable tests

```sh
python3 -m unittest discover deploy/scripts/lab/r88 -p test_r88_contract.py -v
cargo fmt --all -- --check
cargo test --locked --offline -p identity-core --test oidc_id_token
cargo test --workspace --locked --offline
```

Rust's genuine 2048-bit ephemeral RSA test fixture signs the ID
and access JWTs separately with matching nonce, audience,
issuer, subject, `at_hash` and strict signed MFA claims.
It verifies that a correct pair can mint an opaque
unprivileged session; wrong nonce or bogus signed MFA
cannot; absent/untrusted origin, missing/incorrect CSRF,
expired cryptographic proof, malformed handles,
64-session capacity limit and stale/rotated/revoked
secrets fail closed. No physical devices are contacted.

In disposable PostgreSQL CI only, the additional real
Rust async integration test independently signs the same
pair, queries the genuinely restricted identity reader
against TWO synthetic different ISP memberships and
tests exact approved NOC POP. It denies cross-tenant/POP
and forged role, verifies per-request actual sealed
SQL re-authorization and CSRF gating, then expires
the session. That is a **real database and Rust integration
test**, not a fake SQL mock or a customer DB migration.
The CI-only PostgreSQL synthetic password is not
a user secret or a deployable credential.

## Blockers and next priority

**MUST**: real approved external IdP configuration,
separate proof of actual human MFA challenge and
approved `amr` semantics; real HTTPS confidential
authorization-code/PKCE token exchange and validated
provider behavior (in particular R8.7 strictly REQUIRES
`at_hash` while OIDC code flow makes it optional);
independently verified browser public domain/TLS
termination before ever sending Secure cookies.
Then a shared durable/revocable K3s-compatible session
store, explicit host/origin/CSRF middleware and
audited actual PostgreSQL tenant/POP memberships.
Only after that may the real company admin/NOC
Device Manager use pending metadata/review APIs
under server-side menu filtering and backend denial.

**MUST for hardware**: actual private route, authentic
OLT SSH fingerprint/SNMPv3 and real firmware model,
ONT supported genuine CWMP/USP enrollment, approved
manufacturer-specific read-only probes, evidence
freshness and honest network health; firmware
remains a separately approved high-risk later action.
Provider-neutral multi-node K3s and tested offsite
whole-host+real database+cluster DR remain open.

**LARGE RED PRD DEVIATION:** this milestone is the
necessary cryptographic session FOUNDATION and
actual SQL authorization integration, NOT a
signed-in commercial customer dashboard,
operational real device adoption, real MFA,
physical hardware interop or production-ready
SaaS. Do not enable its synthetic proof in
public or treat synthetic signed `amr` as
real enrolled human MFA.
