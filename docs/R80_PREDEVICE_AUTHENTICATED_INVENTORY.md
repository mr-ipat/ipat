# IPAT R8.0 — Pre-device virtual integration: signed identity → PostgreSQL device inventory

**Environment:** private synthetic IPAT lab ONLY; no real ZTE/ONT/RouterOS
access is required to test the complete JWT→tenant/POP database→Rust HTTP
inventory vertical slice. **Actual company OIDC/MFA enrollment, real device
model/firmware, public SaaS APIs and production K3s are NOT completed.**

## What this milestone actually connects

Earlier R7.8 connected signed JWT, restricted PostgreSQL membership and
a server-side menu, but had no data retrieval endpoint. R8.0 adds:

- `deploy/db/migrations/0005_lab_verified_device_inventory.sql`: narrow
  `ipat_platform.list_authorized_lab_devices(issuer,subject,tenant,role,pop)`
  SECURITY DEFINER function owned by the preexisting **NOLOGIN**
  `ipat_identity_lookup_owner`. It returns at most 100 read-only device
  inventory fields. An independent named SELECT-only, FORCE-RLS device
  policy grants access **to the function owner**; `ipat_identity_query`
  and its dedicated CI-only login get EXECUTE on the sealed function,
  never direct SELECT on the devices or subscriber tables.
- Each execution joins the requested tenant to an ACTIVE tenant and
  exact unrevoked/unexpired approved membership whose issuer and
  subject come from the cryptographically verified pinned RS256
  token. The requested role is initially exactly `noc_engineer`,
  with exact nonempty POP, matching membership POP grant and
  the device's **same tenant UUID and exact POP**. Arbitrary
  platform-owner/admin, cross-company and helpdesk requests do
  not inherit this authorization. No device credentials are exposed.
- `apps/control-api/src/tenant_membership_lab.rs` now mounts an
  optional `GET /lab/auth/devices` alongside the previous
  menu endpoint. It shares the existing nonroot private
  `127.0.0.1:3001`, dual opt-in OIDC+scoped DB, independently
  pinned issuer/audience/RS256 public key, dedicated Unix-socket
  owner-only PostgreSQL configuration, and no production bind.
  The Rust query reads the approved membership **and sealed
  device rows in one PostgreSQL statement snapshot**, then
  *also* requires `OperationsInventory` in the Rust
  fail-closed authorization result. A denied token returns 401;
  denied membership/POP/role returns 403; DB outage returns
  503; data responses have `Cache-Control: no-store`.
- The response carries `lab_only=true`, `mfa_verified=false`,
  `real_business_access_enabled=false` and only
  `tenant_slug`, exact `pop_id`, source, limit, count and
  the bounded inventory fields (UUID, POP, kind, vendor, model,
  firmware). No issuer/subject/token/approver, subscriber
  customer_ref, device secret, write capability, public ingress
  or arbitrary SQL is returned.
- The existing `/v1/platform/*`, `/v1/tenant/*` and
  `/v1/operations/*` are STILL 401, even with a signed
  synthetic JWT or forged tenant/role/Host; this is not
  a production customer dashboard. The ordinary Mac preview
  stays synthetic because no operator-reviewed real MFA
  identity realm or real approved tenant/POP membership is
  provisioned on the actual VPS.

## Real disposable test database and two separate ISP simulations

GitHub CI provisions its own ephemeral PostgreSQL 16 instance,
runs R7.0/R7.7's existing base RLS and membership migrations,
then applies `0005` and verifies actual role privileges,
exact tenant A/NOC/pop-a and tenant B/NOC/pop-b positive
inventory plus negative same-subject cross-company/POP,
wrong role/issuer/subject, revoked/expired membership and
suspended tenant. Only AFTER those SQL tests does the
synthetic CI seed create its **disposable-only**
dedicated login reader; it gives the SAME short-lived signed
synthetic subject independent NOC memberships for both
synthetic companies. The Rust Axum integration test must
prove real JWT signing and verification, actual restricted
Postgres login/function call, company A returning only A's
device, company B returning only B's device, other POP
and helpdesk denied and ALL real business endpoints 401.
This is more than a stub but does not certify a hardware
adapter or real human MFA.

