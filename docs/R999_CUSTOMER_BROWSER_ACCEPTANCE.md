# R9.99 — Exact customer human-MFA/browser release gate

R9.98 deliberately stops at an active custom-domain edge with customer_release_go=false. R9.99 is the final per-customer evidence boundary: only a recent independently reviewed real-browser acceptance artifact may create a customer release marker.

Required evidence is exact to one domain/host and contains two distinct tenant identities/hosts. Every item must be true: real human MFA browser login, unauthorized menu hidden, unauthorized direct URL denied, unauthorized API denied, wrong-Host session replay denied, cross-tenant API denied, logout/revocation verified and stale-session denial verified. The evidence requires distinct maker/checker, a nonzero SHA256 of the external browser-test artifact and a review no older than 72 hours.

The root finalizer revalidates the existing R9.98 activation marker, then reads the exact domain using the restricted ipat_domain_ingress_verifier_login function. It does not change DNS, Nginx, certificates, firewall, database lifecycle or network-device state. Only if the database still reports that exact Host as active does it create /var/lib/ipat/customer-releases/<domain>.json with customer_release_go=true.

This source does not fabricate the required external browser artifact. Actual production release therefore remains blocked until the real customer and a distinct control tenant complete the acceptance ceremony against the real public custom domains and human MFA IdPs.
