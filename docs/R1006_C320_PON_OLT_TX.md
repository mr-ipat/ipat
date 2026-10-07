# R10.06 — DEV-01 C320 bounded OLT-side PON transmit power read

R10.06 tests one non-subscriber optical command on the already connected owner-private DEV-01 C320: `show pon power olt-tx gpon-olt_1/1/1`. ZTE C300/C320 command references describe this family as a transmitting optical power query on an OLT PON interface and document that an unavailable optical-module reading can return N/A. The repository parser nevertheless treats the actual device response—not the manual—as authoritative.

The probe has a single compiled command and PON target. It imports the existing pinned private SSH provenance/credential path, requires the current canonical connector status to be enrolled, idle, `device_adopted=false` and `device_writes=0`, caps bytes/lines, removes only the exact command echo, and rejects ONU/serial/subscriber/password/username-shaped output. It has no config/write/reboot/provisioning path.

Initial physical parsing failed closed because DEV-01 uses a small Channel/Tx-power table rather than the one-line manual example. A shape-only classifier disclosed only token classes, not raw values or identifiers. It established the exact safe form `CHANNEL,TX,POWER | separator | GPON_CHANNEL_1,NA`. The parser was narrowed to the documented one-line form or exact channel-1 GPON table, including N/A. It was not generalized to arbitrary table content.

Final bounded physical observation at 2026-10-07T05:31:59Z: command accepted, PON 1/1/1 selected, measurement unavailable (`tx_power_dbm=null`), zero physical writes, no raw CLI, ONU IDs, serials or subscriber data, and both private services remained active. **N/A is not an optical-health result.** This only advances OLT-side optical command shape to PARTIAL/DEGRADED; ONU optical readings and optical diagnosis remain unqualified.

Evidence: `docs/evidence/R1006_DEV01_C320_PON111_OLT_TX_20261007.yaml`.
