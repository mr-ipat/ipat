# R9.79 — Production-shaped security reviewer identity role

Canonical commercial migration order intentionally excludes historical lab migration 0007. That left a schema contradiction: identity_memberships accepted tenant_admin, noc_engineer, helpdesk and auditor, while the production firmware workflow in migration 0017 requires a current security_admin reviewer. R9.79 resolves only that role-contract gap.

Migration 0032 adds and validates a replacement CHECK that also accepts security_admin, then removes the older restrictive CHECK in the same transaction. It does not import any R8.4 lab tables, grants, APIs or adoption-review workflow. Ordinary lookup_active_membership remains unchanged and still does not accept security_admin. The tenant admin capability remains false for a security reviewer.

The firmware-specific sealed current-reviewer function may now resolve a valid security_admin membership. Revoked, expired or suspended-tenant reviewer identities remain invalid. Tenant API and OIDC session issuer are not granted direct execution of the internal reviewer predicate and receive no raw identity table access.

This milestone makes the existing firmware maker/checker source internally consistent. It does not create a public reviewer enrollment UI, upload firmware, validate a vendor signature, execute a worker, change a device or prove a native restore path.
