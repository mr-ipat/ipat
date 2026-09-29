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

## R9.16 actual Site A local cryptography versus hardware interoperability

Owner VPS X25519 key generation/0600 custody and CLI readback were
actually exercised. A real developer Site A PUBLIC key successfully
produced a disabled RouterOS7 B review combined with a SYNTHETIC
throwaway B public key and documentation IP examples. Actual RouterOS
7 peer import/handshake, actual site B identity, real B public key,
OLT SSH authenticated read and true firmware/model are UNTESTED.
Never classify this as actual hardware WireGuard compatibility.

## R9.17 exact actual direct transport versus unverified API claims

DEV-01 owner-reported ZTE C320 at owner-given private IP: earlier
actual VPS credential-free SSH private session returned untrusted
`ZTE_SSH.1.0` banner, no authenticated inventory or exact firmware.
Single actual VPS TCP/443 noauth strict TLS check returned TCP NOT
REACHABLE, credentials/application HTTP requests=0, no device action.
ZTE C320 RouterOS-style API-SSL is NOT an acceptable inferred
protocol; SSH pinned and SNMPv3 are firmware-specific candidates.
Port443 negative result is source/time-specific, not proof of all
HTTPS service ports. Physical identity=UNVERIFIED, site last hop=
UNVERIFIED, actual read-only login=NOT RUN, adoption=FALSE,
health=NOT MEASURED.

## R9.19 C320 action availability evidence

Owner-reported physical DEV-01 ZTE C320 is a known candidate, not an
adopted or verified device. Reconfirmed private SSH banner transport
on owner VPS, but no physical firmware/model/read-only login output.
Offline parser `show card` and `show version-running` validated only
with SYNTHETIC fixtures. Proprietary historical ZXA10 C300/C320
command references describe both CLI commands but require a suitable
operator CLI mode (not necessarily initial low-privilege user prompt).
Exact command layout, privilege mode, noninteractive SSH execution and
firmware on this physical chassis are UNKNOWN until the independent
one-command restricted test. Active alarms, ONT listing/optics and
all writes remain untested or blocked as classified in R9.19.

## R9.19 newest independent network handshake comparison

Actual authorized owner Mac AND worker VPS each observed the private
SSH target TCP transport and SSH2 KEXINIT/selected KEX, but both
preauthentication sessions timed out before the SSH server host-key
exchange, NEWKEYS and listed authentication methods. No password was
sent and no real `show card` command executed. Device identity,
firmware/CLI adapter functionality, SSH authenticated login and
actual health remain UNVERIFIED. This is not a failed ZTE account
password test. Avoid forced firmware/cryptographic changes or blind
retries on production distribution. Require trusted device-side
console/network investigation first.

## R9.20 physical ZTE C320 remains blocked before SSH host-key exchange

Actual owner Mac and VPS credential-free preauthentication attempts
both reached SSH KEX selection but timed out prior to receipt of
SSH server host key and NEWKEYS. No independently sourced actual
physical DEV-01 RSA public host key is yet available. R9.20 offline
pin creation has SYNTHETIC test coverage only; no real OLT CLI
read, chassis/firmware identification or adoption has occurred.

## R9.21 targeted C320 SSH server-state diagnostic

C320 model/firmware are still OWNER-REPORTED, not authenticated.
2013-era ZTE C320 CLI documentation depicts `show ssh` status output
including `SSH init server key`; that example is not the configuration
of physical DEV-01. Both real authorized network paths still have
only PREAUTH KEX evidence, never an actual `show ssh` capture.
R9.21 parser classification is SYNTHETIC ONLY pending a real trusted
console transcript. No actual hardware compatibility promotion.

## R9.22 actual DEV-01 direct-private SSH negotiation (29 September 2026)

