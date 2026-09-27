# R8.4 — Original Rust maker-checker adoption-metadata review, signed MFA claim (lab only)

**Date:** 2026-09-27 · **Milestone:** prepared for independent disposable CI.
**NO PRODUCTION GO.** This software deliberately does not enable real
OLT/ONT adoption, real user login or approved firmware operations.

## Sequence and first-stage prerequisite

The project owner requires a **working Add/List/Condition dashboard before
connecting hardware**, then real authenticated identity, PostgreSQL-backed
inventory, independent adoption review and only then vendor read-only
device evidence. R8.3's interactive private dashboard and separate
signed real PostgreSQL registry are present, but the owner has NOT
provisioned actual company human IdP/MFA and its PostgreSQL security
service accounts. A real authenticated browser-to-PostgreSQL company
dashboard therefore remains **OPEN**, not silently substituted
with fake claims or copied long-lived tokens.

R8.4 prepares the **next independent review trust boundary** using
synthetic short-lived SIGNED MFA-claim test users in real disposable
PostgreSQL. It does not presume that an actual user IdP has completed
MFA enrollment. In the actual owner preview, these routes remain ABSENT
by default, and all three real business API namespaces still deny.

## Genuine software scope

- `crates/identity-core/src/lib.rs` now parses a tightly bounded
  optional `amr` signed JWT claim and marks `signed_mfa_claim=true`
  only when the exact word `mfa` occurs in a cryptographically
  valid pinned issuer's bounded list. A bare header, unverified
  JWT role, Host, arbitrary provider `acr` or claim from
  an unpinned key cannot confer reviewer access. Signature,
  kid, issuer, audience, iat, nbf and short expiry remain mandatory.
  **A signed `amr=mfa` still requires an actual trusted IdP
  configuration and MFA enrollment proof outside this software.**
- `deploy/db/migrations/0007_lab_device_maker_checker.sql`
  extends R8.3 lab-only memberships to `security_admin` and
  defines an independent NOLOGIN reviewer function owner and
  independently scoped reviewer EXECUTE role. The only table
  UPDATE granted to the function owner is the candidate's
  `adoption_state`, not its management IP/model/firmware or
  any real device configuration. Separate FORCE-RLS policies
  explicitly govern its database reads/writes.
- The sealed `lookup_lab_device_reviewer` independently checks
  current active tenant, exact issuer/subject security_admin,
  distinct approval, nonrevoked and unexpired membership.
  The sealed `list_lab_device_review_queue` limits 100 rows
  to a reviewer’s own active tenant and excludes the reviewer's
  own proposed drafts. Its result omits management IP, identity
  secret, subscriber information and audit actors.
- The sealed `review_lab_device_candidate` acquires a real
  PostgreSQL candidate row lock; it denies cross-tenant,
  wrong-role, self-review, inactive tenants, revoked reviewer
  or non-pending review. It atomically updates only metadata
  to `approved` or `rejected`, inserts ONE immutable
  append-only review/audit record with exact distinct actor,
  bounded reason and unique idempotency request. An
  exact same actor/decision/reason/request retry returns the
  same review UUID without creating a second action.
  A different reviewer cannot overwrite the first verdict.
  Candidate `connectivity=unknown`, `health=not_measured`,
  `last_verified_at=NULL` remain enforced; metadata approval
  NEVER means physical device adoption, verified identity,
  live SSH/SNMP/TR-069 or authorization to update firmware.
- `apps/control-api/src/device_review_lab.rs` implements
  opt-in private Axum queue GET and review POST. Both require
  an actual original pinned RS256 access token with signed
  MFA claim PLUS independent exact SQL permission checks on
  every request and an independent restricted PostgreSQL
  reviewer identity. Both only return safe count/status,
  no raw database error/management IP/secret, with no-store
  response headers. There is no browser token-paste
  workaround, automatic superadmin entitlement,
  stale membership cache or proxy header trust.

## Strict deployment and external gates

Default public/K3s and owner Mac preview: new reviewer
endpoints **not installed**. The isolated Rust process mounts
them only when existing private lab identity and scoped
membership have been independently enabled AND an
additional `IPAT_R84_SIMULATED_REVIEW=YES` is specified
on a nonroot localhost process with separate secure
`IPAT_R84_REVIEW_CONNINFO_FILE` owned by the same
service user, regular file mode 0600 in 0700 parent,
containing **only** `user=ipat_lab_device_reviewer`
on a locally authenticated Unix-domain PostgreSQL
socket. An actual production account must never
reuse disposable CI credentials or allow
externally controlled SQL identity parameters.

The external release gates still include provisioning
a **real** human MFA-capable OIDC IdP and verifying
the intended issuer's signed AMR semantic contract;
reviewing the issuer's keys/rotation, active
admin memberships and separate actual reviewer
accounts; current whole-host/offsite DB recovery,
append-only external audit archive, safe TLS
customer login and vendor-specific physical
read-only inventory. No actual server database,
firewall, K3s daemon, device, firmware or
existing owner dashboard token was changed
in this milestone.

## Repeatable tests

On a disposable PostgreSQL 16 integration job,
run migrations 0001 through 0006 and the
previous ordered test fixture, then run:

```sh
python3 -m unittest discover deploy/db/tests -p test_device_maker_checker_integration.py -v
python3 deploy/db/tests/seed_private_http_identity_ci.py
cargo test --locked -p control-api device_review_lab::pg_integration::r84_real_signed_mfa_claim_two_humans_actual_postgres_immutable_review -- --exact
python3 -m unittest discover deploy/scripts/lab/r84 -p test_r84_contract.py -v
cargo fmt --all -- --check
cargo test --workspace --locked
```

CI uses wholly disposable synthetic issuer, cert,
short token and two separate fake people. Real
PostgreSQL tests validate actual column/table/function
privileges, FORCE-RLS, distinct tenant isolation,
atomic review, one immutable audit record, idempotency,
maker rejection, revoked grant, and suspended tenant.
The Rust test independently generates a real
ephemeral RSA key, checks signed `amr:mfa`
versus missing MFA, serves real Axum router
against distinct restricted PostgreSQL reviewer
login and asserts no real business API exposure.
It MUST NOT be presented as physical compatibility
or an actual customer MFA user session.

## PRD gap and priority after this slice

**MUST next:** actual independent Keycloak (or
approved OIDC IdP) setup with verified MFA
configuration; authenticated browser BFF,
secure session handling, signed-in tenant/POP
device list and review workflow menus
server-gated per backend membership.
**MUST after identity:** verified private
network and exact trusted OLT/ONT host
identity, read-only connectivity/firmware
evidence, POP-scoped evidence timestamps,
diagnostics and audit. Firmware write remains
hard-blocked pending vendor recovery plan.
**OPEN:** full multi-provider live K3s,
offsite whole-host + real PostgreSQL
PITR/DR, production custom domain/TLS,
native USP MQTT MTP, complete original
ACS interoperability on exact physical
models/firmware and commercial SaaS GO.

<span style="color:red;font-size:1.4em">
PRD TIDAK LENGKAP: reviewer code and synthetic
signed MFA tests are NOT a live customer IAM,
authenticated real dashboard or physical adoption.
</span>
