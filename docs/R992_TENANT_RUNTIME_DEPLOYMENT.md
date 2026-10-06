# R9.92 — Customer-company tenant runtime deployment

This milestone turns the already tested commercial tenant HTTP/OIDC code into explicit production-shaped services without mixing Platform Owner or device processes.

- `ipat-tenant-api.service`: one shared nonroot loopback BFF on `127.0.0.1:3003`, current exact Host-bound browser sessions on every privileged API call, DB identity only `ipat_tenant_api_login`.
- `ipat-tenant-oidc@.service`: one exact customer Host issuer instance per reviewed deployment record. R9.92 allows a loopback port chosen from 31000–31999 by the root-owned env; each instance keeps R9.71 PostgreSQL durable one-use pending state and uses only `ipat_oidc_session_issuer_login`.
- `r992_prepare_tenant_runtime.sh` creates only nologin users, secure directories and disabled systemd units; it does not install packages, mutate PostgreSQL or touch network/device state.
- `r992_apply_tenant_api.sh` validates root-owned configuration and passwordless local PostgreSQL peer identity, arms timed rollback, then starts only the shared BFF.
- `r992_add_tenant_oidc_instance.sh` validates exact lowercase DNS Host, bounded free loopback port, real-IdP/durable-pending flags, dedicated 0600 issuer files and issuer-only DB peer identity, arms timed rollback and starts one instance.
- R9.87 first-install PostgreSQL bootstrap now requires the tenant OS service identities and maps all four Platform/Tenant runtime roles separately. PostgreSQL still has no TCP listener.

External customer ingress is intentionally not inferred from service activation. A later reviewed ingress step must bind a verified customer Host/SNI certificate, route `/auth/oidc/*` to that Host's OIDC instance and tenant dashboard/API routes to 3003, reject unknown hosts, and then pass real browser MFA plus cross-tenant denial tests. No customer is considered active from source tests alone.
