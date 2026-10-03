# R9.78 — Tenant-admin maker/checker lifecycle for real-POP NOC access

R9.77 establishes typed real-POP read authorization. R9.78 adds the normal operator lifecycle so an administrator never needs a direct database insert.

## Flow

1. A current Tenant Admin opens NOC access. The server lists only currently valid NOC memberships and registered real POPs for the Host-bound tenant.
2. The maker requests one NOC identity → one real POP with a server-computed expiry. Browser submits a bounded duration in hours; it does not choose an authoritative timestamp.
3. The request remains PENDING. The same issuer+subject that requested it cannot approve it.
4. A different current Tenant Admin reviews it. Approval rechecks the target NOC membership, tenant state, POP existence and requested expiry at the moment of review. Rejection is terminal.
5. Approval creates/reactivates the typed grant from R9.77 and immediately affects NOC capabilities. Revocation by a current Tenant Admin sets revoked_at; subsequent reads lose the real-POP scope immediately.
6. Failed approval because the NOC member became ineligible does not silently discard the request. The pending audit record remains until a valid different checker explicitly rejects it.

Legacy exact-Site grants remain outside this workflow and are never converted to real-POP grants.

## Security boundary

The ordinary OIDC session issuer has no workflow execute privilege and raw request/grant/event tables remain unreadable to tenant API login. The commercial API first verifies the durable Host-bound session and Tenant Admin capability; the SQL functions independently repeat current membership checks. Mutations require the existing same-origin CSRF control. NOC users receive 403 from the grant-administration API and no grant mutation menu.

The workflow only changes read authorization metadata. It cannot configure an OLT, move a Site, change a Device, retrieve management endpoints, reveal Vault references, activate a domain or create an NOC identity.

## Reproduction

Before HTTP mounting, the isolated owner-Mac PostgreSQL16 runner applied selected canonical migrations through 0031 and passed 40/40 previous plus lifecycle tests. During development an SQL UNION ordering bug was caught before acceptance and fixed with an explicit subquery. Two later test failures were traced to incorrect test oracles: a supposed idempotent retry generated a different expiry, and a failed approval correctly retained a Pending request. The tests were corrected without relaxing production guards, then 40/40 passed.

GitHub exact-head CI additionally compiles the mounted Rust API and exercises maker request, self-review denial, independent checker approval, immediate NOC capability, NOC 403 on administration, and revoke on the disposable Host-bound Axum/PostgreSQL path. Public IdP/MFA and production infrastructure remain separate gates.
