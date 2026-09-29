# R9.36 — C320 ONT registration path (offline module + deployed private LAB preview)

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
## Actual verification and private rollout (29 September 2026)

Mac source release 9292e9057aba0f422a25372dbae1d92a43641013, checksum verified in an isolated nonroot owner-VPS source checkout. Pinned Rust1.98.1 `cargo fmt --all -- --check` PASS; full locked offline 227/227 synthetic/unit Rust tests PASS (all groups zero failure), followed by exact-source targeted 65/65 `olt-core`+`control-api` Rust tests PASS and actual backend build SHA256 `a03a0fb3a445f45c60d032f5f3ca789abd70d8bba42793f73d68f37519f4ff1b`. Node JS syntax and Python smoke syntax checks passed on owner Mac.

Opt-in nonroot checksum-pinned `deploy/scripts/lab/r936/deploy_private_ont_pre_adoption_preview.sh` `--check` PASS and `--apply` actually replaced ONLY private localhost `:3002` user service, with preexisting verified R9.34 service unit in an owner-only rollback file. Actual live localhost R9.34 compatibility smoke PASS with all eight OLT action POSTs HTTP403; R9.36 smoke PASS proving gate count 2/10, device_adopted FALSE, worker FALSE and new JS blocks real ONT registration. Independent GET repeated same 2/10, original `:3000` health PASS, no established current C320 management sessions in VPS socket snapshot. No live device command was issued for R9.36.

This proves deployed PRIVATE readiness UI and offline draft policy only: there is NO actual trusted device adoption, verified native import restore, real ONT registration, VLAN profile/optical test, production maker-checker or write worker. Rollback unit remains available privately; public CI status for this release is not claimed.
