# R9.19 C320 operator action catalog

This is a private laboratory readiness interface, NOT an authenticated live OLT action endpoint. Direct private SSH transport has been observed from owner VPS; physical device identity, dedicated restricted login and exact firmware capabilities are not independently verified.

Eight backend capabilities are shown: card inventory, running firmware, alarms, ONT listing, ONT optics, ONT provisioning, OLT reboot and firmware upgrade. Card/firmware parser is tested OFFLINE but live read is BLOCKED. Alarm and ONT commands are UNTESTED on actual firmware. The final three actions are HIGH-IMPACT LOCKED. Every private LAB POST to an action returns HTTP403 and creates no job.

Live enablement requires owner console RSA proof, isolated management last-hop, restricted account, exact vendor firmware, real approved baseline, verified worker route, signed tenant MFA and independent reviewer. No browser supplied boolean can unlock a real worker. Exact operator sequence: `docs/SOP_ZTE_C320_READONLY_ADOPTION.md`.
