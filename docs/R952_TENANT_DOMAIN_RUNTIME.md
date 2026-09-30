# IPAT R9.52 — Tenant Domain Runtime

## Scope

R9.52 moves company subdomain/custom-domain routing onto the production control path. It does not turn the historical :3002 lab preview into a public dashboard.

## Data model

Migration: `deploy/db/migrations/0012_tenant_domains.sql`.

A domain record contains tenant UUID, canonical FQDN, domain kind, verification method, lifecycle state, verification timestamp and TLS-readiness flag. FQDN is globally unique. Only active tenant + verified domain + TLS-ready is resolvable.

Platform subdomain example: `customer.ipat.id` with `platform_parent` verification. Custom-domain example: `ipat.fadly.id` with `dns_txt` verification. Examples are not automatically inserted into a live database by the migration.

## Runtime trust

The endpoint is `GET /v1/tenant-context`. It accepts exactly one canonical Host and ignores `X-Forwarded-Host` as authorization input. The response is intentionally safe: tenant slug + hostname, `authentication_required=true`, `business_access_enabled=false`.

The resolver is not a login. Protected APIs must verify OIDC/session identity and current PostgreSQL membership/RBAC+ABAC, then compare that authorized tenant to the resolved tenant.

## Database service identity

Provision a login outside migrations and grant only the NOLOGIN group role:

```sql
CREATE ROLE ipat_domain_reader LOGIN INHERIT
  NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
  PASSWORD '<FROM_SECRET_MANAGER>';
GRANT ipat_domain_query TO ipat_domain_reader;
```

Do not grant table SELECT/INSERT/UPDATE/DELETE. Runtime connection uses a local/private PostgreSQL Unix socket and a mode-0600 owner configuration file.

## Runtime configuration

```text
IPAT_TENANT_DOMAIN_ROUTING=YES
IPAT_TENANT_DOMAIN_DB_CONNINFO_FILE=/absolute/private/path/domain-reader.conf
```

The conninfo file must use `user=ipat_domain_reader`, one absolute Unix-socket host, a database name, no `hostaddr` and no startup `options`.

## Register and activate a platform subdomain

Platform-owned `*.ipat.id` may be verified by control of the parent zone. The platform-admin workflow will create the row as pending, verify the requested label is globally available, provision/confirm the certificate, then atomically mark it verified and TLS-ready.

No domain should be activated by merely receiving a Host header.

## Register and activate a custom domain

1. Create a pending row with verification method `dns_txt`.
2. Generate a random single-use ownership challenge outside Git.
3. Customer publishes the TXT challenge.
4. IPAT verifies DNS from an approved resolver path and records audit evidence.
5. Provision/validate the HTTPS certificate.
6. Set verified timestamp and TLS-ready.
7. Add exact OIDC callback/redirect URI and host-only session policy.
8. Run cross-host isolation tests before business data is served.

For `ipat.fadly.id`, existing DNS pointing is useful routing evidence but is not sufficient by itself for steps 4–8.

## Verification

Static/local:
```bash
python3 -m py_compile deploy/db/tests/test_tenant_domains_integration.py
git diff --check
```

Disposable PostgreSQL:
```bash
python3 -m unittest discover deploy/db/tests -p test_tenant_domains_integration.py -v
```

Rust:
```bash
cargo fmt --all -- --check
cargo test --locked -p control-api tenant_domain::tests -- --nocapture
```

Acceptance requires unknown/pending/suspended/revoked/no-TLS domains to deny, cross-tenant mapping to remain independent, direct table access to be denied, duplicate Host to reject, and forged `X-Forwarded-Host` to have no effect.

## Not completed by R9.52 source alone

Public HTTPS ingress/certificate automation, DNS-TXT challenge service, real browser OIDC/MFA token exchange/session, domain-bound RBAC/ABAC business APIs, and production PostgreSQL deployment/HA remain separate deployment work. Do not advertise the customer dashboard as live until those are actually tested.
