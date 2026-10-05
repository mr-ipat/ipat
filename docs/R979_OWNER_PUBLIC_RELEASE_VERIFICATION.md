# R9.79 — Owner readiness and independent release verification

**Date:** 2026-10-05. The owner reported all five external dependencies READY. This permits verification; it does NOT demonstrate public production acceptance or change the release gate. Keep passwords, actual device configs, recovery codes, MFA QR seeds, tokens and real customer data out of chat and GitHub.

## Five specific evidence requirements

| Reported-ready dependency | Nonsecret evidence from the owner | Independent acceptance |
|---|---|---|
| Provider Rescue/VNC | Redacted, time-stamped successful console administrator session, with the result of id -u and the rescue procedure reference. Do not reboot the currently active private VPS just to generate a screenshot. | Reviewed rescue/recovery drill on a clone or approved maintenance window; prove restored access and unchanged private services. Ordinary SSH without administrator privilege is insufficient. |
| Independent offsite backup | Storage provider/type and nonsecret destination alias, separation and retention; redacted encrypted full-host backup manifest and restore logs. | Build an isolated replacement host from the actual full-host archive. Separately restore the real PostgreSQL base plus WAL at a selected PITR timestamp; verify tenant data/RLS and record measured RPO/RTO. The previous offsite encrypted C320 CLI reference is not a complete host/PG/device-native recovery. |
| Registered domain and DNS | Exact domain spelling, redacted registrar/authoritative-zone panel showing edit rights and an independently readable fresh system-issued challenge TXT. | Validate authoritative DNS from multiple public resolvers, reviewed ingress routing and a browser-trusted certificate for the exact domain. Do not repoint the current laboratory SSH hostname prematurely. |
| Human IdP/MFA | Nonsecret IdP provider and realm plan; redacted MFA enforcement and real sign-in audit for separate Platform Owner, tenant-admin maker, tenant-admin checker and NOC tester. | Run real signed token+MFA/freshness and isolated actual two-company Host/session/API tests, revoke membership and verify immediate deny; never paste tokens or passwords. |
| Physical qualification | Exact manufacturer, model, board/firmware or RouterOS version, management interface and authorized non-customer-impacting test window for ZTE C320, C-DATA, ZTE/VSOL ONT and MikroTik variants. | Redacted timestamped actual CWMP Inform+parameter read and authenticated USP exchange on applicable agent; bounded OLT/RouterOS reads. Require separate vendor-native backup/rollback and independent approval before any physical writes. No untested vendor-wide claims. |

## Read-only baseline observed on 2026-10-05

- Owner-authorized private VPS: Ubuntu Server 26.04, existing nonprivileged SSH user, both existing owner-private services active; DEV-01 C320 currently reports CONNECTED. This is not live layer-1 proof.
- No VPS listeners on port 80/443; external HTTPS443 check failed; no noninteractive privileged deployment identity. Real public SaaS HTTPS and production DB remain unverified.
- The independent public DNS resolver 1.1.1.1 responded NXDOMAIN for **candidate** ipat.id NS, SOA and A. This does not rule out a DIFFERENT real registered domain owned by the user: verify the exact intended spelling before any cutover.
- The unprivileged VPS PATH did not expose restic, pg_basebackup or pg_dump; this does not prove that the separately prepared destination is absent. Restic exists on the owner Mac, but earlier narrow C320 CLI backup is insufficient for full-host/DB/native device recovery.

## Operator procedure: always fail closed

From the reviewed IPAT repository on the authorized owner Mac:

1. Run the read-only readiness manifest script with the EXACT, actually registered intended domain:

    python3 deploy/scripts/production/r979_owner_release_evidence.py --domain EXACT_OWNED_DOMAIN

2. The script checks the already authorized key-only SSH and private VPS loopback and uses bounded public DNS lookup. It outputs sanitized owner-evidence requirements, observed status and source SHA. It never obtains provider-console access, backs up a host, changes any DNS, opens a port, or performs a physical command.
3. The script always exits with status 3 and reports public_go=false. No owner self-reported checkbox is promoted automatically to independently accepted security evidence.
4. Review the actual masked rescue proof and isolated full-host/PG restore before changing the real VPS firewall or installing production infrastructure. After these gates, separately prove real IdP/MFA, trusted TLS/DNS, tenant isolation and exact hardware functionality.

The binding PRD and audit matrix stay production BLOCKED until independently verified real acceptance.