Actual owner-reported C320 offers host keys `ssh-rsa,ssh-dss` and,
on independent owner VPS private SSH tests, exactly the legacy
cipher candidates `aes128-cbc,3des-cbc,blowfish-cbc`. Modern default
SSH rejected hostkey/cipher negotiation. RSA + aes128-CBC with
default group16-SHA512 KEX stalled BEFORE host-key; a separately
bounded, actual VPS credential-free `diffie-hellman-group14-sha256`
negotiation with pinned **ALGORITHMS ONLY** plus RSA/aes128-CBC
SUCCEEDED reaching the server RSA host key and auth-method stage.
Device identity remains UNTRUSTED (network-observed host key is not
independently sourced); no password, actual chassis/card/firmware
read, dedicated restricted user, operational action or customer
baseline has been performed. New exact SSH adapter transport mode
is a verified TRANSPORT PROFILE ONLY, not full vendor compatibility
or adoption. `ssh-dss` was NOT enabled.

Additional R9.22 **network-only** key-continuity check: a further
explicitly bounded credential-free RSA/aes128-CBC/group14-SHA256
owner-VPS handshake again reached SSH auth-method advertisement and
the returned RSA public host fingerprint MATCHED the earlier
owner-Mac and owner-VPS network observations. It remains **UNTRUSTED
FOR REAL DEVICE LOGIN** without a separately sourced actual chassis
console key and independent production account/site approvals.
The temporary untrusted known_hosts observation was deleted.
No OLT model/firmware/card output was collected.

## R9.23 physical C320 operational status and change audit

The user explicitly permits OLT adjustments if needed, but our
actual bounded direct-private SSH group14/RSA/CBC handshake already
reaches authentication methods; a device-side SSH change is NOT
technically evidenced as necessary. Historical C320 vendor CLI
references suggest `show ssh` status can distinguish enabled/disabled,
SSHv1/SSHv2 and server-key initialization, but actual DEV-01 status,
firmware, ACL and restricted account remain UNVERIFIED. Actual OLT
configuration/authenticated read/write commands executed = 0;
actual local-client credential-free negotiation succeeds only with
the verified R9.22 parameters. All eight LAB physical actions stay
disabled pending real trust and tenant authorization. R9.23 offline
change planner cannot establish compatibility or mutate hardware.

## R9.24 real userauth evidence

Actual authorized owner VPS bounded no-credential group14-CBC-RSA
SSH method discovery advertised **`password` only** for the tested
owner-reported account. Server network RSA key was received, still
not independently trusted physical chassis provenance. Existing
R7.9 key-only collector cannot authenticate against the tested
account based on its observed offer; exact firmware and support
for different dedicated accounts remain UNKNOWN. No password or
actual C320 command submitted, no physical adoption, no OLT
configuration changes. Future first-read may require an attended
restricted-account password session after independent site approval.

## R9.25 no speculative SSHv2 key regeneration

Earlier isolated offline status parser's `not initialized` wording
overstated a legacy vendor example. R9.25 corrects SSHv2 statuses
`not initialized` and `disable` to AMBIGUOUS because authenticated
firmware-specific meaning has not been verified and the actual owner
VPS already received a server RSA host key. Real C320 model/firmware,
on-device safe operator role and independent host identity still
UNKNOWN. OLT physical configuration and authenticated read command
count remain ZERO.

## R9.28 actual private alternate C320 Telnet port 323

Owner-approved owner-VPS ONE passive direct private port323 TCP probe
actually SUCCEEDED: 15 bytes of initial Telnet IAC response with
ZERO application writes, authentication data or OLT CLI commands.
This proves Telnet protocol transport response on the specified
route, NOT exact approved physical chassis provenance, real
Telnet login, actual firmware `show card` interoperability, access
role or production worker authorization. The previously proven
private SSH group14-SHA256 transport still reaches password-method
advertisement for the test account. Both paths remain unadopted
pending independently verified hardware identity/role/POP gates.

## R9.30 FIRST ACTUAL C320 authenticated LAB first READ — verified 29 September 2026

