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


## 12. R4.5 actual temporary Mac restic backup and external provider network gate

Actual Mac `restic 0.19.1` repo uses a cryptographically random high-entropy password stored in local macOS Keychain and `RESTIC_PASSWORD_COMMAND`, with no password in Git. Two encrypted snapshots (earlier PARTIAL VPS config and canonical Git source) passed full repository data reading and isolated checksum-verified restore. **FileVault is OFF on the Mac** and the restic Keychain secret has not been independently escrowed, so the Mac is a temporary LAB location, not sufficient secure/independent production recovery. Root-only VPS config/host keys and all future app/database/K3s state remain unbacked up. Source evidence: [R4.5 actual backup + external provider inspection](LAB_ENCRYPTED_BACKUP_EDGE_R45.md).

The external provider Managed Firewall rule set, IPv6 rule posture and provider rollback mechanism remain unknown; external Mac probing established only that TCP/22 was reachable while no connection to the sampled unused service ports completed. **Do not infer that the provider blocks those ports** or permit public Kubernetes API/Flannel VXLAN; review the external provider panel's actual inbound/outbound IPv4+IPv6 rules and verified console recovery before any change. ADR-008/010/017 remain OPEN where recorded.


## 13. R4.6 screenshot-verified external provider allow-all ingress and completed FileVault

**More recent evidence supersedes the earlier Mac FileVault OFF finding.** The operator supplied a completed FileVault setup screen, and independent Mac `fdesetup status` reported **FileVault is On**. The prior encrypted Restic repository and same-Mac Keychain password were confirmed readable after FileVault activation, and a repeat **real isolated restore of the latest canonical source plus previous partial VPS config**, with `restic check --read-data` across all four snapshots/eight packs, succeeded. An independent recovery password escrow and independent second copy still have NOT been verified; the root-only VPS config and real app databases are unprotected by this backup.

The operator's current external provider screenshot shows security group `allow-all` allowing **all inbound IPv4 from `0.0.0.0/0` and all inbound IPv6 from `::/0`**. An independent host audit confirms a global IPv6 address, an IPv6 default route, and SSH listening on **both IPv4 and IPv6**, despite no AAAA in the earlier public DNS lookup. The provider security-group assignment/share scope and any additional Managed Firewall layer remain unknown. **Do not modify a possibly shared `allow-all` group** or mistake key-only SSH for network ingress restriction. [Exact, gated source-CIDR/runbook/rollback plan](EDGE_SECURITY_GROUP_R46.md). No provider/host firewall mutation was executed; owner must first demonstrate actual external provider VNC console login and separate recovery-key escrow.


## 14. R4.7 — user-confirmed external password escrow; SHARED external provider allow-all and failed VNC login

Operator reports **actual VNC Console login has not succeeded**, the external provider `allow-all` group is **shared by multiple VPSs**, and an off-Mac Restic recovery password copy is independently accessible (operator attestation, not audited). It would be unsafe to edit/delete the shared group's allow-all rules or attempt an untested restrictive replacement while the console recovery pathway is unavailable. Provider/guest firewall and K3s state are unchanged.

The new [R4.7 read-only root-config-to-encrypted-Mac backup design and test evidence](ROOT_CONFIG_STREAM_R47.md) allow an owner-interactive sudo-based root-config stream without touching provider or guest network configuration. Its real unprivileged SSH/Restic smoke+isolated restore and deliberate producer-failure/non-snapshot tests succeeded; actual root-privileged capture and root-config restore remain **PENDING USER MAC TERMINAL ACTION**. Sensitive root configuration remains encrypted in Restic with Keychain-only local password retrieval and FileVault ON; this is not full VPS/block backup or AC-07 PostgreSQL restore.


## R4.8 current execution and first-party host firewall proposal

