# R10.33 — Tenant dashboard partial-service availability and deny-by-default

Date: 2026-10-15 Asia/Jakarta.

## Root cause

The commercial Tenant dashboard previously fetched `/api/v1/capabilities` (current membership), then fetched the optional Device Catalog and Branding data within **one** bootstrap try/catch. An HTTP 503 from either secondary service was mislabeled "Sign-in required" and showed a sign-in link, even though tenant membership had just been accepted. The bootstrap could also unhide Devices before the catalog had been loaded/verified. These were UI availability and operator-error bugs, **not** a reason to relax backend access control.

## Correction

- Preserve authoritative capability checks; all menus default hidden.
- Tenant administrators can still use independently authorized POP/Site/Domain/NOC modules during a Device Catalog outage, but **Devices stays hidden** until the server-curated metadata catalog is successfully loaded and verified.
- An unavailable Branding read hides the Branding editor and keeps already-verified authorized operational modules accessible; default branding remains local text.
- A 401/403 from capabilities or a dependent authorized endpoint **hides every module**, closes the active panel and offers sign-in. HTTP 503/unavailable network during initial capabilities verification similarly hides all modules but does **not** falsely claim expired credentials.
- Server-side role/tenant/POP checks, Host-bound sessions and CSRF remain the only source of authority. Client-side degradataion does not grant permissions or permit actions on unqualified equipment.

## Acceptance coverage

- R10.33 source contract is included in pre-existing R9.70 Node CI.
- R10.33 isolated actual-Chrome test uses the commercial source and a loopback-only synthetic API. Cases: Device Catalog HTTP 503 blocks Devices but not independently authorized Sites/POPs; Branding HTTP 503 leaves permitted Device module but hides Branding and avoids incorrect sign-in; capabilities HTTP 401 hides every menu and requires reauthentication. Chrome runner CI is configured to execute these cases after the R10.32 browser regression suite.
- **Do not claim PASS until exact-head CI jobs are observed.** No actual owner VPS commercial Identity/DB/TLS is available at authoring; Remote Desktop Commander remains offline. This is source-level UX/security hardening, not a public deployment.

## Remaining release conditions

The real production PostgreSQL18 HA/PITR, independent full-host restore attestation, recovery root review, trusted TLS Host/SNI and MFA-backed Platform Owner + tenant login, dual-tenant CRUD and exact physical device authorization must still pass. The separately verified R10.29 owner-private C320 read-only lab service is not a replacement for those conditions. PUBLIC_GO=FALSE.
