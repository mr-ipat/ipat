# IPAT — Seven-Day MVP Backlog & Implementation Slices v0.1

**Plan date:** 2026-09-25; target = seven **development days** of integrated laboratory effort. Dates/capacity depend on team availability; calendar dates not promised. **Statuses are PLANNED; no source code/physical tests have been executed in this documentation milestone.**

## 1. Sprint objective, roles, definition of done

**Sprint goal:** demonstrate tenant-safe frontend↔backend auth + original Rust CWMP Inform and one parameter RPC (simulator first) + defined native USP boundary/one simulator flow + device read-only/PPPoE controlled demo where physical access exists + 3 evidence-based diagnostic rules + backup/restore and safe worker state. User-facing workflows can be reduced to CLI/minimal UI if necessary, but no security-critical shortcut may be masked as done.

**Responsible functions (staffing to assign):** Architect/tech lead (architecture integration); Rust protocol engineer (CWMP/USP); product/frontend/full stack (auth/tenant UI); network engineer (physical lab, OLT/MikroTik); SRE (CI/dev infra/backup/K3s); QA/security reviewer (negative tests and write approvals). Staffing is not assumed to be six available full-time individuals.

**Definition of done per story:** file path/commit, migration (if applicable), tested functionality and named exact device/simulator, automated tests + manual verification runbook, sanitized evidence/run ID, observability and failure case, doc/status updates. `Blocked` ≠ `Done`, scaffold-only ≠ completed integration.

**No-go safety controls:** never use production subscriber credentials; no write without lab backup/permission/approval; never use unsecured real RouterOS interfaces; stop release when any delivered path leaks tenant data or bypasses approval.

## 2. Work breakdown ordered by critical path

| Task | Priority | Planned owner | Deliverables / planned paths | Acceptance / dependencies |
|---|---|---|---|---|
| S1-00 repo foundation | P0 | Architect | `Cargo.toml`, workspace apps/crates, `web/`, `migrations/`, `tests/`, `deploy/compose/`, CI and docs | `cargo fmt`, `cargo test` and integration test entrypoint **when code exists**; docs tracked; avoid empty stubs as success |
| S1-01 platform+tenant data model | P0 | Rust + DB | `tenant-core`, persistence migrations, verified domain/membership model | Two synthetic tenants and resource joins enforce tenant invariant; ADR-005 pending final |
| S1-02 identity, RBAC+ABAC | P0 | Security + frontend | `authz-core`, Axum middleware, session config, filtered menu | AC-01/02 UI/direct URL/API/POP negatives; Keycloak proposed pending version pin |
| S1-03 job/outbox and approval | P0 | Rust + DB | `provisioning-core`, `jobs`/`outbox` migrations, worker lease/fencing tests | duplicate delivery/timeout transitions; `unknown` state manual hold; immutable approval plan |
| S1-04 original CWMP | P0 | Rust protocol | `cwmp-protocol`, `cwmp-gateway`, simulator fixtures | AC-03 simulator Inform+InformResponse and 1 tested parameter read; fuzz/bounds/unit parsing |
| S1-05 USP boundary + PoC | P1 | Rust protocol | `usp-protocol`, `usp-controller`, pinned protobuf schemas, simulator | AC-04 one authenticated/correlated message if transport readiness allows; label protocol-only otherwise |
| S1-06 device physical intake | P0 | Network engineer | `DEVICE_MATRIX.md`, lab assets and sanctioned access checklist | Exact model/firmware, backups/management channels; unavailable → BLOCKED, not validated |
| S1-07 OLT read-only adapter | P1 | Network+Rust | `adapters-zte`, `adapters-cdata`, mock fixtures | Basic inventory/read of one actual adapter only if physical/method confirmed; otherwise simulator clearly marked |
| S1-08 MikroTik PPPoE flow | P0 conditional | Network+Rust | `adapters-mikrotik`, CSV validation, dry-run, approval and small job test | AC-05 negative CSV/no-dry-run-write, optional physical write only after recovery/approval |
| S1-09 subscriber/topology | P1 | Full stack+Rust | `inventory-core`, minimal views/fixtures | subscriber→PPPoE→ONT→OLT→distribution model, tenant-filtered |
| S1-10 diagnostics | P1 | Rust+Network | `diagnostic-core`, deterministic fixtures, basic UI/CLI | AC-06 3 distinct hypotheses, source/time/unknown guard |
| S1-11 observability and release | P0 | SRE+QA | structured trace/log/metrics, runbook, evidence index | redaction test, operator steps, source SHA, first end-to-end demo |
| S1-12 backup/restore | P0 | SRE+DB | restore script/runbook (planned) and sanitized evidence | AC-07 actual isolated restore; not merely backup creation |
| S1-13 heterogeneous node | P2 conditional | SRE | `deploy/ansible`, `helm`, `gitops` + node lifecycle runbook | AC-08 only when second node available; otherwise architecture + scripted plan not counted as passing test |

