# IPAT R8.1 — Genuine BBF USP 1.4 protobuf, private read-only virtual-agent proof

**Date:** 2026-09-27 · **Status:** SOFTWARE-INTEGRATED PRIVATE LAB;
not physical agent, MQTT MTP, secure USP session or TR-369 conformance.

## Binding product and hardware-free goals

The IPAT project mandates a native Rust TR-369/USP Controller alongside
original Rust CWMP ACS. The user authorized continued software development
without activating their physical devices until later. R8.1 upgrades
the prior `usp-core` synthetic domain (which formerly operated on
opaque fake bytes) to a real **BBF v1.4 Protocol Buffers**
*bounded, strict, read-only, no-session subset*. The old default
public/production USP path stays CLOSED. This is not a certified
or complete native USP Controller.

Normative upstream resources consulted (historical published v1.4
subset; later protocol revisions and compliance remain work):
- Broadband Forum `usp-record-1-4.proto`:
  https://github.com/BroadbandForum/usp/blob/master/specification/usp-record-1-4.proto
  observed SHA256 of raw file:
  `d32810c332c6ad5b7df3953ad0c8bb9928755c486efca4f57be78effef440435`
- Broadband Forum `usp-msg-1-4.proto`:
  https://github.com/BroadbandForum/usp/blob/master/specification/usp-msg-1-4.proto
  observed SHA256 of raw file:
  `96f18d5f6912c625126c1f1f917b2fc21f4fd6e3474607b9496e8e3dcdcfd3a8`
- https://usp.technology/specification/ for the maintained
  full normative spec. BBF's protocol and IP policies remain
  applicable; the IPAT Rust field subset and independent Python
  fixture encoder were written originally. These field tags
  reference the open official schema, rather than copying
  a third-party runtime or claiming certification.

## Actual new original code

- `crates/usp-core/src/wire14.rs` uses pinned
  Rust `prost = 0.14.3` and manually mapped official v1.4
  schema field numbers to encode a genuine USP `Record →
  NoSessionContextRecord → Msg(Header GET, Body Request(Get))`.
  `encode_offline_get` requires bounded nonempty
  `Device.` data paths and strict endpoint/message IDs;
  it is OFFLINE only, never sends a network message.
- Real `inspect_no_session_get_response` decodes actual
  `Record → Msg(Header GET_RESP, Body Response(GetResp))`,
  nested requested/resolved paths and result parameter
  map entries. A custom bounded pre-decoder scanner
  enforces known protobuf field wire types and rejects
  duplicate singular/oneof and unknown/unsupported
  record, message and nested map fields instead of
  trusting protobuf's last-field-wins behavior.
  Limits: 64 KiB record, 48 KiB inner Msg,
  at most 32 requested paths and 64 resolved paths
  per requested path, 64 parameters per resolved path.
  Structural inspection returns ONLY counts plus
  untrusted claimed sender/destination/message ID
  to the offline Rust caller. It does NOT mint an
  authenticated `VerifiedAgent`, enroll or issue
  a ticket or expose raw parameter values.
- `deploy/scripts/lab/r81/generate_golden_usp.py`
  manually assembles independent protobuf length
  delimiters, fixed official field tags and synthetic
  Get/normal GetResp/correlated GetResp binary
  fixtures. The generator has NO Rust/prost dependency
  and refuses overwriting changed goldens.
- `crates/usp-core/tests/usp14_wire.rs` tests the
  real prost output BYTE-FOR-BYTE against independent
  Python-encoded BBF 1.4 Get golden and inspects
  the independent GetResp golden. Negative tests
  include malformed/truncated/oversized varints,
  bogus read/write paths/endpoint IDs, unsupported
  message type, unexpected record/session/extensions
  and duplicate protobuf oneof.
- Inside `crates/usp-core/src/lib.rs`, a deliberately
  **test-only mock verified peer** bridges the
  actual protobuf response to the preexisting
  tenant-bound synthetic Controller domain. It
  separately proves wrong tenant/peer denial and
  replay denial, while using an independently
  hand-encoded, correlated `r1` GetResp
  fixture. This is MOCK authentication,
  NOT real cryptographic device identity.
- `apps/usp-controller/src/main.rs` adds only
  `POST /lab/inspect-usp14` accepting
  `application/octet-stream` on the explicitly
  opted-in **127.0.0.1:3100** lab listener.
  It validates the real protobuf and returns
  only safe counts, explicit `peer_authenticated=false`,
  `tenant_bound=false`, `usp_session_established=false`
  and `device_operations_enabled=false`.
  Fake tenant/agent headers are ignored,
  parameter values/claims never reflected and
  response headers prohibit caching/sniffing.
  Invalid binary 400, wrong content type 415,
  oversized body 413; no real RPC/session
  or permission is established.
