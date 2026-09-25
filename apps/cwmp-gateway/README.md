# CWMP gateway — original Rust implementation boundary

The first offline security-oriented parser is in `crates/cwmp-protocol`. It accepts a deliberately restricted SOAP 1.1 / CWMP 1.0 synthetic Inform document with explicit XML/node/depth limits and DTD rejection, then produces pure InformResponse XML. This is a **parser/serializer**, not a networked ACS engine.

**Not implemented:** HTTPS/CPE authentication, tenant-device binding, anti-replay, sessions, production SOAP faults, version negotiation, outbound parameter RPCs, and real ONT interoperability. Do not expose this module as a public CPE listener until transport identity and tenant authorization tests pass.

See `docs/PRD.md` AC-03 and `docs/DEVICE_MATRIX.md`. No device/model/firmware support is established by simulator fixtures.