## 3. Day-by-day integration plan

| Day | Main execution (parallel tracks only if personnel available) | End-of-day checkpoint / evidence |
|---|---|---|
| D1 | Freeze v0.1 scope+ADR proposal list; instantiate workspace/CI, capture exact lab inventory and access; seed two tenants; choose simulator fixtures | Compiling repository foundation; device `untested` row detail recorded; unresolved hardware/USP broker blocked visibly |
| D2 | Migrations/tenant context + authz negative tests; CWMP SOAP parser, safe limits and Inform simulator | Both tenant cases pass for first API path; parse valid/invalid Inform; no claim of physical interoperability |
| D3 | OIDC session + entitlement/menu slices; CWMP session and one safe RPC against simulator; begin outbox/job states | AC-01/02 initial negative run, AC-03 simulator run and sanitized evidence |
| D4 | Adapter skeleton tied to observed hardware, MikroTik read-only if authorized, immutable CSV diff/dry-run; normalized inventory/topology | Physical-read result separately tagged; fake/stale observations labeled; dry-run leaves router unchanged |
| D5 | Worker lease/idempotency/approval small workflow; 3 deterministic incident rules; USP native module+simulator PoC (parallel) | AC-05 simulator/physical gate, AC-06, AC-04 PoC evidence or clearly documented blocker |
| D6 | Integrated cross-module E2E, adversarial tenant test, retry/timeout/crash injection; first actual isolated PostgreSQL restore | Repeat AC-01..07; no security bypass; first measured p95 and queue metrics if available |
| D7 | Stabilize/fix; demo tenant-safe triage/provisioning/CWMP; heterogeneous node join **only if ready**; document verification, issue log and next milestone | Acceptance report pass/fail/blocked by AC; actual physical tuple evidence, status + ADR changes; MVP only, not production |

**Critical path:** repository + two-tenant isolation + authorization + durable job safety **before** any real device write. CWMP Inform/RPC is required protocol slice. Physical OLT and USP broker intricacies are parallel and conditional; they must not delay the tenant safety gate or cause fabricated compatibility claims.

## 4. Acceptance test tasks and exact expected negative cases

| Test task | Minimal scenarios | Oracle |
|---|---|---|
| TEST-TENANT | A accesses B via device id, subscriber id, joined topology, raw SQL view, job ID, search/export, host/header spoof, worker payload | Forbidden or indistinguishable not-found per endpoint disclosure policy; **zero** B records or side effects |
| TEST-ROLE | Helpdesk direct URL/REST/export/POP outside assignment, hidden menu snapshot | No forbidden menu and backend policy denies all routes |
| TEST-CWMP | Inform valid, unknown ID, wrong credential, malformed XML/XXE/oversize, allowed RPC, unexpected fault | Only authenticated exact enrolled device creates state; RPC test result recorded per simulator/firmware |
| TEST-USP | Wrong endpoint/topic/tenant, replay, one correlated response or notification | Authenticated authorized identity only; PoC scope explicit |
| TEST-JOBS | Duplicate submit/key, duplicate RabbitMQ delivery, worker crash after device write, expired/self approval, modified diff | No repeat when safely known; ambiguous effect `Unknown` and manual check; no write without valid approval |
| TEST-DIAG | Simulated multi-subscriber uplink, single-ONT optical, PPPoE auth and CWMP-silent-only | Different reasoned hypotheses and insufficient evidence for fiber diagnosis |
| TEST-OPS | Backup→isolated restore, no raw secrets in logs, optional node join/drain | Integrity evidence; sanitized logs; node conditional status correct |