- In K3s test mode (`IPAT_RUN_K3S_LAB=1`),
  the entire private parser is NOT mounted;
  the prior non-operational public health
  endpoint remains the only supported route,
  with `/v1/usp` and any other request still 503.
  By default even the private parser process
  will not start unless an explicit lab opt-in
  `IPAT_RUN_OFFLINE_USP_LAB=1` is supplied.

## How to reproduce (isolated actual Ubuntu 26.04 nonroot)

From an authorized checked-out copy of the exact
IPAT Git commit, without network devices or root:

```bash
python3 deploy/scripts/lab/r81/generate_golden_usp.py
python3 -m unittest discover deploy/scripts/lab/r81 -p 'test_*.py' -v
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo build --locked --offline -p usp-controller
IPAT_R81_SYNTHETIC_HTTP=YES bash deploy/scripts/lab/r81/usp14-private-http-smoke.sh
```

The HTTP script launches and terminates the
original Rust USP Controller binary in a private
temporary environment, asserts exclusive loopback
port 3100, executes real HTTP POST of the
independent binary fixture, rejects forged
identity/operational attempts and malformed
protobuf, and checks no sensitive identity
or fake customer data in HTTP output.
It NEVER contacts external OLT/ONT/routers,
installs K3s or loads kernel modules.
The four static Python reviews alone are NOT
the acceptance evidence: real Rust wire tests,
live loopback HTTP and all existing workspace
tests must also pass.

## PRD MUST gates that remain OPEN

- Actual cryptographic USP Agent endpoint
  verification, controller identity,
  approved agent→tenant enrollment and
  per-tenant broker ACL/topic isolation,
  authenticated TLS MQTT MTP with
  reconnect/backpressure and replay
  persistence; actual negotiated versions,
  session contexts, segmentation, encrypted
  and signed Record payloads; matching
  TR-369 message semantics and real
  agent interoperability.
- Legitimate provisioning workers, durable
  per-tenant action approval, queue fencing
  and audit; per-device feature-level physical
  interoperability, exact model/firmware
  capture, tested safe writes and independently
  verified rollback/recovery for firmware.
- Actual owner IdP/MFA and business dashboards,
  full external offsite VPS/PG backup restore,
  live multi-provider K3s overlay isolation,
  customer-domain HTTPS and production
  release approval.

**RED / PRD DEVIATION (UNTIL PHYSICAL ACCEPTANCE):**
the strict protobuf subset and actual private
process are SOFTWARE LAB only. TC-USP-01
(actual authenticated agent over a real
MTP with verified controller identity)
is NOT RUN. No physical device and no
customer USP traffic were involved.


## Verified feature-code release evidence (immutable)

Feature [PR #90](https://github.com/mr-ipat/ipat/pull/90)
finished at SHA
`82e26a606d2c1411a0f44caa022c26ab8189cc5f`
and GitHub Actions feature run
`36299321859` passed
ALL FOUR independent jobs.
Final code merged to main
`fbed8eaeb93090966e68337ed544ea81c6b0b6c1`;
separate post-code-main run
`36299574222` also
passed ALL FOUR jobs
on exactly that SHA.
The actual ephemeral
Ubuntu26 K3s/Helm smoke
and two separate
disposable PostgreSQL
RLS/backup jobs are
NOT customer cloud or
production recovery tests.

Actual owner Mac and
nonroot Ubuntu26 VPS code
were separately SHA-matched
to that exact code main.
VPS real locked
offline Rust workspace,
real native protobuf
independent golden tests,
mock tenant/peer/replay
correlation, real binary
loopback HTTP positive/
negative tests and
historical boundary
tests all PASSED.
Mac localhost browser
preview was restarted
and three real business
API namespaces denied
forged Host/tenant/role
requests with HTTP401.

Owner Mac encrypted
source-only Restic
snapshot `83505eaa`
passed 136/136 full
encrypted pack reads,
isolated exact Git
source SHA256 restore
and selected historical
PARTIAL readable-root
config restore. This
is NOT a complete
offsite VPS, PostgreSQL
or K3s recovery.
Initial CI revealed
historic static-count,
nondeterministic
R7.7 expiry-fixture
and existing K3s
health sentinel
regressions, all
corrected and retested
BEFORE the successful
feature SHA. Latest
post-final-docs-main
CI and immutable source
sync evidence (if any)
will be attached to
the docs PR after it
runs, not asserted
prospectively here.
