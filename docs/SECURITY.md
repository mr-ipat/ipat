# IPAT — Security Architecture & Threat Model v0.1

**Date:** 2026-09-25 · **Status:** design baseline + implementation proposals; no penetration test or security certification performed.  
**Primary rule:** Deny by default. Multi-tenant isolation is a **security invariant across every subsystem**, not a frontend setting.

## 1. Assets, actors, trust boundaries

**Critical assets:** subscriber PII, device credentials, PPPoE secrets, ACS/CWMP and USP controller/agent keys, router/OLT configs, operational topology, telemetry, plans/quotas, audit trails, tenant backups, signing keys and admin identities.

**Actors:** platform admin (commercial platform control only), tenant roles, automated service identities, enrolled devices, external attacker, compromised device/tenant employee, compromised third-party/VPS or supply-chain dependency. Platform staff cannot directly reveal tenant device credentials by virtue of platform role.

**Boundaries (each enforced independently):** (A) public user/browser and authenticated tenant API; (B) custom-domain ingress; (C) external CWMP/USP device ingress; (D) authorized private OLT/RouterOS management links; (E) inter-service/queue/worker; (F) app↔database/secret store/object store; (G) shared K3s control plane/platform operators; (H) backup/restore and observability planes.

## 2. Essential invariants / defense in depth

| ID | Invariant | Enforcement & negative evidence expected |
|---|---|---|
| SEC-01 | User tenant scope not chosen by headers or hostname alone | Signed OIDC identity + DB membership + verified domain; hostile header/Host tests deny |
| SEC-02 | Unentitled menus inaccessible at **all levels** | UI filtered nav, route guards and backend `authorize(actor, action, resource)`, reject API/query/search/export/notifications |
| SEC-03 | A worker cannot use queue payload to gain tenant rights | Queue contains job ID, DB job provenance and policy re-evaluation, least-privilege service identity |
| SEC-04 | Device belongs to one verified tenant | CWMP authenticated enrollment and serial/OUI validation, USP endpoint identity/broker ACL bound to immutable tenant assignment |
| SEC-05 | Secrets never propagate across tenant or public outputs | External vault/credential refs, short-lived reads, no secret in logs/metrics/screenshot/export/queue blob, rotation |
| SEC-06 | DB shared storage denies cross-tenant reads/writes even on DAL mistake | Scoped FK/queries + PostgreSQL RLS FORCE; app runtime unprivileged, automated isolation tests |
| SEC-07 | Dangerous operations cannot execute from an unapproved/unbounded request | Immutable plan hash, per-item scope, two-person rule where required, expiry, quotas, kill switch, audit |
| SEC-08 | Device failures/retries cannot silently cause duplicate high-impact provisioning | Durable job transitions, idempotency+lease/fencing, read-back/unknown state, manual intervention |
| SEC-09 | Tenant observability/backup/search are also isolated | No high-cardinality secret labels, tenant-scoped audit/export access, backup encryption/access review and restore validation |
| SEC-10 | Platform support access is explicit and time-limited | Tenant-approved (policy-dependent) just-in-time elevation, scope + reason, notification and immutable audit |

## 3. Threat scenarios and mitigation ownership

| ID | Threat/entry | Consequence | Required mitigations & test |
|---|---|---|---|
| T-01 | Tenant/POP IDOR in Axum API, GraphQL/search/export | Cross-tenant PII and device control | Resource-level PEP + DB RLS, ownership on joins/aggregates, property negative tests for each endpoint |
| T-02 | Host / X-Forwarded-Host spoofing, dangling custom domain | Tenant login takeover, session crossover | Edge allowed-host, DNS TXT verification & periodic re-verification, TLS issuance controls, pinned callbacks, host-only cookie + CSRF |
| T-03 | Malicious CWMP Inform spoofing device ID/ACS URL | Cross-tenant device claim and unauthorized config | Auth credential per enrollment, unique-id collision quarantine, per-device tenant binding, rate-limit, reject duplicate/unknown |
| T-04 | SOAP/XML entity expansion or oversized data/fault | Resource exhaustion/parser data exfiltration | No DTD/XXE, parser limits, maximum XML/message depth, timeouts, malformed payload fuzzing |
| T-05 | USP agent/topic spoof, replay, broker topic ACL gap | False state/unauthorized command | TLS authenticated broker identities, per agent/controller ACL, verified tenant mapping, sequence/correlation replay tests |
| T-06 | Rogue/compromised network device and SSRF via address fields | Lateral network/cloud metadata access | Private/tunneled egress, allowlisted verified target IP/DNS/cert, block metadata IP, network policies, no arbitrary URL fetch |
| T-07 | Malicious PPPoE CSV or overbroad bulk job | Account outage/mass change | Strict schema/encoding/size, tenant duplicate checks, preview, immutable diff, two-person approval, rate & blast radius caps |
| T-08 | Ambiguous timeout, duplicate RabbitMQ messages, dead worker | Repeat config execution | DB outbox/idempotency, lease/fencing, per-device lock and unknown/reconcile state; chaos fault injection |
| T-09 | RLS owner bypass / pooled connection setting bleed | Cross-tenant data leak | Separate migration/runtime users, NO BYPASSRLS, FORCE RLS, `SET LOCAL` transaction context, tenant negative tests on pooled connections |
| T-10 | Secrets in structured logs, metrics, DLQ, support bundles | Credential leak | Redaction at source, reference IDs only, scrub test/failure sampling, retention RBAC and security audit |
| T-11 | Shared backups/restore or object key/prefix confusion | Mass tenant breach or silent data mix | Offsite encryption, scoped restore operations, immutable validated manifests, no self-service raw shared DB backup |
| T-12 | Supply-chain dependency, compromised CI/GitOps | Fleet compromise | Pinned deps/lockfiles, SBOM, signed build artifacts, minimum CI privileges and protected deployment approvals |
| T-13 | Compromised platform owner / support account | Cross-tenant blast radius | No automatic device-secret rights, MFA, JIT access and dual approval, segmentation and elevated activity alerts |
| T-14 | Stale topology/telemetry treated as fact | Wrong remediation and service outage | Evidence timestamp and uncertainty, stale gating, human approval for high-impact actions |
| T-15 | Subscriber/user PII exported in diagnostic evidence | Privacy incident | Purpose-limited fields, redacted evidence, row/column policy and retention classification |

