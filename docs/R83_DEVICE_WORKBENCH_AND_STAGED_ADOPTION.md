# IPAT R8.3 — Visible Device Manager, tenant-scoped adoption drafts and truthful status

**Scope:** September 27, 2026. Owner requested actually usable dashboard
**Tambah Perangkat / Daftar Perangkat / Kondisi Perangkat** before the
physical ZTE C320, C-DATA OLT, ZTE/VSOL ONT and MikroTik are powered.
This release provides a genuine local browser→Rust demonstration for all
three UI operations, and a separately gated *genuine PostgreSQL*
candidate-registration API tested with real signed short-lived
synthetic JWTs on an isolated disposable PostgreSQL 16 service.

**It does not claim the real OLT/ONT is adoptable or online today.**
No physical device was contacted; no production PostgreSQL, actual
MFA, customer dashboard, real CPE credential or high-risk firmware
actor was provisioned.

## 1. Visible interactive Device Manager — use it now on the owner's Mac

Use the existing authorized Mac's private SSH tunnel only:
`http://127.0.0.1:48765/lab/device-workbench`.

The existing three-workspace dashboard and main lab page link to this
page. The original Rust `control-api` embeds standalone HTML/CSS/JS
and serves it over private loopback (the normal private Mac tunnel
exposes the server's localhost). Its new `device_workbench_lab.rs`
implements REAL server-side HTTP, in-memory demo-only operations:

- **Add** a `LAB-*` name, `VIRTUAL-*` model, selected
  device kind (OLT, ONT or router), display vendor (ZTE, C-DATA,
  VSOL, MikroTik or Other) and example POP. Maximum 24 fake
  candidates, duplicate name+POP rejected. It NEVER accepts
  management IP, serial number, CPE password or real hostname
  in this demo form/API.
- **List** the newly added fake candidates with server-created
  IDs, filter by POP and device kind. Removal deletes only the
  in-memory fake candidate. These operations survive page
  refresh during that server process, not process restart.
- **Status** is always `PENDING_REVIEW`, connectivity
  `UNKNOWN`, health `NOT_MEASURED`, last verified `null`,
  physical count exactly zero. They NEVER change to fake online
  or green just because a row was added.

All input is independently validated by Rust, names/model
are fake-only, no serial/IP fields, and no code path connects
to the network or invokes a vendor adapter. POST/DELETE
need same-origin localhost Origin+Host and
`X-IPAT-Demo-Only: 1`, refuse browser cross-site
submission, have a 2 KiB HTTP JSON bound, and
use no cookies or browser storage for credentials.
Rendering uses DOM `textContent`, not HTML injection.
The UI is visually tagged *DEMO ONLY* with the
large RED unmet-PRD warning. These controls do
NOT grant admin rights to the three dashboard
presentation choices; only fake records are present.

Endpoints **only when `IPAT_LAB_WEB=1` on loopback**:
`GET /lab/device-workbench`, its same-origin CSS/JS,
`GET/POST /lab/demo/device-candidates`, and
`DELETE /lab/demo/device-candidates/{id}`.
They do not exist in the K3s/public app, and
every real `/v1/platform/*`, `/v1/tenant/*`
and `/v1/operations/*` request remains 401.

## 2. Genuinely persisted PostgreSQL draft registry (separately gated)

The new review-only, **not-production-deployed**
`deploy/db/migrations/0006_lab_device_candidates.sql`
defines `ipat_ops.device_candidates` with fixed
tenant UUID+POP, candidate UUID and unique
issuer+subject+request UUID for idempotent retries,
optional explicitly private RFC1918 IPv4 management
metadata, vendor/kind/model and immutable
`pending_review/unknown/not_measured` defaults.
Approval and last-seen evidence are NOT fabricated.

Two separate NOLOGIN ownership/execution boundaries:
`ipat_device_registry_owner` may SELECT approved
tenant admin/NOC membership and candidate records
and INSERT only candidate drafts under named
FORCE-RLS table/membership policies; it has no
update/delete, subscriber table or device connection.
`ipat_device_registry_execute` has EXECUTE
**only** on the sealed SECURITY DEFINER
`propose_lab_device_candidate(...)` function,
which enforces exact active tenant, nonrevoked/
unexpired approved `tenant_admin` matching
the verified issuer and subject passed by the
trusted app, accepted RFC1918 management IP,
and idempotency key that refuses conflicting
replays. `ipat_identity_query` receives
EXECUTE **only** on the separate
`list_lab_device_candidates(...)` function,
which checks own active tenant, either the
tenant admin role (all own POPs) or NOC role
with an exact explicitly approved POP grant.
Neither execution role gets direct table access.

**Important trust requirement:** Security-definer
SQL does NOT authenticate human JWT by itself.
The application must cryptographically validate
the pinned issuer, audience, key ID, signature
and expiry and MUST NOT grant the query or writer
PostgreSQL service identities to any untrusted
end user. The owner/provisioner must separately
verify MFA, admin enrollment and audit
before any *real* production operation.
No ordinary lab operator is allowed to
run this migration on an existing
customer database or reuse the disposable
CI credentials.

Original Rust private verified endpoints:
`GET /lab/auth/device-candidates?tenant_id=<uuid>&role=tenant_admin`
or POP-granted
`?tenant_id=<uuid>&role=noc_engineer&pop_id=<exact-pop>`.
The read-only service account executes two
independent narrow authorization SQL functions
in ONE SQL statement/snapshot and independently
rechecks the Rust `TenantMembers`
(admin) or `OperationsInventory` (NOC)
permission. Results never contain requester
subject, approval actor or credentials.
`POST /lab/auth/device-candidates/propose` can
INSERT an auditable metadata-only pending draft,
but ONLY with a separate restricted dedicated
PostgreSQL registrar identity, explicit private
nonroot operator opt-in and exact signed
tenant-admin membership. Sample JSON shape:

```json
{
  "tenant_id":"11111111-1111-4111-8111-111111111111",
  "request_id":"d0000000-0000-4000-8000-000000000001",
  "display_name":"LAB-DEVICE-REGISTRATION",
  "pop_id":"pop-a",
  "device_kind":"olt",
  "vendor":"ZTE",
  "exact_model":"VIRTUAL-C320",
  "management_ipv4":"10.26.2.10"
}
```

This JSON is **synthetic CI fixture only**.
No secret is accepted. A future real tenant
registry request may use the physical device's
actual known approved model/IP metadata only
after approved private connectivity and
true identity enrollment (not by this
anonymous demonstration UI).

The real registry private feature needs:
`IPAT_LAB_WEB=1`, `IPAT_LAB_OIDC_VERIFY=YES`,
`IPAT_LAB_SCOPED_MEMBERSHIP=YES`,
`IPAT_R83_REGISTRY_WRITE=YES`,
an independently validated pinned signing
issuer configuration, and TWO independent
user-owned, non-symlink, 0600 PostgreSQL
Unix-socket conninfo files in owner-only
0700 private folders referenced by
`IPAT_LAB_DB_CONNINFO_FILE` (exact reader)
and `IPAT_R83_REGISTRY_CONNINFO_FILE`
(exact registrar). The Rust process refuses
root, remote TCP PostgreSQL, role-changing
connection options, missing identity,
public/K3s mode and unrequested writes.
**None of these actual customer prerequisites
are configured on the owner's live VPS.**

## 3. Actual automated evidence and repeatable QA

Run locally only in a disposable isolated
environment and the original source checkout:

```sh
node --check web/lab/device-workbench.js
python3 -m unittest discover deploy/scripts/lab/r83 -p test_r83_contract.py -v
cargo fmt --all -- --check
cargo test --workspace --locked
cargo build --locked -p control-api
IPAT_R83_DEMO_HTTP=YES bash deploy/scripts/lab/r83/workbench-http-smoke.sh
```

The smoke starts an actual original Rust
Axum 127.0.0.1:3000 listener only if free,
executes real Python urllib GET/POST/DELETE
and verifies HTML/CSS/JS, server-side
persistence across HTTP calls, duplicate/
foreign Origin/fake Host/IP/credential
denials, statuses NEVER live, removed
fake rows, private auth routes absent
without config and all real business
API namespaces remaining 401. It kills
ONLY its own process.

ONLY ephemeral PostgreSQL 16 CI with
`IPAT_PG_EPHEMERAL_TEST=1` tests migration
0006 with 5 genuine database positive/
negative tests and two independent
CI-only reader/registrar login roles.
The additional real compiled Rust
Axum test signs a genuine short-lived
synthetic RS256 JWT, uses SEPARATE
restricted PostgreSQL logins and
writes unique metadata-only drafts
to two independent synthetic ISPs;
then reads the actual committed rows
through tenant-admin and NOC-POP
authorization. Forged Host, token,
cross-POP, changed-idempotency replay,
public IP and secret fields fail closed.
Passing the disposable pilot does
NOT assert human MFA or production
device interoperability.

## 4. Precisely remaining before real hardware adoption

**MUST**: real human IdP+MFA/approval and
production restricted DB setup,
proper tenant admin/POP onboarding,
maker-checker device approval with
append-only audit, per-vendor authenticated
SSH/SNMPv3 or actual CWMP/USP device
enrollment verified against real
model/firmware, private network path
and independently checked vendor identity.
Only verified read-only protocol evidence
may change `connectivity` or `health`
from UNKNOWN/NOT_MEASURED. Explicit
firmware approval, backup and rollback
remain distinct high-risk work.

**SHOULD**: bind the signed candidate
backend to an authenticated real
tenant dashboard with server-side RBAC
and dynamic menu hiding, implement
nonblocking health collection/queue and
NOC evidence freshness and diagnostics.
Do not use one role-switching demo
browser page as a trusted account.

**LATER**: real cross-provider K3s workers,
PostgreSQL HA/PITR/full offsite recovery,
public customer custom domains,
capacity/availability measurements
and all firmware write actuators.

**RED PRD DEVIATION:** A working
demo *Add/List/Status* UI and a
test-proven signed PostgreSQL
metadata draft API are NOT a
fully operational device adoption
platform, real online telemetry,
actual customer MFA dashboard or
approved live equipment/firmware
compatibility.


## Verified release and real owner browser access

GitHub feature PR #94 merged actual
R8.3 visible Rust/HTML/JavaScript
Add/List/Status demo and independently
signed tenant/POP-scoped PostgreSQL
pending-registration code into main at
`94c6b2f07bd63a4ae0578a5915e232b1bd1fa0b4`.
Feature run `36305345728` and
independent real post-code-merge main
run `36306977093` BOTH succeeded
in all four CI jobs, including genuine
disposable PostgreSQL SQL/Rust
two-tenant security tests and
actual compiled Rust HTTP UI
Add/List/Delete smoke on CI.

The owner private Mac tunnel was
restarted from this actual canonical
code SHA. Actual Mac HTTP
`http://127.0.0.1:48765/lab/device-workbench`
and its JavaScript/CSS responded HTTP200.
With synthetic `LAB-`/ `VIRTUAL-`
metadata only, real localhost HTTP
Add 201, List 200 showing exact
PENDING_REVIEW/UNKNOWN/NOT_MEASURED,
and Delete 200 PASSED. Missing
private same-origin write request
was rejected 403; all three
production platform/tenant/NOC
REST namespaces remained 401.
The unprovisioned private real
signed reader/registrar routes
remained 404; no real IdP,
PostgreSQL operator login, device,
probe or firmware operation
was activated. The original
local preview was restarted again
to clear manual demo fixtures,
leaving zero fake candidates.
The same clean merged source SHA
was synchronized by SHA256-verified
Git bundle to actual nonroot
Ubuntu26 VPS and its full
locked offline Rust workspace,
rustfmt and R8.3 static tests
PASSED. A separate duplicate
standalone port-3000 smoke on
the live VPS deliberately
refused preexisting port-3000
preview ownership. The actual
equivalent Mac→private VPS HTTP
test and independent disposable
CI socket test both passed.

Source-only Mac FileVault encrypted
Restic snapshot `3e40cc27`
was all-pack-verified 142/142,
isolated SHA256-exact restored;
selected historical PARTIAL
root-readable restore also passed.
**This is NOT independent
whole-host, customer PostgreSQL
or K3s datastore disaster
recovery.**

The visible Device Manager is
immediately available to the
owner on the authorized Mac.
It is not a real user MFA session
nor an approved hardware adoption
switch. The signed stored
candidate backend is tested and
ready FOR INTEGRATION after
actual IAM/approval and restricted
tenant DB service account provisioning;
do not route customer IPs or
creds through the fake UI.
