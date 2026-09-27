# IPAT R8.6 — Original Rust private OIDC browser authorization-start guard

Date: 2026-09-27 Asia/Jakarta. This milestone addresses
the owner-requested priority of real human operator identity
BEFORE a connected device adoption workflow. The current code
is a hardware-free **OIDC BROWSER FLOW SECURITY SLICE**,
**NOT** a real IdP login, working authenticated tenant dashboard,
customer MFA, or approved physical device access.

## Original Rust functionality

- `crates/identity-core/src/browser_pkce.rs` uses the operating
  system's cryptographically secure `getrandom` entropy,
  three distinct 32-byte random values, and SHA-256 + URL-safe
  unpadded Base64 to produce RFC 7636 `S256` PKCE verifier
  and challenge, plus separate unpredictable `state` and
  `nonce`. It is independently unit-tested against the
  published RFC 7636 sample verifier/S256 expected result.
- `apps/control-api/src/oidc_browser_lab.rs` adds an
  expressly opted-in, nonroot, localhost-only, fake-provider
  browser **GET** `/lab/auth/browser/start`. It only
  constructs a redirect to the owner-configured, exact
  pinned-issuer-associated **Keycloak candidate** HTTPS
  authorization endpoint and registered fixed local callback.
  No untrusted dynamic IdP URL, Host, role, tenant, POP or
  token header is accepted. Referrer-Policy is no-referrer.
- The 303 response includes only `response_type=code`,
  `scope=openid`, explicit `code_challenge_method=S256`,
  S256 challenge, state and nonce. The PKCE **verifier**
  stays server-side in bounded in-memory state and is never
  included in the redirect, browser script, URL, log or HTTP
  response. No client secret exists in this lab.
- The browser state is also carried in a strictly scoped
  `HttpOnly; SameSite=Lax` short-lived **local lab**
  correlation cookie. Exactly 16 outstanding operations are
  permitted per process; all expire after five minutes.
  Query parameters use explicit deny-unknown-fields parsing,
  callback cookie must match the state using constant-time
  equality, and valid state is consumed exactly once.
- The **GET** `/lab/auth/browser/callback` is deliberately
  **NOT** an actual OAuth exchange. Even on a correct state,
  cookie, code-shape and expiry, it destroys the state,
  clears the cookie and returns HTTP **503** with
  `authenticated:false`, excluding the authorization code.
  Invalid, tampered, duplicated-cookie and replay callback
  requests are rejected. An actual IdP may send a code but
  IPAT will NOT redeem, store or trust it in this milestone.
- No new sessions, tenant permissions, device APIs,
  connectivity evidence, physical checks, firmware, K3s
  public interfaces or PostgreSQL mutations are enabled.
  All `/v1/platform`, `/v1/tenant`,
  `/v1/operations` business paths still deny.

## Operator prerequisites and default-deny behavior

The actual owner preview remains the already functional
`http://127.0.0.1:48765/lab/device-workbench` fake-only
Device Manager. The new browser endpoints are ABSENT
unless an authorized nonroot operator provisions an
independently verified pinned issuer public key and
explicitly sets BOTH
`IPAT_LAB_OIDC_VERIFY=YES` and
`IPAT_R86_BROWSER_FLOW=YES` (plus
the existing private `IPAT_LAB_WEB=1`).
The operator-controlled pinned issuer, exact Keycloak
HTTPS authorization endpoint and client ID must be
configured; no keys or secret values are committed
to Git or pasted in chat. Current callback URL is
**fixed to authorized Mac SSH tunnel loopback**
`http://127.0.0.1:48765/lab/auth/browser/callback`,
and is specifically NOT public/customer/production.
The default owner preview has NO real user IdP or
membership configured, so it does NOT expose
these routes. The K3s pod path never mounts them.

Sample owner-controlled non-secret configuration
names: `IPAT_LAB_OIDC_ISSUER`,
`IPAT_R86_KEYCLOAK_AUTH_ENDPOINT` and
`IPAT_R86_PUBLIC_CLIENT_ID`. Do not set these
to a real issuer until independently checking
the issuer's metadata/signing-key ownership,
HTTPS/TLS chain, consented registration and
two-factor semantics. The path suffix for
this initial demo is the Keycloak candidate
`/protocol/openid-connect/auth`; full
provider-neutral discovery, code exchange
and rotating JWKS are NOT implemented.

The local HTTP correlation cookie intentionally
has no production `Secure` attribute because
this proof is bound to a loopback HTTP URL
over the owner's separately established SSH tunnel.
Before true real customer login, use validated
public HTTPS with `Secure` + `__Host-` session
cookies, OIDC metadata/discovery and confidential
server-side client handling, PKCE code redemption,
issuer/audience/nonce validation, robust key rotation,
TLS-certificate and redirect validation, session
rotation/revocation, CSRF and production
browser/SSO threat review. A real human MFA
enrollment/challenge plus separate exact
tenant/role/POP approved PostgreSQL membership
is an additional independent authorization
requirement before a reviewer or NOC may
see actual device records or perform operations.

## Reproducible tests (nonroot Ubuntu 26.04)

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
python3 -m unittest discover deploy/scripts/lab/r86 -p test_r86_contract.py -v
bash -n deploy/scripts/lab/r86/browser-http-smoke.sh
python3 -m py_compile deploy/scripts/lab/r86/browser_http_smoke.py
cargo build --locked --offline -p control-api
IPAT_R86_BROWSER_TEST=YES bash deploy/scripts/lab/r86/browser-http-smoke.sh
```

The real independent `urllib` smoke never follows
the synthetic HTTPS issuer redirect. It generates
a disposable RSA key pair locally and starts the
ACTUAL compiled Rust Axum app at `127.0.0.1:3001`.
Its HTTP assertions verify 303 with complete
S256/nonce/state/redirect/cookie, hostile Host
403, missing/wrong/duplicate correlation
cookie, unknown query rejection, one-use
callback 503 and replay 403, and existing
business API HTTP401. Its temporary key,
listener and files are removed at exit.

Existing real disposable PostgreSQL16
two-tenant candidate and metadata maker-checker
tests are still mandatory; passing this
browser PKCE start proof cannot substitute
actual customer human MFA onboarding.

## Next mandatory milestone and PRD deviation

**MUST next:** owner-approved real OIDC IdP
with a human second-factor enrollment test;
proper HTTPS confidential Authorization Code
+ S256 PKCE code exchange at the server,
validate signed ID token nonce, issue a
short-lived host-only secure server session,
connect session identity to the ACTUAL
restricted signed/verified tenant+POP
PostgreSQL reader/registrar/reviewer,
then render role-filtered tenant dashboards
and matching real server backend controls.

**MUST after that:** independently authorized
OLT/ONT read-only identity binding/firmware
inspection, verified evidence freshness
and health/diagnostic display. Never
conflate adding a registry draft
with connecting to an actual physical device.

**LARGE RED PRD GAP:** the current extra
browser flow always rejects completed
callbacks. No real operator login or
MFA was configured or tested, and the
device-control platform is NOT READY
for real physical adoption or commercial
production. Missing prerequisites
remain explicitly blocked.
