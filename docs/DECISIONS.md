# IPAT — Architecture Decision Register v0.1

**Dated:** 2026-09-25 · **Control rule:** `APPROVED_BASELINE` entries were explicit in `IPAT_PROJECT_BRIEF.md` / Project Instructions, not a claim that they have been implemented; `PROPOSED` require approval; `OPEN` require a decision/evidence. Superseded decisions must record date, rationale and affected docs. **No independent product-owner sign-off on the newly drafted v0.1 docs is represented here.**

| ADR | Status | Decision / proposal | Rationale and constraints | Validation / required next step |
|---|---|---|---|---|
| ADR-001 | APPROVED_BASELINE | Original ACS Rust/Tokio/Axum with CWMP/TR-069; GenieACS not engine | IPAT own core protocol | Test Inform, fault/session and RPC with simulator and individual physical ONTs; independently review standards and licensing |
| ADR-002 | APPROVED_BASELINE | Distinct native TR-369/USP Controller + shared normalized model from day one | Avoid CWMP-only architecture | Pin USP A5 reference feature subset; MQTT only transport candidate; interop evidence |
| ADR-003 | APPROVED_BASELINE | Modular monolith business domain + separately deployable protocol/worker/infra | Reduce premature microservice complexity | Define crate boundaries and test no dependency cycles |
| ADR-004 | APPROVED_BASELINE | Ubuntu Server 26.04 LTS, Rust Tokio/Axum; K3s heterogeneous scale; bounded queues/worker, locks/backpressure; Ansible/Helm/GitOps and Terraform when supported | Portable scaling without duplicate writes | Second node, job failure tests and measurement; database separately HA |
| ADR-005 | PROPOSED | PostgreSQL platform logical metadata separated; operational shared tables with tenant_id, RLS FORCE, runtime no BYPASSRLS; later tenant-dedicated DB tier | Initial ops/cost balance; preserves migration option | Compare 3 models (shared+RLS vs schema per tenant vs DB per tenant) against isolation/restore/cost, load, enterprise residency; security review before final |
| ADR-006 | PROPOSED | Next.js/TypeScript, Keycloak OIDC+MFA, RabbitMQ *work queue*, Prometheus/Grafana | Cohesive common stack matching brief | Dependency/licensing/operations review; verify pinned compatibility on Ubuntu 26.04 |
| ADR-007 | PROPOSED | Native USP initial MTP over MQTT with separate broker trust boundary | Target initial interop, not full transport claim | Compare broker choices, ACL/mTLS, A5 subset vs actual simulator/agent versions; decide prior to full controller |
| ADR-008 | OPEN | Vault/KMS, tenant key model, object store, telemetry retention/partition strategy | Tenant secrecy and stateful scaling | Select providers/design based on compliance budget and storage measurements |
| ADR-009 | MIXED | RBAC+ABAC deny by default, hidden menu + backend denial, high-risk approval, platform-vs-tenant boundary are APPROVED; exact role/action matrix and two-person thresholds PROPOSED | Minimize cross-tenant privilege and device blast radius | Sign off role policy, JIT support policy and emergency access drill |
| ADR-010 | MIXED | PostgreSQL primary/standby + PITR external backups required for production APPROVED; topology/operator tool/RPO/RTO OPEN | Stateful availability independent of app worker count | Production resilience workshop, provision HA, measure failover+restore |
| ADR-011 | APPROVED_BASELINE | Diagnostics are evidence-based fault-domain hypotheses; no fiber cut inferred from missing CWMP alone | Prevent unsafe wrong remediation | 3 synthetic rules S1 then calibrated field evidence |
| ADR-012 | APPROVED_BASELINE | Pilot target list ZTE C320/C-DATA OLT, VSOL/ZTE ONT, MikroTik x86/CCR/RB distribution + customer RB; VSOL GPON OLT not physically confirmed | Prevent unsupported claims | Capture model/firmware and per-feature outcomes |
| ADR-013 | APPROVED_BASELINE | Pilot provisional node 16 vCPU/64 GiB-class/~1 TB NVMe, Ubuntu 26.04; higher variant 16/128/~2 TB for heavier all-in-one pilot | Source approved brief supersedes any alternative conversational sizing | Collect ONT/tenant/retention/inform/job load. 16/32/250 only constrained experiment, not committed baseline |
| ADR-014 | OPEN | Custom-domain verification strategy, tenant OIDC realm/client topology, session cookie handling | Multi-domain security | Threat/test design; legal ownership/registered IPAT domain evidence; choose IdP mapping |
| ADR-015 | OPEN | RLS/tenant quotas, encryption boundaries, tenant deletion and raw backup restore strategy | Commercial isolation/privacy | Contract and disaster-recovery test design |
| ADR-016 | OPEN | Specific supported CWMP/USP RPC feature set and data models on exact physical firmware | Availability varies by device/firmware | Device inventory and lab execution before support claims |
| ADR-017 | OPEN | Selected cloud/VPS, network overlay, Kubernetes datastore/control-plane design and storage classes | Mixed VPS/bare metal constraints | Network/NAT/IO cost and capacity assessment |

