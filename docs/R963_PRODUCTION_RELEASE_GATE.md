# R9.63 — Standalone Site master and the shortest verifiable path to public production

**Status:** production launch BLOCKED; actual owner-private Site/Device operation is a separate milestone. Do not confuse source CI, synthetic PostgreSQL or disposable K3s with the user-facing public production VPS.

## Product flow: one clear owner action per module

1. `Sites & POPs`: create/edit/list/search stable Site master independent from Device Manager; backend counts device references and refuses deletion of assigned Sites. Never automatically assign an arbitrary `POP-A`.
2. `Devices`: select only an already registered Site. With no Site, navigate to the **separate** Site module; after successful Site save return to device creation. The live C320 can be assigned or explicitly unassigned without reconnecting, deleting credentials or running OLT CLI commands. Other candidate vendors remain inventory-only until exact physical qualification.
3. `Platform Admin → Domains`: prepare the temporary public-IP plan and (only after actual zone proof) platform/tenant hostname policy. Plans do not grant tenant/admin identity, issue fake DNS ownership tokens, install TLS or open a network port.
4. **Commercial data plane**: per-company real authenticated tenant Site/Device/domain operations; RBAC/ABAC + independent PostgreSQL tenancy and worker/device boundaries. This is not implemented by the owner-private pilot.

## Prioritized release gates — real VPS, not proposed/synthetic

| Priority | Gate | Acceptance evidence | Current verified state |
|---|---|---|---|
| P0-1 | Rescue-console root and independently recoverable host/DB backups | Actual provider-console root login from the owner's management path; complete encrypted off-host host backup independently rebuilt; separate PostgreSQL WAL/PITR restore; reviewed rollback | **BLOCKED/UNVERIFIED**. The SSH deployment user has no noninteractive sudo; earlier Restic encrypted CLI capture was NOT a vendor-native device or full host/database recovery. |
| P0-2 | Real deployment identity and PostgreSQL tenant runtime | Verified IdP signing/MFA/session, non-exportable runtime secrets, current membership for platform vs tenant roles; production PostgreSQL provisioned with tenant Site/Device/domain tables, RLS and negative cross-tenant tests | **NOT DEPLOYED/UNVERIFIED**; the existing PostgreSQL CI is disposable, not the actual VPS database. |
| P0-3 | Public edge and temporary-public-IP dashboard | Browser-trusted HTTPS and appropriate cert identity (including IP SAN if serving HTTPS by bare IP), dedicated dual-stack ingress allowlist, real auth before app access, external hostile Host/SNI and rollback tests | **BLOCKED**. Live :3002 is loopback-only, 80/443 have no listener and Mac→temporary-public-IP TCP 443 failed. Never publish owner-private :3002 or self-signed privileged admin on HTTP. |
| P0-4 | DNS/tenant company portal lifecycle | Owner proves management/zone control without changing current lab SSH hostname, actual issued per-domain TXT verified via independent DNS, deployed routing mapping, TLS renewal and tenant membership/Host isolation tests | **STAGING ONLY**. The observed temporary public IPv4 `202.162.204.121` is saved in the owner-private domain plan, but does not prove `ipat.id` ownership or active DNS routing. |
| P0-5 | Full PRD functional operation and commercial hardening | Actual per-tenant Site/Device CRUD, native ACS/USP interop on qualified ONT tuples, exact C320/C-DATA feature validation + real native backup/recovery before approved writes; production HA/PITR and adverse-case tests | **PARTIAL / MOSTLY BLOCKED**. Live C320 is Connected with only narrowly tested read commands; new vendors save metadata only. This gate is NOT implied by a working login page or disposable CI. |

**No false shortcuts:** opening 443 without authenticated production application/backup controls is not acceptable; SSH tunnel success does not mean Internet reachability. A customer-supplied hostname cannot become Active before zone proof, trusted Host routing, current tenant membership and a valid certificate.

## Next execution batch (no new demo features)

- Software engineers: deliver standalone Site module (R9.63); then production OIDC/MFA + tenant-POP PostgreSQL runtime, verified platform-owner vs tenant operator menus/backend and deployment agent config reconciliation. Integrate a dedicated TLS ingress with rollback and DNS TXT verification using reviewed infrastructure IaC; test external browser/host abuse.
- Owner-controlled dependency: **first independently verify working provider root rescue console** and provide evidence of a complete restorable host backup and an owned HTTPS hostname/valid bare-IP certificate strategy. Never paste root passwords, one-time owner codes, DNS API secrets or device passwords into chat or the repository.
- Deployment SRE/Security: after owner gates, prove rollback on isolated instance, deploy real minimal authenticated public owner dashboard and read-only C320 view on the intended public IP, then enable independently verified tenant subdomain/custom-domain onboarding and device operations by proven capability. Full physical OLT/ONT writes and high-risk firmware workflow are independently gated and **not** to be silently activated with a public UI release.

The nonprivileged read-only `deploy/scripts/production/r963_public_cutover_preflight.py` inventories **actual** listener/SUDO/TCP443/private connector/domain-plan state and produces machine-readable explicit BLOCKED results. It always returns exit 3; it is not a GO oracle. Tests are in `deploy/scripts/production/test_r963_public_cutover_preflight.py`. See `docs/PROJECT_STATUS.md` for each exact source/real test/deployment run and remaining blockers.
