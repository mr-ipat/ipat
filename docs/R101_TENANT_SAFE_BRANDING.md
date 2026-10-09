# R10.1 — Tenant-safe baseline company branding

IPAT now has a deliberately constrained branding layer for the commercial tenant dashboard. The objective is to make each company visibly distinct without creating an arbitrary HTML, CSS, image, or tracking surface.

## Model

Migration 0042_tenant_safe_branding.sql adds one branding row per tenant:

- display_name: visible company name, 1–80 characters, trimmed, no control characters.
- mark_text: short text or initials, 1–12 characters, trimmed, no control characters.
- accent_token: one of six source-controlled tokens: slate, blue, indigo, emerald, amber, rose.
- compare-and-set revision, current updater identity, and timestamp.
- append-only request events keyed by caller-generated UUID for exact idempotent retry.

No logo URL, inline image, external font, HTML fragment, JavaScript, arbitrary CSS, color value, or remote media URI can be stored by this milestone.

## Authorization

Read access is available only to a current active browser member of that exact tenant with one of the current supported roles. Mutation is restricted to a current tenant_admin. The database independently rechecks tenant state, membership revocation and expiry, and approved reviewer.

The browser never submits an authoritative tenant UUID for branding. The commercial API derives tenant identity from the already authenticated exact Host-bound durable session. The tenant API login has function-only execution and no raw branding, event, membership, or tenant-table rights. The tenant OIDC issuer and generic application role cannot call the branding setter.

## HTTP and UI

GET /api/v1/branding returns exact tenant branding, or a safe fallback derived from tenant_slug when no custom branding exists. PATCH /api/v1/branding requires current Host-bound session, exact HTTPS Origin and CSRF, UUID request id, current revision, and validated fields. Stale revisions and contradictory replay are rejected.

The dashboard applies display_name and mark_text with DOM textContent. The theme is selected only through a source-controlled data-accent token. Branding navigation remains hidden unless current database capability reports Tenant Admin, while other current members can still see the already-approved company brand in the header.

## Acceptance

Required before source completion:

- PostgreSQL 18 isolated security chain covering cross-tenant, non-admin, revocation and suspension, idempotent replay, stale CAS, and raw-table denial.
- Real Axum router plus disposable PostgreSQL Host-bound Tenant Admin save and same-tenant Helpdesk read, wrong-Host and missing-CSRF denial.
- Static Node contract proving safe DOM text handling and the finite theme catalog.
- Full locked Rust regression, PostgreSQL 18 clean first-install migration manifest, and exact GitHub CI.

This milestone is baseline text branding only. Advanced logo uploads and media, package catalog, billing, and actual public two-tenant human-MFA visual acceptance remain separate work.

## R10.15 integration note

Historical standalone R10.1 branding migration path `0035_tenant_safe_branding.sql` conflicts with canonical Subscriber360 migration 0035. The canonical additive integration uses `0042_tenant_safe_branding.sql`; earlier production migrations remain immutable. Integration with recent tenant operations and real human-MFA/public deployment remain separate release gates.
