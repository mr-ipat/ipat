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
| ADR-018 | PROPOSED | Optional IPAT-native host firewall control plane using a narrowly privileged Ubuntu nftables agent, source-scoped dual-stack policy drafts and audit/approval; **no external hosting-provider firewall integration** | Portable tenant-aware operations without requiring vendor-specific APIs; preserve CNI ownership and rescue access | Validate real nftables/iptables-nft/CNI coexistence on disposable node, independent console recovery, root config restore, signed approval matrix, timed rollback and fresh IPv4/IPv6 tests before any apply |
| ADR-020 | APPROVED_SEQUENCE | Product owner defers FR-004 customer subdomain/custom-domain verification and domain-specific session isolation to M2 AFTER a working private integrated laboratory; the prototype remains strictly SSH-loopback-only until separate verified identity, tenant/POP scope and release gates pass. No public multi-tenant entrypoint or relaxation of FR-001/002/003, AC-01/02 is authorized. | Prioritize original ACS/USP/device lab and authentication/authorization without premature public DNS complexity. FR-004/AC-09 stay mandatory before domain-based customer onboarding. | Execute R7.5 locked single private lab endpoint and denial tests. ADR-014 technical domain/OIDC/cookie design stays OPEN until M2 with security review; physical OLT read and real dashboard authentication remain separate incomplete work. |

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

## ADR-019 (OPEN): ZTE C320 controlled firmware upgrade request — R7.1

Owner requests real DEV-01 early read-only pilot and firmware
upgrade. This is a high-risk scope expansion beyond guaranteed
first-week deliverables, NOT implementation approval.
Alternatives: remain read-only pending tests; supervised manual
vendor-led isolated upgrade under independent OLT owner process;
or later native IPAT staged firmware job after exact model/card/
release proof, offline vendor image checksum, two-person approved
maintenance, blast radius caps, backup restore and onsite rollback.
Selection requires actual chassis/board/firmware, official
matching vendor upgrade/rollback release notes and signed
product/security/ISP owner authorization. No candidate
can be labeled implemented or validated by synthetic flags.


## ADR-021 (APPROVED INTENT, TECHNICAL DESIGN OPEN): Fadly customer-host naming — 2026-09-27

Owner clarifies ipat.fadly.id is intended as Fadly company tenant's
future CUSTOM domain and ipat.id as IPAT's future commercial platform
brand; both remain ownership/DNS/TLS UNVERIFIED by IPAT. The current
ipat.fadly.id address is ALREADY used as a vetted SSH target for the
laboratory VPS. Do not silently use that address as an active tenant
selector, repoint it, expose HTTPS, or change the management pathway.
An approved recovery-tested management separation is a prerequisite
before a future domain-routing change. Stable tenant identity comes
from trusted membership records, not IP/FQDN/JWT extra claims.
ADR-020 domain-verification scheduling is unchanged; ADR-014
technical approach and ingress/cookie design remain OPEN.


## ADR-022 — PROPOSED / SYNTHETIC-LAB ONLY: read-only identity query role (R7.7)

Candidate initial trusted-membership adapter design: PostgreSQL
security-definer exact issuer+subject+tenant+role+POP lookup owned by
a narrow NOLOGIN role with two targeted RLS SELECT policies, locked
function search_path, separate NOLOGIN query-only role, and no GRANT
to current ipat_app_runtime. User/token/Host cannot mint membership.
Selected ONLY for disposable lab tests as a security reference:
NOT a production authorization design approval or real IdP binding.
The function's approved_by column is not proof of authenticated review.
Before ADR-022 can be approved: independent threat model review,
audit/MFA grant workflow, fixed application identity broker,
credential/storage isolation and cross-tenant real API RLS tests,
recovery/rollback and negative privilege escalation tests.
FR-001/002/003 remain mandatory despite ADR-020 domain deferral.


## ADR-024 — PROPOSED LAB ONLY: real Axum OIDC→scoped SQL menu adapter (R7.8)

For isolated private demonstration ONLY, join the pinned OIDC verifier
to the separately tested exact-membership PostgreSQL function and
existing fail-closed Rust menu policy using a dedicated restricted
`ipat_lab_identity_reader` role and a private Unix socket. Explicit
double opt-in is mandatory, with separate nonroot 127.0.0.1:3001
binding and no public/K3s activation or real business route access.
The only seeded LOGIN account is disposable CI-only. Actual IdP MFA,
independently approved human membership, audited approver identity,
rotation/revocation handling, transactional RLS data access,
trusted production identity broker and commercial custom domains
remain UNSOLVED. No production approval or automatic live DB
migration is authorized; do not confuse this with full FR-002.


