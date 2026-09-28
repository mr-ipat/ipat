# R9.4 — RouterOS 7 C320 management WireGuard predeployment

Owner confirms a MikroTik RouterOS 7 gateway exists at the C320 site.
Status: OFFLINE design only; no access or changes to real RouterOS,
public Telnet NAT, VPS firewall, routing or customer sessions.
Binding safety controls: R9.0–R9.3, SECURITY and ADR-038–040.
RouterOS version 7 alone does not verify model, firmware patch level,
management interface, VLAN, overlapping IP ranges or recovery method.

## Fixed architecture boundary

- Ubuntu 26.04 restricted IPAT management worker runs WireGuard peer;
  site RouterOS 7 initiates a persistent tunnel to an owner-approved
  VPS management-only UDP endpoint. This is a proposed direction,
  NOT permission to open inbound UDP on an existing shared VPS.
- Route ONLY the chosen isolated OLT management host (/32), not the
  ISP subscriber VLAN, RFC1918 supernet or a default route, through
  WireGuard. Do not place wg-ipat into MikroTik's LAN interface list.
- Telnet TCP/23 must be permitted only from the designated gateway or
  restricted IPAT worker through the verified isolated LAST HOP.
  A VPN over public WAN does not secure a shared/sniffable local hop.
- Deny other worker traffic at both VPS egress and RouterOS forward;
  RouterOS input accepts tunnel control only if strictly necessary.
  Do not modify source NAT, existing default routes, PPPoE or fasttrack
  without independently reviewing the complete existing rule order.
- Retire the WAN public Telnet TCP/321 forwarding as a separately
  approved maintenance action WITH tested out-of-band router recovery.

## Owner-side SAFE read-only discovery (run locally; REDACT outputs)

Do not paste unredacted exports, credentials, interface private keys,
customer IPs or PPPoE users into ChatGPT. A trusted operator can run:

```routeros
/system/resource/print
/system/package/print
/interface/wireguard/print terse
/ip/address/print terse
/ip/route/print terse
/interface/vlan/print terse
/ip/firewall/filter/print terse
/ip/firewall/nat/print terse
```

The `wireguard print` command may expose keys in some display modes:
REMOVE any private-key, preshared-key and sensitive comments before
sharing. Prefer owner-prepared redacted inventory of OS minor version,
router model, management bridge/VLAN, OLT private management host,
VPS source/reachability and recovery capability; do not send full exports.

## Owner-approved pre-change backups and review

Confirm tested WinBox/console/OOB recovery and separately encrypted,
owner-held RouterOS backup/export. Before deployment, record exact
current NAT/filter rule order and management routes in encrypted local
evidence. Owner must review conflict-free WireGuard subnet, exact
private OLT /32 and a VPN UDP port independently permitted by VPS.
Use RouterOS Safe Mode for an attended, pre-reviewed minimal staging
change; it is not a substitute for a proved independent rescue path.
Allow no changes to provider firewall/shared security group until
per-VPS blast radius and recovery are verified.

## Offline address-plan preflight (no changes, no device traffic)

`deploy/scripts/lab/r94/check_address_plan.py` deliberately refuses
public management IPs, a VPN subnet containing the private OLT IP and
VPN conflicts with enumerated existing site/VPS networks. It does NOT
validate actual routes, VLAN separation, NAT rule order or safety.
Example documentation-only synthetic IPs, NOT real site assignments:

```sh
python3 deploy/scripts/lab/r94/check_address_plan.py \
  --vpn-subnet 10.253.77.0/30 --olt-private-ip 192.168.77.10 \
  --existing-network 192.168.77.0/24 \
  --existing-network 10.40.0.0/16
python3 -m unittest discover deploy/scripts/lab/r94 -p 'test_*.py' -v
```

Before implementation: get only REDACTED RouterOS model/minor build,
site gateway ownership/recovery, isolated local management VLAN and
OLT private management /32, existing route-CIDR summary, and planned
VPS UDP permission. Independently pin peer public keys and validate
WireGuard handshakes without exposing keys or touching OLT. Then
separately approve retiring public Telnet NAT under attended rollback.
All SITE-01..06 remain NOT RUN; no Telnet credentials are requested.

## 2026-09-28 exact owner site input (supersedes generic gateway unknown)

Owner confirms gateway hardware is MikroTik **x86 with RouterOS 7**;
OLT is directly reachable on the same local network; owner has
physical or console recovery access. These are REPORTED, not yet
independently inspected. The exact RouterOS 7 patch version,
WireGuard package availability, real OLT private IP and CIDRs, shared
LAN versus physically/virtually isolated management segment, route
return path and existing firewall/NAT rule order are still UNKNOWN.
Direct layer-2 connectivity does NOT prove Telnet cannot be sniffed;
require separately demonstrated dedicated physical port or verified
isolated VLAN/bridge access and restricted switch/router management.

With owner-approved console access, first take a separately encrypted
local x86 RouterOS backup and test recovery without applying any
network changes. Do not place the backup or output of `/export`
with credentials in repository or chat. Stage a new distinct
WireGuard interface, tunnel addresses and narrow /32 routes WITHOUT
altering the existing default route, NAT, PPPoE or customer bridges.
Only after separate review of existing filter order and a tunnel
handshake should source-constrained read-only route verification occur.
Actual public Telnet port-forward closure is a distinct approved
change with console-backed rollback. No live device authentication
until verified last-hop isolation and all R9.1 evidence gates pass.

## Nonintrusive live-distribution rollout option (R9.8 review only)

Reported site is x86 RouterOS 7 in the same local network as C320,
with operator physical/console recovery. BEFORE selecting this option
in a future signed Tenant Admin wizard, inspect actual source/dest
management subnets and confirm an isolated trusted final local hop.
To avoid any static route/configuration change on a busy live C320,
one candidate design is a narrowly scoped WG worker /32 destination
route with source NAT ONLY for worker->one private C320 management
host TCP/23, translated to the site's dedicated management gateway
address. This is merely a PROPOSAL until real return path and existing
NAT/firewall rule order have been independently reviewed. It is NOT
safe to apply blindly on an unsegmented shared customer LAN.

Stage with separate management-only interface/peer and explicit
tenant+site audit; prohibit transit into subscriber VLANs and forbid
C320/ONT write operations. Observe existing PPPoE/session baselines
through independent network telemetry, confirm constrained handshake,
review source ACL and confirm read-only account separately. Reserve an
attended console rollback and independent encrypted router backup.
No OLT default route, VLAN, board, ONT, firmware, PPPoE or firewall
changes have been approved or performed by this documentation.
