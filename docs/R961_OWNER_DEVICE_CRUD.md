# R9.61 — Practical, persistent owner-private device inventory (English-first)

## Operator workflow

`http://127.0.0.1:3002/lab/device-workbench#managed-devices` while the existing owner Mac SSH tunnel is open. The top "Managed Devices" section automatically lists the already connected C320 **DEV-01**, using the actual private connector status, and all metadata-only devices saved via the new real VPS backend. Users can Add, View, Search, Edit and Remove nonconnected device records with immediate durable response. New C-DATA OLT, VSOL/ZTE ONT and MikroTik records may be saved as metadata regardless of vendor adapter support; always display `SAVED_NOT_CONNECTED` until actual separate authentication/physical read is implemented and qualified.

C320 supports Edit Display Name as a durable inventory presentation overlay; its existing sealed SSH credentials, fixed endpoint and active connector stay unchanged. Connected C320 Remove is disabled in the UI **and rejected by the API** with `ACTIVE_CONNECTOR_REQUIRES_SEPARATE_REVOCATION`. Do not unlink or silently wipe the physical connector to fake a delete; a separate owner-approved disconnect/credential-revocation workflow is needed. Metadata-only Remove requires a confirmation and server-checked `expected_revision` + exact `REMOVE <device-id>` request. A successful delete leaves an in-file immutable logical audit event. Editing stale data must fail HTTP 409. Browser never stores secrets or inventory in local storage. All backend routes mount ONLY if `IPAT_R940_PRIVATE_OWNER_READ=YES` and private :3002 canary safety gates are active; `/v1` tenant production API stays denied pending real OIDC/MFA+Host-bound sessions+PostgreSQL RLS.

## Source and tests

- `apps/control-api/src/owner_device_registry_lab.rs`: strict private host/origin/header, fail-closed schema validation, bounded 64-device/2048-event owner file, fsync atomic replacement, symlink/perms defense, optimistic revision and duplicate endpoint rejection, live C320 status composition, real GET/POST/PUT/DELETE.
- `apps/control-api/src/main.rs`: conditional mount alongside the owner C320-only bridge.
- `web/lab/owner-device-inventory.js`, `web/lab/device-workbench.html`, `web/lab/device-workbench.css`: English-first operational UI for actual persisted state, search/detail and normal actions.
- `deploy/scripts/lab/r961/test_owner_inventory_ui.cjs` and inline Rust HTTP/security/persistence/restart tests. Run `node deploy/scripts/lab/r961/test_owner_inventory_ui.cjs`, `cargo fmt --all -- --check`, `cargo test --locked -p control-api`, full GitHub CI and actual private HTTP lifecycle smoke on owner VPS. Testing must not remove DEV-01, credential files, change firewall or touch customer devices.

## Explicit constraints / next implementation

This is a durable, **useful private pilot**, not yet a commercial multi-tenant CRUD release. An owner-local SSH tunnel is the lab access boundary; it is not equivalent to full authenticated tenant authorization. The existing tenant-managed PostgreSQL schema `0016` has Save/List metadata foundations but live HTTPS/OIDC/MFA, host-bound session and update/remove grants/migrations are still required. Do not expose this owner-only JSON registry via a public ingress or copy its synthetic saved records into tenants without verified tenant ownership. Metadata-only devices are never auto-connected or eligible for vendor writes.
