# IPAT — Device & Interoperability Test Matrix v0.1

**Date:** 2026-09-25 · **Scope:** inventory of *intended initial physical test targets*, **not supported-device list**.  
**Current evidence:** user-approved brief supplies vendor/family only; no exact inventory/firmware/serial/test log supplied. **All device-feature combinations start `untested`.**

**R6.0 progress (2026-09-26):** Eight DEV-01..08 *target slots* now appear in the strictly private browser lab; an offline schema validator can create credential-free 0600 metadata staging files. **Physical inventory remains ZERO; all physical tests NOT RUN, compatibility UNTESTED**. No device addresses, serials, credentials, tenant binding or real physical registration have been supplied. See [R6.0 intake and approval rules](DEVICE_TESTING_R60.md).

## 1. Initial confirmed physical pilot targets

| ID | Class | Vendor / family | Exact model | HW rev | OS/firmware | Interfaces to inventory | Features to test | Status | Physical evidence |
|---|---|---|---|---|---|---|---|---|---|
| DEV-01 | OLT | ZTE C320 | C320 (additional board/line-card details **TBD**) | TBD | TBD | Remote SSH CLI fixed two-read candidate; SNMPv3 telemetry candidate; verify BOTH on exact observed firmware | OLT inventory, PON/ONU read-only, alarm signal; no untested writes | `untested` | None supplied |
| DEV-02 | OLT | C-DATA | **TBD exact model** | TBD | TBD | Verify SNMP/CLI/API physically | Discovery/read-only, PON/ONU/optical signal | `untested` | None supplied |
| DEV-03 | ONT | VSOL | **TBD exact model** | TBD | TBD | Verify CWMP TR-069 and data model; USP optional only if actual agent exists | Inform, authentication, 1 known safe parameter RPC | `untested` | None supplied |
| DEV-04 | ONT | ZTE | **TBD exact model** | TBD | TBD | Verify CWMP TR-069 and data model; USP optional only if actual agent exists | Inform, authentication, 1 known safe parameter RPC | `untested` | None supplied |
| DEV-05 | Router (distribution) | MikroTik x86 | **TBD** | TBD | RouterOS version/build **TBD** | Verify `api-ssl` or HTTPS REST where version allows | Read inventory, PPPoE secret/session, controlled demo batch | `untested` | None supplied |
| DEV-06 | Router (distribution) | MikroTik CCR | **TBD exact CCR** | TBD | RouterOS version/build **TBD** | Verify `api-ssl`/REST | Same scoped profile | `untested` | None supplied |
| DEV-07 | Router (distribution) | MikroTik RB | **TBD exact RB** | TBD | RouterOS version/build **TBD** | Verify `api-ssl`/REST | Same scoped profile | `untested` | None supplied |
| DEV-08 | Customer router | MikroTik RB | **RB951Ui-2HnD (reported, not verified)** | HW revision not observed | **RouterOS 7.23.7 (reported, not verified)** | HTTPS REST candidate; private route and trusted TLS pending | One authenticated HTTPS GET of system/resource, no changes | `untested` | Model/version reported by operator 2026-09-26; no physical read yet |

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


## R5.0 synthetic USP boundary evidence (NO real-agent interoperability)

Twelve `usp-core` in-memory synthetic enrollment, tenant/peer-spoof, replay, malformed reply, request-correlation and capacity checks plus three optional `usp-controller` loopback-router health/deny tests passed on an isolated actual Ubuntu 26.04 checkout. **No official USP Record/Msg protobuf, MTP, broker or real CPE agent was used.** This is `synthetic-domain-only` evidence, **NOT TC-USP-01 acceptance** and not an upgrade of any physical device test tuple from `untested`. [R5.0 scope](USP_SYNTHETIC_R50.md).

## R6.1 customer router update (NOT a physical compatibility result)

Operator reported RB951Ui-2HnD with RouterOS 7.23.7 for DEV-08. Hardware board revision, dedicated restricted account, verified private management reachability and trusted HTTPS certificate remain unverified. The single read-only R6.1 probe must pass independently; a local successful probe remains unreviewed evidence until the authorized test record and tenant assignment are separately checked. All physical matrix test results remain NOT RUN and compatibility UNTESTED. See `MIKROTIK_CUSTOMER_R61.md`.

## R6.2 DEV-08 native Rust offline normalization (still NOT RUN physically)