## ADR-025 — APPROVED DIRECTION / ACTUAL TRANSPORT PROPOSED (R7.9)

Owner clarifies that the initial DEV-01 ZTE C320 does NOT require
a direct physical Layer-1 connection or on-site serial-console
session to manage/read it; it must be accessible by a supported
remote management protocol over a private network path. Candidate
order: vendor-documented SSH CLI for bounded read-only
chassis/card/running-version capture; SNMPv3 telemetry as
separate candidate once real firmware, MIB and configuration
are independently verified. Existing original ACS TR-069
remains for CPE/ONT; do not assume an OLT-side TR-069 API
or generic REST on this specific firmware. R7.4 strict
physical console packet remains a conservative OPTIONAL
legacy route; new R7.9 remote-read packet is an ALTERNATIVE,
requiring owner authorization, true private route,
independently pinned SSH host identity, verified
read-only account, read-exec command support and encrypted
private capture, but not a physical cable. Actual
vendor/firmware interop and all writes remain unapproved.
A tested independent recovery route, valid vendor
firmware image, rollback and maker-checker approval
ARE mandatory BEFORE any disruptive firmware changes.
See docs/C320_REMOTE_AND_K3S_PROVIDER_NEUTRAL_R79.md.

## ADR-026 — APPROVED PROVIDER-INDEPENDENCE / PROPOSED NETWORK DETAIL

Owner requires portable heterogeneous K3s across VPS
providers without requiring their external firewall
APIs or IPAT integration with third-party firewalls.
Pilot design candidate: one low-latency single server
(control plane) and heterogeneous x86_64/arm64 workers
across provider boundaries over an independently managed
encrypted private WireGuard tunnel `wg-ipat`. K3s
Flannel VXLAN is tunneled INSIDE WireGuard (never
raw public UDP 8472), 6443 reached only by trusted
private tunnel peers. The host firewall MAY be absent
only where effective ingress isolation is independently
verified, not as a replacement for network security.
A multi-provider three-member etcd cluster is NOT
automatically accepted: protect HA server quorum
with low-latency same-site private networking.
PostgreSQL HA and whole-host offsite DR remain
independent requirements. Provider-independent
OFFLINE plan code is not production installation
approval; ADR-017 live network/rollback and
external recovery gates remain OPEN.


## ADR-028 — PROPOSED PRIVATE LAB ONLY: pinned JWT to sealed tenant/POP inventory (R8.0)

Approved implementation scope for VIRTUAL pre-device integration:
connect the already pinned signed-JWT verifier to a second,
narrow FORCE-RLS-protected PostgreSQL SECURITY DEFINER
read-only inventory function, and join its results with an
active exact membership from the SAME DB statement
snapshot in actual private Rust Axum. Limit the first
inventory endpoint to exact POP-scoped NOC membership:
tenant_admin broad POP access requires separate reviewed
backend semantics, not a guessed default. No service
table SELECT, direct browser SQL, shared runtime-role
grant, user-controlled SET ROLE, auto-generated
platform privileges or real hardware writes. A
NOLOGIN function owner gets SELECT on the FORCED
RLS devices table through its own named SELECT
policy, and only an EXECUTE-only separately
provisioned query identity may invoke it.
Production use still needs trusted actual human
OIDC/MFA approval provenance, restricted real
service identity, review of schema-owner threat
boundaries, eventual pagination and security
assessment. No approval to migrate an actual
customer DB or expose real company APIs.


## ADR-029 — APPROVED VIRTUAL USP WIRE SLICE / FULL MTP STILL PROPOSED (R8.1)

While the real devices are unavailable, implement a
native original Rust read-only USP v1.4 **Protocol
Buffers subset** based on Broadband Forum's
published `usp-record-1-4.proto` and
`usp-msg-1-4.proto`, using a pinned `prost`
dependency and separately hand-encoded
synthetic binary golden fixtures. The subset
handles offline no-session Get serialization
and strictly bounded structural GetResp
inspection. Its external claimed from/to
endpoint IDs MUST NOT become trusted tenant
identity; cryptographic VerifiedAgent
constructors remain restricted to independently
authenticated protocol adapters. Duplicate
or unrecognized record/message/oneof
fields are rejected, not interpreted
as unsupported commands; no automatic
write, session, transport or enrollment.
A local-only opted-in Axum parser endpoint
may report safe counts with explicit
false auth/tenant/session flags, but cannot
accept actual managed-agent messages.
In K3s mode even that parser is ABSENT.
TLS MQTT MTP with independently bound
client/cert/broker ACLs and durable
tenant-scoped enrollment remains OPEN;
the v1.4 subset is NOT certification
or a silent claim of latest spec coverage.
See docs/R81_NATIVE_USP14_PROTOBUF_PRIVATE_VIRTUAL_AGENT.md.