## 5. Cut-line / scope management if capacity is constrained

- **Never cut:** two-tenant backend deny-by-default security tests, protocol endpoint identity binding, job dry-run/no write bypass, truthful result statuses, docs/incident evidence. If safety checks fail, do not demo real write.
- **P0 core lab slice:** CWMP simulator Inform+1 safe RPC, basic OIDC/menu/API, PostgreSQL tenant model+job outbox, synthetic diagnostic evidence, backup restore.
- **Defer from demo (do not pretend done) in this order if blocked:** actual USP broker interop beyond simulator, multi-OLT physical telemetry, graphical topology polish, second hetero node, batch real changes without lab readiness. Keep their **architecture** and open issues because product MUST still requires them.
- **Seven days is a target, not completion guarantee**; preserve exact pass/fail/blocked per AC and tag physical vs simulator in QA report.

## 6. Post-S1 epics & sequencing

| Epic | Dependencies | Release gate |
|---|---|---|
| E-M1-CWMP | `TC-CWMP-01/02` results and secure CPE onboarding | Prioritized RPC/profile interoperability with per-device regression |
| E-M1-USP | MQTT/broker ADR and actual agent inventory | Authenticated multi-device/tenant Controller with trace/retry/conformance suite |
| E-M1-OLT-MIKROTIK | Physical lab access, vendor docs and security policy | Per-model/firmware feature capability, safe write/reconcile/rollback tests |
| E-M2-TENANT-DOMAIN | ADR-005/014, security review | Verified custom domain and hard multi-tenant tests, plan enforcement |
| E-M2-DIAGNOSTICS | Real topology/event data after pilot | Evaluation dataset, operator feedback, freshness and false-positive metrics |
| E-M3-RESILIENCE | Measured sizing and ADR-010 | K3s multi-node tests, DB failover/PITR RPO/RTO, backup/object/queue HA |
| E-M3-SECURITY | End-to-end policy coverage | Threat-model closure, penetration test, incident-response tabletop |
| E-M3-NATIVE-FIREWALL (SHOULD/LATER) | ADR-018 proposed; root-config restore PASS; actual console/rescue still unverified, ADR-017 CNI OPEN | Pure dual-stack firewall-policy Rust dry-run first; later privileged nftables agent only after immutable diff, ABAC/dual approval, CNI coexistence review, isolated rollback drill and independent fresh SSH/IPv4/IPv6 tests; never manage third-party hosting firewall APIs |
| E-M4-COMMERCIAL | Legal/contract decisions & verified readiness | Tenants/packages/support and privacy readiness, billing only separately scoped |

## 7. Reporting template (use on D7)

```text
Sprint commit SHA:
Lab topology and exact devices:
AC-01: PASS/FAIL/BLOCKED; evidence:
...
AC-08: PASS/FAIL/BLOCKED; second-node context:
Physical validated tuples: [list only if evidence exists]
Simulator-only results: [list]
Security failures / high-risk stop conditions:
Performance measurements (environment, workload, p50/p95/p99):
DB restore evidence and data-integrity check:
Architecture decisions revised:
Next priority / owner / blockers:
```


## R4.9 actual S1-04 subtask progress