The new `crates/routeros-core` crate and independent Python-Rust
redacted-evidence synthetic contract are implementation and
simulator test evidence only. Exact operator-reported DEV-08
RB951Ui-2HnD / RouterOS 7.23.7 remains `untested`
physically: Mac has no established private router-lab config,
verified lab management route, dedicated user/TLS CA or safe
authorized device transcript. A syntactically valid
`UnreviewedInventory` or mode-0600 staged evidence file is
never equivalent to owner permission, device identity, live
compatibility or verified tenant assignment. Physical
TC-ROS-03 remains NOT RUN. [R6.2 scope](ROUTEROS_RUST_R62.md).

## R6.3 DEV-08 SSH connectivity vs verified router identity

The owner-supplied public SSH endpoint was TCP-reachable and
returned an untrusted RouterOS-like SSH banner. However,
the owner's Mac **previously saved a DIFFERENT RSA server
fingerprint** for that exact endpoint, so strict OpenSSH
denied the connection *before the password or any RouterOS
read-only command could be sent*. This is a security
STOP condition, NOT a tested RB951 firmware or
device-management capability. Host identity cannot be
established by scanning the same untrusted public endpoint
again. Independent direct-LAN SSH fingerprint comparison
is required; actual hardware test TC-ROS-03 remains NOT RUN,
physical registration ZERO and the owner-reported
RB951Ui-2HnD/7.23.7 tuple UNTESTED. See
[the R6.3 independent SSH identity guide](MIKROTIK_SSH_HOST_TRUST_R63.md).

## R6.4 DEV-08 SSH one-read adapter prepared; real hardware STILL UNTESTED

The owner-reported RB951Ui-2HnD / RouterOS 7.23.7
has a new OFF-BY-DEFAULT SSH read-only laboratory
transport that *could* later issue one exact three-field
`/system resource` command after independent direct-LAN
SSH host identity proof, dedicated restricted key and
human recovery/scope sign-off. Its new strict synthetic
Python-to-Rust evidence bridge is a parser/integration
test only. The reviewed endpoint's historical/current
RSA host keys still differ and the owner's actual router
has NOT been authenticated, inventoried, connected or
declared physically compatible. The private browser
shows the previous blocker as STATIC and
`physical_devices_enrolled=0`. See
[restricted R6.4 lab scope](MIKROTIK_SSH_R64.md).

## R6.5 reported customer RB router still physically UNTESTED

The owner-authorized public SSH listener was reachable in
prior read-only checks but the owner's Mac has a different
previously trusted RSA server key. The alternate
default API-SSL and HTTPS ports were not verified
reachable in a separate no-credential exact-target
test. These observations alone do **not** verify
the hardware or actual installed services of any
customer router. The claimed DEV-08 RB951Ui-2HnD,
RouterOS 7.23.7 remains operator-reported only;
no password sent, trusted WinBox session proven,
actual physical board/version read, TR-069
session initiated or RouterOS configuration
written. TC-ROS-03 NOT RUN. See
[the R6.5 staged access plan](MIKROTIK_CUSTOMER_R65_PREP.md).

## R6.6 CWMP synthetic RPC availability is NOT real ONT interoperability

The new original Rust SOAP/CWMP 1.0 profile
can construct/read one `Device.DeviceInfo.SoftwareVersion`
GetParameterValues request/response and sanitize
a CWMP fault in synthetic tests, behind a sealed
synthetic admission session. Actual Axum HTTP
loopback parser tests DO NOT produce
authenticated ACS sessions or reach
any VSOL/ZTE ONT. No exact device model,
actual enabled CWMP version, firmware,
ACS URL/certificate or successful live Inform
has been observed. `TC-CWMP-01` and all
physical per-device RPC results remain
NOT RUN; zero physical devices enrolled.
See [R6.6 tests/release gaps](ACS_CWMP_R66.md).

## R6.7 mTLS TLS 1.3 gateway tidak membuktikan kompatibilitas CPE

Server TLS1.3 wajib client cert
lulus tes CA, no-client, CA palsu,
EKU salah serta hostname/CA server
salah dengan CA laboratorium sintetis.
Tidak ada ONT nyata yang
menjalankan handshake terhadap IPAT,
dan sertifikat klien sintetis
bukan identitas asli perangkat.
Tidak ada InformResponse jaringan
kepada perangkat atau RPC yang
diterima dan dicatat dari ONT.
VSOL/ZTE dan semua target fisik
tetap `untested` per exact model/
firmware/protokol; jumlah enrolled
fisik tetap nol.
[Detail milestone](ACS_MTLS_R67.md).

