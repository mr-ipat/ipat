# R8.2: Original Rust CWMP virtual ONT — genuine local SOAP HTTP

Date: 2026-09-27. Scope: hardware-free, strictly synthetic, localhost-only
CWMP 1.0 **partial** HTTP integration with the existing original Rust
`cwmp-protocol` parser, serializer and strict read-only RPC parser.
This milestone does **not** enable the production ACS.

## What actually runs

`apps/cwmp-gateway/src/main.rs` adds THREE separately gated routes
to the existing Rust Tokio/Axum laboratory gateway:

1. `POST /lab/virtual-ont/inform` accepts ONLY the fixed fictional
   `SYNTHETIC / 001122 / FAKE-ONT / FAKE-NOT-PHYSICAL` CPE
   identity, exact `0 BOOTSTRAP` event and `synthetic-only`
   CWMP correlation ID. The handler uses the real existing CWMP
   strict XML parser, then the original `inform_response()`
   serializer to return actual SOAP 1.1/CWMP 1.0
   `InformResponse`. Any different/real CPE serial or
   unrecognized event returns 403; malformed/DTD returns 400.
2. `GET /lab/virtual-ont/read-request` serializes the
   exact existing original Rust CWMP 1.0
   `GetParameterValues` RPC for only
   `Device.DeviceInfo.SoftwareVersion`
   and fixed synthetic request ID `ipat-synthetic-rpc-01`.
   No arbitrary parameter, password, write, firmware or
   configuration RPC is allowed.
3. `POST /lab/virtual-ont/read-reply` runs the original
   strict correlated SOAP response/fault parser against
   exactly that fixed synthetic request ID/parameter.
   It reports ONLY `parameter_count` or whether a
   synthetic fault occurred; it deliberately excludes
   device values, fault strings and all fake identity
   contents. Forged SOAP correlation, duplicate parameter,
   different path, write method or DTD is rejected.

**Important:** These separate demonstration calls do
NOT establish a single atomic authenticated real CWMP
session and do NOT imply actual ACS Inform→RPC device
communication or trustworthy tenant enrollment. The
underlying `cwmp-admission` already has a separate
memory-only synthetic session/replay/lease model;
real durable authenticated HTTPS sessions still need
integration after device identity and actual firmware
tests.

## Security and readiness boundary

All three extra routes are ABSENT by default. They
are registered ONLY with BOTH
`IPAT_RUN_OFFLINE_CWMP_LAB=1` and
`IPAT_R82_ENABLE_VIRTUAL_ONT=YES`;
even then the process is hard-bound to
`127.0.0.1:3300`, and the routes are disabled
under `IPAT_RUN_K3S_LAB=1`. The existing
production-looking `/cwmp` stays HTTP 503
regardless of forged Host, tenant,
mTLS-verification headers or any supplied
valid SOAP document. No TLS identity,
subscriber ownership or actual POP is
accepted through the simulator.
Input is globally capped at 64 KiB
on ALL routes, and unsafe or unsupported
SOAP/XML fails closed. All simulated
success responses use `Cache-Control: no-store`
and never expose the input serial, actual
device credentials, RPC value or error detail.
Source and fixtures contain NO real owner
device addresses or passwords.
Private HTTP is intended only for an
authorized machine's ephemeral/nonroot
loopback exercise. It must NEVER be
reverse-proxied as a real device ACS.

## Reproducible acceptance on nonroot Ubuntu 26.04

Run in the canonical checkout or a CLEAN throwaway
worktree without another program bound to port 3300:

```sh
cargo fmt --all -- --check
cargo test --locked -p cwmp-gateway
cargo build --locked -p cwmp-gateway
python3 -m unittest discover deploy/scripts/lab/r82 -p test_r82_contract.py -v
bash -n deploy/scripts/lab/r82/virtual-cwmp-http-smoke.sh
IPAT_R82_SYNTHETIC_HTTP=YES bash deploy/scripts/lab/r82/virtual-cwmp-http-smoke.sh
```

