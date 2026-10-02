# R9.60 English-first C320 capability and execution center

## What actually works

In the existing authorized, SSH-tunneled owner-private :3002 dashboard, the fixed ZTE C320 OLT has newly completed credential enrollment and reports Connected. Exact physical tests 2026-10-02 show 3 active chassis cards and 5 firmware-inventory lines; firmware board/version mapping is not reconciled. The live PON ONU counts read returned HTTP 503 and remains a defect, despite overall C320 connectivity. New server-owned `GET /lab/c320-owner-action-catalog` enumerates 16 grouped operations with a live connection gate and ONLY two bounded executable read endpoints. The English section in `/lab/device-workbench` renders the backend catalog, displays prerequisites, and can invoke the existing exact `CARDS` and `FIRMWARE` read routes. It is **NOT** full OLT CLI support or writable OLT/ONT adoption; client cannot supply custom device address or command.

## User-visible controls and gates

- **Runnable, subject to current connection:** active line-card count and firmware inventory row-count. Results appear in-page with device timestamp and an explicit 'no physical writes' message. Actual per-board version identification is not yet qualified.
- **Degraded:** ONU/PON counts, because the most recent actual physical test failed HTTP 503. Historical manually observed ONU IDs cannot be presented as live per-ONU data.
- **Visible in private owner lab, unqualified/nonexecutable:** per-ONU status, optical signal, alarms, port traffic and statistics, device-native backup, ONU provisioning/deprovisioning, VLAN/service profiles, real ONT CWMP/USP management, controlled reboot, firmware upgrade, native configuration restore. High-impact controls MUST be hidden entirely in production for tenant users without permissions, not merely greyed out. Lab lists describe scope gaps rather than implying availability.
- **PRD:** English `en-US` default customer/operator UI. `id-ID` optional once localization catalogs and translations are validated. Historical private dashboard copy remains bilingual/noncompliant until a separate complete migration; do not advertise GA localization as completed.

## Files, tests and standalone verification

- Backend: `apps/control-api/src/c320_live_lab.rs` pure fail-closed catalog and GET route; `apps/control-api/src/device_workbench_lab.rs` separately served owner JS.
- Web: `web/lab/c320-action-catalog.js`, `web/lab/device-workbench.html` server-catalog-bound controls and in-page results.
- Tests: Rust `owner_catalog_never_authorizes_an_unqualified_command`, Node `node deploy/scripts/lab/r960/test_operator_capabilities.cjs`, pinned `cargo test --locked -p control-api`, full CI. Independently verify no execution URL for any write or unqualified read; failed authorization must reject in backend too.
- Live rollout: compile isolated pinned commit on nonroot VPS, checksum binary, retain r959 preview for rollback, replace only private owner canary :3002, smoke test live server catalog and verified read outcomes without invoking an unqualified command or altering root firewall/production database. The existing connector retains credentials and does not restart as part of UI rollout.

**Remaining requirements:** repair `REFRESH` parser from sanitized offline/vendor evidence and revalidate physical output; enumerate exact running board/software and supported read-only vendor commands; deploy real MFA/tenant Host-bound API, audit/logging, PostgreSQL worker and tested native recovery before any write action. Record all physical results in `DEVICE_MATRIX.md` by exact tuple. Never ingest physical secrets in code or chat.
