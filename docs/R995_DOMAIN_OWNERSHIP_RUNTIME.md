# R9.95 — Dedicated DNS ownership verifier runtime

R9.94 deliberately split DNS ownership authority from ingress activation authority. R9.95 makes the existing native Rust TXT ownership worker actually deployable as its own production-shaped nonroot service.

## Runtime boundary

- Dedicated nologin OS identity: `ipatdnsverify`.
- Dedicated PostgreSQL login: `ipat_domain_verifier_login`, peer-authenticated over local Unix socket only and passwordless. PostgreSQL TCP 5432 remains disabled by the R9.87 bootstrap.
- Dedicated service-owned config directory `/var/lib/ipat-domain-ownership-verifier` mode 0700 and conninfo mode 0600. It is intentionally not nested under root-only `/etc/ipat`, which would make the file unreachable to the worker.
- `control-api` starts with only `IPAT_TENANT_DOMAIN_VERIFIER=YES`; startup explicitly rejects co-location with tenant API/OIDC, Platform Owner API/OIDC, lab, K3s lab, private canary or owner-device service modes.
- The worker has no HTTP listener. It can call only ownership queue + ownership-only lifecycle wrapper established by R9.94; enablement verifies that raw `tenant_domains` SELECT and the historical broad lifecycle function are both unavailable.

## Deployment flow

`r995_prepare_domain_ownership_verifier.sh` creates the service identity and installs the hardened unit but leaves it disabled. `r995_enable_domain_ownership_verifier.sh` is a separate explicit reviewed root step. It validates local DB/socket syntax, verifies peer identity exactly, checks raw-table and broad-function denial, validates systemd, creates the private service-owned conninfo atomically, and enables the service with rollback that disables it and removes the just-created config on failure.

The service runs with `NoNewPrivileges`, zero Linux capabilities, strict filesystem/kernel/device/namespace protection and only AF_UNIX/AF_INET/AF_INET6 because DNS resolution is its only network responsibility.

## Acceptance evidence

Development caught two deployment defects before release: the first config location under `/etc/ipat` was incompatible with existing root-only directory traversal and was moved to a dedicated service-owned `/var/lib` directory; the first PostgreSQL18 bootstrap edit added the peer identity to `pg_ident` but omitted its HBA line, causing a real disposable peer-auth failure. The HBA was corrected and the entire clean-host bootstrap rerun.

Current isolated evidence: source/bootstrap tests 13/13 PASS; Ubuntu26 systemd verify PASS; pinned Ubuntu26/PostgreSQL18 clean bootstrap PASS with both DNS-ownership and ingress-verifier peer roles and no TCP5432; owner-VPS pinned Rust1.98.1 full `control-api` 111/111 PASS. No production VPS service, PostgreSQL cluster, DNS, firewall or physical device was changed.

Actual production DNS ownership remains unaccepted until the intended server is safely deployed after recovery prerequisites and a real customer TXT challenge is observed and transitioned through the restricted service.