The offline synthetic Inform admission portion of S1-04 now compiles in a separate `cwmp-admission` Rust crate, with **11 unit tests PASS** on actual Ubuntu 26.04. It is *not* an HTTPS/mTLS adapter, production sessions, a real RPC, physical ONT testing or full AC-03 acceptance; S1-04 remains **IN PROGRESS**. No security shortcuts or live deployment are authorized while trusted TLS enrollment and out-of-band network recovery remain blocked. The independent S1-05 native USP controller still has no runtime implementation.


## R5.0 native USP simulator milestone status

The distinct native Rust `usp-core` synthetic controller boundary and explicit optional `usp-controller` loopback health-only binary compile and have 12 + 3 synthetic unit tests passing on actual Ubuntu 26.04. The controller's real USP protobuf Record/Msg and broker/MTP trust path are NOT IMPLEMENTED; a locally correlated struct is not a TR-369 wire message. **S1-05 remains IN PROGRESS and PRD AC-04 is not met.** Version/schema pinning and actual authenticated wire-level simulator are the next MUST subtask. Never expose a public USP ingress or approve ADR-007 broker choice without the required security comparison.


### R5.1 lab data-boundary slice
- MUST: isolated, explicitly opted-in synthetic PostgreSQL 16.9 schema/RLS/role tests with negative cross-tenant cases; independent restored synthetic database row-checksum and RLS checks. Do not claim live backend persistence or production HA.
- SHOULD: trusted OIDC-to-tenant verification inside backend, non-superuser pool with transaction-local scope, deny on invalid context and post-transaction connection reuse.
- LATER: tenant-safe backup access controls, production PostgreSQL HA/PITR, independent host recovery and actual AC-07 customer-data restore evidence.


### R5.2 diagnostic synthetic work
- MUST within this isolated slice: verified source+timestamp synthetic observations, 3 differentiated distribution/access/PPPoE scenario tests, a CWMP missing-only insufficient-evidence guard, no cross-tenant/POP correlation or auto-remediation, and contradictory/stale negative tests.
- SHOULD next: actual authenticated topology and normalized read-only OLT/router/CWMP/USP signal integration, DB-backed provenance with row isolation, deterministic replay and operator-facing audit evidence.
- LATER after field evidence: accuracy/uncertainty calibration, downstream impact completeness, production alert routing and controlled remediation approvals.

### R5.3 offline safe job core
- MUST (simulator-only): tenant/router-scoped immutable synthetic plan and key; self-approval denial; approval TTL, 60-second fenced synthetic worker lease, no duplicate claim on globally shared router, expired-lease `Unknown` **with router quarantine**, and negative tests. Eight Rust tests and three static contracts passed on isolated Ubuntu/Mac before GitHub review.
- STILL MUST for S1-03 / AC-05: actual verified operator/service identity, persisted PostgreSQL outbox and transactional fencing, independently verified RouterOS inventory and dry-run diff, audited two-person approvals, side-effect uncertainty reconciliation, redacted operator evidence. Current simulator must never be wired to a live writer.
- LATER: load/failure injection with multiple workers and a real database/broker on recovery-approved isolated infrastructure; before any physical PPPoE write, owner-approved test backups, access and maintenance window.

### R5.4 lab-only durable job/outbox progress
- MUST in this isolated slice: executable additive lab-only PostgreSQL migration for tenant+POP job/outbox tables, forced RLS/no runtime DML, scoped router FK, exact idempotency and immutable plan digest, synthetic bounded approval/lease state transitions, ambiguous router quarantine and same-transaction outbox trigger. Run negative integration tests and logical new-database restore in disposable PostgreSQL 16.9 CI only.
- STILL MUST before actual S1-03/AC-05: trusted OIDC session and membership, verified POP from database, signed/verified worker identity, approved roles, actual global physical router registry, nonprivileged purpose-limited SQL mutation API, publisher and crash-safe claim/reconcile with redacted audit and real device read-only diff. No actual PPPoE write allowed.
- LATER: independent PostgreSQL host/PITR and offsite recovery, authenticated multi-worker crash/failover and benchmark, real physical interoperability, production release security review.

