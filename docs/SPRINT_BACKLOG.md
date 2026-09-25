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
