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


## ADR-034 — APPROVED LAB-ONLY RUST S256 BROWSER INIT; COMPLETE BFF OPEN (R8.6)

The original private browser Authorization Code PKCE
initiation may be implemented and independently
tested while actual IdP/human MFA is unavailable.
The first reviewed provider shape is the
Keycloak-candidate exact pinned issuer-associated
HTTPS authorization endpoint, NOT a claim of
provider-neutral discovery. Use OS entropy, one-time
state, independent nonce, RFC7636 S256, bounded
pending proof, fixed Mac-loopback callback and
HttpOnly same-site correlation. Even valid returned
state+code may only be discarded and return HTTP503;
no customer session, role, token exchange, reviewer
rights, device access or live user MFA assertion
may result from the synthetic test. Actual approved
human IdP+MFA, confidential HTTPS BFF and secure
tenant-scoped signed-in dashboard are distinct
OPEN deployment/review milestones.


## ADR-035 — APPROVED OFFLINE OIDC TWO-SIGNED-TOKEN BINDING; REAL BFF OPEN (R8.7)

Prepare an original Rust strict, OFFLINE, actual RSA
pinned-issuer verification of the browser ID token
AND the independently audience-verified access token.
Require exact configured client audience and any azp,
matching signed issuer/subject, original server-private
nonce, constant-time SHA256 at_hash of the exact
independently signed access JWT, five-minute auth_time
and bounded signed mfa method on BOTH tokens.
Return opaque issuer/subject/min expiry ONLY;
never grant tenant, POP, reviewer, session or
hardware operation based on synthetic signed inputs.
No anonymous or simulated token injection into
actual browser callback is approved.
Real independently proven IdP human MFA,
confidential HTTPS PKCE code exchange,
server-side Secure session and separate
approved tenant/role/POP SQL membership
remain binding OPEN MUST gates.
This strict lab method REQUIRES at_hash as an
explicit local profile, though OIDC Core 1.0
section 3.1.3.8 makes at_hash OPTIONAL for an
Authorization Code Token Endpoint response.
Do NOT assume a real Keycloak issuer emits it;
prove actual provider behavior before choosing
the real BFF policy. A future code-flow method
may instead use verified confidential HTTPS
endpoint binding if documented/reviewed, but
must never fabricate at_hash or silently reuse
this strict method under weaker assumptions.
See docs/R87_PINNED_OIDC_ID_TOKEN_NONCE_AT_HASH_OFFLINE.md.


## ADR-036 — APPROVED OFFLINE IDENTITY-ONLY OPAQUE BFF SESSION; LIVE LOGIN OPEN (R8.8)

The approved development order requires a genuine secure
browser session primitive backed by cryptographically
verified independent OIDC ID+access signed pair and
separately approved exact tenant/POP SQL membership
before attempting actual OLT/ONT adoption. The new
original Rust SessionVault issues separately random
256-bit opaque handle and CSRF, stores only SHA256
digests plus bounded issuer/subject/expiry, caps
memory at 64, enforces cryptographic token expiry
and five-minute idle, and supports explicit revocation.
A separately UNMOUNTED private Control API bridge
first requires the R8.7 strict nonce/at_hash/MFA
signed-pair proof AND the genuine restricted sealed
SQL active membership lookup, then rechecks that SQL
on EVERY tenant/POP read or mutation. Host/origin
MUST be independently checked; mutation also
requires constant-time CSRF proof.
The _Secure HttpOnly SameSite=Strict __Host-_
cookie helper is a future HTTPS policy fixture
and is NEVER actually emitted from today's HTTP
lab. No actual IdP/MFA, confidential network
token exchange, secure production TLS, shared
durable session storage, device privileges or
production API route is approved by this ADR.
See docs/R88_OFFLINE_SIGNED_PAIR_SEALED_SQL_SESSION_FOUNDATION.md.


## ADR-037 — APPROVED INTERNAL SESSION→SCOPED DEVICE READ, HTTP STILL BLOCKED (R8.9)

While owner-approved real human OIDC MFA,
verified provider-specific confidential
authorization-code/PKCE redemption and
secure HTTPS browser session transport
are absent, add ONLY an internal unmounted
original Rust BFF function connecting the
R8.8 genuine signed identity-only opaque
session to the R8.3 genuinely sealed
tenant/POP SQL candidate inventory.
A cookie never stores or grants an
implicit tenant, role, POP or device
permission. Every request authenticates
the current session and queries
separately current exact signed issuer/
subject, active approved per-company
membership and optional exact NOC
POP in one PostgreSQL snapshot; the
restricted reader has EXECUTE-only
permissions. Returned metadata has
no management IP, credentials or
unverified physical telemetry.
The existing real `/v1/*` and
live device endpoints remain denied;
actual customer UI login is a separate
mandatory implementation/review gate.
Full production approval NOT GRANTED.


