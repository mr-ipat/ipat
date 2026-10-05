# R9.87 — PostgreSQL 18 local-socket first-install bootstrap

R9.86 proves current IPAT security migrations on PostgreSQL 18, which is the current Ubuntu Server 26.04 package baseline observed on the authorized VPS. R9.87 adds a production-shaped first-install staging bootstrap, not a production HA declaration.

The bootstrap is root-only, Ubuntu 26.04-only, explicit-opt-in and refuses any existing PostgreSQL cluster. It validates a SHA-256 manifest covering the exact 23 currently selected schema/security migrations before installing or applying anything. PostgreSQL is configured with listen_addresses empty, Unix socket /run/postgresql, SCRAM as the password-encryption policy, WAL level replica, and no TCP listener.

## Identity separation

R9.85 must have already created the nonlogin Linux service users ipatpapi and ipatpoidc. During first bootstrap only, R9.87 creates a temporary nologin Linux identity ipatpgmigrate. PostgreSQL peer mapping temporarily maps both OS postgres and ipatpgmigrate to DB superuser postgres, solely because current reviewed migration files create/transfer object ownership among many NOLOGIN roles and cannot run under an ordinary CREATEROLE role. An actual PostgreSQL 18 test failed with: must be able to SET ROLE "ipat_schema_owner".

After all hash-pinned migrations complete, bootstrap peer mapping is fully replaced by final runtime-only mappings:

- ipatpapi -> ipat_platform_session_api_login
- ipatpoidc -> ipat_platform_session_issuer_login
- local postgres remains peer-only for host administration
- all other local authentication is rejected

The ipatpgmigrate OS account is deleted before success is declared. No database password is stored in Git or runtime env.

## Package and rollback constraints

The installer suppresses Ubuntu postgresql-common automatic main cluster creation, installs PostgreSQL 18 only after checking the Ubuntu 26.04 candidate, creates only cluster 18/ipat, and rolls back the newly created cluster and temporary migration user on bootstrap failure. Existing clusters cause immediate refusal and are never modified.

R9.87 does not alter host firewall, provider ACLs, Nginx/DNS, K3s or any physical device. It does not enable a public PostgreSQL TCP socket.

## Acceptance evidence

On owner Mac, a pinned ubuntu:26.04 disposable container executed the complete first-install path: package installation, cluster creation, 23 hash-pinned migrations, temporary migrator removal, zero TCP 5432 listeners and successful passwordless restricted peer login for Platform API and Platform OIDC. The disposable test emitted R987_UBUNTU26_DISPOSABLE_BOOTSTRAP_PASS.

This is still NOT HA/PITR production acceptance. Before public commercial cutover, IPAT still requires an independent standby/failover design, encrypted off-host WAL/base backup, restore into an independent host, measured RPO/RTO, recovery reconciliation, actual root execution on the intended VPS, real IdP secrets/MFA and trusted HTTPS ingress.
