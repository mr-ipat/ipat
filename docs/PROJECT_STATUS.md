# IPAT — PROJECT_STATUS (interim repository bootstrap)

**Date:** 2026-09-25 Asia/Jakarta
**Status:** R0 repository bootstrap / D1 safe Rust scaffold. This file is an interim status pending import of the complete previously prepared PRD v0.1 documentation package.

## Completed
- Created private GitHub repository `mr-ipat/ipat`.
- Prepared a new, isolated working directory; did **not** overwrite a pre-existing local folder named `ipat`.
- Copied the transferred brief byte-for-byte (SHA-256 verified against the upload).
- Added deny-by-default policy example, synthetic two-tenant unit tests, local-only Axum health route, minimal GitHub CI and secret-aware Git ignore.

## Scope and security
- **No server connection or deployment; no credentials collected, changed or committed.**
- Device interoperability: ALL UNTESTED. ACS and USP are NOT YET IMPLEMENTED.
- Neither database tenancy strategy nor MQTT broker selected; corresponding ADRs remain PROPOSED.
- Initial code is a foundation only; an authenticated OIDC adapter, DB RLS, high-risk approval and negative end-to-end tests must precede real device operations.

## Tests and evidence
- GitHub API access and private repository visibility verified. The first push was blocked because the GitHub OAuth credential lacks `workflow` scope; workflow excluded from the initial published commit pending explicit account reauthorization.
- Rust toolchain was not found on the connected workstation.
- Local Rust compile/tests: NOT RUN; remote CI: NOT RUN (workflow scope blocked).
- Physical and Ubuntu 26.04 tests: NOT RUN.

## Pending import / next actions
1. Import the complete 2026-09-25 PRD/architecture/security/ADR/device/deployment/backlog documentation package; reconcile this interim status, do not discard milestones.
2. Project owner signs off proposed ADRs (tenant PostgreSQL design, USP MQTT/broker, auth provider) before irreversible implementation.
3. Generate and commit Cargo.lock after dependency resolution.
4. Implement trusted identity verification and actual negative tenant API tests (no request header as authority).
5. Obtain exact lab model/firmware and Ubuntu server SSH access via least privilege.
