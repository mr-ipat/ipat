# R8.7 — Genuine pinned OIDC ID-token nonce + at_hash check, OFFLINE only

Date: 2026-09-27 Asia/Jakarta. Context: the product owner requires
**real authenticated tenant UI and safe device adoption before touching
real equipment**. The prior R8.6 PKCE S256 browser start/callback is
strictly local, synthetic and ALWAYS discards an authorization code
with HTTP 503, because no independently approved external human IdP
and confidential HTTPS code-token exchange have been provisioned.
Do not mistake R8.7 for live customer authentication.

## Actual original implementation

`crates/identity-core/src/oidc_id_token.rs` implements
`PinnedIssuer::verify_offline_browser_pair(client_id,expected_nonce,id_token,access_token)`
with cryptographically verified signed RS256 ID and access JWTs
against the SAME independently provisioned pinned issuer.
An ID token must target the exact approved browser client
(no array/surprising audience, mismatched authorized party or
untrusted token-supplied JWK) while the access token must
independently target the existing IPAT control API audience.

The ID and access tokens MUST agree on issuer and subject;
ID token nonce must match the separately generated R8.6
server-private unpredictable state (constant-time equality),
and `at_hash` MUST equal the left 128 bits of SHA256
of the exact verified access-token octets, encoded as
unpadded URL-safe Base64, also compared in constant time.
Both signed tokens require bounded `amr:["mfa"]`.
This is a conservative reviewed IPAT profile, not a
claim that arbitrary external OIDC providers use identical
MFA attribute semantics.

Both JWTs enforce pinned `kid`, exact RS256 algorithm,
disallow token header remote key URLs, validate strict
audience and HTTPS issuer, enforce expiry/not-before/issued
time, bounded token lifetime, and ID token `auth_time`
no older than five minutes. Any missing/bogus/stale
ID field, bogus key, tampering, typ confusion, wrong
client, wrong nonce, mismatched subject, mismatched
access token, false MFA or stale authentication
fails closed.

The opaque successful return type contains
ONLY independently verified issuer/subject and
minimum of the two signed expirations: it
intentionally has NO tenant, role, POP, admin
privilege, token material, browser session,
public exposure, serializer or Debug.
No SQL queries, HTTP requests, client secrets,
provider onboarding or network device contacts
occur in this verifier. Existing R8.6
callback still returns HTTP 503, and every
real business `/v1/*` endpoint remains
default denied. No real human MFA was
provided or performed.

## Reproducible nonroot Ubuntu 26 tests

```sh
python3 -m unittest discover deploy/scripts/lab/r87 -p test_r87_contract.py -v
cargo fmt --all -- --check
cargo test --locked --offline -p identity-core --test oidc_id_token
cargo test --workspace --locked --offline
```

Five genuine Rust tests independently generate ephemeral
2048-bit RSA pairs using openssl, sign separate
synthetic ID/access tokens and exercise correct
acceptance plus nonce, `at_hash`, wrong client,
missing/stale signed MFA and `auth_time`,
wrong RSA signing key/kid/token type,
wrong subject and timing-negative cases.
The source-level contract adds four separate
guards and the full existing CI runs
them on genuine Ubuntu, alongside the
existing real PostgreSQL maker-checker
and lab-only virtual device tests.
These are verified software tests,
not actual IdP provider conformance.

## Explicit remaining PRD MUST work

1. An independently consented, real HTTPS
   identity provider and real operator
   human MFA enrollment/challenge with
   documented meaning of signed `amr`.
   Reviewed provider secret/JWKS lifecycle,
   acceptable rotation and TLS controls
   must precede any token acquisition.
2. Confidential backend HTTPS authorization-
   code redemption with PKCE S256 and
   exact IdP token endpoint, TLS trust
   and response content-type; securely
   bind the *server-owned* nonce and
   real issued ID+access token using R8.7.
3. Host-only Secure HttpOnly browser
   sessions created ONLY after real MFA
   and database-approved exact
   tenant, role and POP; CSRF,
   revocation/rotation, generic login
   errors and logout. The existing
   two-company restricted SQL registry
   and maker-checker endpoints must use
   that identity via server-side ABAC.
4. Only then enable reviewed private
   read-only ZTE C320/C-DATA and ONT
   firmware-matched protocol adapters
   after owner proves actual route,
   vendor host identity and independent
   recovery/maintenance permissions.
   Real physical UNKNOWN/NOT_MEASURED
   must not be silently marked healthy.

**LARGE RED PRD DEVIATION:** Despite
R8.6 browser PKCE and R8.7 real signed
two-token verification code, IPAT
does NOT have an enrolled real IdP/MFA
operator or an authenticated customer
browser session, production Device
Manager, full actual ACS/USP device
connection, live network condition,
firmware approval or full verified
offsite disaster recovery.
