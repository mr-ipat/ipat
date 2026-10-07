# R10.07 — Tenant-safe PPPoE bulk CSV dry-run and independent review

This milestone closes the non-writing commercial planning slice of PRD FR-020/AC-05. It does not claim physical MikroTik compatibility and deliberately contains no RouterOS execution, claim, or lease function.

## Operator flow

1. Current authenticated Tenant Admin opens the PPPoE plans module. Unauthorized roles do not receive the menu capability and the backend still independently rejects every API call.
2. Operator selects an existing same-tenant MikroTik router metadata row whose management transport is exactly routeros_api_ssl, lifecycle is SAVED, Site is valid and explicitly assigned to a real POP. SAVED does not mean physically connected.
3. Operator supplies strict CSV with exact header:

       subscriber_id,action,username,profile,secret_ref

   One to 128 data rows are accepted. Supported actions are create, update, and disable. Client parser handles quoted commas and escaped quotes, rejects malformed quoting, wrong column count, wrong header, and wrong action. There is no password column.
4. Backend revalidates every row. Each subscriber must already be a same-tenant Subscriber360 profile bound to the exact selected distribution router. Current subscriber PPPoE username becomes the immutable before_username. create requires no current username; update and disable require a current username. create and update require a tenant-bound vault://tenant/<exact-tenant-uuid>/pppoe/... reference; raw secrets are not returned.
5. Server computes SHA-256 from exact router and ordered normalized items. Request UUID is also the plan UUID and idempotency basis. Same request plus same payload is idempotent; changed digest, payload, or key collision fails closed. Maximum blast radius is 128 rows.
6. Durable plan starts awaiting_approval, execution_allowed=false, physical_readback_verified=false, basis=subscriber360_declared_not_router_readback, planned rate 20/min.
7. Maker cannot review their own plan. A distinct current Tenant Admin may approve or reject. Approval expires within 30 minutes, but still leaves execution_allowed=false.
8. List and detail endpoints never return Vault references or secret values. Raw plan, item, and audit tables are denied to tenant API, OIDC, and generic application roles.
9. There is intentionally no physical RouterOS write endpoint, queue claim, lease, or executor in R10.07.

## Security boundary

Current canonical membership vocabulary still has only tenant_admin, noc_engineer, helpdesk, and auditor. Therefore this milestone uses two distinct current Tenant Admin humans for maker/checker and does not falsely claim the final PRD role split of provisioning_officer and security_admin plus Product Owner and ISP pilot-owner approval. Before any actual RouterOS write, role vocabulary and policy must be expanded, real MFA freshness checked at high-risk approval and execution, physical router state read immediately before apply, approved diff matched against actual state, per-router execution rate enforced rather than merely stored, timeout and partial success forced into Unknown plus reconcile, and tested native backup and rollback established.

## Evidence and reproduction

- PostgreSQL migration: deploy/db/migrations/0037_pppoe_batch_dry_run.sql
- SQL negative/integration test: deploy/db/tests/test_pppoe_batch_dry_run_integration.py
- Isolated PG16 runner: IPAT_R1007_SYNTHETIC_DOCKER=YES bash deploy/db/tests/r1007_docker_repro.sh
- Actual Axum plus disposable PostgreSQL test: commercial_tenant_api::pg_integration::r1007_real_pg_axum_pppoe_batch_dry_run_never_executes_routeros
- Strict CSV/UI contract: node deploy/scripts/lab/r1007/test_pppoe_dry_run_ui.cjs
- Production PostgreSQL18 first-install manifest includes migration 0037 before reserved 0038.

AC-05 remains BLOCKED for physical acceptance until a real exact MikroTik model, RouterOS, and API-SSL tuple is read, current PPPoE state is safely reconciled, and one narrowly approved physical operation with rollback and readback passes. This source milestone is not device compatibility evidence.