### R5.5 infrastructure safety critical path
- MUST now: independently verify canonical Mac/GitHub/Ubuntu source, existing encrypted source recovery, live host read-only conditions and fail-closed external recovery/perimeter/ADR gates; record result without installing services or changing host/provider firewall.
- MUST external before any privileged work: actual out-of-band login, separate-host encrypted full restore, dedicated per-node IPv4+IPv6 source-restricted ingress and tested rollback, independent PostgreSQL PITR/HA readiness and private K3s networking architecture sign-off. No shared external group editing or hosting-provider integration.
- SHOULD on disposable separately recoverable machines: independently tested PostgreSQL physical base backup+WAL restore, native nftables failure/rollback drill with CNI coexistence, pinned isolated K3s bootstrap/restore and node crash handling. LATER: measured production multi-node HA, real OIDC/POP identity, audit/security review and customer-data acceptance.

### R5.6 next K3s milestone — real target-OS disposable node, not live VPS
- MUST first: pin an official stable K3s binary and checksum; use a disposable GitHub Ubuntu 26.04 host to run real embedded-etcd K3s with private-only API, single-node readiness, CoreDNS, a disposable test pod/internal DNS and temporary local etcd snapshot. Enforce non-GitHub refusal and CI-only sandbox guards.
- STILL BLOCKED before actual VPS install: real independent working rescue console, complete encrypted separately restorable host image/data and full independent recovery, dedicated dual-stack effective perimeter controls without editing shared external groups, approved ADR-017 private CNI/control-plane topology and tested independent rollback. Static/CI success does not waive these live security and disaster-recovery requirements.
- NEXT after actual disposable node proof: test controlled K3s systemd lifecycle and **second-machine** datastore snapshot restore, isolated nftables+CNI coexistence, source-scoped IPv4/IPv6 ingress and interrupted node recovery; then deploy health-only control-api without exposing synthetic CWMP/USP/RouterOS features.

### R5.7 K3s recovery progress
- DONE in isolated laboratory: real Ubuntu 26.04 QEMU K3s systemd source node; CoreDNS/pod DNS; embedded-etcd snapshot plus secret-safe server-token transfer; restore to a different VM; stale Node/pod cleanup; post-restart fresh-pod health proof; post-restore snapshot; nftables default-drop coexistence; intentional SSH lockout and precise timed rollback.
- MUST before live-node install: real out-of-band console login, full independently restorable live-host recovery, dedicated effective IPv4+IPv6 perimeter, signed ADR-017 CNI/private node network/datastore design and ADR-018 decision if native host firewall will apply.
- SHOULD next after R5.7 merge: use the recovered disposable K3s environment for IPAT Helm/deployment manifests, pod security, service accounts/network policy and application health checks without exposing CWMP/USP or real device writes.

### R5.8 MUST: disposable K3s health-only Rust app deployment
Build original Rust `control-api` and separate synthetic `usp-controller` into non-root scratch OCI images; validate exact private Helm manifests with negative mutation tests; deploy them inside the **already isolated** ephemeral Ubuntu 26 K3s job, verify actual health and deny-by-default routes, record precise CI and encrypted merged-source recovery. SHOULD next: independently instrument and test real OIDC-issued membership/POP claims before enabling data APIs and complete isolated multi-node/private-CNI trials. LATER/production: only after independent OOB rescue, full-host recovery, dedicated dual-stack ingress, approved ADR-017/018 and stateful DB PITR may any live server services be installed or exposed.

### R5.9 web access boundary
- MUST: source-controlled, explicitly enabled Mac-local browser preview,
  actual Rust HTTP/CSP/denied-request test, fail-closed SSH forwarding,
  six negative source/deployment contracts and exact-source encrypted backup.
- SHOULD: decide and approve ADR-006 frontend/OIDC provider and ADR-014
  verified custom domains; build authenticated login/membership/POP test
  before showing real tenant-specific menus or customer data.