**Later actual verification supersedes earlier "root backup pending" notes.** On 2026-09-25 the owner completed an interactive sudo-based read-only root-config SSH stream directly into the Mac's encrypted Restic repository. The assistant independently inspected actual snapshot `abaa9827`, ran `restic check --read-data` over **11 snapshots/20 packs**, and used `--verify-root` to restore into an isolated, cleanup-protected private directory. It verified gzip/tar integrity, root-only `etc/sudoers` presence, the expected managed key-only SSH directives and removal of plaintext test artifacts. **PASS for SELECTED ROOT CONFIG RECOVERY ONLY.** SSH host private keys, all VM filesystem contents, application/database state and future K3s token/datastore are intentionally excluded. No snapshot/whole-machine restore or separate recovered-host boot has been tested; off-Mac Restic recovery secret remains an owner attestation, not independent auditor access.

**First-party firewall scope change:** this repository must not integrate with named hosting-provider firewall APIs. An optional IPAT-owned Ubuntu node firewall control plane is proposed in [FIREWALL_CONTROL_PLANE.md](FIREWALL_CONTROL_PLANE.md), and a new `firewall-policy` Rust crate implements **in-memory non-executable** dual-stack proposal validation. Global host firewall changes require platform-scoped RBAC+ABAC, MFA, two-person approvals for risky changes, a dedicated privileged agent, independent out-of-band console access, full recovery rehearsals, safe nftables/K3s coexistence and auditable automatic rollback; none of those runtime privileges or actions have yet been implemented. Tenant accounts cannot gain `CAP_NET_ADMIN` or host firewall access by choosing a menu item. The existing external shared permissive security group remains an **environment constraint, not a product integration**, and MUST NOT be changed by IPAT.


### R4.9 original CWMP offline identity and replay admission

A pure, **non-networked** Rust `cwmp-admission` unit-testable module enforces exact synthetic pinned-client identity and immutable tenant binding, bounded parser/session/replay capacities, rejects duplicate device/cert and duplicate CWMP correlation IDs, and refuses a different peer to end an active lease. It deliberately exposes **no public trusted-peer constructor** until a correctly audited real TLS adapter exists. Simulator enrollment/identity tests do not prove mTLS, transport security, durable replay safety or physical ONT compatibility. Future public endpoints must uniformly deny all identity/tenant mismatches, accept only genuine mTLS cryptographic evidence, preserve persistent state after restart, implement actual CWMP retry semantics and redact serial/certificate values in audit. [Full scope](CWMP_ADMISSION_R49.md).


### R5.0 synthetic native USP trust boundary (real USP wire support pending)

A separate `usp-core` Rust test-only controller/domain module now models exact tenant-bound agent enrollment and pinned synthetic client SPKI identity, constrained read-only planning, bounded pending/replay and strict synthetic response correlation. Crucially `VerifiedAgent` and `TrustedOperator` have **no production-constructible public proof constructors**. A spoofed broker topic, response endpoint or tenant request header cannot be treated as proof when the real MTP is developed. The optional `usp-controller` binary binds only loopback **if explicitly enabled**, with health-only routing and blanket 503 for attempted USP/data requests; it has not been run on the live VPS. No real certificate/OIDC verifier, binary protocol, broker ACL, durable anti-replay, high-risk operations or physical agent tests are delivered. Read [R5.0 exact scope and known threats](USP_SYNTHETIC_R50.md).


## R5.1 — synthetic PostgreSQL schema isolation (new source; acceptance requires actual CI)

Initial lab-only `ipat_platform` + `ipat_ops` schema migration uses tenant-scoped composite device/subscriber keys, `FORCE ROW LEVEL SECURITY`, a restricted runtime role and deny-without-trusted-transaction-scope policies. Tests explicitly cover missing/invalid tenant context, cross-tenant writes, denied platform reads and restricted `TRUNCATE`. See [R5.1 precise scope and actual test procedure](POSTGRES_TENANT_R51.md). **No verified OIDC/session binding, safe runtime SQL pool, real customer records, live PostgreSQL node or production PITR are present.** A direct client that can issue arbitrary SQL/`SET LOCAL` is NOT automatically an authenticated tenant; role and connection admission remain prerequisites.

