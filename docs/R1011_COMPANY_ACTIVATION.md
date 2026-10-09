# R10.11 — Two-person company activation and initial tenant domain bootstrap

The Platform Owner reservation flow creates only a Suspended company. R10.11 adds the next controlled lifecycle step without bypassing production identity or DNS/TLS controls.

A current Platform Owner maker submits an exact suspended tenant, customer-owned hostname and routing mode, the already provisioned external IdP issuer plus subject for the first Tenant Admin, a bounded membership expiry, and a SHA-256 digest of independently reviewed external activation evidence. A different current Platform Owner checker must approve. The maker cannot self-approve.

Approval is atomic: exactly one initial tenant_admin membership is created, one customer domain is created in pending_dns with its persisted TXT challenge, and only that reserved tenant becomes active. Rejection creates none of those effects. Exact request replay requires an identical request UUID, domain UUID, hostname, routing mode, target admin identity, expiry and evidence digest.

The tenant becoming active is not customer release. Tenant browser session issuance still requires an active verified domain. Therefore the first Tenant Admin cannot obtain a browser session on the new hostname until DNS ownership verification, routing, trusted HTTPS/SNI and real OIDC/MFA acceptance independently complete.

The Platform Owner API derives maker and checker identities from current Host-bound Platform Owner sessions. Tenant API, tenant OIDC issuer and ordinary application runtime roles receive neither raw activation-table access nor the underlying activation functions. Browser retries preserve request/domain IDs and expiry after uncertain transport failure.

This solves the first-domain bootstrap cycle: the initial domain is created as pending through independently reviewed platform activation; once that domain is active, normal tenant-admin self-service can manage additional domains.

Still not production proof: real human IdP/MFA provisioning, public DNS TXT control, certificate issuance/renewal, hostile Host/SNI and cross-tenant browser tests, independent full-host plus PostgreSQL PITR restore, and physical device qualification remain separate release gates.
