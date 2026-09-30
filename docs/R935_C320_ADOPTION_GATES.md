# R9.35 — Physical C320 adoption gate execution plan

Status: in progress; this file is a runbook, NOT permission to change hardware. Treat isolated owner LAB hardware as LIVE during adoption. Never send real credentials or raw CLI captures to Git, ChatGPT or public CI.

## Verified evidence carried from R9.34

- Actual owner-interactive SSH scripted `show card` returned three known INSERVICE physical cards, with bounded offline Rust normalization and owner-only audit.
- Owner-executed off-VPS encrypted Restic CLI reference capture passed isolated byte-identical recovery.
- Neither result proves device-native configuration import/recovery, independent device identity, firmware compatibility, limited service account or automatic tenant adoption.

## Ordered acceptance gates (do not skip)

1. Preserve existing protected private evidence, encrypted Restic snapshot and verified rollback instructions. Do NOT expose the 119980-byte CLI transcript in dashboard or logs.
2. Independently attest physical chassis/management SSH host key (console/OOB or otherwise verifiable owner-approved evidence) and prove POP last-hop isolation before persistent worker enrollment.
3. Obtain exact vendor documentation matching the observed C320 controller/card firmware and validate a DEVICE-NATIVE export/import/restore rehearsal with rollback, approvals and maintenance controls. A recovered CLI transcript does not qualify.
4. Create a dedicated least-privilege device service account only after backup/native rollback and separate operator review. Prove allowed read commands and denied config/credential/firmware commands using an approved non-disruptive negative-test method; never use the current privilege-15 test account for an unattended worker.
5. Reconcile GTGHK versus reported GTXK firmware file type and absent PRAM MVR evidence. Record current alarms and ONT inventory only after their actual read syntax/semantics are separately established.
6. Validate real tenant OIDC MFA and a distinct qualified reviewer with immutable approval record, device/POP scopes and expiry. Neither platform admin nor owner CLI access implies tenant authority.
7. Implement a bounded, audited, read-only production device worker with per-device locking, policy re-evaluation, immutable sanitized receipts, timeouts, no fallback to Telnet and deny on changed host key/firmware/role.
8. Test owner-approved read-only physical end-to-end inventory through the worker and compare exact model/firmware and freshness to independent capture. Test failure modes: host-key change, missing account role, missing reviewer, stale/ambiguous results, retries and cross-tenant denial.
9. Only after all evidence passes, review the adoption transition independently. Firmware/configuration writes remain separate high-impact approvals; read-only adoption never authorizes firmware update.

## Current machine-readable LAB report

`GET /lab/c320-actions` (existing local private read CSRF guard) reports `adoption_gate_report`, a historical display only. It checks 10 existing/current facts; 2/10 are currently verified. Unknown, missing and non-boolean entries fail closed. This is NOT an API to approve gates; no endpoint or production worker is enabled by the report, even if synthetic flags all become true.

**R9.35 stop conditions:** no independent physical proof; no firmware-specific vendor-native restoration rehearsal; no proven restricted account; no tenant MFA/reviewer; no bounded worker. Stop before any persistent hardware write, credential change, firmware action or SaaS `ADOPTED` state if any are missing.

**Verification scope:** Rust unit tests in `apps/control-api/src/c320_actions_lab.rs` cover the displayed historical gate count and missing/non-boolean inputs. These tests require the pinned Rust build environment and do not replace hardware tests. Public repository/CI must contain synthetic-only evidence.