The tests are intentionally NOT executed against any
customer PostgreSQL. New schemas/functions must
receive independent production review and live operator
account provenance before deployment. The synthetic
role has a known CI-only test password which MUST
NEVER be used outside the disposable instance.

## Reproducible testing

On the authorized nonroot Ubuntu 26.04 repo for static and
offline coverage ONLY:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
python3 -m unittest discover deploy/db/tests \
  -p test_r80_inventory_contract.py -v
python3 -m unittest discover deploy/scripts/lab/r79 -p 'test_*.py' -v
```

The **genuine PostgreSQL and signed HTTP** checks are CI-only
and must run against an explicitly throwaway `ipat_synthetic`
database with `IPAT_PG_EPHEMERAL_TEST=1`; both real test
results are tracked in `.github/workflows/ci.yml` alongside
the original negative RLS/backup and Ubuntu26 K3s jobs.
Do not set that CI flag against a production VPS database.

## Physical device test when equipment is available

When the owner's C320 is available later, its live IP-based
management channel and exact firmware/board fingerprint
are separately reviewed. R7.9's strict SSH read-only
candidate can collect the two known SHOW table outputs
over a trusted private route; a separately designed
SNMPv3 adapter may be substituted only after verifying
real firmware and vendor MIB support. Both channels
feed the **same logical future inventory schema**.
No L1 cable is necessary for a working remote management
protocol; that does not authorize OLT firmware writes
without independently tested recovery.

## Remaining MUST/SHOULD/LATER

- **MUST:** real IdP with MFA, independently audited
  membership enrollment, properly restricted dedicated
  production service account, bounded tenant/POP RLS
  for every business API/job/ACS/USP/queue, real
  dashboard data rendering and device protocol simulators
  with fixture versioning.
- **SHOULD:** private-lab simulator for OLT/ONT/RouterOS
  without device access, virtual tenant operational
  diagnostic scenarios, real measured cross-provider
  private WireGuard worker cluster on a second disposable
  VPS after external recovery safety gates.
- **LATER (production-gated):** customer public domains,
  complete hardware/firmware acceptance matrix,
  high-risk approved firmware actuators, PostgreSQL
  replication/failover/PITR and completed independent
  whole-host restore.

**PRD DEVIATION / RED NOTICE:** as of this milestone,
a genuine customer can NOT yet sign in, no real business
dashboard retrieves production data, and no physical OLT,
ONT, RouterOS or heterogeneous live cluster acceptance
has passed. R8.0 is an actual **pre-device synthetic**
integration slice, not enterprise production completion.


## Verified synthetic end-to-end release candidate

Original feature code `1ef12afd7232c04e49d732427fc1182fb1978bdd`
passed GitHub run `36291884873` all FOUR independent
jobs. The disposable PostgreSQL job's logs
showed 4/4 actual SQL integration tests
AND the actual `r80_real_signed_jwt_to_postgres_tenant_pop_inventory`
signed RS256→restricted reader→actual Rust
Axum HTTP test `1 passed, 0 failed`.
The same synthetic database deliberately
contained prior provisioning/outbox router
fixtures, so the correct API assertions
validate EVERY returned row's exact
authorized POP and presence of the
known independent device belonging
to EACH of the two ISPs, not an
artificial exactly-one-device
assumption. A separate earlier
bad expiry fixture violating the
database's CHECK was corrected before
this verified final feature SHA.
The actual temporary CI test
password, synthetic JWT and random
fixture devices are not deployed on
the real user's production VPS.

This is a meaningful hardware-free
backend vertical slice, but **not**
actual human MFA, complete UI
business authentication, a live
device session or company onboarding.
Full source and independent
post-merge backup/source-sync
evidence are recorded separately
only AFTER the actual final merge.