## R7.1 DEV-01 ZTE C320: offline parsing is NOT physical interoperability

Original DEV-01 row and TC-OLT-01 stay untested/NOT RUN.
R7.1 Rust parser and secure offline owner-file importer recognize
bounded synthetic CLI table shapes for show card and
show version-running; they do NOT use or contact a real OLT.
All exact chassis/controller/PON/uplink cards, HW revisions,
management channel capability and running firmware remain unknown.
Vendor command availability/output may differ by actual firmware.
No physical result, credential, serial, tenant assignment or
compatibility claim is produced from an offline parsing pass.
New proposed TC-OLT-FW-01 firmware test: NOT SCHEDULED/NOT RUN;
requires separate scope approval, official matching release
notes and image checksum, tested backup+recovery, verified
alarm-free state, two approvers, customer impact window
and onsite console before an actuator could exist.
See PRD_DEVIATIONS_R71.md for visible red status ledger.

## R7.2 image digest is not C320 firmware interoperability

Only an operator-supplied SHA-256
claim for an owner-private local
file can be compared bytewise
by the new offline check. No
vendor image or live physical
C320 was downloaded or used.
Installed chassis, controller,
uplink and GPON board revisions
and running build are unknown.
No firmware compatibility
claim, OLT SSH/SNMP test,
TC-OLT-01 or TC-OLT-FW-01
pass is created by SHA-256.
DEV-01 remains `untested`
and firmware actuators remain
disabled; see
[C320 integrity runbook](C320_FIRMWARE_INTEGRITY_R72.md).

## R7.3 offline C320 output correlation — NOT physical acceptance

The Rust offline parser now differentiates `CfgType` and `RealType` in
synthetic `show card` text and matches `MVR` only against those two
observed names at the **same slot**. Mismatched names/slots and boot-only
rows have negative tests. Feature and post-merge-main four-job CI passed;
see `PROJECT_STATUS.md` R7.3. **No DEV-01 session was executed**:
TC-OLT-01 remains `NOT RUN`, ZTE C320 physical support `untested` and
firmware upgrades disabled. Do not promote an offline parser format test
to physical evidence. Hardware, build, private owner-approved access and
recovery are mandatory for the first actual field capture.


## R7.4 DEV-01 dedicated nonproduction lab readiness (2026-09-27)

Equipment owner declares DEV-01 and other test devices reserved for
testing, NOT connected to live distribution. This is unverified operator
intent, not observed network isolation or device discovery.
A strict PRIVATE and OFFLINE packet validator
(deploy/scripts/lab/r74/lab-readiness.py) checks owner-declared preflight
metadata for DEV-01 with zero network/firmware capability; a passing
declaration is HUMAN_REVIEW_REQUIRED, never physical acceptance.
A real device session still requires the exact actual controller/PON/uplink
cards/build, verified management-channel trust, dedicated read-only account
and private recovery/backup. TC-OLT-01 remains NOT RUN, DEV-01 UNTESTED;
firmware writes remain DISABLED. See C320_ISOLATED_LAB_R74.md.


## R7.9 DEV-01 remote management ALTERNATIVE, NOT a physical L1 requirement

Product owner chooses actual IP-reachable management for
DEV-01 instead of a direct serial cable from IPAT.
Historical C320 documentation supports family-level
SSH CLI and SNMPv3 features, **not** guaranteed on
owner's unknown firmware. New opt-in remotely callable
two-command SSH candidate and synthetic mock
SSH→real compiled Rust parser contract are implemented.
No real device route, independent current hostpin,
read-only account or exact board/firmware evidence
has been observed. DEV-01 status stays `untested`,
TC-OLT-01 stays `NOT RUN`. The previous R7.4
extra-strict on-site-console checklist is ONE
conservative alternative, not a precondition
on this separate operator-authorized READ-only
R7.9 path. Firmware upgrade still requires
separate maintenance/recovery proof and explicit
maker-checker, never implicit connectivity.


## R8.1 native USP virtual protobuf evidence ≠ physical agent compatibility

Native original Rust serializer/parser
now processes actual BBF v1.4 no-session
Get/GetResp protobuf binary in an isolated
nonroot virtual test, with a separate
mock trust-bound synthetic Controller.
This validates NO specific VSOL, ZTE
or other ONT's USP Agent support,
broker binding, model, firmware,
data model or interoperability.
TC-USP-01 (authenticated actual
agent over real MTP) remains
NOT RUN. All physical device
matrix status labels remain
unchanged/untested until exact
model/firmware/evidence is available.


