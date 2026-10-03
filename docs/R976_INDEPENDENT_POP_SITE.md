# R9.76 — Independently administered POP master and explicit tenant Site association

The tenant operator problem was a structural ambiguity: legacy managed_device.pop_id actually referenced the existing tenant Site code, while legacy NOC grants also used an exact Site code. Changing only a label or silently promoting a Site code into a real POP could widen NOC read permissions or incorrectly move historic equipment.

## Supported future commercial operator flow

- POPs is a first-class independent tenant module with its own create, list, rename, guarded removal, CAS revision and append-only tenant-scoped audit. POP code cannot be reused after deletion; historical identity is retained in the audit.
- Sites remains a separate independent tenant module. A Site may have exactly one optional explicit parent POP. A new Site with selected POP is created atomically with a same-tenant compound FK; old records remain unassigned and are never guessed or mapped automatically.
- Existing Sites may be explicitly associated, reassociated or unassigned using tenant-admin session, CSRF, a compare-and-set Site revision, the exact selected same-tenant POP and append-only actor attribution. Devices still select their registered Site, not a POP; no device state or actual hardware setting changes when linking.
- Deleting a POP with assigned Sites is denied by both the business function and the compound FK. Archiving physical equipment and deleting a Site remain governed by their pre-existing independent controls. Second-phase migration 0029 validates the staged FK, subject to independently approved production backup and maintenance.
- Site/POP controls remain hidden for non-admin actors and all routes independently authenticate a current Host-bound session; database functions recheck active tenant-admin membership. Direct NOC POP-CRUD access is denied.
- **Legacy NOC access does not silently expand:** existing identity_pop_grants.pop_id remains an exact historical Site code, even after that Site is assigned to a new real POP. The new real POP hierarchy does not itself issue or expand NOC grants. A separately reviewed, typed real-POP grant migration/workflow and exact NOC authorization tests remain a later mandatory requirement before declaring full POP-scope production coverage.

## Reproduction and limitations

IPAT_R976_SYNTHETIC_DOCKER=YES bash deploy/db/tests/r976_docker_repro.sh creates no public PostgreSQL port, applies selected reviewed migrations 0001–0029, then tests admin roles, NOC non-escalation, two-tenant same-code distinct POPs, linked and unassigned Sites, atomic parent resolution, CAS stale-revision denial, protected deletion, tombstone/reuse and append-only audit. R9.70 real Axum + disposable PostgreSQL HTTP integration is extended to cover admin and NOC user flows. The exact test results and GitHub CI/merge will be recorded in PROJECT_STATUS only after successful execution.

This does NOT declare physical POP network topology discovered, actual device movement, qualified OLT/ONT/Routers, public SaaS deployment, real customer MFA or any proprietary device operation complete.