- BLOCKED for public deployment: actual usable independent rescue console,
  whole VPS separate-host restore, verified dedicated IPv4+IPv6 perimeter,
  signed ADR-017 private cluster topology, production PostgreSQL HA/PITR
  and high-risk authorization audits. An information-only local browser
  is not permission to install live K3s or publish the API.

### R6.0 device testing preparations

- MUST (deliverable): eight exact approved target slots in the strictly
  local read-only lab dashboard; zero real hardware enrollment status;
  offline exact model/HW revision/firmware metadata schema with safe
  private staging, strict unknown/secret/placeholder/address rejection,
  Rust HTTP tests and real CI regression.
- SHOULD (only with authorized real hardware): capture each tested unit's
  board/model/firmware evidence and isolated management method out of Git;
  implement its first verified read-only adapter or authenticated CWMP
  Inform and independently test positive tenant identity and negative
  cross-tenant paths before any true device enrollment.
- BLOCKED (not claim complete): any actual physical device is connected,
  USP support, true OLT/RouterOS adapter support, subscriber view,
  privileged actions, public customer dashboard or commercial compatibility.
  Secrets, actual device IPs and serials must not enter chat or Git.

### R6.1 DEV-08 targeted real router test
- MUST now: register operator-*reported* RB951Ui-2HnD RouterOS
  7.23.7 in DEV-08, leaving physical enrollment and compatibility
  at zero. Add one dedicated safe/offline-by-default HTTPS REST
  first-read probe with strict TLS and secretless output; exercise
  malicious/wrong model/firmware, private address, permissions and
  unauthorised-operation negatives with no physical I/O.
- MUST before *actual physical* read: approved non-disruptive
  customer-router test scope, independent router recovery/backup,
  isolated Mac-to-router private path, www-ssl TLS certificate
  validation, restricted single-purpose account and separate
  two-flag explicit one-GET operator execution.
- LATER: reviewed physical identity evidence + tenant-bound
  backend onboarding; full Rust adapter, managed config, Wi-Fi,
  PPPoE, customer and firmware functions require independently
  tested RBAC+ABAC, per-firmware checks and controlled writes.
  No claim that all features are ready.

### R6.2 native Rust RouterOS domain and evidence bridge
- MUST in laboratory: implement standalone reusable Rust
  routeros-core normalizer for bounded exact DEV-08 read-only
  response, negative ambiguous/mismatched/secret/duplicate tests;
  add closed-schema offline evidence parser and non-network
  local CLI; verify cross-language Python synthetic payload
  stripping + forged enrollment denial and pinned Rust CI.
- MUST for the first *real* physical test: operator supplies
  independent dedicated private management route, actual
  least-privileged REST account, valid trusted RouterOS TLS
  certificate, customer permission and tested non-disruptive
  recovery. Only run existing Mac one-GET R6.1 helper after
  both explicit local approvals and record reviewed physical
  evidence before changing DEV-08 compatibility status.
- LATER: trusted per-tenant device assignment, OIDC/MFA,
  production Rust network connector and isolated read-only
  API. Wi-Fi, PPPoE, firewall, updates and device writes
  need their own scopes, reviews and physical tests.

### R6.3 unexpected router SSH host-key change — block authentication
- MUST: retain old known_hosts record unchanged; never send
  the user-posted password or automate accepting the new
  untrusted public endpoint fingerprint.
- MUST: independently compare new observed public RSA key
  to the **same actual** router over a separate trusted
  direct-LAN management channel with known device hardware;
  verify any expected legitimate key regeneration, and
  rotate already-shared credentials using a trusted path.
- COMPLETE (pre-CI): implement single-host, zero-credential
  SSH fingerprint read-only validator with default mismatch
  exit 4, no hidden known_hosts rewrite, optional strict
  owner-controlled independent proof file, mocked negative
  tests and exact-target live unauthenticated denial test.
- BLOCKED: actual RouterOS SSH login/read, any physical
  support claim or customer-router configuration until
  host identity, limited account, safe read-only scope
  and equipment non-disruption are independently verified.
