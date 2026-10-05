# R9.82 — Temporary VPS IPv4 admin, customer-owned company domains

Owner decision: initial commercial staging no longer depends on registered IPAT brand ipat.id. Use the EXISTING owner-private VPS temporary public IPv4 plan (most recent recorded candidate 202.162.204.121) as the future *platform administrative* address. Each customer company provides its independently owned exact custom domain. Preserve the existing laboratory SSH hostname ipat.fadly.id unchanged.

## Intended end-to-end flow

1. The separate authenticated Platform Owner reserves a company as Suspended. Reservation alone does not create tenant member identities, activate billing/domains or enable physical device operations.
2. After independently approved company activation and real tenant-admin MFA, that company's administrator saves their exact own custom hostname and permitted DNS routing mode in the company dashboard. The existing restricted PostgreSQL function issues and persists a unique per-domain ownership TXT.
3. Only the authenticated same-company tenant-admin session may GET /api/v1/domains/{id}/dns. The server reads the exact existing domain through a tenant-bound database function. It returns the saved TXT name/value, routing records from a configured ingress target and an explicit ready-or-hold decision. A missing/cross-tenant/disabled domain is denied. This endpoint never invents a replacement TXT token.
4. If the reviewed ingress destination is the temporary VPS IPv4, an authorized customer can see a candidate A record for their exact custom hostname without requiring ipat.id. CNAME/NS output requires real separately configured corresponding targets. Routing instructions remain HOLD while actual DNS ownership or reviewed HTTPS ingress readiness is absent. Neither a candidate record nor readiness flags alone activate a company or hostname.
5. Independent DNS TXT verification, routing health, per-customer trusted TLS/SNI and the current tenant Host-bound membership are still required before real activation and external acceptance.

## Literal-IP Platform Owner requirements

The dedicated nonroot Platform Owner process on loopback 127.0.0.1:3005 may accept an exact public IPv4 as the pinned platform Host only with explicit IPAT_R982_TEMPORARY_IPV4_MODE=YES and IPAT_R982_IP_SAN_TLS_REVIEWED=YES, on top of the existing reviewed HTTPS edge setting and a separately created approved platform_console_hosts entry. These configuration settings are operator attestations, not independently observed proof. Literal IPv4 browser access requires a CA-trusted HTTPS certificate containing the EXACT IPv4 in an iPAddress SAN; a brand-name DNS SAN or browser security exception does not qualify. No private control service is published directly.

Read-only acceptance probe on the authorized owner Mac:

    python3 deploy/scripts/production/r982_temporary_ip_tls_preflight.py --ip EXACT_APPROVED_VPS_PUBLIC_IPV4

The probe checks IP eligibility, match with the existing owner-private approved plan, VPS port 443 listener, real externally reachable TCP443 and default-CA exact-IP SAN TLS. It ALWAYS returns public_go=false and exit 3 because provider Rescue Console, independently restored full host and real PostgreSQL PITR, real confidential human MFA/IdP, production database and independently reviewed ingress are separate release requirements. No firewall or device actions occur.

Only after actual engineering review may an operator configure a routing target such as IPAT_CUSTOM_DOMAIN_DNS_MODE=a_record and IPAT_CUSTOM_DOMAIN_IPV4=THE_REVIEWED_VPS_IP; IPAT_CUSTOM_DOMAIN_ROUTING_READY=YES and IPAT_R982_CUSTOM_DOMAIN_TLS_EDGE_READY=YES must reflect independently tested deployments, not the mere existence of this code.

## Scope and evidence

Tests: Rust platform exact IPv4 Host and same-tenant saved-domain guidance, real disposable PostgreSQL16 plus Axum customer request and DNS-instruction flow through the existing R9.68 test, Node static dashboard contract, and read-only Python IP SAN TLS negative tests. Public DNS/HTTPS/MFA are NOT proven by these tests. C320, C-DATA, ONTs, MikroTik physical interoperability, recovery and full PRD public production gates remain separately audited.

## Final source acceptance and live limitation (2026-10-05)

Owner Mac/VPS real current temporary-IP plan matched 202.162.204.121. No listener on the target public HTTPS port, public TCP443 unavailable and no verified browser-trusted exact iPAddress SAN certificate; no production cutover was performed. The source was SHA-verified against local tested code. GitHub PR #175 exact final head d2dc0db846ceaf85d1af60bc9cf9275cbf6d207c passed CI run 37259075082 all 4/4 jobs including the existing R9.68 real disposable PostgreSQL + Axum browser domain flow and newly isolated R9.82 checks; merged to main d160e7ffe051dd910700b83eeeabc4337952c29e. Independently tested 106/106 owner-VPS control-api Rust, 4/4 owner-Mac Python negative checks, Node contract and Rustfmt PASS. Production access to the new console/custom-domain onboarding remains gated by real trusted IP SAN TLS, human MFA issuer, independently recoverable host/DB and real authorized company activation.
