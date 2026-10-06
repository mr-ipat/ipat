# R9.93 — Exact customer-domain HTTPS/SNI ingress

R9.93 converts a customer-owned verified DNS name into a production-shaped HTTPS ingress without reusing the Platform Owner backend.

## Required sequence

1. R9.92 tenant API and the customer-specific OIDC issuer must already be active on exact loopback only: shared tenant API 127.0.0.1:3003 and one reviewed customer OIDC port 31000–31999.
2. The customer domain must already have completed the independent TXT ownership gate. The installer also requires the exact persisted `ipat-domain=<uuid>` value and rechecks `_ipat-verify.<host>` through a public resolver before any ingress mutation. The customer then points exactly one public A record to the reviewed VPS IPv4. Before trusted TLS exists, the bootstrap Nginx site serves only ACME HTTP-01 and returns 503 for every application path; no sign-in page or API is exposed over plaintext HTTP.
3. The installer verifies the exact A record through an independent public resolver, validates both loopback backends, arms a 10-minute rollback and installs an exact server_name only. A staging ACME run always deletes its certificate and rolls back the site.
4. Production obtains a certificate for the exact customer DNS name, verifies the DNS SAN, installs the HTTPS site, routes only /auth/oidc/* to that customer's OIDC instance and routes the remaining dashboard/API paths to the shared tenant API. The Host passed upstream is a fixed rendered exact customer hostname, never a forwarded client-selected authority.
5. The script verifies trusted HTTPS locally with curl --resolve, verifies the real OIDC start route returns its redirect, performs a Certbot dry-run renewal with deploy hook, and only then cancels the rollback timer.

Unknown Host/SNI values are never mapped to a customer backend by this site. Existing Platform Owner services 3005/3006 and private/lab services are not referenced.

## Security boundary and remaining gate

The script never changes firewall/security-group configuration, PostgreSQL state, tenant-domain activation state or physical devices. It intentionally does not call record_tenant_domain_check. Successful HTTPS installation therefore does not by itself change a tenant domain from ownership_verified/routing_ready/tls_ready to active. A separate restricted verifier/activation milestone must persist independently observed routing/TLS evidence and then perform final activation through the existing ordered domain lifecycle. This prevents a root deployment script from silently granting tenant authority.

Source acceptance requires Bash syntax, six fail-closed source tests and pinned Nginx 1.28 bootstrap+TLS syntax. Real production acceptance additionally requires actual customer TXT proof, public A propagation, real ACME issuance, external browser MFA and cross-tenant hostile Host/SNI tests.