## R5.3 sealed job simulator and explicit production write prohibition
The synthetic `provisioning-core` enforces tenant + router scoped test-only actor/worker proofs, separate checker identity, bounded approval expiry, immutable idempotent proposal comparison and global per-router claim serialization. Expired leases are `Unknown` and retain the router quarantine; no recovery or executor is wired. Private proof fields have no production constructors. **These are synthetic security properties only**: no identity provider, durable database transaction/outbox, production approval audit, verified router ownership, encrypted credentials, real RouterOS diff or crash-resistant cross-node fencing is implemented. Real `BulkPppoeWrite` stays denied by `authz-core`. Scope and negative tests: [R5.3 review](PROVISIONING_SIMULATOR_R53.md).

## R5.4 synthetic job/outbox data boundary (not authenticated runtime)
The R5.4 lab migration force-enables tenant+POP RLS and grants `ipat_app_runtime` read-only SELECT on new jobs/outbox; no user-issued SQL write path or real worker exists. A separate NOLOGIN schema-owner trigger writes a redacted outbox row atomically after each allowed synthetic transition. Cross-tenant/POP router FKs, immutable plan digest, exact synthetic distinct approver names, one-hour approval TTL, maximum 60-second lease and global unresolved-router-UUID uniqueness are defense-in-depth **data constraints**, not an OIDC/MFA identity verifier or canonical device registry. Real high-risk approval identity, binding user membership/POP to transaction scope, per-service least privilege, durable publisher, reconciliation, audit and external effects require independent implementation and adversarial tests before network writes are possible. See [R5.4 test plan](POSTGRES_JOB_OUTBOX_R54.md).

## R5.5 production admission stop condition
The nonprivileged `deploy/scripts/production/readiness.py` checks only private source/GitHub/VPS hashes, key-only SSH, Mac FileVault, existing encrypted source-snapshot presence and simple host-readiness facts. Its return value is intentionally always `NO_GO`; it never changes host firewalls, externally managed shared rules, K3s or PostgreSQL. Independently tested console recovery, complete independently recoverable full host and database backups, dedicated IPv4+IPv6 effective ingress, signed high-risk change/rollback approvals and selected network/HA architecture remain mandatory manual gates. No arbitrary attestation fields, screen capture or user-controlled headers can override them. See [R5.5 evidence and stop conditions](PRODUCTION_INFRA_RECOVERY_R55.md).

## R5.6 disposable K3s and live-installation isolation
The actual Ubuntu 26.04 GitHub-hosted runner is the only target authorized by `k3s-ubuntu26-ephemeral-ci.sh`. Non-CI machines, existing K3s installations, unverified OS/architecture, public-address selection and unpinned downloads fail closed. The real ephemeral CI experiment does not add any inbound application service to IPAT's VPS; isolated snapshot/token/kubeconfig artifacts are not published. No external hosting-provider firewall API integration or mutable shared Security Group rule is included. Production admission remains `NO_GO` until real independent console/rebuild recovery, a dedicated effective dual-stack perimeter and signed ADR-017 CNI/network and datastore/rollback design have been independently verified. See [R5.6 lab and limits](K3S_UBUNTU26_R56.md).

## R5.7 K3s/nftables recovery findings
R5.7 confirms three security-relevant failure modes in disposable Ubuntu 26.04 QEMU tests: a new-host embedded-etcd restore requires custody of the original K3s server token; restored stale Node and pod objects can direct workloads toward a dead source node until explicitly reconciled; and systemd timer default `AccuracySec=1min` can make an apparently short firewall rollback nondeterministic. The repository now requires root-only token handling, explicit stale-node/pod cleanup, a newly created pod to prove runtime recovery after K3s restart, `nft -c` validation, an isolated IPAT test table, explicit dangerous-drill opt-in, and `AccuracySec=1s` with no randomized delay. These controls were actually exercised on disposable VMs. They do not replace independent live-console access, full live-host recovery, dedicated dual-stack perimeter verification or production approval.