## ADR-038 — APPROVED PUBLIC TELNET NO-AUTH NETWORK EVIDENCE ONLY (R9.0)

Owner authorized one exact public IPv4/alternative TCP321 as a ZTE
candidate OLT management endpoint. Real Mac TCP and Telnet IAC were
observed, and separately an UNAUTHENTICATED peer banner claimed ZTE.
The actual VPS source TIMEOUT was observed independently. Approve
a separately gated original one-host Python receive-only preflight
(never commands, credentials or raw banner) and historical, timestamped
Mac-only dashboard evidence; never infer trusted OLT/C320 or health.
Public plaintext Telnet MUST NOT receive ANY actual or test credential.
Physical access requires independently verified private encrypted
management transport or authenticated SSH/SNMPv3 if actual firmware
supports it, verified device identity, dedicated restricted account,
per-tenant MFA/approval/audit and firmware-matched read-only acceptance.
No public edge firewall changes, Nusa integration, firmware upgrade,
live K3s setup, production tenant enrollment or C320 compatibility
is authorized by this one-time network observation.


## ADR-039 — APPROVED ADOPTION READINESS GATES; READ PROBE EXECUTION STILL BLOCKED (R9.1)

Metadata maker-checker approval is insufficient to contact physical
equipment. Before any future read-only probe intent can be created,
require four separate current evidence gates: secure management path,
device identity, dedicated read-only account and recovery plan.
Store evidence append-only with bounded lifetime, exact tenant/
candidate/reviewer identity and latest-verdict fail-closed semantics.
The attester MUST be the same separately approved security-admin
reviewer who approved the metadata and must still hold active own
tenant membership. Expose only boolean readiness through the
restricted identity reader and opaque-session bridge; never expose
management IP, evidence contents or secrets there. A true
read_probe_eligible value is ONLY a prerequisite result and MUST NOT
enqueue, claim or execute any network operation. A later durable
read-probe intent/outbox and vendor adapter remain separate review
milestones, and all firmware/write actions remain blocked.


## ADR-040 — R9.2 APPROVED DEVELOPMENT-ONLY NONEXECUTABLE READ INTENT

Implement a durable, per-tenant/candidate unique immutable read-probe
request and same-transaction PRIVATE audit after R9.1's FOUR latest
unexpired gates and maker-checker approval are independently checked
again by a narrow PostgreSQL SECURITY DEFINER function. Only active
own-tenant, exact-POP NOC can request; actual browser BFF mutation
requires signed independent ID/access opaque identity, fresh SQL
membership, trusted Host/Origin and correct CSRF. The dedicated NOLOGIN
function owner, EXECUTE-only writer and separate restricted query reader
are distinct. The sole possible state is awaiting_separate_execution_review;
private audit published_at is permanently NULL. The function is
UNMOUNTED and has no broker, worker, management transport, execution
lease, credentials, firmware commands, physical traffic or power to
authorize a subsequent network probe. A future worker MUST revalidate
readiness again, use independent verified site route/device identity,
per-device durable leases, explicit human approval and recovery before
any actual device connection. Production release NOT approved.
See docs/R92_IMMUTABLE_NONEXECUTABLE_READ_INTENT.md.

## ADR-041 — APPROVED PRODUCT DIRECTION / DEPLOYMENT STILL GATED: connection-method choice

Decision 2026-09-28: IPAT must not universally require WireGuard,
VPN or MikroTik RouterOS >=7. Tenant Admin SHALL ultimately select
supported direct secure management (verified SSH/SNMPv3 authPriv),
WireGuard, compatible IPsec or a separately verified IPAT site gateway.
RouterOS 7 is required only for MikroTik built-in WireGuard, not for
IPAT device management in general. A device with ONLY Telnet MUST
be reached solely across an independently verified trusted isolated
private last hop; public Telnet cannot carry authenticated management
credentials. Dashboard choice is the product-facing interface, but
all real tenant rights, reviews and provisioning remain backend-only.
R9.5 UI selector is a lab prototype, NOT deployed tunnel control.
Supersedes any reading of R9.3/R9.4 that WireGuard is universally
required; those earlier documents describe one candidate path only.

