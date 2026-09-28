# IPAT R9.1 — Device Adoption Readiness Gates

**Date:** 2026-09-28 Asia/Jakarta
**Status:** SOFTWARE CANDIDATE — persistent evidence and signed-session projection;
NO physical read, NO live adoption, NO firmware operation.

## Goal

Close the unsafe gap between "metadata approved" and
"IPAT is allowed to touch a physical device".

R8.3 provides tenant-scoped pending device metadata.
R8.4 provides distinct maker-checker metadata approval.
R8.8/R8.9 provide an opaque signed-identity session primitive
and a current PostgreSQL tenant/POP-scoped candidate projection.
R9.1 adds a separate fail-closed evidence state machine.
Even an approved candidate stays **NOT READ-PROBE ELIGIBLE**
until every required operational prerequisite has fresh evidence.

## Four required gates

The disposable-first PostgreSQL migration
deploy/db/migrations/0008_lab_device_adoption_readiness.sql
adds immutable append-only attestations for exactly:

1. secure_management_path — independently reviewed encrypted/private
   management transport or another approved secure vendor management path.
2. device_identity — independently verified actual device identity,
   exact model and firmware evidence; a banner or IP address is insufficient.
3. readonly_account — dedicated least-privilege read-only management
   identity is independently verified. No password/secret is stored here.
4. recovery_plan — recovery/rollback and operational escape path has
   evidence appropriate to the planned physical read.

An attestation contains tenant/candidate IDs, one gate, a
verified|blocked verdict, SHA-256 evidence reference, bounded note,
attester identity, capture time and expiry. Evidence validity is limited
to at most seven days. Raw evidence, credentials, management transcripts
and secrets are deliberately stored elsewhere under operator control.

## Metadata approval is not device permission

read_probe_eligible is true only when ALL conditions hold in one
sealed PostgreSQL projection:

- active own tenant and current role/POP membership;
- candidate has maker-checker approved metadata;
- immutable approved review record exists;
- latest unexpired state of all four gates is verified;
- connectivity remains unknown;
- health remains not_measured;
- last_verified_at remains NULL.

A later blocked attestation overrides an earlier positive gate.
Expired evidence does not qualify. Revoked/expired tenant membership
is rechecked on every read.

**Critical boundary:** read_probe_eligible=true does NOT execute a
network probe, enqueue a worker task, authorize firmware, change health,
or establish a physical device session. It means only that the
preconditions to REQUEST a future separately controlled read-only
probe have passed.

## Role separation

New NOLOGIN roles:

- ipat_device_readiness_owner: SECURITY DEFINER function owner with
  only the table reads/append-only insert needed to evaluate readiness.
- ipat_device_readiness_execute: EXECUTE-only attestation boundary.

The existing ipat_identity_query gets EXECUTE only on the safe
readiness projection. It receives no direct candidate, review or
attestation table privilege.

The attestation function independently requires the exact same
signed-identity security_admin who performed the candidate's
approved metadata review, an active own tenant, unrevoked/unexpired
approved membership and an already-approved candidate. This avoids
letting an unrelated actor silently turn metadata approval into
device readiness.

R9.1 does not claim this security-admin evidence is real-human MFA.
Production still requires the R8.5–R8.8 external IdP/MFA/BFF gates
to be independently deployed and verified.

## Browser/session boundary

apps/control-api/src/browser_session_lab.rs adds the UNMOUNTED
adoption_readiness_for_session() bridge. It:

- authenticates the opaque signed-identity session;
- requires independently verified trusted host/origin;
- calls only the sealed PostgreSQL readiness projection;
- rechecks exact requested tenant/role/POP;
- returns only candidate display metadata plus readiness booleans.

It deliberately omits management IP, evidence hash/note, attester,
credentials, firmware secret, private key and raw network evidence.

The function is **not mounted in main.rs**. Actual browser HTTP
exposure remains blocked until a real confidential OIDC code exchange,
independently proven human MFA and safe HTTPS BFF session exist.

## Acceptance tests

Static source/security:
python3 -m unittest discover deploy/scripts/lab/r91 -p test_r91_contract.py -v

Real disposable PostgreSQL:
python3 -m unittest discover deploy/db/tests -p test_device_adoption_readiness_integration.py -v

Real Rust signed-session/DB bridge:
cargo test --locked -p control-api browser_session_lab::tests::r91_signed_session_reads_real_pg_adoption_gates_without_probe_execution -- --exact

The SQL integration must prove role separation, forced RLS,
append-only evidence, four-gate positive path, blocked/expired
fail-closed behavior, idempotency, wrong reviewer/cross-tenant denial,
pending candidate denial, exact POP restriction and membership
revocation recheck.

The Rust integration must use actual independently signed synthetic
OIDC tokens, an opaque server session, real restricted PostgreSQL
reader/writer/reviewer/readiness identities and prove that an
approved candidate is initially ineligible, becomes eligible only
after all four gates, then immediately becomes ineligible after a
new blocked identity attestation.

## RED PRD DEVIATION — physical adoption is still blocked

R9.1 prepares the system so a physical device cannot be contacted
merely because somebody clicked "approve". It does **not** solve the
remaining external dependencies:

- actual human IdP login and independently proven MFA;
- secure private/routed management path from the IPAT worker;
- actual pinned OLT/ONT/router identity and firmware;
- dedicated real read-only account;
- independently verified operational recovery;
- durable worker request/claim model for physical read-only probes;
- vendor-specific live SSH/SNMPv3/CWMP/USP interoperability;
- evidence-to-health freshness and diagnostic correlation;
- production PostgreSQL/K3s/offsite disaster recovery.

The next implementation milestone should create a durable
**read-probe intent** queue that may be generated only from a fresh
R9.1 eligible projection and still requires a separately authorized
worker adapter. No firmware/write actuator belongs in that milestone.
