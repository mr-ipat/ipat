# IPAT R10.30 — Tenant and Platform Dashboard UX Audit

Date: 2026-10-15 Asia/Jakarta. Baseline: canonical GitHub main 7afb09f1. Source: Tenant/Platform HTML, CSS and JS. This is a source and synthetic browser audit, NOT a live authenticated production audit.

## Findings and corrections

| Priority | Evidence | Implemented correction |
|---|---|---|
| P0 | Device Save created a new idempotency request ID on each HTTP retry, allowing duplicates after ambiguous server commit. | Reuse one request ID for unchanged form data until acknowledged. Different fields get new intent. Existing PostgreSQL enforces same-key replay. |
| P1 | Empty Site selector interrupted Add Device and did not return to registration. | Clear no-Site guidance; independent Site create; automatic return to Devices with new Site selected. POP remains independent. |
| P1 | PPPoE success banner told operators another Tenant Admin must approve, contrary to role policy. | Correct text to distinct Security Admin; no change to approval backend or worker readiness. |
| P1 | Tenant navigation lacked current-page state and keyboard focus, and async completion could overwrite newer status. | ARIA current page and controls, focus on panel heading, generation guard for stale status messages. |
| P1 | Tenant/Platform had no skip link or explicit focus-visible styles. | Semantic skip-to-main, keyboard focus styles, responsive tables/forms, reduced motion and minimum control sizes. |

## International baseline and limits

Target WCAG 2.2 A/AA, especially keyboard navigation, perceivable focus, form labels and instructions, and understandable status feedback. Sources: https://www.w3.org/TR/WCAG22/ and https://www.w3.org/WAI/tutorials/forms/labels/ . This milestone is **not a claim of full WCAG conformance**. Dedicated screen-reader, 200% zoom, visual contrast, localization and physical keyboard tests remain outstanding.

## Executed verification

- Node JavaScript parse and CI-invoked static authorization/UX contract PASS.
- Actual Chrome headless on operator Mac over an isolated local synthetic HTTP API: 3 PASS, testing capability-denied menus, Site-missing/create/resume, and Device Save HTTP 503 then 201 with identical request UUID and correct Site.
- No real identity session, DNS, TLS, PostgreSQL production, customer records, devices, firmware or network changes. The private C320 dashboard R10.29 remains separate and operational.

## Residual PRD acceptance

Real production browser Host/SNI/MFA, tenant role changes, direct API denial, expiry, Site and Device CRUD persistence, Screen Reader/keyboard and viewport/contrast testing remain required before commercial GO. Browser request IDs persist only during the current page lifecycle; cross-reload ambiguous submissions need server-side reconciliation before full production acceptance. No PRD security requirements were removed.
