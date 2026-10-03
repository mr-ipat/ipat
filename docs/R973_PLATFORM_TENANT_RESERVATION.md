# R9.73 — Platform Owner creates a suspended company reservation

This milestone is a SQL source and disposable-database acceptance boundary, not a public Platform Admin dashboard, automatic subscription or a live customer tenant.

## International-standard company onboarding sequence

1. A separate platform OIDC/MFA server must cryptographically verify Platform Owner, resolve the owner from the independent platform principal store and bind the request to a platform-controlled hostname. Browser fields or a tenant-admin session can never authorize platform operations.
2. Platform Owner selects a unique company slug. Backend generates a request UUID and tenant UUID server-side, then calls a dedicated NOLOGIN platform executor with the verified issuer and subject. SQL rechecks current approved unexpired nonrevoked platform identity.
3. The reservation function creates only a Suspended tenant and an actor-attributed, immutable, unique request record. Exact replay is idempotent. Replays with changed tenant, slug or actor fail, as do duplicate slugs, expired/revoked principals and tenant admin or OIDC issuer attempts.
4. A separate audited review must verify contract, tenant-admin invitation acceptance, distinct tenant MFA account, platform-authorized hostname and verified DNS/TXT/ACME/TLS before a future activation workflow may change Suspended state.
5. Company activation, invitation, billing, branding and Platform Admin HTTP/UI are NOT implemented in this migration. An unactivated reservation cannot resolve as an active commercial Host-bound tenant session.

Dedicated NOLOGIN execute role is NOT inherited by the existing commercial tenant API or session issuer. Before any HTTP route uses it, separately verify live platform IdP/MFA, host ownership, CSRF, audit and request rate limits.

Reproduction: IPAT_R973_SYNTHETIC_DOCKER=YES bash deploy/db/tests/r973_docker_repro.sh. Disposable no-published-port PostgreSQL16; selected canonical ordered migration dependencies to 0026 and 17 SQL scope/replay/negative tests. No physical equipment, public listener or owner-VPS production database is changed by the reproducer.
