# R9.19 C320 operator action catalog

This is a private laboratory readiness interface, NOT an authenticated live OLT action endpoint. Direct private SSH transport has been observed from owner VPS; physical device identity, dedicated restricted login and exact firmware capabilities are not independently verified.

Eight backend capabilities are shown: card inventory, running firmware, alarms, ONT listing, ONT optics, ONT provisioning, OLT reboot and firmware upgrade. Card/firmware parser is tested OFFLINE but live read is BLOCKED. Alarm and ONT commands are UNTESTED on actual firmware. The final three actions are HIGH-IMPACT LOCKED. Every private LAB POST to an action returns HTTP403 and creates no job.

Live enablement requires owner console RSA proof, isolated management last-hop, restricted account, exact vendor firmware, real approved baseline, verified worker route, signed tenant MFA and independent reviewer. No browser supplied boolean can unlock a real worker. Exact operator sequence: `docs/SOP_ZTE_C320_READONLY_ADOPTION.md`.

## Verified implementation and rollback

The operator Mac source was transferred as a checksum-verified Git delta
into independent unprivileged owner-VPS checkout
`/home/openai/.cache/ipat/r919-release/src`, built with pinned locked
offline Rust 1.98.1 (full `control-api` test suite and rustfmt PASSED).
Actual binary SHA256:
`7478f575a8bb2bb9f984ba15a8dd6cd18b68beb3db78d6a7aeaef2e906a8c9a4`.
The real owner VPS ran the reviewed checksum-pinned nonroot loopback
upgrade `deploy/scripts/lab/r919/deploy_private_c320_catalog.sh`
(SHA256 `7511ba0c9efbacb17a2d8ab16ebe27a10c957a5bc760943961202c6cd3ab446a`),
installed versioned user unit
`deploy/scripts/lab/r919/ipat-r911-preview.service`
(SHA256 `7d8e09dc50ecc52c4dc15514f98f0b2bdb1b2295538a34f2186b352a5bb47c37`),
then actually passed `actual_private_c320_catalog_http_smoke.py`
(SHA256 `2c5e477ca4f1b75e6d9f495e08bfeebe105e91c08865d28a26f7864eaca9b488`)
against `127.0.0.1:3002`: all eight actions disabled, unknown or
high-risk action POST HTTP403, existing historical hardware adoption
FALSE, unchanged original `:3000/healthz` HTTP200. Absolutely no live
OLT authentication or OLT/ONT/router provisioning was performed.

Nonroot owner rollback (ONLY if the private preview needs restoring):

```sh
ssh ipat-lab
cp -p /home/openai/.cache/ipat/r919-release/rollback-user-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
```

This private on-login LAB unit still has no verified production uptime
or actual signed Tenant Admin session. The relevant initial pilot
SOP is `docs/SOP_ZTE_C320_READONLY_ADOPTION.md`.

An independent actual owner Mac test established a temporary SSH
loopback forward to the owner VPS, loaded new R9.19 private HTML,
queried the running Rust catalog and confirmed all eight actions
non-dispatchable. A real HTTP POST attempting `READ_CARD_INVENTORY`
was rejected HTTP403 and reported zero device network actions. The
temporary operator Mac tunnel was terminated after the test. This
is actual HTTP UI/backend integration proof, NOT physical SSH to
C320. The owner VPS checked its nonroot private proof folders and
found no stored independently sourced console host-key proof; do
not try to infer OOB trust from the historical network fingerprint.
