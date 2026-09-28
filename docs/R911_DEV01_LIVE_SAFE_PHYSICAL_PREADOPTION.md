# R9.11 — Live-distribution DEV-01 private SSH pre-adoption checkpoint

Date: 2026-09-28 Asia/Jakarta. Treat C320 as LIVE distribution with
active ONTs/subscribers. This milestone performed one bounded physical
private SSH credential-free handshake after a first 9-second timeout;
20-second bounded retry returned banner `ZTE_SSH.1.0` and the same
observed RSA SHA256 fingerprint as prior R9.10. No public Telnet
credential test, login, show command, firmware, ONU, router or
customer management-plane change was performed.

## Implementation

- `deploy/scripts/lab/r911/private_ssh_identity_probe.py` validates
  exact RFC1918 host and TCP port and requires a nonroot opt-in before
  ONE bounded credential-free SSH handshake. Legacy SSH allowances
  are process-local only and apply to NO credentials or commands;
  host verification is never bypassed or declared successful.
- `web/lab/physical-intake-evidence.json` is dated HISTORICAL physical
  transport evidence with no addresses/secrets and ALL six physical
  admission gates false. It NEVER claims online/healthy/adopted.
- Private `GET /lab/device-physical-evidence` exposes the static record
  with no-store; Device Manager renders the gate checklist and fails
  closed on unexpected positive values. This is a lab status surface,
  NOT privileged Tenant Admin onboarding or live telemetry.
- Five R9.11 offline tests are included in the existing R9.0 CI unit
  step; Rust Axum tests check no-store and zero-adoption on HTTP.

## Required actual no-impact acceptance — NOT MET

1. Independently match the RSA fingerprint through trusted local
   owner console or verified out-of-band inventory, not network scan.
2. Independently prove dedicated management LAN isolation, SSH
   compatibility and actual read-only credentials. Default factory
   account is not acceptable for a live distribution OLT.
3. Before any authenticated connection, capture owner-trusted OLT,
   ONT, router and PPPoE baselines with incident abort thresholds;
   arrange on-site physical/console recovery.
4. Make exactly one approved bounded read-only request on the isolated
   trusted route after maker/checker approval and signed human MFA.
5. Abort immediately if alarms, control-plane load, session stability
   or any subscriber metric worsens. Record genuine chassis serial,
   board/model and actual firmware before approving vendor adapter.
6. Only then record measured health; never infer no customer impact
   from silence. No upgrade/config push until a separate change review.

One can never absolutely guarantee zero performance effect when
connecting to a live OLT; these gates minimize, observe and bound it.

## Reproduction, noninvasive scope

Run offline/no-device tests from a clean checkout:

```sh
python3 -m unittest discover deploy/scripts/lab/r90 -p test_r90_preflight.py -v
node --check web/lab/device-workbench.js
cargo fmt --all -- --check
cargo test --locked -p control-api
```

ONLY with renewed exact owner approval and on the Mac/site worker
already proven to have an appropriate PRIVATE route, a nonroot
operator may execute ONE bounded SSH no-credential fingerprint
observation (never run as a scheduler):

```sh
IPAT_R911_APPROVE_ONE_NOAUTH_PRIVATE_SSH_PROBE=YES \
  python3 deploy/scripts/lab/r911/private_ssh_identity_probe.py \
    --probe --private-ipv4 YOUR_TRUSTED_PRIVATE_IPV4 --port YOUR_APPROVED_PORT
```

This requires no keys or OLT credentials and never indicates verified
identity; the default outcome is untrusted SSH key or unavailable
transport. It must not be used for repeated polling of a live OLT.
No physical authentication step is enabled until independently
verified source trust plus isolated last hop and read-only account.
Private Axum evidence is mounted only in explicit non-K3s lab mode;
a separate regression ensures it is not mounted by normal public API.

## Safe parallel loopback canary for the real VPS

To demonstrate the new LAB physical-intake panel without restarting
the existing SSH-private 127.0.0.1:3000 app, R9.11 permits a SEPARATE
opt-in identity-free/private-only canary at 127.0.0.1:3002 using
`IPAT_LAB_WEB=1 IPAT_R911_PRIVATE_CANARY=YES`. The normal lab still
binds 127.0.0.1:3000; the distinct identity lab still uses :3001.
Private canary never exposes physical connection functionality or
secret-taking HTTP routes. It refuses non-lab, actual OIDC, restricted
membership, registry/reviewer options and K3s mode. Review real
per-VPS loopback listener before attempting side-by-side deployment.
No live OLT or site-router connectivity is required to display
historical transport evidence. This page is a PRIVATE DEMO accessible
only through an approved SSH tunnel, NOT an authenticated tenant
dashboard. Never enter actual keys or device passwords into it.

## Actual private canary on owner VPS: independently exercised

On the actual nonroot Ubuntu owner VPS, source-only Git bundle was
created from reviewed Mac checkout, verified by SHA-256 and Git bundle
base check, and checked out separately at exact tested code SHA
`621ad09ae466283898afd4f5de951ab3dffedc68`. The canonical VPS
main workspace and old :3000 process were NOT modified. Bounded
low-priority single-job `cargo build --locked --offline -p control-api`
produced a binary with SHA256
`39d3bb51d6b5359c48af00732d185cd0c46aab65b02ab207ec26a206b0cec233`.

`private_loopback_canary_smoke.py` starts a temporary nonroot, no-OIDC
process at 127.0.0.1:3002, tests the evidence JSON and true status
claims, GET 200 / no-store, POST 405, both forged business API GET
401, loopback-only bind, old :3000 health 200 before/after, then
terminates and checks the canary listener is gone. Initial script
attempts FAILED locally on a TIME_WAIT port-bind check and
case-sensitive HTTP header check, then were CORRECTED. The final
actual VPS full canary smoke PASSED. No root, SSH policy, firewall,
customer data, site gateway or OLT operation was modified.
The new panel was smoke tested on actual VPS as a TEMPORARY canary,
NOT left running as an always-available service or rolled out to
production/tenant users. Existing :3000 main LAB remains unchanged.
