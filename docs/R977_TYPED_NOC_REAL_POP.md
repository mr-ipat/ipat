# R9.77 — Typed NOC authorization for real POPs

R9.76 separated real POP from Site but retained historical identity_pop_grants.pop_id as an exact Site-code grant. R9.77 adds a separate typed real-POP grant store. There is no migration, inference, or fallback from a Site grant to a POP grant.

A real POP grant is valid only while the tenant is active, the target POP exists, the NOC membership is current, the grant is unexpired and nonrevoked, and requester/approver identities are distinct. Tenant API and OIDC issuer cannot insert or read the raw grant table. This milestone intentionally has no browser grant-write path; reviewed grant lifecycle is a separate next step.

Commercial capabilities return legacy exact-Site scopes and real POP scopes separately. Legacy NOC endpoints keep exact-Site semantics. New real-POP NOC endpoints require an explicit typed grant and list only Sites currently associated with that POP plus secretless saved Device metadata. Moving a Site to another POP changes real-POP visibility immediately without changing legacy Site permission.

The UI labels each scope as Real POP or Legacy exact Site. NOC receives no mutation routes, management endpoints, secret references or physical command controls.
