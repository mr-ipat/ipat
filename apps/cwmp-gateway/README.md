# CWMP gateway — original Rust implementation boundary

The first offline security-oriented parser is in `crates/cwmp-protocol`. It accepts a deliberately restricted SOAP 1.1 / CWMP 1.0 synthetic Inform document with explicit XML/node/depth limits and DTD rejection, then produces pure InformResponse XML. This is a **parser/serializer**, not a networked ACS engine.

**Not implemented:** HTTPS/CPE authentication, tenant-device binding, anti-replay, sessions, production SOAP faults, version negotiation, outbound parameter RPCs, and real ONT interoperability. Do not expose this module as a public CPE listener until transport identity and tenant authorization tests pass.

See `docs/PRD.md` AC-03 and `docs/DEVICE_MATRIX.md`. No device/model/firmware support is established by simulator fixtures.


## R4.9 offline simulator milestone

[CWMP offline admission test evidence and limitations](../../docs/CWMP_ADMISSION_R49.md). The new pure `cwmp-admission` Rust crate enforces synthetic per-device enrollment tied to tenant ID and pinned simulated client-SPKI identity, bounded active sessions/replay tracking and response only after admission. Its `AuthenticatedPeer` is **sealed against public construction** until a properly reviewed real mTLS transport adapter exists. This is **NOT** an HTTP/HTTPS gateway, a durable CWMP session, a `GetParameterValues` RPC or tested physical ONT support. No internet listener is authorized. The gateway executable remains NOT IMPLEMENTED.
