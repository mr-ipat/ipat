# R9.8 — Unmounted connection-draft signed-session bridge

Development checkpoint 2026-09-28: code is ORIGINAL Rust in
`apps/control-api/src/browser_session_lab.rs` and intentionally
UNMOUNTED. `submit_nonexecutable_connection_draft_for_session`
accepts only fixed method/gateway enum and scoped synthetic candidate,
revalidates actual opaque signed session, trusted origin, exact CSRF,
fresh independently restricted tenant-admin SQL membership, then
proposes a draft through a SEPARATE EXECUTE-only PostgreSQL client.
Migration 0010 independently repeats own active tenant and candidate
POP and inserts one immutable audited nonexecuting method-choice draft.
It accepts no gateway secret, real endpoint, route, device credentials,
Telnet authentication or firmware operation.

Offline static scope check R9.8 passed; remote installed pinned Linux
rustfmt stdout transformation was idempotent. R9.8 REAL joined
signed-MFA-to-PG bridge acceptance and HTTP production deployment
have NOT been tested or enabled. Real IdP enrollment, MFA, encrypted
secret vault, safe owner-site gateway integration and dedicated
revalidation worker remain separate MUST acceptance gates. Do NOT
mislabel this partial backend seam as complete dashboard tunneling.
