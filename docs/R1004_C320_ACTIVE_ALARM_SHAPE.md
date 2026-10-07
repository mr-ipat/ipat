# R10.04 — DEV-01 active-alarm command shape evidence

R10.04 addresses the next exact ZTE C320 read-only diagnostic gap after the R10.02 aggregate PON acceptance. Historical owner-private evidence showed that this exact firmware help advertised crtv-active and that show alarm crtv-active was syntactically accepted, but the output had never been safely interpreted.

A new one-shot nonroot probe uses exactly that one fixed read command through the existing strict pinned owner-private SSH provenance and sealed credential. It has no dynamic command, no config mode, no write/reboot/firmware/ONU provisioning path, bounded bytes/pages/time, and never returns or persists raw alarm records, alarm source values, ONU IDs, serials, subscriber data or raw CLI transcript.

The first current physical run exposed only a 23-byte three-token line. Audit recognized this as the exact remote command echo, so the first probe shape was not used as alarm evidence. The parser was corrected to remove only one byte-exact command echo and the physical read was repeated. Final current result: command accepted, no rejection signature, exact echo removed, zero payload lines / zero payload bytes, zero pages and zero physical writes; both existing private services remained active.

Critical interpretation boundary: an empty payload after a successfully accepted command is NOT promoted to "no active alarms" because exact C320 V2.1.0 alarm semantics have not been independently corroborated. The operation catalog therefore advances alarms from NOT_QUALIFIED to DEGRADED, with no execution endpoint and an explicit unresolved-semantics notice. This improves product truth without fabricating device health.

Evidence: docs/evidence/R1004_DEV01_C320_ACTIVE_ALARM_SHAPE_20261007.yaml.

Acceptance for this milestone is command-shape/current-live read only. Optics, per-ONU details, SNMP, traffic, alarm semantic parsing, native restore and every physical write remain open.
