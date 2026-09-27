# IPAT R8.5 — Real independently provisioned IdP trust/MFA preflight

**2026-09-27. Source scope: original Rust trust-verification utility,
not a live customer IdP, authenticated browser session or device
adoption. Physical C320/ONT/MikroTik and full production remain OPEN.**

The owner requires a signed-in, persistent tenant/POP/device dashboard
before connecting actual hardware. R8.3 already has genuine interactive
fake-only Device Manager plus a separately signed JWT→real PostgreSQL
pending candidate backend. R8.4 added the independently restricted
reviewer maker-checker and append-only audit. The missing security
prerequisite is to independently establish the operator's ACTUAL
human OIDC issuer/MFA semantics and safe browser session, not invent
a local password database, trust a request header or paste a token
into an unsafe public demo.

## What R8.5 really implements

New standalone original Rust binary
`crates/identity-core/src/bin/oidc-mfa-preflight.rs` invokes the
SAME existing independently pinned `identity-core::PinnedIssuer`
signature verifier used by existing API authorization. A nonroot
operator must provide an independently verified HTTPS issuer URL,
exact audience and RSA key ID, plus the corresponding official
JWKS RSA public key converted to a PEM held in a private owner 0600
regular file inside an owner 0700 directory. This utility ONLY
accepts a short-lived access JWT through bounded protected stdin;
it refuses a terminal to reduce accidental echoed bearer typing.
It checks original strict RS256 signature, exact pinned issuer,
audience, kid, issued/not-before/expiry and maximum 15-minute
lifetime, then checks the exact SIGNED `amr` list includes
the exact value `mfa`. All token key, identity, role,
POP and tenant contents remain private; a pass prints only
the bounded three-line generic preflight status.

ALL of the following are *separate* external operator checks,
not automatically satisfied by a signed `amr:mfa` alone:
- Approved ACTUAL human IdP with independently owned
  admin/backup accounts, enabled second-factor enrollment
  and a second-person verified login+MFA challenge.
- Actual HTTPS issuer TLS chain, DNS, pinned trusted RSA
  key provenance/rotation and provider-specific official
  documentation that `amr:mfa` truly represents the
  desired successful second-factor event. Keycloak
  is a CANDIDATE IdP, not silently an approved deployment.
  Do not assume the default server emits this claim:
  require independent client AMR mapper verification.
- Correct OIDC authorization-code+S256 PKCE confidential
  browser BFF design and secure host-only HttpOnly Secure
  SameSite session cookie with CSRF, state/nonce and
  refresh/logout; no implicit or password grant flow.
  That BFF is STILL **NOT IMPLEMENTED** by this binary.
- Actual approved tenant/role/POP memberships and
  genuine production restricted PostgreSQL service
  credentials supplied only to the server over a
  private Unix socket after independent recovery gates.

**PRD RED GAP:** No human MFA account was actually enrolled
or challenged, no browser login works, no company customer
login is provisioned, and no real OLT/ONT is reachable/
adopted as a result of this utility. An MFA signed-token
preflight pass NEVER turns on a real dashboard, SQL writer,
reviewer, customer API or firmware process.

## Guarded operator rehearsal, with NO example secrets

Run on a dedicated authorized nonroot host after independent
source review and only after an OIDC IdP exists:

```sh
cargo fmt --all -- --check
cargo test --locked -p identity-core --test oidc_cli
cargo build --locked -p identity-core --bin oidc-mfa-preflight
./target/debug/oidc-mfa-preflight --requirements
```

For a future real IdP check, the operator independently sets
`IPAT_R85_REAL_IDP_PREFLIGHT=YES` and
`IPAT_R85_ISSUER`, `IPAT_R85_AUDIENCE`,
`IPAT_R85_KID` and `IPAT_R85_PINNED_PEM_FILE`.
The public key file must have 0600 permissions under a
0700 directory owned by the running nonroot account,
with no symlink or additional hard links. Supply the
independently obtained short-lived access token via a
secure protected FD on standard input to
`oidc-mfa-preflight --verify`, WITHOUT placing
bearers in shell arguments, shell history, a file
in this repo, chat transcripts or ordinary command logs.

A successful synthetic signed-claim check emits
`R85_PINNED_SIGNED_MFA_CLAIM_PREFLIGHT=PASS`,
`ACTUAL_HUMAN_MFA_ENROLLMENT_AND_BROWSER_SESSION=NOT_VERIFIED`
and `PRODUCTION_DEVICE_OR_BUSINESS_ACCESS=DENIED`.
All denied cases print only a generic diagnostic,
without token/subject/tenant content. There is no
runtime HTTP server, enrollment or secret material
created. Do not enable existing R8.4 review
endpoints on production solely because this binary passed.

## Actual automated evidence

The independent Rust integration test
`crates/identity-core/tests/oidc_cli.rs` generates a
fresh disposable 2048-bit RSA key using OpenSSL
for each fixture (no checked-in key), signs short-lived
synthetic access JWTs, starts the ACTUAL compiled
Rust binary as a subprocess and pipes the JWT through
a private anonymous stdin FD. Tests assert valid
signed synthetic `amr:mfa` positive and negative
missing explicit opt-in, missing MFA, wrong RSA
kid, independent attacking RSA key, invalid or
oversized bearer, world-readable public key
and symlink public key. It also asserts that
success output does not reveal attacker
tenant/role claims or synthetic subject,
and never claims actual human login.
These are strong software tests only;
a true IdP/MFA protocol and token-claim
conformance test must be run once actual
approved operator accounts are available.

CI runs the independent CLI process tests,
four R8.5 static safety contracts, full
locked Rust workspace, and previous
disposable PostgreSQL maker-checker,
physical-restore and disposable
single-node Ubuntu26 K3s jobs.
No actual hardware, real customer IdP,
production PostgreSQL, live K3s
or network firewall is touched.

## Product critical path after this milestone

**MUST NEXT:** approve the independent human
IdP and actual second factor, implement native
OIDC Authorization Code+PKCE server-side BFF
and verified session, connect its signed-in
context to current independently authorized
tenant/POP database routes and reviewer UI;
test forged Host/cookies, CSRF and
cross-tenant response isolation.
**MUST AFTER:** approved verified remote private
OLT C320/C-DATA read-only management SSH/
SNMPv3 capability tests by exact firmware;
actual ONT TR-069/USP mutual trust;
fresh evidence-driven health/status and
append-only auditable per-device tasks.
Firmware remains a separate
high-risk maintenance workflow after
independent recovery proof.
