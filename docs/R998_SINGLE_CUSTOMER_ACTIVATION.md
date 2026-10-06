# R9.98 — Exact one-customer OIDC and custom-domain activation

R9.98 turns the existing customer-domain primitives into a deliberately single-customer activation path. It does not enable a global customer-domain timer and cannot activate a different domain through an arbitrary Host header.

## Authority boundaries

Migration 0034 adds `ipat_platform.get_tenant_domain_activation_result(uuid)`, executable only by `ipat_domain_ingress_verifier`. It exposes only one exact custom domain that belongs to an active tenant, has already passed DNS ownership, uses the reviewed A-record path, is not disabled and is in an ingress-relevant state. It cannot mutate anything and does not grant raw `tenant_domains` SELECT.

The one-shot verifier runs as the existing nonroot `ipatdverify` identity with only `ipat_domain_ingress_verifier_login`. It accepts one exact domain UUID and retrieves Host/TXT/state only through the restricted function.

## Mandatory Platform Owner acceptance first

A real Platform Owner browser/MFA acceptance is required before customer activation. The reviewed evidence must match the active R9.97 literal-IP edge and attest real human MFA login, immediate logout/revocation, wrong-Host replay denial and stale-session denial under distinct maker/checker review no older than seven days.

This evidence permits customer activation work; it does not grant a customer release GO.

## Single-customer flow

For exactly one domain ID, customer Host, OIDC instance and OIDC port:

1. Validate R9.97 Platform edge + real Platform Owner human-MFA acceptance.
2. Use the restricted ingress-verifier role to confirm the exact stored domain ID/Host/TXT and current non-active ingress state.
3. Add one exact customer OIDC instance through R9.92. Its own env must already describe a verified confidential IdP.
4. Run R9.93 staging ACME for that exact customer Host; it must validate and roll back.
5. Run R9.93 production ingress for the exact Host; it rechecks public TXT and a single exact public A target, trusted DNS SAN, tenant API Host and OIDC start.
6. Run the R9.98 one-shot verifier as `ipatdverify`. It rechecks TXT, A and trusted TLS/OIDC at every relevant transition and advances only the requested domain through `routing_ready → tls_ready → active`.
7. Read the final exact state through migration 0034 and require `active`.

The R9.94 multi-domain timer is deliberately not enabled by this workflow.

## Rollback and retry

Before database state reaches `active`, global rollback removes the just-added customer OIDC instance and invokes the exact R9.93 rollback created for the production customer ingress. Partial non-active database states such as `routing_ready` or `tls_ready` are intentionally preserved; they do not resolve as active tenant domains and can be safely reverified on a reviewed retry.

Once the exact verifier returns final `active`, database state becomes the authoritative boundary and destructive rollback is disarmed. A local activation marker is evidence-only and any later marker-write failure must not remove a now-active public ingress.

## Success is not customer release

R9.98 activation evidence explicitly records `tenant_human_mfa_browser_acceptance=REQUIRED` and `customer_release_go=false`. A separate external tenant browser acceptance must still prove the real tenant user's MFA, correct menu entitlements, API denial, logout/revocation, wrong-Host/cross-tenant replay denial and customer domain behavior before commercial release.

No physical device operation, firmware action or firewall mutation is part of R9.98.

## Pre-commit consistency audit
The final database transition returning exactly `active` is treated as the authoritative commit acknowledgement; the nonroot verifier performs no fallible read or network operation after that point. R9.93 also supports a production-only root-owned 0600 rollback-path export under `/run/ipat-r998-r993-rollback-*.path`, so the outer R9.98 transaction can invoke the exact customer-ingress rollback without heuristic directory discovery. These controls prevent DB-active/edge-rolled-back and edge-active/OIDC-rolled-back split states caused by orchestration bookkeeping failures.
