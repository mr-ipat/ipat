# R10.03 — C320 capability catalog follows physical evidence

R10.02 changed the exact DEV-01 PON 1/1/1 aggregate-count result from degraded to physically accepted through the protected private HTTP path. The owner capability catalog must not keep displaying that verified read as DEGRADED.

R10.03 changes only the catalog truth state: when DEV-01 is currently connected with credentials enrolled and writes disabled, `onu_counts` is `AVAILABLE_READ_ONLY` with the already existing `/lab/c320-owner-live-refresh` endpoint. If connection/credential/write/adoption safety conditions fail, the same central `read` gate removes AVAILABLE_READ_ONLY just as for cards and firmware. The endpoint itself still independently enforces private Host+CSRF, fixed IPC action, strict sanitized aggregate parser, no serial/subscriber data and zero writes.

No other capability is upgraded. Per-ONU detail, optics, alarms, traffic, native backup/restore, provisioning, reboot and firmware write remain NOT_QUALIFIED / approval-gated. This is deliberately evidence-driven: a connected device does not make an untested command executable.
