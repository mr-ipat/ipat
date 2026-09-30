# R9.37 — C320 ONU discovery and ONT service configuration foundation

Status: CODE TESTED / PHYSICAL ADOPTION BLOCKED / NEW UI NOT DEPLOYED. Date: 29 Sep 2026. Do not equate synthetic parser passing with verified physical compatibility or available ONU IDs. Zero new C320 hardware commands in R9.37.

## Implemented MUST source slice

- `crates/olt-core/src/onu_discovery.rs`: strict bounded 16KiB synthetic `show gpon onu uncfg` response parser, rejecting unsafe bytes, unexpected fields, malformed/duplicate serial, out-of-range PON locations and ambiguous output. Its reported provisional ONU index is **NOT an allocatable ID**. Full registered ONU inventory and exact CLI response validation remain absent.
- `crates/olt-core/src/ont_registration.rs` (R9.36): validated, isolated review-only ONT draft for currently known card slot `1/1/1`. No real serial accepted in LAB UI, no hardware commands generated; no verified vendor ONU type and allocation proof.
- `crates/olt-core/src/ont_service_plan.rs`: offline review-only bridge/802.1Q service VLAN, customer VLAN, ETH-port and verified external TCONT/GEM/profile references; denies missing exact model profile and unsupported capability. Boolean and reference collections here are synthetic caller inputs, **NOT attestations** or real device-discovered profiles. `ONT_SERVICE_WRITES_ENABLED=false`.
- `apps/control-api/src/c320_actions_lab.rs` adds PRIVATE protected historical GET `/lab/c320-ont-feature-readiness`, explicitly reporting only offline modules, still 2/10 adoption gates and false real discovery/model/profile/register/config/rollback. `web/lab/c320-first-real-inventory.js` shows this additional conditional feature status in private inventory.

## Scope still required before real ONT adoption

MUST: independent physical OLT host key and isolated last-hop; genuine native config export/import recovery drill; scoped account and negative tests; exact firmware verification; signed tenant MFA distinct checker and audited durable worker; one confirmed read-only physical unconfigured ONT inventory and registered ONU state from matching firmware; exact ONT model/OUI and tested `onu-type` profile; PON port and available ONU ID collision proof; verified TCONT/GEM bandwidth and bridge service mapping; single-ONT immutable approved plan plus bounded activation, read-back, rollback and optical/traffic acceptance.

SHOULD after single-ONT MVP: multi-ONT inventory pagination with freshness and discrepancy flags; VLAN-per-subscriber plan previews; PPPoE customer association; profile templates by tested ONT variant; batch dry-run with per-PON locks and approvals; optical trends and subscriber correlation; OMCI capability visibility.

LATER / separate certification: ONT routed-WAN credentials, Wi-Fi SSID/security, VoIP, multicast IPTV/CATV, firmware transfer/reboot, TR-069/USP coordination, multi-vendor provisioning, zero-touch registration and self-service tenant bulk flows. These require model+firmware-specific real interoperability; no generic ZTE template is an authorization to execute.
