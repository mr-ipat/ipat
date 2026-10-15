# R10.32 — Exact Device Save read-after-write acceptance

Date: 2026-10-15 Asia/Jakarta

## Defect and remediation

The commercial Tenant dashboard previously treated `POST /api/v1/devices` followed by any HTTP-200 response from `GET /api/v1/devices` as a successful Save. An empty or stale tenant inventory could therefore clear the form and show success even when the **exact newly created device ID** was missing. This violates the operator requirement that Save must result in a visible inventory entry.

Fix: keep the existing per-intent `request_id` and input fields until the successful `POST` response's saved ID is present in the refreshed tenant-scoped inventory in lifecycle `SAVED`. The comparison is by exact ID, never display name, Site label, or vendor. On missing ID, empty list or GET failure, show "Device Save could not be reconciled" and instruct an unchanged retry using the same idempotency key. No direct device command, credentials or token is stored on the client.

A successful POST is not evidence that a physical OLT is Connected. The UI continues to distinguish metadata SAVED from physical adoption, and the server still validates the current authenticated tenant, role, Site, curated catalog, CSRF and request key. Database migration not needed: PostgreSQL already supports same-request replay.

## Coverage / truthful statuses

- `deploy/scripts/lab/r1032/test_device_save_readback_contract.cjs`: executes the **production visibility predicate** in Node's isolated `vm` with positive exact SAVED ID and negative empty list, wrong ID, ARCHIVED ID, null/numeric ID. Also asserts reconciliation order: list refresh and ID verification precede clearing form and pending key.
- Added to existing R9.70 CI dashboard contract.
- Updated R10.30 synthetic Chrome fixtures. New scenario models POST201 → GET200 empty → safe unchanged retry POST201 → GET200 containing exact ID, and verifies same request ID with no premature success. Existing POST503 and list GET503 scenarios also now return saved records on subsequent successful list reads.
- **Execution pending:** GitHub exact-head CI must run; actual Mac Chrome scenario has NOT been executed for R10.32 because Remote Desktop Commander was offline when this source was authored. Older R10.30 Mac Chrome evidence does not qualify this updated path.
- **Production status:** NOT DEPLOYED and NOT VERIFIED on commercial VPS. The public environment still lacks approved PostgreSQL18 HA/PITR, reviewed root-owned recovery evidence, human MFA/TLS, and tenant DNS Host routing. PUBLIC_GO=FALSE.

## Follow-up

Once authorized Mac/VPS control returns, rerun `node deploy/scripts/lab/r970/test_tenant_dashboard_contract.cjs` and `python3 -m unittest discover -s deploy/scripts/lab/r1030 -p 'test_dashboard_browser_flow.py' -v` with Chrome. Then deploy to commercial environment **only after** root recovery, database, trusted TLS, and MFA prerequisites and verify real Save/read for two distinct tenant identities with direct cross-tenant API rejection. Do not represent synthetic browser evidence as commercial release acceptance.
