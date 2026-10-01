# R9.53 — Combined private domain dashboard and C320 operator connector

## Scope and observed results — 2026-10-01
The baseline is canonical GitHub R9.21, not the divergent older R9.52 local tenant-domain implementation. The reviewed private C320 lab operator modules and the newer R9.50 one-time-code UI are added without replacing the R9.21 domain registry, DNS instructions, lifecycle, or deny-by-default main routes.

- Actual Ubuntu 26.04.1 owner VPS, nonroot listener 127.0.0.1:3002.
- Rust pinned 1.98.1: 68/68 merged control-api tests PASS; locked offline release build SHA256 619a8499e6e4b44b4de40aba4654b3fe3b2fbc0b464885f94644d38982b4aa9a.
- Mac Node source contracts: Add Device 6/6, status indicators 6/6 and prior R9.21 dashboard synthetic test PASS.
- Ubuntu Python isolated encrypted connector tests 5/5 PASS.
- Rootless user-systemd private canary rollout: preflight PASS, HTTP smoke PASS, new binary ACTIVE with retained previous binary and automatic rollback on a failed smoke.
- Actual C320 connector status from Unix socket: draft saved, no device credentials enrolled, no verified current read, no adoption, writes disabled.
- ONE bounded owner-authorized fixed C320 TCP preflight from the VPS returned TCP_REACHABLE_AUTH_NOT_TESTED. No SSH credential, CLI or configuration command was sent during this test.
- Existing A record for ipat.fadly.id was observed pointing to the configured VPS address, but public HTTP/HTTPS :80/:443, real IdP/MFA, production PostgreSQL runtime, domain Save BFF, TLS certificates and tenant public session are still unavailable.

## Reproduction and rollback
With approved SSH access, open the Mac terminal and use SSH tunnel:
  ssh -N -L 3002:127.0.0.1:3002 ipat-lab
Then browse /lab/dashboard-preview, /lab/domain-settings and /lab/device-workbench on http://127.0.0.1:3002.

For this exact private canary source/binary on the owner VPS:
  bash deploy/scripts/lab/r953/deploy_private_integrated_preview.sh --preflight
  IPAT_R953_PRIVATE_DEPLOY=YES bash deploy/scripts/lab/r953/deploy_private_integrated_preview.sh --apply

The deployment script is pinned to the owner VPS stage path and exact reviewed binary checksum. It will refuse unrecognized prior units or security-gate changes, and automatically restore the previous :3002 unit if HTTP assertions fail. It never touches :3000, root firewall, DNS, public ports, K3s, actual OLT configuration or the dedicated persistent connector service.

Manual rollback for a later problem: remove only the R9.53 user-systemd override named r953-integrated.conf, then run systemctl --user daemon-reload and systemctl --user restart ipat-r911-preview.service. Keep the previous binary intact.

## Next MUST / SHOULD / LATER
MUST: privileged, recoverable and separately backed production PostgreSQL deployment; apply all migrations through 0014; restricted runtime login role; real HTTPS ingress and tenant-bound OIDC/MFA/BFF; automatic DNS TXT/routing/TLS verification; tenant-scoped persistent Device Manager and audit. Do not advertise public readiness before independent browser tests.

MUST for C320: via private dashboard the authorized owner securely enters the existing single-use lab code and dedicated read-only device credential (never in chat or Git). Validate independently trusted device identity, exact model/card/firmware, reviewed isolated path and backup/no-impact baseline; only then perform ONE bounded read-only inventory operation and record evidence. Do not call TCP reachability adoption.

SHOULD: integrate a generic private adapter/worker contract and deployable provisioning profiles after real identity and durable storage are working. LATER: firmware/configuration writes with separate approval, rollback and physical interoperability evidence. No unattended firmware upgrade.