## R8.3 owner-visible staged device candidates — NOT interoperability evidence

The new private synthetic Device Manager displays
the prospective OLT/ONT/router vendor categories
ZTE/C-DATA/VSOL/MikroTik before physical
connectivity. Its LAB-/VIRTUAL- entries
are deliberately NOT actual inventory:
registered=synthetic only, connectivity=UNKNOWN,
health=NOT_MEASURED, last_verified=null.
A separate exact-tenant PostgreSQL draft
schema now exists as code and must be
tested on a disposable database; the real
model/board/serial/firmware, approved
management peer fingerprint and correct
private routing are NOT known until the
owner's device access is available.
Real OLT ZTE C320 read-only management
uses the separately gated R7.9 remote
candidate and R7.1 offline evidence
parser; real ONT CWMP and USP are
the later independently authenticated
hardware interoperability test tracks.
No matrix row becomes 'supported'
from entering a form or passing
synthetic tests.


## R9.0 DEV-01 owner-authorized PUBLIC network endpoint evidence (NOT hardware test)

At 2026-09-28 ~09:52 WIB, one exact user-approved public IPv4/TCP321
returned TCP+Telnet IAC from the owner Mac. A separately bounded
no-credential Telnet OPTION-REFUSAL-only exercise elicited an
UNAUTHENTICATED 'ZTE' marker; it is merely a server claim.
Repeatable original R9.0 receive-only preflight observed 15 initial
IAC bytes from the Mac and wrote a redacted 0600 private evidence
record. The actual IPAT Ubuntu26 VPS TCP path to the same address
TIMEOUT twice. No login, host-identity pin, authenticated C320 board,
model/firmware, PON card, CLI, physical read or trusted tenant/POP
mapping was measured. This is evidence_type=network_transport,
NOT evidence_type=physical_device_interop.
DEV-01 exact device and protocol compatibility remain UNTESTED;
TC-OLT-01 remains NOT RUN; connectivity UNKNOWN and health
NOT_MEASURED. See R90_REAL_ZTE_CANDIDATE_TELNET_NETWORK_FIRST_CONTACT.md.

## R9.3 DEV-01 site path acceptance — PENDING, not interoperability

Owner reports DEV-01 ZTE C320 public NAT TCP/321 to plaintext Telnet/23.
R9.0 owner-Mac unauthenticated handshake is the only observed device-path
evidence; actual VPS path timed out. No real ZTE identity, firmware,
privileged login, vendor command or SSH/SNMPv3 capability verified.
SITE-01..SITE-06 are all NOT RUN. See
`docs/R93_C320_SECURE_SITE_ACCESS_GATE.md`; TC-OLT-01 NOT RUN.

## R9.4 owner-reported site topology (no physical acceptance)

Owner reports a MikroTik x86 RouterOS 7 connected directly on the
same local network as DEV-01 ZTE C320, with owner-side physical or
console recovery available. Version/build, actual private management
IP/VLAN isolation, gateway baseline and restoration remain UNVERIFIED.
No WireGuard session, live Telnet login or TC-OLT-01 test performed.

## R9.6 DEV-01 live-distribution assumption

For future testing, assume this reported C320 is LIVE with active ONTs:
no physical authentication, polling, upgrade or configuration change
until isolated management and approved read-only safeguards are proven.
Owner-reported endpoint TCP/Telnet can be passively tested with ZERO
transmit bytes; this is NOT hardware identity or device health.
R9.6 backend selection tests are synthetic-only, TC-OLT-01 NOT RUN.

## R9.7 durable connection metadata is NOT ZTE interoperability

New disposable-only tenant method/gateway draft does not include
private/public IP, credentials, exact device identity, management
route, tunnel key or live probe. DEV-01 authentic physical TC-OLT-01
remains NOT RUN. No performance/ONT impact measurements exist yet.

## 2026-09-28 VPS owner-authorized passive DEV-01 repeat observation

At 2026-09-28T06:19:51Z the actual authorized nonroot IPAT VPS
performed ONE bounded public TCP/321 receive-only R9.0 check to the
owner-provided numeric DEV-01 candidate endpoint. It returned TIMEOUT.
Zero credential bytes were sent, no Telnet commands, login, site route
or hardware changes occurred. Owner Mac's earlier credential-free
TCP/Telnet handshake DOES NOT prove VPS reachability or device identity.
Physical TC-OLT-01 remains NOT RUN; connectivity UNKNOWN,
health NOT_MEASURED and exact model/firmware UNVERIFIED.

