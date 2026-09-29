# R9.44 — IPAT C320 operational GUI based on functional requirements research

Date: 2026-09-29. Source basis: `IPAT_PROJECT_BRIEF.md`, latest `docs/PRD.md`, `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/DEVICE_MATRIX.md`, `docs/DECISIONS.md`, and `docs/PROJECT_STATUS.md` from current project feature branch. Reference for user-visible feature categories only: public `https://zetset.id/`. Do NOT copy another vendor's code, artwork, protected UI or device-compatibility assertions.

## Reference feature mapping and evidence

The reference vendor publicly advertises multi-OLT monitoring, ONU activation, real-time PON/ONU traffic, topology mapping, auto backup/restore, ACS URL push, NAT-reachable VPN, per-technician RBAC/audit, sync, OpenAPI, reusable VLAN/GEM/T-CONT and browser terminal. These are **vendor claims on its own website**, not proof IPAT implements them or proof all firmware supports their operations. IPAT adds original Rust CWMP ACS + native USP, tenant isolation, fleet diagnostics and supervised approval. Existing IPAT lab has reliable manual C320 inventory count evidence on PON 1/1/1 (72 configured, 0 online, 0 unconfigured in submitted 2026-09-29 snapshot), historically authenticated C320 card and firmware reads, and a separate *private only* temporary agent that is **not** commercial adoption.

## Delivered as R9.44 source (deployment NOT YET verified)

- New `web/lab/c320-operator-console.js`: read-only operational dashboard controlling **existing** private `GET /lab/c320-owner-agent-state`, `GET /lab/c320-owner-manual-onu-snapshot`, `POST /lab/c320-owner-live-refresh`, `POST /lab/c320-owner-live-cards`, and `POST /lab/c320-owner-live-firmware`. Strict JSON validation and text-only DOM updates, explicit actual/manual timestamps, status, session/command activity, no dynamic network endpoint, serial, arbitrary CLI or credential input. A real physical success is displayed ONLY upon successful authenticated bounded owner-agent command response with timestamp; failed physical read never converts historical manual data into live telemetry.
- `web/lab/device-workbench.html`: a new prominent operational GUI before the legacy adoption diagnostics including ZTE C320 DEV-01 device selector, exact supported PON 1/1/1 selector, four ONU summary counters, two hardware metadata counters, busy/offline session status, three *bounded read* action buttons and an explicit high-risk-write-denied warning.
- `web/lab/device-workbench.css`: responsive operator GUI on desktop/mobile; no screenshots, styling or source borrowed from reference competitor.
- `apps/control-api/src/device_workbench_lab.rs`: serves the new JS through the existing private-only workbench router with original no-store headers and in-process route check. **Existing physical device rejection APIs and tenant security unchanged.**
- `deploy/scripts/lab/r944/test_operator_console.cjs`: statically checks private route wiring, allowed fixed operations, known supported PON, original physical endpoints, and manual/live evidence separation. This source test alone is not end-to-end live device verification.

## Acceptance and honest limits

| Priority | Acceptance requirement | R9.44 source status |
|---|---|---|
| MUST | Human sees C320 status, historical manual 1/1/1 ONU counts, last evidence date, current agent presence, one-click only-existing approved card/firmware/PON summary operations | SOURCE CODE ADDED; runtime integration & physical click NOT VERIFIED |
| MUST | Unsupported PONs disabled; failed/offline reads never masquerade as success; serials/credentials never rendered | SOURCE GATED; independent HTTP/user browser tests still needed |
| MUST | Existing privileged physical writes remain rejected; user no arbitrary CLI; original producer worker unchanged | DESIGNED / CURRENT PHYSICAL POST DENIAL historical evidence; regression on new deployment pending |
| MUST | Runtime operator panel actually loads JS after rebuild, under private :3002/SSH tunnel; three buttons yield real UTC evidence while owner agent is active | NOT YET VERIFIED |
| MUST | Real hardware implementation obtains separate least-privilege device account, independently attested host pin, per-tenant MFA+ABAC, immutable audit and native restore rehearsal BEFORE actual ONU/VLAN/reboot/firmware change | BLOCKED |
| SHOULD | Strict redacted per-PON/per-ONU/optical/history/traffic/alarm adapter by **physically verified C320 firmware command**, read-only first | NOT IMPLEMENTED |
| SHOULD | Permanent tenant-specific bounded collector/queue, cache with freshness TTL and reconciliation-before-retry; domain routing never implies authorization | NOT IMPLEMENTED |
| LATER | Fleet multi-vendor matrices (C-DATA first), alarm map, fault correlation, PPPoE staged mass job, firmware canary maintenance/rollback, full tenant dashboards | NOT IMPLEMENTED |

**Critical:** New source in GitHub is not deployed to VPS and cannot be described as functioning from the browser until runtime rebuild and HTTP smoke are observed. Owner recently reported repeated `BrokenPipeError` from R9.42; R9.43 concurrent-status fix is **deployed and its simulated IPC test passed**, but a real C320 read after the fix has not yet been observed via GUI. No unverified compatibility, customer-affecting reset, firmware upgrade or production ADOPTED status.
