# R9.84 — Temporary literal-IPv4 trusted HTTPS ingress

R9.82 selected the existing temporary VPS public IPv4 as the first Platform Owner administrative target. R9.83 added the separate Platform Owner OIDC/MFA issuer source. R9.84 adds the privileged, reviewed ingress installer source needed to terminate browser-trusted HTTPS on that exact IPv4 without repurposing ipat.id or the private C320 dashboard.

## Current external capability verified 2026-10-05

Let’s Encrypt now generally issues certificates containing IP-address identifiers. IP certificates use the shortlived certificate profile and have a validity period of about 160 hours. Current Certbot supports --ip-address; webroot IP support requires Certbot 5.4 or later. The webroot flow requires an already reachable HTTP server for the exact IP challenge. R9.84 therefore uses Nginx only for HTTP-01/HTTPS termination and does not use the Certbot Nginx installer.

## Two-phase production runbook

1. r984_prepare_ip_tls_prereqs.sh runs only as root on Ubuntu Server 26.04 with an explicit opt-in. It simulates APT first, refuses removals and mixed apt Certbot, installs only Nginx/snapd/CA prerequisites while preventing daemon auto-start, installs or refreshes current classic Certbot snap, verifies Certbot >=5.4, leaves Nginx stopped and requires no 80/443 listener afterward. It never alters firewall, DNS or device state.
2. r984_apply_ip_tls_ingress.sh requires an exact globally routable IPv4, ACME contact, root, explicit reviewed opt-in and phase staging or production.
3. Staging phase first: snapshot prior Nginx site/default state, arm a ten-minute systemd rollback before public Nginx mutation, serve only exact-IP HTTP-01 webroot, request a Let’s Encrypt staging IP certificate using shortlived/webroot/ip-address, verify exact iPAddress SAN with OpenSSL, delete staging lineage, restore prior Nginx state and disarm rollback.
4. Production phase: before any Nginx mutation, dedicated Platform Owner API 127.0.0.1:3005 and OIDC issuer 127.0.0.1:3006 must already be exact loopback listeners and answer expected local probes. Wildcard backend listeners are refused.
5. Production certificate uses the same short-lived webroot/IP profile. Unknown Host values return 421. Port 80 redirects only to the configured literal IP. /platform/auth/oidc/* routes only to loopback 3006, everything else only to loopback 3005. Private 3002, tenant BFF 3003 and tenant issuer 3004 are never proxied.
6. A deploy hook rechecks exact IP SAN, runs nginx -t and reloads Nginx after renewal. A bounded randomized timer invokes ordinary certbot renew, never force-renewal. A dry-run with deploy hooks and a local default-CA HTTPS check are mandatory before rollback is disarmed.
7. Any failure while rollback remains armed restores prior Nginx state and removes newly created renewal hook/service/timer. Existing renewal artifacts cause hard refusal rather than overwrite. This is a first-cutover procedure, not an uncontrolled upgrade script.

## Files

- deploy/nginx/r984-platform-ip-bootstrap.conf.template
- deploy/nginx/r984-platform-ip-https.conf.template
- deploy/scripts/production/r984_prepare_ip_tls_prereqs.sh
- deploy/scripts/production/r984_apply_ip_tls_ingress.sh
- deploy/scripts/production/r984_nginx_template_smoke.sh
- deploy/scripts/production/test_r984_ip_tls_ingress_source.py
- deploy/systemd/ipat-ip-cert-renew.service.template
- deploy/systemd/ipat-ip-cert-renew.timer

## Acceptance and limits

Source acceptance includes shell parsing, nine fail-closed/static policy tests, pinned disposable Nginx syntax for bootstrap and TLS templates, and nonroot refusal checks. Real privileged scripts are deliberately not executed against the live owner VPS until an independently approved root/recovery path and both production Platform Owner services/database are ready.

R9.84 does not create PostgreSQL, Platform Owner principals, IdP client secrets, physical-device access or customer domains, and never opens provider firewalls. If external TCP80/443 is blocked, staging ACME fails and rollback preserves prior state; the solution is reviewed provider/network ingress, not automatic firewall bypass.

Public production acceptance still requires real Platform Owner API and issuer connected to production PostgreSQL, actual human MFA, exact external IPv4 ACME staging and production issuance, browser testing from an independent network, full-host and real PostgreSQL PITR restore evidence, then separate customer-domain TLS/SNI and two-tenant hostile-isolation tests.
