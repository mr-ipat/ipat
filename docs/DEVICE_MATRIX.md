# IPAT — Device & Interoperability Test Matrix v0.1

**Date:** 2026-09-25 · **Scope:** inventory of *intended initial physical test targets*, **not supported-device list**.  
**Current evidence:** user-approved brief supplies vendor/family only; no exact inventory/firmware/serial/test log supplied. **All device-feature combinations start `untested`.**

## 1. Initial confirmed physical pilot targets

| ID | Class | Vendor / family | Exact model | HW rev | OS/firmware | Interfaces to inventory | Features to test | Status | Physical evidence |
|---|---|---|---|---|---|---|---|---|---|
| DEV-01 | OLT | ZTE C320 | C320 (additional board/line-card details **TBD**) | TBD | TBD | SNMP / vendor-supported channel: verify | OLT inventory, PON/ONU read-only, alarm signal; no untested writes | `untested` | None supplied |
| DEV-02 | OLT | C-DATA | **TBD exact model** | TBD | TBD | Verify SNMP/CLI/API physically | Discovery/read-only, PON/ONU/optical signal | `untested` | None supplied |
| DEV-03 | ONT | VSOL | **TBD exact model** | TBD | TBD | Verify CWMP TR-069 and data model; USP optional only if actual agent exists | Inform, authentication, 1 known safe parameter RPC | `untested` | None supplied |
| DEV-04 | ONT | ZTE | **TBD exact model** | TBD | TBD | Verify CWMP TR-069 and data model; USP optional only if actual agent exists | Inform, authentication, 1 known safe parameter RPC | `untested` | None supplied |
| DEV-05 | Router (distribution) | MikroTik x86 | **TBD** | TBD | RouterOS version/build **TBD** | Verify `api-ssl` or HTTPS REST where version allows | Read inventory, PPPoE secret/session, controlled demo batch | `untested` | None supplied |
| DEV-06 | Router (distribution) | MikroTik CCR | **TBD exact CCR** | TBD | RouterOS version/build **TBD** | Verify `api-ssl`/REST | Same scoped profile | `untested` | None supplied |
| DEV-07 | Router (distribution) | MikroTik RB | **TBD exact RB** | TBD | RouterOS version/build **TBD** | Verify `api-ssl`/REST | Same scoped profile | `untested` | None supplied |
| DEV-08 | Customer router | MikroTik RB | **TBD exact RB** | TBD | RouterOS version/build **TBD** | Verify accessible interface and management route | Read-only inventory; controlled changes only after safety review | `untested` | None supplied |

**NOT confirmed for physical pilot:** VSOL GPON OLT. Track only in future candidate list unless test access is explicitly confirmed.

## 2. Granularity and status semantics

A device is never globally “compatible”. Every feature row must be keyed by **vendor + exact model + hardware/board revision where relevant + exact firmware/build + protocol+version + feature/operation + test topology + date**. Different firmware/board/API means a different row and fresh evidence. Status vocabulary:

- `untested`: no IPAT test evidence for the exact tuple.
- `partial`: some expectations pass, others fail or remain blocked; write down the precise bounds.
- `validated`: all declared acceptance steps for a **specific exact tuple/feature** passed in physical test and artifacts were reviewed. This does **not** mean vendor-wide or TR-069/TR-369 standard conformance.
- `blocked`: test run cannot complete due to hardware/access/safety; not evidence of either compatibility or incompatibility.

Do not mark a simulator as physical. Maintain a separate `evidence_type=simulator|physical|interop-suite`. Never post unredacted subscriber info, credentials, serials in public documents or Git.

## 3. Per-feature test case ledger — blank until performed

| Test ID | Device target(s) | Protocol | Required expectation | Precondition | Initial result | Evidence file/run ID |
|---|---|---|---|---|---|---|
| TC-CWMP-01 | DEV-03/04 individually | CWMP | Authenticated Inform, InformResponse, session closed | Actual model/firmware known, CPE isolated lab | NOT RUN | TBD |
| TC-CWMP-02 | DEV-03/04 individually | CWMP | One supported read-only parameter RPC and correct response | Confirm path from observed data model first | NOT RUN | TBD |
| TC-USP-01 | Simulator/actual agent only if available | USP | Authenticated message correlation and controller identity | Broker MTP and agent capability confirmed | NOT RUN | TBD |
| TC-OLT-01 | DEV-01 | Vendor/SNMP channel | Read-only OLT inventory, PON status/alarm | Access methods/vendor MIB verified | NOT RUN | TBD |
| TC-OLT-02 | DEV-02 | Vendor/SNMP channel | Same read-only feature with C-DATA exact model | Model + firmware and access confirmed | NOT RUN | TBD |
| TC-ROS-01 | DEV-05/06/07 each | API-SSL or HTTPS REST | Read inventory, PPPoE secret and active sessions under restricted role | Actual RouterOS/build and API confirmed | NOT RUN | TBD |
| TC-ROS-02 | One authorized lab router only | API-SSL or HTTPS REST | Invalid CSV rejected, dry-run no changes, approval diff and idempotency, controlled small write | Backup, recovery/rollback, maintenance window | NOT RUN | TBD |
| TC-ROS-03 | DEV-08 | Available secure interface | Read-only customer router inventory | Reachability and explicit tenant access | NOT RUN | TBD |
| TC-DIAG-01 | Simulator | Deterministic rules | 3 distinct incident hypotheses with evidence and uncertainty | Synthetic topology/event fixtures | NOT RUN | TBD |
| TC-SCALE-01 | Two provisioned compute nodes | K3s + job queue | Join/drain and no duplicate job execution in defined fault tests | Node B available; observable queue/DB | NOT RUN | TBD |

