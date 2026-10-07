# R10.08 — C320 access/PON aggregate evidence to Subscriber360 diagnostics

## Goal

Use the already physically accepted aggregate DEV-01 ZTE C320 PON 1/1/1 count shape as one bounded diagnostic signal without exposing ONU IDs, serials or subscriber payloads and without turning that signal into an automatic root-cause claim.

This milestone does not ingest the private DEV-01 observation into production PostgreSQL. The production database/runtime is not deployed on the current VPS. It adds the exact normalized data contract and inference rules that a future approved telemetry worker must use.

## Semantic contract

The signal access_pon_all_configured_offline means all of these are true for one exact OLT/access-device observation:

- configured is greater than zero;
- online equals zero;
- offline equals configured;
- evidence is recent and has a SHA-256 digest;
- exact tenant distribution router and exact access OLT exist in the same Site/device scope;
- no subscriber identifier is stored on the aggregate observation.

It does not mean distribution uplink down, fiber cut, OLT unhealthy, or customer outage by itself.

diagnostic-core emits access_pon_segment only when current Subscriber360 topology is VERIFIED, the aggregate access signal is fresh, and at least two fresh subscriber_unreachable observations map through verified topology to the same exact access device.

If distribution-uplink-down evidence is also present for the same scope, the result is conflicting_evidence. Missing topology, one subscriber only, stale evidence, wrong access device, cross-Site device, or malformed counts remain insufficient or denied. Every result keeps requires_operator_review=true and remediation_permitted=false.

## Database boundary

Migration 0038_access_pon_aggregate_diagnostics.sql adds an optional access-device foreign key to normalized diagnostic observations and permits it only for the exact aggregate PON signal. A dedicated record_access_pon_aggregate_observation function is granted only to ipat_diag_ingest_exec. Tenant API and general application runtime retain no raw-table or ingest authority.

The function requires the distribution router and access OLT to belong to the same tenant and Site, and it accepts only the exact all-configured-offline count shape. No credentials, raw CLI, ONU IDs, serial numbers or subscriber payloads are stored.

## Pre-publication evidence

- diagnostic-core: 13/13 PASS.
- full control-api Rust suite: 113/113 PASS.
- selected disposable PostgreSQL16 commercial/tenant/domain/Subscriber360 suite: 62/62 PASS.
- existing physical DEV-01 aggregate evidence remains the R10.02 result: configured 72, online 0, offline 72, physical writes 0, serials false.

The historical sanitized R10.02 evidence motivates this signal shape but is not silently inserted into any production tenant.

Still required: exact-head CI, PostgreSQL18 clean first-install verification with migration 0038, production telemetry worker identity, real tenant mapping, real production database, and external browser/diagnostic acceptance.
