# R9.94 — Least-privilege customer-domain ingress verifier and ordered activation

R9.93 can install a customer-owned exact Host HTTPS ingress after checking the saved TXT token and public A record, but deliberately does not update database activation state. R9.94 closes that source/runtime gap without allowing the DNS ownership worker to become a general activation authority.

## Authority split

- Existing `ipat_domain_verifier_login` remains the **DNS ownership verifier**. Migration 0033 revokes its old direct execute right on the broad lifecycle function and gives it only `record_tenant_domain_ownership_check`, which accepts `ownership_verified` or a `check_failed` scoped to `pending_dns`.
- New `ipat_domain_ingress_verifier_login` is a separate login/NOLOGIN role pair. It receives only `list_tenant_domain_ingress_checks` plus `record_tenant_domain_ingress_check`. It cannot claim ownership, cannot read `tenant_domains` directly, cannot skip database state order, and its `check_failed` scope is limited to `ownership_verified`, `routing_ready`, or `tls_ready`.
- Candidate rows are limited to active tenants, exact custom domains, verified DNS ownership, `a_record` routing, expected `_ipat-verify.<host>` metadata, and states that still need ingress work.

## Evidence sequence

The nonroot `ipatdverify` runtime rechecks the **current persisted TXT token through resolver 1.1.1.1 before every advancement**. It then requires the exact public A result to equal the reviewed VPS IPv4. TLS evidence is collected only by connecting to that exact IP with the customer Host/SNI, normal CA validation, hostname verification, a successful tenant landing request, and an OIDC-start HTTP 303. The resulting evidence hashes bind the customer TXT token, exact Host, exact reviewed IPv4 and TLS certificate fingerprint. Immediately before `active`, TXT, A and trusted HTTPS are rechecked again.

Database ordering still independently enforces `ownership_verified -> routing_ready -> tls_ready -> active`; a verifier cannot skip a state even with a fabricated hash. DNS-verifier and ingress-verifier roles are cross-denied in disposable PostgreSQL tests.

## Production packaging

`r994_prepare_domain_ingress_verifier.sh` creates the dedicated nologin OS identity and installs the hardened oneshot service/timer **disabled**. `r994_enable_domain_ingress_verifier.sh` is a separate reviewed root action: it validates the exact globally routable IPv4, verifies peer authentication to only `ipat_domain_ingress_verifier_login`, confirms no raw table SELECT privilege, validates systemd units, atomically writes a root-only nonsecret environment file, and enables the timer with rollback on failure. It does not modify firewall, DNS, PostgreSQL schema, packages, tenant membership or physical devices.

The PostgreSQL18 first-install bootstrap maps `ipatdverify` only to `ipat_domain_ingress_verifier_login` over the local Unix socket; TCP 5432 remains disabled. The ingress service is hardened with no capabilities, `NoNewPrivileges`, strict filesystem protection, namespace/kernel/device protection and a bounded two-minute timer with jitter.

## Acceptance completed in the source candidate

- Existing commercial/RLS/Platform regressions plus R9.94 PostgreSQL split-role/state tests: 49/49 PASS on disposable PostgreSQL.
- Owner-VPS rootless pinned Rust 1.98.1 full `control-api`: 110/110 PASS after changing the Rust ownership verifier to the ownership-only wrapper.
- Pinned disposable Ubuntu 26.04/PostgreSQL18 full first-install: PASS with ingress peer identity and zero TCP5432 listener.
- Pinned disposable Ubuntu 26.04 `systemd-analyze verify`: PASS.
- R9.94 source/bootstrap tests: 13/13 PASS.

These are source/disposable acceptance only. No real customer domain is activated by these tests. Actual customer production PASS still requires a real tenant/company, authoritative TXT and A, trusted certificate, real IdP MFA login, external hostile Host/SNI isolation, production recovery evidence and approved deployment on the intended VPS.
