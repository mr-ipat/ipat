# IPAT R8.9 — Real signed BFF session → sealed tenant/POP pending-device inventory (UNMOUNTED)

**Scope:** implement the missing trust-safe backend connection between
the original R8.8 cryptographic identity-only browser session and the
R8.3 actual PostgreSQL tenant/POP pending adoption candidates.
This is intentionally INTERNAL RUST SOFTWARE + genuine disposable DB CI,
not a claim that owner Mac now has a real company-login dashboard.

## What is implemented

`apps/control-api/src/browser_session_lab.rs` adds a bounded
`PendingDevice` result type and `pending_devices_for_session`:
- Re-authenticates each opaque cryptographic identity-only cookie via
  the original OS-random SHA256-digest-only `BrowserSessionVault`,
  with independently verified host/origin passed by the future HTTPS BFF.
  Browser-supplied tenant, role or claims NEVER confer entitlements.
- Makes ONE actual parameterized PostgreSQL statement with a
  MATERIALIZED exact active membership check and LATERAL sealed
  `list_lab_device_candidates` call. PostgreSQL independently
  rechecks issuer, subject, tenant, role and exact POP, active
  company status, unrevoked and unexpired approval every time.
  Auth failures and DB errors produce no result.
- Allows an explicitly approved NOC engineer for its exact POP
  or a separately approved company tenant admin for its own
  tenant. A multi-company operator needs independent membership
  in EACH company; no platform-owner implicit device access.
- Returns at most 100 bounded metadata rows. No management IP,
  subscriber record, credential, server cookie, fake telemetry
  or firmware contents enter the result. Review status and
  connectivity/health are reported only as already stored
  by trusted data owners; this path never modifies them.
  Never equate a pending registration or approved metadata
  with physical adoption or successful device connectivity.

## Exact regression acceptance

`deploy/scripts/lab/r89/test_r89_contract.py` statically verifies
no accidental public routing, the independent per-request SQL
functions, strict safe result contract and genuine disposable
PostgreSQL CI execution.

A separate genuine CI-only Rust integration uses independently
signed short-lived 2048-bit RSA synthetic ID+access tokens and
the original nonce/MFA claim verifier, then real restricted
PostgreSQL 16 reader and separate registrar service accounts.
The test opens a genuine opaque+CSRF identity-only session
only after restricted approved membership and registers
two fake devices in two separate fake ISPs. It checks that
a NOC session for company A and POP A can list A without
seeing B or other POPs, that independently granted
company B POP B may be queried only with the distinct
actual current B membership, and that forged session,
wrong tenant/POP/role, invalid origin and expired
session are all denied. Neither reader nor registrar
has direct SELECT/INSERT on operational tables.
No physical traffic or actual user data are involved.

```sh
python3 -m unittest discover deploy/scripts/lab/r89 -p test_r89_contract.py -v
cargo fmt --all -- --check
cargo test --workspace --locked --offline
# R8.9 REAL two-ISP PostgreSQL test requires disposable GitHub CI environment;
# never run its CI-only fixture against any actual customer database.
```

## Scope and PRD red warning

**MUST before browser activation:** actual owner-approved human
OIDC+MFA service with independently tested `amr` semantics;
secure confidential HTTPS authorization-code/PKCE exchange,
tenant-custom-domain TLS/origin policy, actual restricted
nonroot PostgreSQL identities, audited human tenant membership
and maker-checker approval, actual Secure HttpOnly host-only
cookie transport and CSRF/logout, revocable durable shared
session store for horizontally scaled K3s, and fully backend-
authorized hidden UI menus. The current owner lab UI continues
to use FAKE-only volatile Add/List/UNKNOWN and actual
`/v1/*` namespaces reject unauthorized requests.

**MUST before real device condition:** approved authenticated
private C320 SSH/SNMPv3 host/fingerprint and genuine running
firmware+boards evidence, model-specific ONT genuine CWMP/USP
peer identity, actual read-only checks with recorded freshness,
diagnostic correlation and explicitly separate high-risk
firmware backup/recovery/approval. Do not pass real device
credentials through a synthetic browser form.

**LARGE RED PRD DEVIATION:** This is a necessary verified
INTERNAL trust-to-database integration, **NOT** a real
customer login, mounted production device manager,
physical adoption, active monitoring or full product GO.


## Verified release and owner-private controls (2026-09-28)

Feature PR #105 merged exact original
Rust source into main SHA
`bd788f43e19d94edf5d8eb518053fc2610370a70`.
The separately executed feature
GitHub run `36368344427`
and independently executed
post-code-main run
`36368663242` both
completed SUCCESS 4/4
independent jobs each,
including real PostgreSQL
two-company synthetic
signed OIDC→original
restricted reader→opaque
session→POP-specific
candidate data and
negative cross-tenant,
role, POP, origin and
expiry tests.

GitHub private, owner
Mac and actual authorized
nonroot Ubuntu26 VPS
were synchronized to
the exact source code
SHA by SHA256-checked
Git bundle fast-forward.
Final canonical VPS full
locked offline Rust
workspace, rustfmt and
4/4 R8.9 static
security checks passed.
The owner-private
localhost preview
initially refused old
stale PID state correctly,
then was guardedly
stopped and rebuilt
from the matching new
source. Live actual
Mac GET Device Manager
returned 200; unprovisioned
real customer signed
registration GET returned
404 and ALL THREE real
business API GET
namespaces returned 401.
No new R8.9 public
browser route was mounted.

Owner Mac FileVault
encrypted SOURCE-only
Restic snapshot
`65b40fa7` all encrypted
packs read 158/158
and isolated SHA256-exact
source restore passed;
selected historical
PARTIAL readable root
config SHA256 restored.
NOT whole-host/customer
PostgreSQL/live K3s
or offsite real DR.

**PRD STILL OPEN:** no
actual human MFA IdP,
real confidential HTTPS
PKCE exchange, Secure
cookie login into
commercial operational
device UI, actual
OLT/ONT health evidence
or physical firmware
compatibility proof.
