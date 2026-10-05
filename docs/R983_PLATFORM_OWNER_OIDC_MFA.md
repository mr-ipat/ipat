# R9.83 — Dedicated Platform Owner OIDC/MFA browser issuer

R9.80–R9.82 provided the current Platform Owner PostgreSQL session, API/dashboard and temporary literal-IP host model but intentionally had no real browser login issuer. R9.83 closes that source boundary with a separate confidential Authorization Code + PKCE issuer process.

## Security and flow

- Dedicated exclusive nonroot IPAT_R983_PLATFORM_OIDC_ISSUER_SERVICE=YES process binds only 127.0.0.1:3006. It cannot coexist with Platform Owner API, tenant API/issuer, DNS verifier, C320/private lab, K3s lab or other device services.
- Exact pinned HTTPS IdP issuer, confidential client ID+secret and exact pinned RSA signing key/KID are configured only from reviewed deployment files/values. Secret/key files must be absolute, owner-only directory mode 0700 and file mode 0600, single hard-link, no symlinks.
- Browser route /platform/auth/oidc/start enforces exact Host and creates 180-second server-side PKCE state. It redirects only to the pinned issuer, includes S256 challenge, nonce, prompt=login and max_age=300. No access token, secret or PKCE verifier enters URL/logs.
- Callback /platform/auth/oidc/callback accepts exact state/cookie/code, one-use in-memory PKCE proof, fixed HTTPS token endpoint without redirects, bounded response and default system CA. The identity verifier requires matching signed access+ID token issuer/client/subject, nonce and at_hash, bounded token lifetime, fresh auth_time, and signed amr containing mfa.
- The verified token returns only issuer+subject+expiry. It does NOT accept realm roles/tenant claims. PostgreSQL issue_platform_browser_session independently requires an unrevoked separately approved platform_owner principal plus exact preapproved platform_console_hosts record. Session issuance uses only ipat_platform_session_issuer_login; tenant issuer has no execute right.
- Platform session and CSRF secrets are fresh independent 256-bit URL-safe values; PostgreSQL stores only SHA-256 digests. Max platform session is ten minutes and further bounded by the signed token expiry. Browser cookies remain Secure; session is HttpOnly and SameSite Strict; OIDC state is HttpOnly SameSite Lax only for IdP callback.
- Platform API root now exposes only a same-origin “Sign in securely” link to /platform/auth/oidc/start; it cannot mint or bypass authentication.

R9.83 deliberately uses one explicit singleton issuer for the short PKCE pending map. Restart safely invalidates pending logins; durable browser sessions remain PostgreSQL-backed. Horizontal HA of this login issuer is LATER and requires a dedicated platform pending-state store before multiple issuer replicas. The Platform Owner API itself remains separately scalable from this login edge.

## Evidence and non-goals

Actually exercised before GitHub publication:
- exact-source rootless pinned Rust 1.98.1 full control-api suite: 110/110 PASS;
- synthetic RSA signed ID+access pair with current MFA/auth_time, approved Platform Owner principal+console Host in disposable PostgreSQL16, ISSUE-only session creation, API-role validation, immediate principal revocation denial and explicit tenant-issuer denial: 1/1 PASS;
- initial Docker DB routing fixture failures were resolved by using a dedicated disposable Docker network, not by relaxing authorization;
- all temporary PostgreSQL/network fixtures are removed after acceptance.

This does NOT prove a real human IdP tenant exists, a real Keycloak/other provider client has been provisioned, a human has completed MFA, or public HTTPS is active. Production acceptance requires an independently controlled IdP client, actual human Platform Owner login with MFA, callback over browser-trusted HTTPS, logout/revocation tests, external hostile Host/session replay tests, and full recovery evidence. Never commit client secret, MFA seed or real tokens.