The owner states this exact target is a disconnected zero-customer
TEST LAB, with LIVE adoption change discipline. On ACTUAL private
C320, the owner-authorized temporary Telnet323 and separately exact
RSA/aes128-CBC/group14-SHA256 SSH interactive sessions BOTH
successfully logged in and independently observed `show card` and
`show version-running` without configuration edits. The real
Telnet session also read `show ssh` confirming enabled ver2.0 local
CHAP; one `show alarm active` probe returned syntax error and alarms
remain UNTESTED. Three real observed physical cards:
slot 1/1/1 GTGHK (configured GTGH), slot 1/1/3 PRAM, slot 1/1/4
SMXA, all card-reported INSERVICE. Five real version records:
slot 1/1/1 `GTXK` MVR V2.1.0/BT V4.0.16 (file-type alias vs
GTGHK UNVERIFIED), slot 1/1/4 SMXA MVR V2.1.0/BT V4.0.13/FW
V2.1.0. Slot 1/1/3 PRAM had NO MVR version row.

Actual first manually transcribed no-secret card snapshot was
accepted by protected owner-VPS offline Rust card-only normalizer
(3 cards, 3 INSERVICE). Never imply raw byte-exact log, independent
host key/chassis identity, verified firmware all slots or commercial
adoption. R9.30 branch contains narrow actual-shape Rust parser
regression (synthetic fixture only in public mirror) and immutable
private LAB readiness metadata. Physical automatic poll, alarms,
ONT inventory, hardware config/firmware writes: NOT VERIFIED.

## R9.31 actual lab C320 pre-change backup stage

Using exact private owner-authorized bounded SSH, actually read
privilege=15, available on-device startup cfg file listing, V2.1.0
system-group and HISTORICAL alarm counter. No active alarm assessment.
For this firmware `show startup-config` was rejected (historical
other firmware docs do NOT establish compatibility). With nonpersistent
`terminal length 0`, actual `show running-config` returned 119980
bytes, complete vendor config end marker and original CLI prompt.
Owner-private 0600 actual sensitive full snapshot and SHA256 receipt
are kept on owner VPS outside Git. First independent off-host
Restic encrypted backup+restorability TEST not yet possible: guarded
sensitive transfer was rejected by tool; do not reroute covertly.
All persistent OLT CLI configuration changes, password rotation,
privileged account creation, PON/ONT commands and firmware: ZERO.
Production automatic adoption stays BLOCKED; the real lab manual
read proof from R9.30 remains validated.

## R9.33 actual off-VPS recovery accepted and new read-only CLI compatibility

Owner Mac actual Restic encrypted off-VPS C320 manual CLI reference
119980 bytes SHA256 matches owner VPS raw owner-private original;
full isolated byte-identical encrypted-restic restore completed and
local non-secret 0600 receipt verified. STILL NOT vendor-native
import/export or rehearsed OLT device-side recovery. New bounded
actual SSH `show username` returned TWO explicit privilege15 local
user entries; no trusted limited-privilege service user discovered.
Actual `show alarm ?` advertised `crtv-active`; that read-only
command was accepted by firmware, but output NOT yet semantically
validated or a certified current health metric. All hardware writes
ZERO. Actual first cards/firmware still only the three previously
measured slots and five version rows; vendor GTGHK/GTXK mapping
unverified and PRAM MVR unreported. Separate PRIVATE LAB historical
inventory endpoint makes exact verified rows visible without
misrepresenting active telemetry or production adoption.

## R9.34 real C320 first ONE-SHOT scripted operator SSH `show card`

Source 3afe20b approved ephemeral operator LAB command tested on
actual private C320 SSH from owner VPS: exactly ONE actual bounded
read-only `show card` command returned three actual cards matching
the independently recorded initial owner Telnet and SSH snapshots
(1/1/1 GTGHK, 1/1/3 PRAM, 1/1/4 SMXA), all INSERVICE.
Actual independently inspected protected offline Rust normalizer
parsed 3/3; private file SHA256
5dc6aebaa162de7899fdec974377e1b9631647bdbab3b2b1502e20ce6810a722.
No device config writes, no persisted test password, active owner
socket closed. This establishes a working software LAB read adapter,
NOT unattended production adoption, safe limited role or certified
firmware alias/ONT/health/upgrade compatibility.

