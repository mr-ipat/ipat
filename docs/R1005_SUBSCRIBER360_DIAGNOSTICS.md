# R10.05 — Tenant-isolated Subscriber 360 and evidence-based diagnostics

R10.05 implements the first commercial Subscriber 360 + diagnostic slice on the existing tenant, POP, Site and managed-device masters. It does not configure PPPoE, ONTs, OLTs or routers and does not expose credentials.

## Subscriber 360 contract

- The canonical ipat_ops.subscribers table is extended rather than replaced. A tenant-admin may declare an exact Subscriber ID, display name, optional PPPoE username, exact registered POP/Site, required distribution router, optional access OLT/ONT and an opaque ONT reference that is not a serial/password field.
- POP/Site/device foreign keys are tenant-compound. Distribution/access devices must belong to the exact tenant/Site and be current non-archived metadata.
- New or changed topology is declared, never silently verified. A distinct non-login topology verifier may promote only the exact current tuple with bounded timestamp and evidence SHA256. topology_declared_at blocks stale evidence after a topology change.
- Tenant Admin has CAS create/update. Scoped helpdesk/auditor/NOC readers receive only currently authorized Subscriber 360 rows.

## Diagnostic evidence contract

A separate ingest executor stores only normalized bounded signals. Subscriber-specific and neighbor evidence must identify an exact subscriber on that distribution path. Missing CWMP alone never implies fiber damage.

Each event stores tenant/path identifiers, normalized signal, bounded source identifier, timestamp and evidence SHA256. No raw CLI transcript, CPE credential, PPPoE password or packet payload is stored. Browser freshness is limited to 1–3600 seconds.

Rust diagnostic-core returns distribution-path, ONT/access, PPPoE-authentication, conflicting-evidence or insufficient-evidence. Distribution, ONT and PPPoE fault-domain hypotheses require current verified topology. Every result requires operator review and remediation_permitted is always false.

## Authorization and UI

/api/v1/capabilities exposes separate can_read_subscribers, can_manage_subscribers and can_read_diagnostics. Read-scoped users see Subscriber 360/Diagnostics without edit controls; Tenant Admin sees declaration controls. Direct API calls reauthenticate the current Host-bound session and PostgreSQL functions re-evaluate current scope.

## Acceptance

- Disposable PostgreSQL16 selected commercial regression plus R10.05: 59/59 PASS after audit hardening.
- PostgreSQL18 production migration manifest includes migration 0035 with exact SHA256 and source bootstrap checks.
- Node UI contract PASS. Full Rust, clean PostgreSQL18 first-install and exact-head GitHub CI remain required before source closure.
- Physical telemetry producers remain independently unqualified until actual device tests pass.

## Pre-publication isolated evidence
Exact candidate source passed pinned Rust1.98.1 formatting, diagnostic-core 11/11 and full locked offline control-api 113/113 on a clean rootless owner-VPS checkout. A clean pinned Ubuntu26/PostgreSQL18 first-install rehearsal applied the 26-file manifest including migration 0035 and passed all restricted local-socket identity/no-TCP checks. These are source/disposable proofs, not production telemetry or public tenant acceptance.
