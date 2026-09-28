# R9.2 — Durable nonexecutable read-probe intent, exact signed-session scope

Date: 2026-09-28 Asia/Jakarta. Status: SOFTWARE IMPLEMENTATION FOR
DISPOSABLE TEST, NEVER an authorization to touch actual devices.
Binding docs: PRD v0.1, SECURITY, DEVICE_MATRIX, ADR-039, R9.1.

## Product reason

Owner's workflow is Add OLT/ONT → permanent tenant and POP inventory →
separate maker/checker metadata approval → independently proven secure
management path, physical identity, dedicated read-only account and
recovery → NOC requests a *recorded* first read. R9.1 implements immutable
readiness evidence and never equates a Telnet banner with verified hardware.
R9.2 implements the NEXT boundary: the NOC can file a single persistent
and auditable but **permanently nonexecutable lab request**. In particular,
the known owner-Mac public Telnet connectivity and actual VPS TIMEOUT from
R9.0 cannot bypass the secure-private-route readiness gate.

## New real original source

- `deploy/db/migrations/0009_lab_read_probe_intent.sql` defines
  `ipat_ops.device_read_probe_intents` with immutable candidate,
  tenant UUID, POP, signed issuer/subject, client-generated idempotency
  UUID and the **sole** permitted state
  `awaiting_separate_execution_review`.
  `must_revalidate_before_execution` is permanently TRUE.
  A separate immutable `device_read_probe_intent_audit` row is created
  in the SAME SQL transaction; `published_at` MUST remain NULL.
  This audit is explicitly NOT an executable broker/job outbox.
- Sealed SECURITY DEFINER `request_lab_read_probe_intent` is owned
  by a dedicated NOLOGIN, no-BYPASSRLS role and EXECUTE-granted only to
  its independent service role. It supports **ONLY** an active own
  `noc_engineer` with exact approved POP and the current R9.1 sealed
  SQL projection: maker-checker metadata approved, latest unexpired
  verified four readiness gates and no recorded actual live status.
  It serializes same-candidate submissions, permits exactly one
  immutable request per tenant/candidate, returns the same ID only for
  an exact same-signer/request retry, and returns NULL otherwise.
  No inherited access to subscriber or device credential tables.
- `list_lab_read_probe_intents` returns only own currently approved
  tenant/POP intent IDs, candidate ID, audit-safe status, request time
  and mandatory revalidation flag. This read has EXECUTE grant only
  to the existing restricted identity-query role, not the writer.
  No management IP, raw evidence, OLT secret, Telnet endpoint or
  fake live health is returned.
- `apps/control-api/src/browser_session_lab.rs` adds
  **UNMOUNTED** `submit_nonexecutable_read_intent_for_session`.
  It requires a cryptographically verified opaque browser identity
  session, trusted independent Host/Origin, matching CSRF on
  mutation, separately queried fresh exact membership and a
  distinct PostgreSQL EXECUTE-only intent writer. The SQL function
  separately repeats the actual readiness and membership checks
  at insertion time. The HTTP production and private demo router
  intentionally mount NONE of these methods.

## Verification boundary

R9.2 disposable PostgreSQL integration exercises real sealed
function privileges and FORCE RLS, actual two-company candidate
metadata and independent maker-checker review, four real recorded
readiness gates, missing-gate rejection, wrong exact POP, other
company, signer forgery, changed idempotency replay, later blocked
identity evidence and the one-to-one immutable private audit.
The mandatory disposable Rust signed ID/access→opaque
CSRF session test also creates one intent through a
separate actual PostgreSQL login and tests forgery, wrong POP,
missing trusted origin, successful exact retry and immutable
nonexecutable state. These tests run only with exact synthetic
`ipat_synthetic` PostgreSQL 16 environment in CI.

Reproduce static no-device test:

```sh
python3 -m unittest discover deploy/scripts/lab/r92 -p test_r92_contract.py -v
cargo fmt --all -- --check
cargo test --workspace --locked --offline
```

Real disposable integration on a separate fresh PostgreSQL 16
requires migrations 0001–0008, the existing per-milestone
seed harness, and then:

```sh
python3 -m unittest discover deploy/db/tests -p test_read_probe_intent_integration.py -v
python3 deploy/db/tests/seed_private_http_identity_ci.py
cargo test --locked -p control-api browser_session_lab::tests::r91_signed_session_reads_real_pg_adoption_gates_without_probe_execution -- --exact
```

Do NOT point the fixtures at owner/live PostgreSQL, issue the
disposable fake credentials to actual users or enable the
lab-specific SQL migration as a customer production upgrade.
CI itself runs the complete preceding ordered disposable suite.

## Explicit red PRD gap

**NOT DONE:** actual live user IdP confidential authorization-code
exchange and MFA completion, real TLS BFF session/multinode vault,
actual security-admin evidence intake UX, real operator approval
and audit signing, independently secure site-to-VPS route, actual
ZTE C320 serial/cards/firmware/SSH host key, vendor-safe SNMPv3
or ACS/USP device enrollment, reusable stateful worker with
durable per-device lease and revalidation-at-claim, physical
OLT/ONT connectivity/health, full offsite host/PG/K3s DR and
commercial dashboard. NO physical command, packet, firmware
or actual health reading is triggered by the R9.2 intent.
R9.0 documented that owner Mac only had untrusted
public Telnet negotiation while the actual VPS route timed out;
NO CREDENTIAL may be sent on that plaintext channel.
