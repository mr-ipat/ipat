# R10.01 — DEV-01 C320 exact sanitized card / running-firmware tuple

R10.00 re-established fresh physical CARDS and FIRMWARE counts but still left the exact board/firmware tuple unresolved. R10.01 adds a one-shot evidence collector that uses only the same two already-qualified read-only commands (show card, show version-running) through the existing owner-private credential and strict host-key path. It does not restart the live connector, does not accept a dynamic command/host/user/port, does not enter configuration mode, and returns no serial, subscriber data or raw transcript.

Before the physical read, the parser was exercised on both owner Mac and an isolated source checkout on the owner VPS. The first physical execution correctly failed closed because it assumed MVR FileType must equal exact CfgType or RealType. A second diagnostic-only implementation exposed only sanitized slot/type/MVR candidates, not raw CLI. The observed real mismatch was GTGH / GTGHK card metadata versus GTXK MVR. The code did not add a wildcard or inferred vendor alias. The final evidence pass retained that relationship as unresolved and captured HW/SW/MVR build data without weakening correlation rules.

Physical observation at 2026-10-06T06:51:43.778916Z:

| Slot | CfgType | RealType | HardVer | Card SoftVer | MVR FileType | MVR | MVR build | Result |
|---|---|---|---|---|---|---|---|---|
| 1/1/1 | GTGH | GTGHK | V1.0.0 | V2.1.0 | GTXK | V2.1.0 | 2017-07-03 00:28:55 | unresolved vendor alias |
| 1/1/3 | PRAM | PRAM | V1.0.0 | V1.01 | — | — | — | no MVR row observed |
| 1/1/4 | SMXA | SMXA | V1.0.0 | V2.1.0 | SMXA | V2.1.0 | 2017-01-17 01:04:45 | exact mapping |

Additional sanitized firmware rows: slot 1/1/1 GTXK BT V4.0.16 (2018-05-09); slot 1/1/4 SMXA BT V4.0.13 (2017-04-26) and SMXA FW V2.1.0 (2017-06-23).

The owner-private final JSON was mode 0600 and SHA-256 df5ae23228e6db82003bf20a93ca7cb4450fae9bb1985a60edf9c7b81d8d6a06. Both existing private services remained active after every read. Physical writes = 0.

Acceptance boundary: this upgrades DEV-01 inventory detail only. It does not qualify GTXK as a vendor alias for GTGH/GTGHK, does not reconcile PRAM firmware behavior, and does not authorize PON/ONU/alarm/SNMP/native restore/configuration/firmware operations. AC-12 remains PARTIAL.