## R5.8 synthetic-only K3s packaging boundary
Only explicit disposable GitHub CI runs bind the original Rust health-only app processes on their pod network. The separate native USP stub has no USP Record/MTP, and the Control API device route returns anonymous 401 regardless of fake tenant headers. Helm app templates remain ClusterIP-only, non-root, no token mounts/capabilities/host namespaces, read-only and default-deny application egress with smoke-pod-only ingress. A strict rendered-Helm validator rejects unsafe drift, and CI tests deliberate negative policy mutations. Cluster smoke traffic does not itself prove the selected CNI enforces every policy, and none of this authorizes a real VPS installation. See [R5.8 lab application boundary](K3S_APPLICATION_PACKAGING_R58.md).

## R5.9 local-only web preview (not an authentication perimeter)

The optional Rust browser status preview is absent unless explicitly requested,
and the executable masks it when binding to the K3s pod network. Default
security boundary is `127.0.0.1:3000`; the only approved preview path is the
authorized Mac's strict-host-key key-only SSH tunnel bound to
`127.0.0.1:48765`. Its CSP forbids external resources/inline scripts,
framing and forms; read-only status exposes no operational data or secrets,
and anonymous device requests still return 401. There is no pseudo-login,
trust in user headers or private tenant menu rendered to unauthenticated
users. NEVER route this through public ingress, proxy or alternate domain.
Production OIDC, real RBAC+ABAC, domain ownership, verified recovery and
IPv4+IPv6 isolation remain unverified and are not replaced by this preview.

## R6.0 hardware intake trust boundary and sensitive-field denial

A planned vendor/target row is NOT a discovered or assigned physical device.
The private laboratory dashboard displays only generic public test-family
planning data with an explicit zero real devices count. Browser GET requires
the already reviewed Mac-loopback-only SSH preview; default/K3s routers
return 404 and mutation endpoints 405. No private management IP, credential,
serial or subscriber info may be embedded in HTTP or source-controlled
fixtures. The independent offline Python intake tool uses an allowlist of
required metadata keys and protocol *candidates*, duplicate-key rejection,
bounded strict field syntax, sensitive-field/IPv4 pattern denial and exclusive
0600 write under an owner-only 0700 directory outside Git; the output is
ALWAYS marked unverified and unconnected. A credential-free local file must
never be interpreted as approval, secure tenant binding or proof of protocol
support. Actual physical enrollment and read-only probing require explicit
lab/tenant owner permission, dedicated least-privileged identity, isolated
reachable channel, audited review, exact firmware record and negative tests.

## R6.1 RouterOS initial physical-read safety

One customer router's reported model/version never proves physical
reachability or service support. The candidate REST probe requires
owner permission, separate isolated lab route confirmation, owner-Mac
run, *two explicit* real-GET environment gates, RFC1918 single target,
independent TLS CA/hostname verification and dedicated local mode-0600
netrc; it does not use admin credentials, skip certificates, perform
discovery, auto-retry or any HTTP write. Only /rest/system/resource
is allowed, and untrusted/raw response fields including serials,
secrets and addresses are not emitted. The allowed evidence file is
new mode-0600 outside Git. Data still cannot be exposed via the
unauthenticated lab UI; even a successful unreviewed probe does not
provide true OIDC/tenant binding or any write permission.
A qualified human must ensure router www-ssl is source restricted
and account policy is custom least-privilege (read,rest-api only
as compatible), unlike the overprivileged built-in read group.

## R6.2 RouterOS raw JSON/evidence trust boundaries