## ADR-042 — PROPOSED, NOT APPROVED: live C320 no-change gateway return path

For the owner-reported active distribution ZTE C320 with directly
connected x86 RouterOS7, consider an independently scoped secure
site VPN plus one-destination source NAT on its dedicated management
hop to avoid a live OLT static-route change. THIS IS A PROPOSAL,
not a customer-network change or proof the last hop is isolated.
An alternate independently secured gateway/protocol may be selected.
Requires verified real topology, management VLAN and return path,
existing RouterOS filter/NAT order, owner console/recovery, approved
worker identity, signed tenant review and observed no-impact baseline.
Public Telnet credentials remain forbidden. No tunnel/gateway action
or real C320 provisioning was performed by this ADR.

## ADR-043 — Development-only isolated parallel UI canary

For safe progression without disrupting the previously running
SSH-private Device Manager on the actual VPS, new preview builds may
run ONLY when explicitly opted in on hardcoded `127.0.0.1:3002`,
with actual/synthetic OIDC and tenant-write options disabled, and
no K3s/public deployment. Original :3000 and :3001 must remain
unmodified. Canary test MUST verify actual loopback bind and negative
real-business API responses and terminate cleanly. This is not
a substitute for live MFA Tenant Admin, approved device management,
independently pinned legacy SSH host identity or private VPN rollout.

ADR-043 implementation note 2026-09-28: the temporary smoke runner
was complemented by an actual reviewed nonroot systemd user unit on
the owner VPS, restricted and verified to 127.0.0.1:3002 while :3000
kept serving. Existing host Linger=no remains unchanged; this
unit is an on-login private LAB preview, not persistent enterprise
availability or permission to operate real OLT/ONT hardware.

ADR-043 addendum — R9.12 uses the SAME isolated, nonroot, on-login
private preview service and exact :3002 port; it does NOT enable
actual tunnel provisioning or replace existing :3000. Upgrades must
pin source bundle and compiled binary, review unit checksum, back up
the old unit, retest actual HTTP deny-by-default behavior and retain
immediate nonroot rollback. Linger remains OFF, no HA claim.

## ADR-044 — R9.13 no-credential first-read and strict legacy mode

FIRST live-C320 read uses a separately verified/pinned host RSA key,
site-isolated private management route and dedicated restricted
public-key credentials, never the factory/default account disclosed
in chat. Process-scoped SSH RSA/AES128-CBC may be considered only
for observed exact firmware after review, not as a global downgrade.
An actual worker DEFAULT_ROUTE_ONLY is explicitly inadequate as
proof of private site access; do not grant adoption on that basis.
Real OLT commands and tunnel/router activation remain unapproved
until the full SITE and operational no-impact acceptance ladder passes.

ADR-044 addendum: one ephemeral, encrypted owner-Mac→VPS reverse
Unix-socket relay may be used for specifically authorized NONAUTH
transport checks while durable site gateway is unavailable. It must
not persist, must close after one test, cannot carry live customer
OLT credentials, cannot claim independently isolated last hop, and
cannot substitute for true tenant-owned dashboard-managed tunnel.

## ADR-045 — APPEND-only physical site proof is NOT operational admission

R9.14 six missing site proofs are modeled as expiring independent
security-admin-attested digests in separated no-login PostgreSQL
privileges, NEVER as an executable job or actual physical adoption.
Approved candidate maker and site reviewer must be distinct; latest
BLOCKED/expired evidence resets the affected gate. No BFF HTTP
mount, real MFA, live worker, VPN activation or owner secret intake
is authorized by this database migration.

## ADR-046 — APPROVED design direction: Site A central, Site B self-applied

Owner specified IPAT dashboard/server as Site A, not a remote Router B
configuration-pushing controller. WireGuard is optional; use direct
private address for independently demonstrated same/connected network
management, not merely an RFC1918 address. For external sites, use
verified reachable public Site A endpoint (or an independently
verified routed private interconnect). Generate the reviewed pairing
profile at Site A; Site B operator applies it LOCALLY and retains its
own private key. Site A handles only its own listener, peer and tenant
scope after real signed identity, maker/checker, recovery and narrow
route approval. IPsec remains an alternative in future implementation,
not feature-complete. R9.15 LAB-only design preflight is NOT deployment
approval and carries no hardware-adoption privileges.