## ADR-030 — APPROVED PRE-DEVICE RUST ACS VIRTUAL SOAP / NO PUBLIC ADMISSION (R8.2)

The owner approves continuing software-first ACS development
while actual test ONTs/OLTs are still offline. Implement
three actual LOCALHOST HTTP SOAP proof routes in the
ORIGINAL Rust CWMP gateway, using the existing strict
Inform parser+serializer and one strict read-only
GetParameterValues/GetParameterValuesResponse RPC.
Admit ONLY an immutable fake CPE identity/event and
one fixed non-secret parameter. Never assume HTTP Host
or claim headers authenticate a real device. Separate
opt-in environment flags plus loopback and absence
under K3s mode are mandatory. The live `/cwmp`
endpoint remains HTTP503 pending real peer admission,
durable session/replay, trusted device→tenant binding,
interoperability and security review. This virtual
two-call demonstration is NOT a persisted/authenticated
CWMP session or certified TR-069 compatibility.
Actual production implementation remains OPEN.


## ADR-031 — APPROVED PRE-DEVICE DEVICE WORKSPACE AND DUAL PRIVATE DRAFT REGISTRY (R8.3)

The owner's visible complaint is binding: IPAT MUST provide
device registration, inventory lists and truthful condition
visibility BEFORE any physical OLT/ONT adoption exercise.
We implement a local interactive strictly SYNTHETIC
demo linked from all existing private dashboard previews,
and a distinct separately approved original Rust
pinned-RS256→PostgreSQL tenant-admin pending-draft backend.
They MUST NOT silently share the demo's identity,
authorization, temporary storage, data or privileges.
A separate NOLOGIN write function with immutable
pending_review, connectivity unknown and health
not_measured, plus isolated EXECUTE-only
registrar/reader accounts and row-level tenant/POP
membership checks prevent falsely online devices.
A duplicate request UUID with changed metadata MUST
fail. No firmware, real network scan, automatic
approval, customer public API, stored secret,
anonymous real-device import or tenant-wide NOC
permission is approved. Real MFA, auditable
maker-checker approval, evidence-backed
online/health transitions and validated vendor
protocol integration remain binding OPEN MUST
gates; approval of this design does NOT
approve live user enrollment or DB migration.


## ADR-032 — APPROVED METADATA MAKER-CHECKER LAB / PRODUCTION AUTH STILL OPEN (R8.4)

Implement a distinct sealed `security_admin` reviewer
role/SQL executor for candidates proposed by
`tenant_admin`, but only for **metadata review**.
Database enforces separate signed issuer+subject,
active tenant/current unrevoked privileges,
atomic single-verdict row lock, append-only audit,
bounded rationale and idempotency; neither approval
nor self-declared device identity authorizes
physical device enrollment, health claims or
any firmware operation. Gate Rust reviewer
endpoints with an independently validated
signed exact `amr:mfa` flag from configured
pinned issuer, an additional private opt-in,
nonroot, localhost, and a dedicated
Unix-socket reviewer service identity.
Actual Keycloak/approved IdP `amr` semantic
contract and real user enrollment, real
signed-in browser BFF, secure operator login
and production recovery are STILL PROPOSED/
OPEN; synthetic CI MFA claims cannot satisfy
actual human MFA controls.


## ADR-033 — APPROVED INDEPENDENT PINNED REAL-IDP PREFLIGHT; REAL IAM STAYS GATED (R8.5)

Before real customer company sign-in or any physical device
adoption, independently establish an actual operator-controlled
OIDC HTTPS issuer, pinned signing-key provenance and verified
human second-factor semantics. Add an original standalone
unprivileged bounded Rust process that only checks a
short-lived pinned RS256 access JWT's exact `amr:mfa`
and refuses operator-revealing output. Do not trust tenant,
roles, POP or human MFA enrollment solely from a token;
only existing separately approved actual DB membership
can grant a role, and actual IdP 2FA configuration and
challenge remain independent external human gates.
The readiness CLI cannot mount HTTP, create sessions,
adopt devices, authorize firmware or enable K3s.
Keycloak/OIDC browser BFF provider/deployment choice
remains PROPOSED until actual independently approved,
fully protected production operator enrollment.
See docs/R85_INDEPENDENT_REAL_IDP_SIGNED_MFA_ADMISSION_PREFLIGHT.md.