An authenticated HTTPS result is untrusted until independently
reviewed against the precise authorized device. The original Rust
domain bounds raw bytes and field cardinality, rejects duplicate
fields and mismatched exact reported model/architecture/version,
drops unapproved fields, and returns only explicitly unreviewed
metadata. The independent private evidence parser *rejects unknown
fields*, duplicate JSON keys, missing allowed fields, fabricated
promotion flags, unsafe HTTP methods/resources, wrong identity and
invalid time syntax. Syntax is explicitly NOT an authenticity
signature, authorization claim or proof of physical interoperability.

The additional Rust owner-local CLI validates only an absolute path
to a redacted, non-symlink mode-0600 evidence file in an owner-private
directory outside its source repository; output never reproduces
payloads or paths. Its success expressly keeps enrollment, tenant
trust and compatibility false. Python-to-Rust synthetic cross-contract
fixtures verify fake serial, IP and password suppression and refusal
to promote a forged tenant record. No actual router traffic is needed.

## R6.3 immutable SSH host key conflict before any router credential is sent

The customer-router public SSH endpoint is reachable and presents
a MikroTik-style SSH banner, but the currently presented RSA key
**does not match** the authorized Mac's prior pinned RSA key
for the exact endpoint. OpenSSH strict checking correctly aborted
before any authentication. Do not interpret a public key scan
or software banner from the same network path as proof of
device identity. No chat-disclosed device password was
transmitted, written to local files, injected into process
arguments or stored in source, and no host trust was overridden.
A changed key requires an independent trusted direct-LAN
identity comparison, owner acceptance of the exact device,
investigation of possible different NAT/port-forwarding,
and credential rotation using a separate trusted path.
A new no-credential, no-login, one-host SSH fingerprint
preflight plus mocked denial tests are in
[MIKROTIK_SSH_HOST_TRUST_R63.md](MIKROTIK_SSH_HOST_TRUST_R63.md).
No configuration-write permission or live physical feature
claim follows from an SSH host-key match alone.

## R6.4 safe SSH transport preconditions and no-login default

Actual owner-supplied router SSH credentials were never used
after the Mac detected an unexpected SSH host RSA key.
R6.4 adds a **disabled-by-default** key-only SSH first-read
helper limited to the exact initial DEV-08 board/version and
a fixed three-field resource read. Its single untrusted
public RSA keyscan cannot authorize access: the fingerprint
must match an independent, owner-controlled direct-LAN
verification file with mode 0600 in a mode-0700 directory.
An existing historical key mismatch requires a separate
explicit operator opt-in. The helper uses a disposable
one-host exact public key pin, `-F /dev/null`, strict host-key
verification with only RSA SHA-2 algorithms, a dedicated
mode-0600 local SSH identity, BatchMode, public-key-only,
agent disabled, no passwords, no proxy, no port forwards,
bounded timeout and bounded command output. It never
rewrites historical known_hosts, stores a credential in
CLI args/CI/Git, or changes device configuration.

Offline --requirements and --preflight never contact the
router. No real --read may occur until the owner has
independently verified physical identity, safely rotated
the already-shared password from trusted management,
provided explicit non-disruptive customer permission,
established recovery and a restricted short-lived key.
Syntactically valid local fingerprint/evidence files
are only asserted inputs; they are NOT cryptographic
proof of the stated trusted-LAN origin or a verified
tenant/device assignment. Malformed model/version,
unapproved method or forged enrollment evidence is
rejected by the Python sanitizer and separate Rust
closed-schema verifier. All physical tests remain
NOT RUN until trusted human evidence is reviewed.

## R6.5 temporary customer router password authorization and alternatives

