# R9.36 — C320 ONT registration path (offline source-only stage)

Owner objective: register/configure ONTs immediately after a genuinely accepted physical C320 adoption. Current actual status: physical `ADOPTED=false`, production ONT writes forbidden. Hardware-as-live rules apply even in zero-customer isolated lab.

## Implemented offline boundary

`crates/olt-core/src/ont_registration.rs` validates a single isolated syntactic registration draft: exact device `dev-01`, physically observed GTGHK card slot `1/1/1`, bounded port/ONU ID, uppercase 4-letter vendor plus eight hexadecimal-character synthetic serial form, constrained profile reference and idempotency key. Known serial and position collisions reject the draft. This is deliberately not ONT model compatibility validation, resource reservation, discovered ONU inventory or actual configuration generation. `ONT_EXECUTION_ENABLED=false` always.

The existing private dashboard hardware POST actions remain HTTP 403. No credentials, real serials, untested vendor CLI writes, user-selected shell commands, network access, firmware downloads, or persistent job creation are added by this stage.

## Remaining ordered integration MUST

1. Independently certify device identity, last-hop isolation and exact firmware, test native recovery and restricted service identity according to `R935_C320_ADOPTION_GATES.md`.
2. Obtain bounded actual read-only PON/ONU inventory, discover unconfigured ONU and approved ONT model/profiles on EXACT observed C320 firmware. Do not infer from manuals of another release.
3. Implement tenant-scoped verified device/POP ownership, authenticated MFA requester, independent maker-checker with immutable plan hash, approval expiry, change ticket and service baseline. Collect real ONT serial only through this authenticated production boundary, not private synthetic demos, chat or public CI.
4. Check exact PON/card capability, occupied IDs and serial collisions on fresh verified inventory. Lock one physical OLT and target PON; stage a single ONT first. An expired/ambiguous operation must reconcile by reading physical ONU state before retry.
5. Implement exact tested firmware-specific adapter with no generic shell interpolation. Capture redacted audit and before/after native-config backup; perform bounded approved registration and separately approved service profile/bridge/VLAN provisioning with read-back and abort/rollback.
6. Prove one actual ONT (exact brand/model/firmware) registers successfully, has intended service config and passes optical and upstream connectivity checks. Keep other ONTs unvalidated until their separate physical acceptance.

No vendor command shown in unrelated online manuals is automatically approved for this hardware. Registration and firmware update are separate high-impact change types.