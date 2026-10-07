# R10.09 — Commercial tenant role separation for PPPoE dry-run

R10.07 intentionally stopped at two distinct Tenant Admins because the canonical commercial membership vocabulary did not yet contain Provisioning Officer or Security Admin. R10.09 closes that source-level PRD mismatch without enabling a RouterOS write.

## Commercial roles

Migration `0039_commercial_role_separation.sql` expands the durable tenant membership vocabulary to the PRD roles:

- `tenant_admin`
- `system_admin`
- `security_admin`
- `noc_manager`
- `noc_engineer`
- `provisioning_officer`
- `helpdesk`
- `field_technician`
- `auditor`

Existing role scope semantics are preserved. Tenant Admin, System Admin, Security Admin and Provisioning Officer are tenant-wide identities and therefore require `p_pop IS NULL` in the sealed current-membership lookup. NOC Manager, NOC Engineer, Helpdesk, Field Technician and Auditor remain explicitly POP/Site-grant scoped. No role is accepted merely because it appears in a browser token; all authorization uses the current PostgreSQL membership and revocation state.

## PPPoE separation of duties

Provisioning Officer may list plans, inspect non-secret plan items and create a new exact dry-run. Security Admin may list plans, inspect non-secret plan items and approve/reject an awaiting dry-run. Tenant Admin cannot see the PPPoE module merely because it is Tenant Admin and receives backend `FORBIDDEN` for direct list/create/review calls. Provisioning Officer cannot review a plan, and Security Admin cannot create one. Revoking the current Security Admin membership immediately removes review capability.

Existing independent-subject protection remains in the database as a second boundary. `execution_allowed=false`, `physical_readback_verified=false`, and the absence of any RouterOS execute/claim/lease function are unchanged.

The tenant UI receives separate `can_create_pppoe_plans` and `can_review_pppoe_plans` values from the backend. The PPPoE navigation appears only to one of the two workflow roles; the create form itself is hidden from review-only identities. Direct API calls are independently denied.

## Acceptance

Disposable PostgreSQL tests create three real current synthetic identities: Tenant Admin for inventory topology, Provisioning Officer as maker, and Security Admin as checker. They test wrong-role denial, current revocation, tenant isolation, non-executable approval, role vocabulary and raw-table boundaries. A separate Axum + real disposable PostgreSQL test checks browser capabilities and direct HTTP denial for Tenant Admin and the wrong PPPoE role.

This milestone remains non-writing. No RouterOS command, password, device credential, live VPS production database, firewall or customer public service is modified. Physical MikroTik RouterOS/API-SSL readback and any later reviewed execution workflow remain independently required by AC-05.
