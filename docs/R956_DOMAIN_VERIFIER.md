# R9.56 — DNS ownership verifier on the canonical domain registry

This work ports the isolated Rust DNS TXT verifier to the R9.53 canonical
UUID-based tenant_domains schema. The background process does not register
customers, issue TLS, grant tenant authority or adopt devices.

## Ordered operation

1. Only an authenticated and currently authorized Tenant Admin can save a
   domain request using migration 0013, generating a unique TXT challenge.
2. Migration 0015 exposes a bounded queue of pending custom-domain challenges
   from active tenants to the existing restricted verifier capability role.
3. A dedicated nonroot Rust worker checks the exact DNS TXT challenge through
   the system resolver, including concatenated TXT chunks. An exact match
   produces SHA256 evidence and invokes the sealed lifecycle function for
   ownership_verified only.
4. Missing or incorrect DNS does not advance the domain. The worker records
   bounded DNS_TXT_MISMATCH or DNS_TXT_LOOKUP_FAILED errors.
5. Independent services must still verify routing readiness, valid TLS and
   correct authenticated tenant/session binding before production activation.

## Deployment requirements (currently NOT fulfilled on the VPS)

- PostgreSQL production-shaped cluster installed, restricted runtime login
  roles and separate off-host backup/recovery proven. Apply migrations through
  0015 in order under change control; do not re-run non-idempotent migrations.
- Unix-socket peer mapping for ipat_domain_verifier_login with no database
  password in a config file. Place private conninfo in a 0700 nonroot-owned
  directory, file 0600, without symlinks. Example contents (placeholder):
  host=/var/run/postgresql user=ipat_domain_verifier_login dbname=ipat
- Run the separate worker with these environment variables:
  IPAT_TENANT_DOMAIN_VERIFIER=YES
  IPAT_TENANT_DOMAIN_VERIFIER_DB_CONNINFO_FILE=/private/path/conninfo
  IPAT_TENANT_DOMAIN_VERIFIER_INTERVAL_SECONDS=60
  Execute the control-api binary separately from the HTTP systemd service.
- Real Tenant Admin MFA/OIDC BFF Save must be mounted before customers can
  create domain requests. DNS ownership success alone is NOT readiness to
  point, route, issue TLS, or activate.
- Read-only offline checks: pinned cargo fmt, cargo test --locked,
  isolated PostgreSQL integration. No test claims public DNS/TLS deployment.

## C320 owner path

The VPS connector is running, and DEV-01 draft remains preserved. On the
owner Mac through the existing private SSH tunnel, open
http://127.0.0.1:3002/lab/device-workbench . The owner obtains the one-time
lab setup code using the exact private SSH command displayed inside
Advanced Security and pastes it locally alongside an approved dedicated
read-only C320 device credential. Never put either credential in chat/Git.
Only after enrollment and independent device host-identity checks may the
operator execute the fixed, bounded read-only inventory flow. Firmware or
other physical writes remain independently blocked.

## Recovery

Stop only the dedicated verifier service if DNS verification behaves
unexpectedly. Do not drop migration 0015 in a used database. Preserve audit
evidence, pending requests, old release binary, and separately proven backup.
TXT resolution may be stale due to DNS caches; failures remain pending.