ADR-046 implementation clarification: R9.15 may produce an OFFLINE
nonexecuting, disabled=yes RouterOS7 Site B pairing preview only;
Linux Site B package generation and IPsec remain future separately
tested adapters. For separately proven existing bidirectional private
management access, generate no redundant VPN package. No amount of
synthetic CLI metadata, remote TCP SSH handshake or approved database
metadata alone may authorize live device management.

## ADR-047 — Site A development key staging is NOT production tunnel

R9.16 development IPAT Site A locally generates original X25519 key
material (0700 owner folder, separate 0600 public/private files), and
provides only its public key for Site B operator's MANUAL disabled
review. B returns only its independently generated public key, not a
private key; IPAT never pushes config to B. Production activation
REQUIRES a real encrypted, recoverable central vault, genuinely
signed MFA and scoped tenant/site authority, independent B approval,
verified network route and change-controlled listener/rollback.
Dev-only keys may not silently be promoted to tenant production keys.

## ADR-048 — APPROVED direct-first per-device protocol + Indonesian UI terms

Owner corrects earlier metaphor: operational UI uses *Server Pusat
IPAT*, *Gateway Lokasi*, *Jaringan Manajemen* instead of Site A/B.
These are locally chosen terminology, not a claim of a formal SNI
mandate. Real connections first use an independently verified existing
routed network and manufacturer/firmware-specific secure protocol;
WireGuard and other tunnels are optional, not forced for every
OLT/ONT/router. RouterOS API-SSL/REST HTTPS are MikroTik-specific;
never infer C320 HTTPS/API compatibility from network reachability.
Site gateway config remains self-applied after operator review if a
tunnel is selected. The existing `site_a/site_b` internal lab JSON
names remain temporarily for compatibility; replace under a separate
versioned production BFF contract, NOT as new dashboard language.
Authenticated physical DEV-01 adoption still REQUIRES independent
console host RSA identity, restricted account, last-hop isolation,
actual read-only command and live baseline/MFA approval before RUN.

## ADR-049 — Public synthetic CI MIRROR, private operational original

Never set the existing operational `mr-ipat/ipat` PRIVATE Git history
public merely to bypass hosted CI quota: it contains active device
management addresses and historical network audit evidence. User's
request for free public CI is satisfied through a NEW clean-source,
synthetic-only PUBLIC mirror `mr-ipat/ipat-open-ci`; no original Git
history or previous CI logs are included. Every mirror refresh must
independently sanitize IP/domain/fingerprint fixtures and scan for
secrets, with synthetic CI result provenance mapped to exact private
source SHA. GitHub workflow installation remains blocked until owner
updates OAuth `workflow` scope or installs checked-in staged YAML via
GitHub UI. Public GitHub-hosted STANDARD runner minutes may be free;
this never makes production deployment or physical device tests free
of separate review. Preserve original private source and live evidence.

## ADR-050 — Owner-only offline C320 capture parser before operational adoption

The first genuine C320 action pipeline retains only two strictly
allowlisted read-only commands (`show card`, then optionally
`show version-running`) AFTER independently pinned SSH host identity,
restriction of the device account, reviewed POP last-hop and live
impact baseline. The new olt-core offline CLI only parses PRIVATE
owner-supplied evidence after such a read: it NEVER opens a network
socket or executes OLT commands, rejects unsafe input/output
permissions and overwrites, stores only bounded normalized owner-only
JSON and hashes, and NEVER marks actual physical device adopted.
More read-only commands (alarm/ONU optical power/ONT inventory) require
exact firmware-specific vendor support and independent device tests.
Any write operation remains maker/checker-controlled and production
role-gated; test default/admin login does not certify least privilege.

## ADR-051 C320 individual action catalog is evidence-based

Do not equate an offline parser test or historic SSH banner with real OLT interoperability. All live actions stay disabled until actual identity, scoped restricted login, firmware and tenant controls are independently proven. High-impact actions additionally need separately approved maintenance and rollback. Keep private LAB readiness metadata separate from a future real tenant-authorized worker.

## ADR-052 — Trustworthy physical host key cannot be derived from network SSH

R9.20 makes the independently trusted chassis console RSA public-key
handoff a separate offline, owner-only operation. A same-fingerprint
match between owner-asserted console key and previous network
observations is necessary but INSUFFICIENT for an active OLT login;
source provenance, independent reviewer, bounded production access,
nonprivileged account, POP isolation and baseline remain independent
hard gates. Never retry legacy SSH with password or disable pinning
to work around a preauthentication KEX stall.