## 4. RBAC + ABAC model v0.1 (proposal subject to owner approval)

- **Default:** every action/resource/scope denies unless explicitly listed. New frontend menu, endpoint, command, event consumer or export must ship with a named policy and a denial test.
- **RBAC** establishes coarse role; **ABAC** checks tenant, site/POP, region, device/subscriber ownership, operation risk, maintenance window, role session strength (MFA), ticket and expiry. Server calculates attributes from trusted database and signed identity, not request fields alone.
- **Policy inputs:** `subject`, `service_actor`, `action`, `resource_type/id`, `verified_tenant_context`, `attributes`, `auth_strength`, `requested_plan_hash`, `approval`, `time`. Policy output `allow/deny` with reason code; no hidden implicit admin wildcard.
- **Policy matrix initial suggestion:**

| Role | Typical permitted scope | Explicit non-entitlements |
|---|---|---|
| Platform owner | Create tenant, plans and aggregated billing/usage metadata | No implicit device-secret/tenant subscriber read or network write |
| Tenant admin | Tenant membership, eligible role delegation, tenant settings | Platform operations and global data; high-risk write without approval |
| System admin | Scoped infrastructure/config operations | Bypass security policy or secret access by role alone |
| Security admin | Policy, access review, privileged approval per separation rule | Approve own high-risk request; cross-tenant unapproved secret access |
| NOC manager | Tenant/POP diagnostics, operations and eligible change approval | Platform/global access and unrestricted high-risk execution |
| NOC engineer | Read inventory/telemetry/incidents; approved scoped diagnostics | User access/secret export/unapproved bulk write |
| Provisioning officer | Scoped onboarding and create dry-run/request | Self-approval or unrestricted bulk change |
| Helpdesk | Minimal subscriber view and permitted diagnostics | Bulk provisioning, full configs, device secrets, restricted menus |
| Field technician | Assigned work and device subset during work window | Org-wide subscriber export, policy editing, unrelated POP data |
| Auditor | Scoped audit events and approved evidentiary export | Mutations, secret reads, jobs |

**MVP proof:** render one helpdesk and one provisioning actor, test allowed vs disallowed menu/route/API/POP/tenant; full role matrix implementation before commercial release.

## 5. High-risk classification & approval lifecycle

**Baseline high-risk examples:** PPPoE bulk create/update/delete, OLT config writes, mass reboot/reset, firmware push, secret reveal/export/rotation, platform role elevation, bulk subscriber export, cross-tenant support access, emergency device changes. Risk tier and thresholds per tenant/device/service to be finalized (`ADR-009`).

Lifecycle: `draft → validate → dry-run/diff immutable → request approval (reason/ticket/TTL) → distinct qualified approver → execution lease → per-item verify/reconcile → signed/redacted audit`. If plan hash, target set, risk class or operator scope changes, revoke approval and require new approval. Emergency break-glass needs policy, MFA, reason, timebox, just-in-time credentials, alarms, post-facto review; never silently bypass all controls.

Worker independently verifies unexpired approval, policy and plan hash immediately prior to any write. Limits include maximum devices per batch, per-router concurrency/rate, tenant daily quotas and kill switch. No automatic high-impact remediation in diagnostic engine without separately approved policy + tests.

## 6. Secrets, cryptography, and tenant data

- Ingress/APIs use TLS; USP MQTT TLS plus topic ACL if approved; device RouterOS `api-ssl` or HTTPS only with validated certificates for staging/production; CWMP auth/TLS strategy adapts to measured ONT capabilities without silent security downgrade.
- Deploy secrets out of Git: KMS/Vault product selection open. Secret references in DB/job payload; least-privileged service identity per service/tenant scope, runtime retrieval and audited rotate/revoke. Do not give generic platform-admin direct tenant-secret read.
- Encrypt tenant PII and backups at rest with managed keys; evaluate per-tenant envelope keys and rotation cost; backups and restore operators with dual-control and export risk classification. Distinguish platform data vs customer data and support audit retention.
- Synthetic examples: `${IPAT_TEST_DB_PASSWORD}`, `${IPAT_LAB_ROUTER_PASSWORD}` etc are placeholders, NEVER usable production secrets. Evidence must redact Authorization, SOAP credentials, PPPoE secret, JWT and subscriber PII.