Owner permission for temporary password login does NOT authenticate
a server whose RSA SSH host key unexpectedly differs from the
previously pinned owner-Mac record. No credential should be
transmitted to that public endpoint until the actual physical
router is independently identified from a separate trusted
direct-LAN WinBox/console path and the discrepancy is explained.
The separate new owner-Mac keypair permits creation of a
restricted SSH read-only identity **after** that verification;
never use the already disclosed general-purpose password.
Default binary API-SSL and HTTPS availability checks were
unauthenticated and did not find a verified-reachable TLS endpoint.
TR-069 provisioning requires a separate authenticated and
TLS-trusted client-to-ACS path. No provider firewall,
RouterOS management service, account or production
cluster was changed. See
[the R6.5 operator access protocol plan](MIKROTIK_CUSTOMER_R65_PREP.md).

## R6.6 CWMP first read RPC, sealed session and private HTTP trust boundary

The only newly generated ACS RPC is the
allowlisted non-mutating `GetParameterValues`
for `Device.DeviceInfo.SoftwareVersion`.
Parsing rejects SOAP/correlation/method/type/
namespace/oversize/DTD violations and sanitizes
CWMP faults to numeric codes without logging
untrusted freeform fault strings. Rust admission
may advance only a sealed already-admitted
synthetic peer/tenant/opaque lease after
a genuinely empty POST; session evidence
never enables enrollment or config writes.
The running Rust gateway is **not yet an ACS
device endpoint**. It starts only with
a local explicit opt-in, binds exclusively to
127.0.0.1, and always denies external-style
`/cwmp` requests, including spoofed
`x-client-cert-verified` / tenant claims.
The safe synthetic parser route never issues
SOAP responses and echoes no device identifiers.

Threat-model blockers remain independent
real TLS CA/mTLS cryptographic CPE proof,

trusted operator enrollment, anti-replay
durable multi-pod ownership, resource
exhaustion limits at TLS edge, real HTTP
timeout/keepalive correctness, redacted
auditing and exact per-firmware physical
interop. No live TLS certs, public listener,
real subscriber secrets, hosted perimeter,
PostgreSQL or K3s were changed.
See [R6.6 acceptance boundaries](ACS_CWMP_R66.md).

## R6.7 pembuktian nyata mutual TLS dan penolakan identitas palsu

Biner Rustls TLS1.3 mTLS yang benar
hanya membuka 127.0.0.1:3433 setelah
opt-in eksplisit dan sertifikat
CA klien/server+private key yang sah
dari folder pemilik pribadi. Verifier
harus menerima CA tepercaya,
memvalidasi masa berlaku serta
EKU clientAuth; tanpa sertifikat,
CA klien palsu dan sertifikat
serverAuth di sisi klien ditolak.
Klien juga menguji verifikasi
CA/DNS server tanpa mengizinkan
TLS trust bypass. Berkas hanya
dimiliki akun nonroot, parent 0700,
file 0600 dan no-symlink/hardlink.
Sertifikat sementara dikunci dalam
folder tes sekali pakai, dibersihkan
setelah server tes berhenti.

**Batas keamanan yang belum
terpenuhi:** ini autentikasi
*transport berbasis CA*, BUKAN

otorisasi device/tenant. Bahkan
sertifikat valid CA tidak
boleh mengakses endpoint /cwmp
(503). Tidak ada koneksi bridge
kepada sealed Rust synthetic
AuthenticatedPeer, belum
ada pin SPKI spesifik perangkat,
sertifikat dicabut belum diuji,
belum ada private production
issuer/secret rotation, trusted
operator approval, persistent
session atau model firmware
interop. Tidak ada public ingress,
firewall maupun customer secret
yang diubah. [Kontrak R6.7](ACS_MTLS_R67.md).

## R6.8 role-preview UI dan API deny/default wajib dibedakan

R6.8 menyediakan tiga
tampilan UI berbasis
data sintetis yang semua
dapat dipilih dalam satu
browser lab privat.
Ini sengaja **bukan**
RBAC atau akun
platform/tenant sungguhan.
Privat bukan pengganti
login OIDC dan cookie
tenant yang benar.
API platform/tenant/NOC
masih menolak semua
GET/POST/DELETE dengan
HTTP 401 bahkan jika
Authorization, Host,
X-Tenant-Id dan role
palsu dikirim klien.
Endpoint browser preview
tidak ada pada K3s
public-pod bind,
memakai CSP same-origin

