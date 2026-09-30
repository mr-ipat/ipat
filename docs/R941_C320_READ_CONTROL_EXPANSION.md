# R9.41 — Three real owner-supervised C320 read controls, private panel deployed

## Actual functional scope

Private nonroot owner-VPS Control API `127.0.0.1:3002`, forwarded via the already tested Mac SSH tunnel, now has THREE specific owner-supervised physical **READ-ONLY** operations. `Device Manager` displays separate buttons for: (1) PON `1/1/1` ONU summary (`show gpon onu uncfg`, `show gpon onu state gpon-olt_1/1/1`, `show run interface gpon-olt_1/1/1`), (2) installed C320 card service count (`show card`), (3) running firmware inventory row count (`show version-running`). All five exact commands have previously appeared in owner-approved REAL C320 manual or authenticated one-shot read evidence. New cards/firmware actions DO NOT expose raw firmware versions/serial numbers, do not claim GTGHK/GTXK alias reconciliation, and do not certify alternate firmware compatibility.

A temporary owner-interactive read agent accepts only local Unix messages `REFRESH`, `CARDS`, `FIRMWARE`. Client may not set device address, port, arbitrary CLI, credential, serial or VLAN. Owner password only in local hidden TTY, no unattended credentials stored. Shared quota **five total clicks across all three operations during one 15-minute activation**, one successful or failed request consumes a slot, minimum ten seconds between requests. The account remains a TEMPORARY privileged LAB identity and the SSH RSA key is observed and pinned but NOT independently OOB-attested. Public/K3s tenant APIs never mount these endpoints. Commercial multi-tenant access still denies.

The private Rust bridge creates fixed Unix request per route and returns field-allowlisted, revalidated JSON. Real fresh physical success is established only after the owner starts the agent and a returned timestamp comes from actual physical command execution. Agent offline gives explicit HTTP503 and no fake success. User-facing stage is **SUPERVISED LAB READ ENABLED**, not physical OLT production `ADOPTED`, not authenticated tenant operations, and definitely not OLT writing/ONT registration/firmware upgrade. Hardware POST action catalog still HTTP403.

## Verified release and actual deployment

- Owner Mac commit source `b6d6a73` synced to checksum-verified isolated owner VPS `/home/openai/.cache/ipat/r941-stage/src`. Pinned Rust 1.98.1 `cargo fmt --all -- --check` PASS; exact release workspace **245/245 Rust tests PASS** and no failures; local owner-agent Python suite **4/4 PASS**. `cargo build -p control-api --locked --offline -j1 -q` PASS; private backend SHA256 `6e4084becdacfa57f525639e1cffbcf1e7dc50c2e3dc48e507730f60d7de2ee5`.
- Exact old R9.40 private service-unit SHA256 `4a2404537ec27c1a97aa22c34fb7281a9c910db8fa81648d00f95aa9e0bc3189` and old private binary `55144b871d49e93482306dc7250e645d59ffa2a0cfeb9756ca6f6c6b5e1fd38c` checked before scoped nonroot private panel replace via `deploy/scripts/lab/r941/deploy_read_controls.sh`. Owner-only old unit preserved at `/home/openai/.cache/ipat/r941-release/rollback-r940.service`. No host firewall/K3s/original :3000/database/production controls/device commands touched.
- New PRIVATE unit `ipat-r911-preview.service` actually active, new backend checksum MATCH, Mac private `http://127.0.0.1:3002/lab/device-workbench` HTTP200 and owner-manual ONU snapshot HTTP200. Actual HTML contains all three operational button IDs. New private deploy HTTP smoke PASS: all three physical read buttons backend HTTP503 WITHOUT the owner-agent, forged Origin HTTP403, original :3000 health HTTP200. New positive synthetic-only Unix IPC smoke PASS: both CARDS and FIRMWARE HTTP POST actually reached strict local Unix messages, field allowlist dropped intentionally injected synthetic confidential field, reports read-only and adopted FALSE, socket removed. **No new physical OLT commands were executed in these tests.**

## Immediate real owner activation

From Mac Terminal (local only, no password in ChatGPT):

```bash
ssh -tt ipat-lab 'cd /home/openai/.cache/ipat/r941-stage/src && IPAT_R940_OWNER_READ_AGENT=YES python3 deploy/scripts/lab/r940/owner_supervised_c320_read_agent.py --owner-terminal'
```

Enter phrase `START_ONE_OWNER_PRIVATE_READ_ONLY_C320_AGENT` and the existing LAB SSH password at the hidden terminal prompt. Leave terminal running. On that same Mac open `http://127.0.0.1:3002/lab/device-workbench` and click ONE selected physical read button. Observe timestamp and redacted counts; do not assume a successful read if the panel says error. Ctrl-C to terminate agent afterward; socket unlinks. If the agent refuses a real physical firmware table, keep other read results separate and update only the exact matching parser after independent sanitized evidence.

## Remaining MUST / SHOULD / LATER

MUST for real OLT tenant provisioning: independent chassis/OOB SSH identity and management isolation; firmware GTGHK/GTXK and PRAM board version reconciliation; device-native tested export/import recovery; dedicated restricted read-service user and negative permissions evidence; tenant OIDC MFA/ABAC and distinct signed reviewer; bounded audited real production worker with no privilege-15 test account; real isolated ONT detected in new fresh `uncfg` scan; exact compatible ONT model/firmware, truly current conflict-free ID and validated VLAN/TCONT/GEM; maintenance-approved one-ONT execution, optical/service read-back, idempotency/rollback. Last real `uncfg` observation zero discovered ONU; physical connection/registration not available from code alone. Never enable mass writes/firmware or declare PROD_ADOPTED from owner's broad request alone.

SHOULD after individual firmware validation: alarms, optical levels, per-PON/per-ONU inventory and alert telemetry in private/tenant dashboards, immutable redacted actual-read audit. LATER: C-DATA/other vendor physical adapter matrices, autonomous multi-node queue+K3s, mass PPPoE/OLT/ONT controls, fleet firmware staged upgrade and production-grade tenancy with DR testing.