## R9.36 C320 ONT registration adapter status — 29 September 2026

Pure synthetic `olt-core::ont_registration` single-draft validation now exists and was covered by actual owner-VPS Rust unit tests. This module DOES NOT issue ONU/ONT registration commands or prove support for any ONT model, SN registration method, GPON profile, VLAN/service flow or physical firmware. The actual C320 only has independently repeated lab SSH scripted `show card` results on its known GTGHK slot 1/1/1; observed GTGHK↔GTXK file-type and missing PRAM MVR remain unresolved. OLT production ADOPTED = FALSE; ONT physical register/config test = NOT RUN; all physical action POST endpoints remain disabled.

## R9.37 strict ONU/ONT parser and bridge/VLAN feature status (29 Sep 2026)

C320 real physical `show gpon onu uncfg` response: **NOT OBSERVED by R9.37**. Public old C320 CLI documents describe a candidate syntax but are not firmware-matched proof. Rust synthetic parser for candidate unconfigured ONU table and offline bridge/VLAN/TCONT/GEM profile reviewer tested; **all physical ONU discovery, ONT registration, service profile activation, model/firmware and optical/traffic validation remain UNTESTED**. No real ONT serial, ONU ID, unconfigured inventory, VLAN or profile from actual C320 was ingested. Historical 1/1/1 GTGHK board observation cannot prove supported port occupancy or chosen ONT capability. Commercial physical adopt and ONT write state FALSE.

## R9.38 owner-lab fresh ONU inventory staging and historical real configuration counters

Protected actual R9.31 complete C320 running-config owner-VPS transcript (119980 bytes; owner-only 0600, NOT exported) independently parsed locally for sanitized counters: 171 `onu N type` declarations within five `gpon-olt` configuration sections; 72 declarations in historical `gpon-olt_1/1/1`. This does NOT establish these ONUs physically present, operational, or still configured today. An earlier generic lstrip count found 174 ONU-prefix lines across *all* config contexts; use 171 as the narrower interface-scoped count, not interchangeable totals. Fresh exact-firmware `show gpon onu uncfg`, `show gpon onu state` and `show run interface gpon-olt_1/1/1` owner-interactive capture is prepared but not yet actually executed or certified. Physical ONU registration remains UNTESTED; no free ID may be allocated based on historical counts.

## R9.39 — Owner-attested real ZTE C320 CLI response on PON 1/1/1, 29 September 2026

**Actual manual commands supported by this physical firmware in owner-provided interactive Telnet LAB output**: `show gpon onu uncfg` returned `%Code 62310-GPONSRV : No related information to show.` (zero reported unconfigured at the specific snapshot); `show gpon onu state gpon-olt_1/1/1` returned 72 configured status rows and footer `ONU Number: 0/72` (all were `enable`, `OMCC disable`, `OffLine`, GPON channel); `show run interface gpon-olt_1/1/1` returned 72 matching-count configured ONU declarations, each with configured type label `ZTEG-F623`. Registered ONU serials were included in owner chat transcript and MUST NOT be recopied to docs/Git/frontend/logs or used as enrollment inputs from chat. This is dated **manual owner-attested** real observation, not unattended worker evidence or automatically verified ID-by-ID equality. Configured type labels alone do not certify connected model/firmware or multi-vendor support. Zero unconfigured NOW does not prove optical port fault without a test ONT connected.

Real C320 command-specific offline strict Rust response parser added in `crates/olt-core/src/c320_real_inventory.rs`; R9.38 one-time owner SSH reader corrected to exact scoped command and observed C320 response shape. Actual scheduled operator-executed SSH reread via newly corrected script, native restore, signed production adoption and actual physical ONT provisioning remain NOT VERIFIED; maintain `device_adopted=false` and physical ONT writes HTTP403.