dan no-store, hanya
membaca endpoint status
laboratorium dan tidak
mengirim data pelanggan.

Pure dashboard policy
menguji isolasi peran
platform vs tenant dan
POP, tidak mengizinkan
mass-write. Policy
belum dipakai dalam
runtime sebelum
token OIDC/MFA
terverifikasi, domain
diikat ke membership
DB, FORCE RLS,
session CSRF dan
audit action diuji.
Lihat [kontrak R6.8](DASHBOARDS_R68.md).

## R6.9 verified signature is not tenant or administrator authority

The new pinned RSA RS256 JWT verifier requires exact issuer,
single audience, fixed key ID, time-bounded exp/nbf/iat,
short token lifetime and rejects asymmetric/symmetric
algorithm confusion, forged signatures, JOSE remote
key resolution and malicious identity claims.
No token-provided role, group, tenant or POP
is trusted or promoted to RBAC subject.
Operator-local public-key file is owner-only
in an owner-only directory, opened using
no-follow protection. The private probe
does not log or echo verified subject
and exists only by explicit loopback opt-in.

A cryptographically valid JWT MUST NOT be
mistaken for authenticated Keycloak login,
verified MFA, trusted active membership or
business authorization. Existing platform,
tenant and NOC business endpoints remain
401 even on signed token and spoofed headers.
Real Keycloak discovery and JWKS provenance,
key rotation/revocation, OIDC PKCE/nonce/state,

host-only tenant cookies, CSRF, operator-
authorized memberships and database-backed
resource/POP-specific RLS remain unimplemented.
No public port, provider firewall or
production secrets change in R6.9.
See [identity proof and blockers](IDENTITY_OIDC_R69.md).

### R7.0 tenant identity schema candidate, no runtime entitlement granted

R6.9 PinnedIssuer only validates an RS256
token signature, not authorization.
New R7.0 synthetic PostgreSQL candidate
separates approved identity membership
by tenant+issuer+subject+role from
exact POP grants via composite FK,
and platform principals into an
independent table. All three tables
have ENABLE/FORCE RLS and grant the
existing application DB runtime role
**NO access**. Tests reject runtime
read/write/enumeration, wrong-tenant
POP assignment, unapproved role,
structural expiry and missing approver.
Superuser fixture in disposable CI
is NOT the intended operational
operator-approval mechanism.

No online actor can currently read
the membership tables or create
trusted DashboardSubject from them.
JWT custom roles/tenant claims,
Host or X-Tenant-ID still must be

ignored for privilege. Before any
enablement: trusted IdP OIDC
discovery/JWKS pin rotation, PKCE,
MFA assurance, independent approval,
backend policy, RLS session-scope,
audit, CSRF and custom-domain
ownership verification.
[Exact R7.0 tests/gaps](DASHBOARD_MEMBERSHIP_R70.md).

### R7.1 native OLT read-only boundary and protected firmware gate

ZTE C320 parsing runs offline on bounded text; it rejects
control characters, unexpected table layout and duplicate slots.
The offline importer checks exact filenames, absolute owner
directory 0700, file mode 0600, no symlinks/hardlinks, rechecks
file identity with O_NOFOLLOW and never echoes raw CLI transcripts.
The binary has NO live equipment transport, firmware image
upload, dangerous configuration command or web listener.
Even all firmware check flags only mark HUMAN_REVIEW_ONLY;
the flags themselves are not trustworthy signed attestations.
Actual write automation must additionally prove exact
device/card/version compatibility with authenticated operator
and maker-checker authorization, tested restoration, onsite
independent rescue and maintenance/customer impact controls.
Do not distribute customer firmware images or management secrets
in Git/chat. This feature does NOT bypass seven blocked
live-production infrastructure safety gates.