## ADR-053 — Read existing SSH daemon state via trusted chassis console

The actual dual-path C320 SSH sessions timed out between KEX selection
and receipt of a host key, before credentials. A historical vendor
manual describes `show ssh` including server-key initialization.
A single site operator-approved trusted-console STATUS read is the
next diagnosis, not automatic legacy cipher downgrade, public Telnet,
key regeneration, repetitive password attempts or a forced VPN.
IPAT's R9.21 helper analyzes a PRIVATE offline status transcript,
marks real chassis trust FALSE and never authorizes OLT configuration.
If the actual firmware differs, require exact vendor documentation.

## ADR-054 — CONFIRMED per-device legacy SSH group14-SHA256 transport

An actual single bounded PRIVATE owner-VPS C320 SSH probe established
that `ssh-rsa` plus `aes128-cbc` plus specifically
`diffie-hellman-group14-sha256` reaches server host-key and SSH
userauth method negotiation with ZERO credentials or OLT commands.
Earlier actual `diffie-hellman-group16-sha512` choice stalled before
server key exchange. Introduce a PROCESS-SCOPED optional C320
transport profile; never weaken server-global SSH, enable `ssh-dss`
or infer physical chassis trust from a network-host-key handshake.
Real trusted console RSA source, restricted account, isolated POP
ACL and signed tenant approvals remain independently blocking gates
for actual C320 `show card` and later device management.

## ADR-055 — R9.23 no speculative live OLT server config edits

Owner authorizes operational adoption and conditional SSH changes,
but current actual group14/RSA/aes128-CBC direct-private transport
already reaches server authentication. The original privileged test
account, a recurring network-only RSA key, and synthetic firmware
CLI cannot independently prove the true active chassis/POP scope.
An offline-only site change planner is introduced to classify a
REAL trusted-console `show ssh` transcript without executing
commands. Do NOT change or regenerate a working server key or enable
SSH globally merely to work around a CLIENT-SIDE negotiation issue.
Actual site changes require independently trusted hardware identity,
exact firmware, secure restricted collector role, local console
recovery, live baseline and independent approved maintenance;
all actual OLT server configuration commands executed are ZERO at
this milestone. Client-side exact-device group14 profile remains
verified and scoped to a single SSH process.

## ADR-056 — R9.24 observed `password`-only C320 test account

An ACTUAL credential-free bounded C320 owner-VPS probe successfully
reached userauth and received ONLY `password` for the owner-reported
test username. The preexisting key-only R7.9 collector is INCOMPATIBLE
with that observed account until independent dedicated user setup
proves a real `publickey` offer. Do not infer an SSH server config
defect or edit the OLT merely because the existing collector enforces
keys. Support a separately approved, trusted-host, on-site operator
ATTENDED first read if actual minimally privileged account only
supports password; commercial automation requires isolated credential
vault, independent true MFA, POP scope/worker and audit not yet
implemented. This observation is account/time-specific, not a
universal vendor capability statement.

## ADR-057 — SSHv2 server-key initialization status is ambiguous

Actual C320 group14/RSA/AES128-CBC handshake already returns the RSA
server host key and advertises test-user password authentication.
Historical firmware manuals show `SSH init server key : not initialized`
or `disable` even with SSHv2 enabled. These fields alone SHALL NOT
indicate a defective SSHv2 RSA key or trigger regeneration, SSH
daemon restart or live crypto changes. R9.25 offline triage and
conditional site change planner now classify both values as
SSHV2_FIELD_AMBIGUOUS while retaining UNKNOWN physical chassis
identity. Real first-read requires external trusted-console
provenance, restricted user and explicit site risk controls.

## ADR-058 — Physical acceptance cannot be synthesized from owner plan metadata

Current real authorized owner Mac C320 private intake contains only
`plan.json`; current VPS has no verified site console capture.
Introduce strict separate R9.26 nonroot local assessment to name
precise missing physical files and independent tenant/POP production
authorization gates without disclosing raw CLI or accepting browser
boolean attestations. Even a matching offline key+card/version capture
is insufficient to mark device ADOPTED; trust provenance, actual
restricted role, live baseline, tenant OIDC/MFA and dedicated audited
non-demo production worker must be independently proven. Do not
repeat uncredentialed SSH scans to substitute for these missing facts.