The smoke script launches the ACTUAL compiled original
Rust Axum gateway, waits for the exclusive
127.0.0.1 listener, and a separate Python
`urllib` client sends REAL SOAP over the
loopback TCP/HTTP socket. An independent
ElementTree reader confirms the real
`InformResponse` structure/correlation and
`GetParameterValues` request structure.
It then sends a SOAP GetParameterValuesResponse
with an intentionally fake secret-like firmware
string and a separate fault containing synthetic
private text, verifying that NEITHER leaks.
Negative HTTP tests include forged headers,
actual-looking serial, wrong event, wrong
correlation, disallowed write/parameter,
wrong media type, DTD, oversize and unchanged
production `/cwmp` denial.

The script refuses absent explicit opt-in
and preexisting port listeners; its cleanup
terminates ONLY the subprocess it launched.
The CI job repeats real Rust/Python tests
as well as the preexisting R8.1 BBF USP 1.4
real protobuf virtual-agent tests,
R8.0 genuinely restricted synthetic
two-tenant PostgreSQL HTTP tests,
disposable Ubuntu 26.04 real single-node
K3s and disposable PostgreSQL backup/restore.
Neither CI nor this smoke is a real physical
CPE, production DR, multi-provider cluster
or security certification.

## Explicit acceptance cut-line

**MUST before actual devices:** independently
authenticated real HTTPS/TLS/mTLS where the
specific ONT firmware supports it (else
vendor-compatible verified secure admission),
trusted immutable tenant/device enrollment,
persisted bounded CWMP session/retry state,
full negotiated device-specific Inform/RPC
semantics and redacted interoperability
logs on the owner's actual VSOL/ZTE ONT.
**MUST before real SaaS:** human MFA,
audited tenant/POP membership, end-to-end
server-authorized business API,
production K3s isolation and independent
whole-host+PG+datastore disaster recovery.
**SHOULD:** virtual ONT simulator with
stateful HTTP request/response sequence
and additional versioned ONT fixtures,
together with an independently
authenticated USP MQTT MTP pilot.

<font color="red">PRD NOT COMPLETED:</font>
the real `/cwmp` service is **intentionally
HTTP 503**, no real ONT Inform/parameter
read has occurred, full CWMP/TR-069
interoperability and firmware/provisioning
are NOT validated. This is a demonstrable
original Rust and HTTP virtual SOAP
software slice, not a release GO.


## Independently verified feature-code checkpoint

PR #92 source commit
`3911c846f3b0e28ae5d62ff466a95a3d2bbb7680`
passed GitHub run `36301589491`
with all four independent
CI jobs SUCCESS. Merged
main code SHA
`d8e75c8a8f2e12dad15797135960e5de9bb30067`
passed independent separate
post-merge main CI
`36301824513` **4/4 SUCCESS**.
The real compiled Rust
CWMP SOAP HTTP smoke
and previous real USP
protobuf, real ephemeral
PostgreSQL multi-tenant
signed JWT service, and
disposable K3s/recovery
tests ran in that suite.

Owner Mac encrypted
source-only Restic
`a44374c5` was fully
read (138/138 packs),
exact source SHA256
isolated restored,
and a separately
selected historical
PARTIAL root-readable
archive restored.
Actual canonical nonroot
Ubuntu26 checkout,
Mac and GitHub matched
the exact code SHA by
SHA256-verified fast-
forward Git bundle.
Real VPS full locked
workspace/fmt and true
Python HTTP→Rust SOAP
smoke PASSED from final
canonical source.
Owner Mac protected
loopback preview restarted,
unprovisioned user auth/
inventory routes stayed
404, while forged Host/
tenant/role/business GET
in three namespaces
all returned HTTP401.
These facts do NOT
validate any actual
VSOL/ZTE firmware or
physical ACS session.