## 1. Decision questions with suggested owner / due gate

| Question | Suggested accountable role (not assigned) | Evidence needed | Due gate |
|---|---|---|---|
| Q-01 PostgreSQL tenant strategy and shared-backup controls? | Principal Architect + Security Architect | threat model, isolation/restore cost and performance comparison | Before schema freeze beyond S1 |
| Q-02 USP subset and MQTT broker selection? | Rust Protocol Lead + Network Engineer | BBF A5 mapping, agent inventory, security PoC | Before M1 protocol expansion |
| Q-03 Exact hardware/firmware/management methods? | Lab Owner + Network Engineer | asset intake form and authorized connectivity | S1 D1; if unavailable mark blocked |
| Q-04 Who approves bulk PPPoE and emergency writes? | Product Owner + Security Admin + ISP pilot owner | signed role matrix, blast-radius policy, break-glass drill | Before any real write |
| Q-05 Keycloak, Vault/KMS and external OSS licensing/ops model? | Platform/Security leads | deployment/security review and pinned compatibility | Before commercial pilot |
| Q-06 Real SLO, RPO/RTO, expected size and telemetry retention? | Product Owner + SRE | tenant/ONT volume, poll/Inform rate, benchmark, budget | Before production architecture capacity freeze |
| Q-07 Customer data residency, customer domains and privacy clauses? | Product Owner + Legal/Compliance | target jurisdictions and contract terms | Before sale/customer onboarding |
| Q-08 HA topology, cloud/bare metal provider, network boundary? | SRE + Principal Architect | available provider APIs, routing/link tests, measured failover | Before M3 HA deploy |

## 2. Decision process

1. Write decision request, alternatives, trade-offs, proof required, owner and requested date; annotate as `PROPOSED`, do not silently treat as implemented or approved.
2. Security-sensitive decisions require product owner/security sign-off; vendor support requires lab evidence per tuple. Decision record should include `decided_at`, `decided_by`, link to test/evidence, migration implications and rollback strategy.
3. On approval/change: update **this file**, affected `ARCHITECTURE.md`/`SECURITY.md`/`PRD.md`, `SPRINT_BACKLOG.md` and `PROJECT_STATUS.md` in the same tracked change; document superseding ADR rather than deleting rationale.

## 3. Sources and external reference maintenance

Binding decision source: `IPAT_PROJECT_BRIEF.md`. Reference versions checked 2026-09-25: Broadband Forum *in force* TR-069 Amendment 6 Corrigendum 1, TR-369 Amendment 5, and TR-181 Issue 2 Amendment 21. Standards change/interop impact evaluated at each release; standards publication is not implementation certification.
