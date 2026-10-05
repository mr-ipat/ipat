# R9.85 — Platform Owner production-shaped nonroot runtime services

R9.84 can expose browser-trusted HTTPS only after the separate Platform Owner API (3005) and OIDC/MFA issuer (3006) already run correctly on loopback. R9.85 adds an explicit first-install runtime package for those two processes.

## Separation

- ipat-platform-api.service runs as Linux user ipatpapi, reads /etc/ipat/platform-api.env, executes /opt/ipat/bin/control-api and is additionally restricted by systemd IPAddressDeny=any / IPAddressAllow=localhost.
- ipat-platform-oidc.service runs as Linux user ipatpoidc and reads /etc/ipat/platform-oidc.env. It must reach the separately approved external IdP but the Rust process itself binds only 127.0.0.1:3006.
- Both services drop Linux capabilities, use NoNewPrivileges, strict filesystem/kernel hardening and separate database identities.
- Neither service contains physical-device credentials or routes.

## Two-step install

r985_prepare_platform_runtime.sh is root-only, Ubuntu26-only and explicit-opt-in. It installs a checksum-pinned already-built control-api binary, creates two nologin service users, creates root-only /etc/ipat and an ipatpoidc-owned 0700 secret directory, installs the service units, and leaves both services disabled/stopped.

The operator or secret manager then provisions, outside Git:
- root-owned 0600 platform-api.env and platform-oidc.env staging files;
- ipatpoidc-owned 0600 client-secret and pinned IdP public-key files inside /var/lib/ipat-platform-oidc/secrets;
- PostgreSQL local authentication mapping that allows ipatpapi -> ipat_platform_session_api_login and ipatpoidc -> ipat_platform_session_issuer_login without embedding a database password in repository/source.

r985_apply_platform_runtime.sh validates the exact public IPv4, required runtime gates/Host equality, env ownership/mode, dedicated OIDC secret ownership/mode/link count, database socket directories, and actual restricted PostgreSQL login for each Linux service identity using psql -w. It never edits PostgreSQL configuration or roles.

Before service activation it arms a ten-minute rollback. It installs final root-only env files, enables/starts both services, then proves exactly one 127.0.0.1:3005 listener and one 127.0.0.1:3006 listener with no wildcard bind. It checks the Platform API exact-Host sign-in page and a no-side-effect OIDC 404 probe. Any failure stops/disables both services and removes runtime env files.

## Secrets

The repository contains examples only. The examples use documentation IP 203.0.113.10 and placeholder IdP values. No client secret, password, MFA seed, production signing token or device credential belongs in source control.

## Acceptance boundary

Source tests check separation, hardening, checksum-pinned first install, exact DB roles, secret-file constraints, loopback listeners, rollback, and absence of firewall/device commands. systemd-analyze verification is run on Ubuntu 26.04. Actual production acceptance still requires a production PostgreSQL instance/roles and authentication mapping, real confidential IdP client+human MFA, securely provisioned secret/key files, root execution of prepare/apply, and then R9.84 staging/production IP certificate cutover.
