# R9.81 — Independent Platform Owner HTTP and Suspended company dashboard

The canonical R9.80 session grants **Platform Owner-only database privileges**. This milestone makes a separately deployed Rust/Axum browser API and English Platform Owner dashboard consume only that restricted session. It does NOT reuse the tenant API, tenant issuer, or private C320 connector.

## Browser and API behavior

- GET / deliberately returns HTTP 503 with instructions that a separately approved real confidential OIDC/MFA issuer must be deployed first. A synthetic or header-based Platform Owner login would be a security vulnerability, not a completed feature.
- GET /platform/dashboard requires a current platform-only session bound to the exact deployment Host and existing trusted, pre-approved platform-console hostname. Unauthorized or revoked sessions cannot load dashboard HTML.
- GET /api/v1/platform/reservations lists bounded nonsecret company reservation metadata through the reviewed PostgreSQL session-bound function. There is no tenant data or device credential access.
- POST /api/v1/platform/reservations checks the exact Host, HTTPS Origin, separate CSRF and current database session. The browser retains its UUID pair for timeout-safe exact replay. The database rechecks identity and creates only a Suspended tenant. No tenant-admin account, custom DNS, billing or live customer activation is fabricated.
- POST /api/v1/platform/logout revokes the current platform session in PostgreSQL and expires both separate platform cookie names.
- The page hides controls until authenticated and uses textContent, not innerHTML. The backend and PostgreSQL, not the frontend, independently enforce rights.

## Deployment boundary

The exclusive IPAT_R981_PLATFORM_SERVICE=YES process binds only 127.0.0.1:3005, requires a nonroot identity, an externally reviewed TLS edge and exact platform Host, and uses only the dedicated ipat_platform_session_api_login via an absolute local PostgreSQL Unix socket. The process refuses simultaneous lab, DNS verifier, commercial tenant BFF or confidential login-issuer modes. Runtime environment opt-in does not constitute real HTTPS or DNS ownership proof.

Files: apps/control-api/src/platform_owner_api.rs, apps/control-api/src/main.rs, web/console/platform/, deploy/scripts/lab/r981/test_platform_console_source.cjs. Reuses reviewed migration deploy/db/migrations/0032_platform_owner_browser_sessions.sql.

Acceptance: locked Rust unit tests, browser-source contract, and the **actual router with disposable PostgreSQL16** after 0032 must pass separately. The isolated HTTP test covers missing sessions, wrong Host, missing CSRF, idempotent suspended-only company creation, contradiction on replay, read-only list and immediate logout. The database identities and sessions in tests are synthetic; do not treat them as real approved human MFA.

**Still pending and release-critical:** actual real Platform Owner OIDC/MFA confidential issuer; approved real platform Host DNS TXT/trusted TLS; production PostgreSQL HA/PITR and independently restored host backup; verified Rescue Console; public browser tests against distinct actual companies; customer activation and independently qualified physical device tuples. There is no public production GO from this milestone.

## Final exact-source acceptance (2026-10-05)

GitHub PR #174 exact reviewed head a09f69bad197bffef6419b9ddd57a8ad0a3b9e7d was built from twelve SHA-verified GitHub blobs with the same Git tree 3f53b9181a00ec900b1014ca6fea9eee415a3840 as the locally tested source. GitHub Actions run 37256543507 completed SUCCESS in all four jobs (unit-tests, postgres-rls-restore, postgres-physical-recovery-lab and disposable Ubuntu26 K3s). The postgres-rls-restore job specifically completed the new R9.81 real Rust/Axum HTTP against actual isolated PostgreSQL16 after migration 0032. PR #174 was marked ready and squash-merged with exact-head protection to main a90df9fc15b8adda356da10bd0d8268f3f6eb5a8.

Prior to exact CI, the owner-Mac static browser contract passed and a rootless isolated owner-VPS checkout of the exact R9.81 source passed pinned Rust 1.98.1 formatting, the named Platform Host/CSRF Rust unit test and the full control-api test suite 104/104. A temporary no-public-port PostgreSQL16 and reverse SSH loopback created for local test setup were removed without claiming local real HTTP success; the successful actual HTTP/DB test evidence comes from GitHub CI. Both owner-private C320 services stayed active, and no physical device operation or production PostgreSQL migration occurred.
