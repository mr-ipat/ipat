# R9.15 — Site A IPAT centralized tunnel ownership (user direction)

Status: approved architectural DIRECTION; prototype is safe lab-only.
IPAT / centralized dashboard acts as **Site A**, owns central listener,
peer onboarding and tenant policy. Site B (RouterOS7 or Linux) receives
a reviewed pairing profile and **the site operator applies it LOCALLY**.
No dashboard-initiated SSH/API configuration push to customer routers.

For Site B already reachable via an independently verified *private
bidirectional* route, select direct-private without a redundant VPN.
RFC1918 IP syntax alone, or an existing VPS default gateway, is
NOT evidence of private site reachability. A remote Site B uses a
publicly reachable, independently verified Site A endpoint (or a
separately verified approved private cross-site network). Do not
blindly send traffic to a VPS private address from the Internet.

## Intended data flow

1. Tenant admin at IPAT Site A chooses tenant/site, method and central
   endpoint. Site A validates actual route, VLAN/last-hop isolation,
   address overlaps, existing routes and change evidence.
2. For WireGuard, Site A generates/stores its private key locally in
   a secure secret manager and exposes only its public key, approved
   endpoint, peer subnet, port, and narrow allowed routes in a B
   pairing profile. B independently generates/retains its own private
   key and submits ONLY its public key and reviewed site metadata.
3. Site A stages its OWN listener/peer after genuine signed MFA,
   independent maker/checker and explicit rollback/health proof.
   The download package instructs the Site B operator to import and
   approve the peer on Router B; the panel NEVER pushes to B.
4. After Site B locally applies configuration, IPAT observes the
   handshake and tests ONLY the approved management host /32,
   return-path isolation and documented service baseline. Do not
   connect OLT or mark it ONLINE based solely on tunnel handshake.
5. B operator can rotate keys/revoke locally; A revokes its own peer,
   preserving immutable audit and rollback. Physical OLT adoption is
   a separate gated one-command read plus chassis/firmware evidence.

## Implemented, verified boundaries

- `deploy/scripts/lab/r915/site_a_plan.py` pure offline review:
  exact no-secret JSON schema, private/public endpoint classification,
  direct-private versus Site A WireGuard / IPsec plan, restrictive
  /30 and exact management /32 routes, and existing subnet collision
  refusal. It ALWAYS returns REVIEW_ONLY, no active configuration,
  no credentials, no pushes, no real route measurement or adoption.
- Private LAB Rust POST `/lab/demo/site-a-plan` validates fixed
  *synthetic* choices only; returned Site A/Site B direction and
  topology classification remain nondeployable.
- Private Device Manager offers hub endpoint scope, gateway, topology
  and method selections and explicitly states the no-push policy.
  This UI is NOT actual tenant-controlled WireGuard provisioning yet.

## IMPORTANT: Site A routes versus Site B routes

OLT remains on local Site B management LAN. Site A's WireGuard peer
AllowedIPs can include ONLY B's tunnel /32 and the reviewed target
OLT /32. Site B's peer policy should allow ONLY Site A tunnel /32
(or additional strictly approved return routes); no default route,
customer VLAN or generic RFC1918 supernets. B needs a separately
reviewed OLT return path or narrowly reviewed management SNAT; neither
is assumed or auto-installed. A tenant-specific worker may reach the
OLT ONLY after independent physical site gates are satisfied.
## Actual DEV-01 acceptance blockers (not implementation successes)

The owner-reported ZTE C320 SSH banner is observed but host identity
has not been independently confirmed at the chassis console.
VPS `ip route get` to owner-supplied C320 private address is still
DEFAULT_ROUTE_ONLY; no verified durable management tunnel exists.
Owner-reported site RouterOS7 model, exact patch, interface inventory,
bridge/firewall order, isolation and recovery have NOT been
independently checked. No owner-specific public keys, signed
Tenant Admin MFA or full no-impact live baseline were supplied.
The server's nonroot `wg` binary was absent at last check. We do NOT
install it, configure a public UDP listener, alter firewall, change
customer routes or use factory ZTE credentials without full review.

To reproduce synthetic offline planning, make a 0600 owner-only JSON
file outside the repository containing only the documented nine
topology fields, then execute:

```sh
python3 deploy/scripts/lab/r915/site_a_plan.py \
  --local-reviewed-topology-json /ABSOLUTE/PRIVATE/PATH/topology.json
python3 -m unittest discover deploy/scripts/lab/r915 -p 'test_*.py' -v
```

All example addresses must be documentation-only and not inferred as
real site assignments. The output is design guidance, not proof of
physical isolation, real site route, or permission to modify a live
OLT or router.