## 4. Evidence schema required for every actual test run

```yaml
run_id: "LAB-YYYYMMDD-NNN"            # placeholder
executed_at_utc: null                  # ISO 8601
performed_by: "ASSIGNED_LAB_OPERATOR"
source_commit: "GIT_SHA_PLACEHOLDER"
environment: "isolated_lab"
evidence_type: "physical"             # simulator | physical | interop-suite
vendor: "VENDOR"
exact_model: "MODEL"
hardware_revision: "HW_REV"
firmware_build: "EXACT_BUILD"
protocol: "CWMP_OR_USP_OR_SNMP_OR_ROUTEROS_API"
protocol_version: "EXACT_VERSION"
feature: "EXACT_OPERATION"
device_identifier_redacted: "REDACTED_ID"
preconditions: []
steps: []
assertions: []
observed_behavior: []
outcome: "blocked"                    # blocked | fail | pass
validation_status: "untested"         # untested | partial | validated
limitations: []
sanitary_evidence_refs: []
reviewed_by: null
```

## 5. Intake form — information still required before any compatibility claim

- **ZTE C320:** chassis/board/line-card/PON card, OS/firmware build, installed SNMP MIB, usable CLI/SNMP version and credential scope, number of ports/ONT, management reachability from isolated IPAT lab.
- **C-DATA OLT:** complete model/product line, board revision, firmware build, SNMP MIB/vendor CLI/API guide and sanctioned access.
- **VSOL/ZTE ONT:** vendor part number/model/product class, hardware rev, bootloader/firmware build, CWMP URL/setup, supported data model (TR-098/TR-181/vendor), factory/configurable ACS auth method; whether any actually expose USP Agent (never assume).
- **MikroTik:** exact x86/CCR/RB SKU, RouterOS major/minor/build, `api-ssl` vs REST availability, valid certificate setup, custom restricted operator/PPPoE privileges, lab PPPoE test accounts and router backup/rollback capability.
- **Lab network:** management VLAN/subnets and allowlisted egress design (private, not published), CPE behind NAT details, external uplink constraints, test subscriber count, safe maintenance windows, independent device access for recovery, capacity of second heterogeneous server.
- **Evidence governance:** written permission to test, named test operator, credential placeholders, approval authority for writes, evidence encryption/retention and exact acceptance procedures.

## 6. Procedure and acceptance by phase

S1: synthetic/simulator testing allowed for endpoint design; if any exact physical device available, perform discovery/read-only and one safe, authorized parameter/read first, then MikroTik small approved PPPoE write on isolated accounts if recovery exists. Label *simulated vs real* for each output. No OLT config writes S1 by default. Commercial gate: contract/physical regression per firmware, malformed/negative security tests, resilience/rollback and authorization gates; `validated` attached per feature only.

**Initial disposition:** All `TC-*` above **NOT RUN**, all physical device profiles `untested`. This is transparent project status rather than a failure verdict for any vendor.


## R4.9 simulator evidence only (NOT physical compatibility)

On the actual Ubuntu 26.04 lab, `cwmp-admission` eleven **offline synthetic** unit tests passed: enrollment/tenant/SPKI binding, SOAP InformResponse after admission, unauthenticated cross-tenant/device spoof rejection, repeated ID, bounded leases/replay, malformed SOAP/XML, wrong-peer lease completion and XML escaping. This exercises no actual TLS transport, broker, RPC or physical ONT and therefore does **not** change any DEV-01 through DEV-08 `untested` physical matrix status or claim a completed TC-CWMP-01/02 field test. Evidence/code and limitations: [R4.9 CWMP synthetic scope](CWMP_ADMISSION_R49.md).