## R9.10 owner-provided private SSH endpoint (unauthenticated)

At 2026-09-28 the actual owner Mac established TCP/SSH with the
reported private candidate management address TCP321. Banner
`ZTE_SSH.1.0`; peer offered ssh-rsa/ssh-dss and legacy CBC ciphers.
Explicit per-process compatibility testing reached server
RSA host key but stopped before authentication on strict host-key
verification. Identity, chassis, exact firmware, read-only account,
management segment isolation and worker/VPS private path remain
UNVERIFIED. Physical read-only TC-OLT-01 NOT RUN. Health NOT_MEASURED.

## R9.11 second bounded live private SSH observation

One 9-second noauth SSH handshake attempt timed out. Following an
adjustment to a 20-second MAX subprocess budget, ONE more noauth
physical candidate probe returned `ZTE_SSH.1.0` and repeated the
R9.10 RSA SHA256 fingerprint. No authentication or CLI commands.
This is stable identity-observation evidence ONLY across observations;
it is not independent identity trust, verified exact model/firmware,
validated last-hop safety or physical adoption. Physical read=NOT RUN.

## R9.13 real worker path and candidate transport status

Actual nonroot IPAT VPS route-table-only check to owner-reported
private C320 endpoint returned DEFAULT_ROUTE_ONLY; network packets=0.
Original Mac legacy SSH banner/RSA fingerprint observation remains
UNTRUSTED until separately compared on actual trusted site console.
R9.13 pinned legacy RSA/CBC SSH support is offline-only tested; no
public-key restricted user login, C320 firmware or fixed `show`
interop was physically verified. Real worker route=NOT VERIFIED,
physical adoption=NOT RUN, health=NOT MEASURED.

## R9.13 actual credential-free relay observation

Owner Mac→VPS strictly temporary reverse encrypted AF_UNIX port
forward allowed VPS one passive inbound TCP SSH banner read:
`ZTE_SSH.1.0`, 20 bytes, zero credentials/commands. One versioned
helper repeat passed and independently verified relay socket removed.
This is NOT proof of actual ZTE C320 chassis/model/firmware, OLT
restricted login, isolation or long-lived worker reachability.
Physical OLT TC-OLT-01 authenticated read remains NOT RUN; adoption
FALSE and health NOT MEASURED.

## R9.14 six physical proofs now represented but NOT supplied

Device DEV-01 owner-reported ZTE C320 remains physical UNVERIFIED.
No independently obtained console RSA fingerprint, restricted SSH
account/firmware command result, measured live baseline or dedicated
VPS route proof has been received. New six-gate metadata does NOT
change physical adoption FALSE or live health NOT MEASURED.

## R9.15 actual DEV-01 versus topology plan

Owner-reported site router MikroTik x86 RouterOS7 and C320 management
private address are NOT independently inventoried. R9.15 allows
simulated Site A direct-private or Site A WireGuard listening design;
it does NOT prove VPS-to-OLT private access. Previous nonroot VPS route
check was DEFAULT_ROUTE_ONLY, and real site management last-hop and
firmware remain unverified. OLT adoption remains NOT RUN.

## R9.15 direct-from-VPS private SSH transport actually observed

One bounded exact private-IP SSH handshake FROM the actual IPAT VPS
returned `UNVERIFIED_PRIVATE_SSH_HOST_KEY`, `ZTE_SSH.1.0` and the
same untrusted RSA fingerprint earlier observed from Mac, with ZERO
credentials/commands. Linux route metadata still DEFAULT_ROUTE_ONLY
but that does not prevent observed private host transport reachability
via an upstream route. Real dedicated management last-hop isolation,
OOB fingerprint proof, restricted account and firmware UNKNOWN.
Prefer direct-private as a candidate, NOT as an approved/validated
physical adoption path. No firmware, chassis serial or health measured.


## R9.53 DEV-01 fresh integrated private status (2026-10-01)

Owner VPS private connector status: draft saved, credentials not enrolled, no verified current physical read, production adopted FALSE, writes FALSE. One bounded fixed allowlisted no-credential TCP probe returned TCP_REACHABLE_AUTH_NOT_TESTED. This is transport reachability only; exact current chassis/card/firmware and secure authenticated read remain pending. Historical earlier R9.30/R9.34 owner-observed reads are not silently promoted to a fresh production worker validation.
