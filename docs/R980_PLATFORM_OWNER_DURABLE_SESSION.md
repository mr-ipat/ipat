# R9.80 — Platform Owner durable browser session: SQL security boundary

R9.73 implemented a suspended-only company reservation, but no independent Platform Owner browser session. R9.80 adds an isolated, Host-bound, digest-only, short-lived session and current Platform Owner checks. It deliberately remains a SOURCE/DISPOSABLE milestone until real human MFA, HTTPS and the separate BFF are integrated and tested.

1. A separately approved platform console Host requires actual validated ownership and trusted HTTPS readiness before session issuance. Migration does NOT automatically provision any hostname or infer that a staging hostname exists.
2. A separate dedicated issuer database login starts without a password. The actual production issuer must independently verify signed OIDC ID/access tokens, nonce, real human MFA and current identity BEFORE issuing a session. PostgreSQL does not pretend to perform OIDC verification.
3. Sessions are bounded to ten minutes, four minutes idle and four concurrent current sessions. Only SHA-256 session/CSRF digests reach PostgreSQL. Every call checks current Platform Owner membership and the exact trusted non-disabled Host.
4. Suspended-only reservation, paginated listing and logout are callable only from verified Platform Owner session functions. Issuer/subject come from the existing validated session, not the browser payload. Mutations require independent CSRF.
5. Tenant API, tenant OIDC issuer, and Platform API logins receive no platform session issuance, raw table or cross-domain authority. The platform BFF cannot directly execute the prior raw reservation function.

Reproduce synthetic-only acceptance:

    IPAT_R980_SYNTHETIC_DOCKER=YES bash deploy/db/tests/r980_docker_repro.sh

The test creates disposable PostgreSQL16 without publishing a port, migrates the selected schema through 0032 and exercises current session/Host/CSRF, principal revocation, exact idempotent replay, cross-role denials and the existing commercial regressions.

NOT IMPLEMENTED/VERIFIED HERE: an actual separately authenticated Platform Owner OIDC HTTP process/UI, production TLS/DNS, customer activation, real MFA, production PostgreSQL or host/PG PITR restoration. This SQL-only milestone does not authorize public access.
