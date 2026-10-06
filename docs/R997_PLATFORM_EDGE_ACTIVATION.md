# R9.97 — Production PITR-gated Platform Owner public edge activation

R9.97 is the activation phase that follows a successful R9.96 foundation. It intentionally activates only the shared platform edge and shared tenant API. Customer OIDC instances and customer HTTPS ingress remain separate per-company workflows.

## Mandatory PITR acceptance before activation

A root-owned 0600 evidence file must attest the exact R9.96 foundation source and all of the following:

- offsite repository was verified;
- a distinct-host restore was performed;
- pg_verifybackup passed;
- a named PITR target was recovered;
- tenant RLS validation passed on the restored database;
- streaming standby failover was verified;
- fencing was verified;
- measured RPO and RTO are within separately approved targets;
- a different maker and checker approved the evidence within seven days.

This evidence gate is still an operational attestation, not a replacement for audit artifacts.

## Activation order

After all preflight checks, R9.97 activates:

1. Platform Owner API + Platform Owner OIDC on exact loopback.
2. Shared commercial tenant API on exact loopback.
3. DNS ownership verifier.
4. Nginx/Certbot prerequisites without auto-start.
5. Let’s Encrypt short-lived literal-IP staging ACME validation, which is rolled back.
6. Production short-lived literal-IP certificate and exact-IP HTTPS edge.

The existing R9.84 production installer independently validates exact IPv4 SAN, Certbot renewal dry-run, CA-trusted HTTPS, exact Platform API routing and OIDC start redirect.

The orchestrator does not enable customer OIDC instances, customer HTTPS ingress or the customer ingress activator. Those require independent per-company exact Host/TXT/TLS acceptance.

## Rollback

Before activation, no runtime/public listener may already exist. If any runtime, ownership verifier, prerequisite or ACME step fails, the global rollback disables Platform API/OIDC, shared tenant API and ownership verifier and removes just-created runtime environment files. The R9.84 edge subscript keeps its own transactional Nginx/certificate rollback.

The R9.96 foundation and PostgreSQL cluster are not deleted by R9.97 rollback.

## Success state

Even after a trusted literal-IP certificate and public OIDC-start route are active, R9.97 writes `/var/lib/ipat/r997-platform-edge.json` with `public_go=false`. A real independent human MFA browser acceptance, session revocation/replay tests and external hostile Host tests are still required before production GO.