## 7. Logging, monitoring and incident response

Every authorization, protocol identity binding, high-risk request/approval, execution, failed/replayed job, device access and restore emits structured event with timestamp UTC, sanitized subject/tenant/resource/correlation ID, policy rule/result and request hash (not payload secret). Central logs should be append-only/tamper-evident according to product tier; audit role reads via tenant-verified policy. Rate-limit security alarms; observability tenant labels cardinality/security reviewed.

P0 incident triggers: any confirmed cross-tenant access; compromised agent/credential; mass write unintended; accidental raw backup exposure; unauthorized platform support access. Runbooks: disable ingress/service actor, quarantine impacted device/tenant tokens, pause affected queue and prevent replay, preserve redacted evidence/immutable audit, root-cause, tenant notification per contract/law. No live credentials in ChatGPT or Git issues.

## 8. Verification gates

| Gate | Minimum automated/manual evidence | Phase |
|---|---|---|
| G-01 | Cross-tenant and cross-POP negative tests for all delivered API + jobs + export/search, authz snapshot UI | S1 and every release |
| G-02 | Unknown/untrusted device not automatically enrolled; malformed CWMP fails safe | S1 simulated, C physical |
| G-03 | Bulk PPPoE dry-run does not write; expired/self-approval/plan diff denied | S1 simulated/lab, C all supported adapters |
| G-04 | DB RLS owner/pool bypass and `SET LOCAL` tests; denied path cannot leak row counts/IDs | S1 on selected schema, C full tables |
| G-05 | USP ACL spoof/topic/replay and tenant mapping tests | PoC S1 if transport works; C conformance |
| G-06 | Secret redaction in logs, audits, DLQ, exports, traces, metrics and test artifacts | S1 representative, C comprehensive |
| G-07 | Static dependency scan, SCA/SBOM, image signed build + threat model review | C |
| G-08 | Penetration test and backup-isolation/restore/failover security drill | Before commercial |

## 9. Open questions, dependencies and privacy

Unresolved: precise role/action matrix and maker-checker thresholds; MFA for all roles vs privileged only; Keycloak version/multi-tenant realm strategy; data residency and lawful retention/deletion; vault/key ownership; client certificate/device enrollment fallback; external broker ACL choice; audit retention and tamper-resistance technology; tenant-specific encryption and shared physical-backup contractual disclosure. Track in `DECISIONS.md` and assign reviewers before commercialization.

**References:** `IPAT_PROJECT_BRIEF.md` as source of decisions; `ARCHITECTURE.md` and `PRD.md` for scope. PostgreSQL RLS mechanics: https://www.postgresql.org/docs/current/ddl-rowsecurity.html. No independent security audit or physical test has occurred in this documentation milestone.

## 10. Ubuntu restricted laboratory SSH posture (2026-09-25 execution note)

This implementation note does **not** approve an architectural change. The independently executed Stage-1 installer successfully provided Rust/C compilation, but its privileged SSH policy report showed that remote root and password-based SSH remained enabled in evaluated synthetic-loopback contexts. The host still has **no VPS snapshot or complete off-host recovery copy**. Current Stage-2 key-only policy and timed automatic rollback are strictly **PREPARED, NOT APPLIED**; see [the controlled Stage-2 runbook](../deploy/scripts/lab/STAGE2-SSH.md) and [actual evidence](PROJECT_STATUS.md). The owner's separate interactive `CONSOLE_READY` and `DISABLE_PASSWORD_SSH` confirmations are mandatory before applying globally visible SSH policy changes. Provider firewall and K3s networking must be reviewed in separate transactions; do not treat SSH hardening source-code tests as evidence that the live firewall, backup or application isolation is secure.

## 11. Actual Stage-2 SSH outcome and K3s security prerequisites (2026-09-25)

**Later verification supersedes the historical pre-change state in section 10.** The owner successfully executed the gated, six-minute timed-rollback SSH Stage 2 and confirmed its completion. An independent NEW Mac public-key SSH session succeeded; the root-owned early config snippet explicitly disables root and password SSH, the rollback marker is absent and a password-only diagnostic was refused by the actual `openai` server connection. We have **not** independently exercised privileged effective-policy checks for every possible SSH Match/source address, nor a failed-SSH timed-rollback drill. SSH port and firewall were unchanged. Do not conflate key-only SSH with network ingress security.

The real Ubuntu 26.04.1 rootless K3s host preflight is [recorded separately](LAB_K3S_READ_ONLY_2026-09-25.md). Provider ingress, public Flannel/Kubernetes port exposure, full encrypted independent recovery, multi-node private overlay and production K3s datastore remain unverified. **Do not expose TCP/6443 or UDP/8472 publicly or deploy customer data until those gates pass.** Architecture decision ADR-017 remains OPEN.
