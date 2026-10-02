# IPAT R9.58 — Operator-controlled firmware implementation

## Source deliverables
- `deploy/db/migrations/0017_operator_firmware_workflow.sql`: tenant-scoped immutable firmware artifact metadata, per-device change intent, independent attestor-only evidence, separate `security_admin` approval, operator-only execution-intent transition, append-only events and exact restricted roles. Apply after 0016 using a verified maintenance/backups procedure, never against an unreviewed production DB.
- `deploy/db/tests/test_operator_firmware_workflow_integration.py`: disposable PostgreSQL synthetic authorization, request, cross-tenant, approval, attestation, event, duplicate and revocation tests. It contains no actual firmware bytes or device I/O.
- `apps/control-api/src/firmware_workflow.rs`: unmounted verified-session/CSRF/origin BFF primitives to stage artifact metadata and create/list/review/execute requests. Identity and tenant scope must be derived from a genuine production Host-to-tenant session, never browser headers.
- `crates/firmware-core`: exact-adapter contract and synthetic, fail-closed worker orchestration tests; `UnsupportedFirmwareAdapter` remains default.
- `web/console/firmware`: production-oriented interface source gated by a real BFF capability response. It is not deployed/publicly mounted yet.

## Safe tests
Run the pre-existing CI's ordered disposable PostgreSQL migrations/tests 0001-0016 first on a throwaway PostgreSQL 16 `ipat_synthetic` only. With the existing exact CI synthetic environment (`IPAT_PG_EPHEMERAL_TEST=1`, `PGHOST=127.0.0.1`, `PGDATABASE=ipat_synthetic`, `IPAT_PG_SYNTHETIC_PASSWORD=local_ci_synthetic_only`), run `python3 -m unittest discover deploy/db/tests -p test_operator_firmware_workflow_integration.py -v` then all downstream historical PG tests. Run `cargo fmt --all -- --check`, `cargo test --locked -p firmware-core`, `cargo test --locked -p control-api` and the complete locked workspace CI. No real firmware image or network device is required or contacted by these tests.

## Physical pilot and release gates
1. Deploy genuine MFA/OIDC, durable tenant sessions, verified Host routing, tenant-scoped PostgreSQL roles and encrypted artifact storage with server-side SHA-256 plus independently validated signed/vendor release metadata; register exact device+board+running versions.
2. Implement the separately authenticated attestor that records **actual** vendor artifact signature/manifest, device identity, backup/restore result, service-impact baseline, independent recovery console and physically qualified exact adapter. Do not let an operator-generated boolean or checksum impersonate any of these.
3. Mount protected API routes and role-filtered UI. Implement a durable, crash-safe outbox/worker lease and per-device lock; re-check all approval, object, version, maintenance-window, identity and recovery gates at actual dispatch. Show progress, pause/abort where vendor-supported, audit and post-upgrade live readback with alert on mismatch.
4. Validate on isolated non-customer ZTE C320 using the **actual** board list and manufacturer-approved exact image. Record hash/vendor provenance, backup/native restore drill, onsite console, signed two-person review, actual pre/post version and service health, and adapter/firmware-specific recovery. Only then mark this tuple supported. Repeat separately for C-DATA, ONTs and MikroTik RouterOS where the vendor-specific mechanism is established.

R9.58 source intentionally does **not** upload firmware bytes, provide a physical driver, enable automatic upgrades or change any live infrastructure. These dependencies must be closed before calling the full operator firmware ecosystem usable.
