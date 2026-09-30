# R9.16 — Actual local Site A dev-key custody and Site B manual pairing review

Status 2026-09-28: implemented and tested on owner nonroot IPAT VPS
as **developer-only staging**, not operational tunnel/adoption.
The owner chose IPAT dashboard/VPS as Site A. Site B generates and
retains its OWN private key and a local operator configures Router B;
IPAT must NEVER push Router B configuration via SSH/API.
For an independently verified existing private network, avoid a new
WireGuard tunnel entirely. If B is outside, the approved Site A hub
requires a reachable public endpoint; private RFC1918 syntax alone
never proves intersite bidirectional reachability.

## Delivered

- `deploy/scripts/lab/r916/site_a_keypair.py`: original local X25519
  keypair generator/readback; no dependency on GenieACS or remote
  connection. Nonroot owner-only 0700 folder, 0600 non-symlink exact
  public/private files, atomic exclusive creation, never overwrite,
  fail-close on partial writes, outside source repo. Only public
  44-character WireGuard-compatible key is ever returned. This is
  NOT a production secret manager or durable backup.
- `deploy/scripts/lab/r916/reconcile_site_a_pairing.py`: strictly
  local Site A public-key readback, independently submitted Site B
  PUBLIC key (never Site B private key), exact no-secret owner-only
  topology, bounded UDP port, disabled reviewed RouterOS7 B commands.
  For independently verified direct-private route, NO VPN package.
  IPsec and Linux B pairing remain unsupported; no fake execution.
- Private LAB Rust GET `/lab/dev-site-a-public-key` reads only the
  exact owner-only Site A `public.key` leaf, with no-follow/owner/mode
  checks. Private LAB POST `/lab/demo/site-a-manual-pairing` matches
  Site A key to a submitted B public key and validates narrow /30
  host allocations and private OLT /32, returning ONLY three
  `disabled=yes` RouterOS B review lines. Never sends any command,
  stores B key or applies WireGuard. Real MFA, endpoint reachability,
  conflict-free on-site subnets, return path, OLT/ONT baseline and
  independent maker/checker all remain explicitly FALSE. Requests
  accept NO private keys, passwords or unknown fields.
- Device Manager private LAB panel now shows the *dev-only* Site A
  public key supplied by backend (if configured), and accepts only
  Site B PUBLIC key plus planning values. It renders disabled text via
  `textContent`, never HTML injection or active script execution.
  Public key input and planning values are NOT production site authority.

## Actual Site A VPS developer-only evidence

`cryptography` X25519 was available on owner VPS, but `wg` command
was NOT present at previous inspection; no `wg-quick`, UDP listener,
firewall change or live Router B/OLT management has been authorized.
Actual VPS staged ONE dev-only keypair in:
`/home/openai/.local/share/ipat/r916-dev-keys/site-a-dev01-lab/`
with directory 0700 and key files 0600. The private key MUST remain
on this host; it has no verified production vault, independent backup
or active WireGuard listener. The operator-only script read its public
key back and independently validated that X25519 public matches the
sealed private key; no key content was logged or committed.

An ACTUAL offline owner-VPS test combined that real *developer* Site A
PUBLIC key with an ephemeral synthetic (NOT REAL Router B) PUBLIC
key and documentation-only topology. Renderer returned exactly THREE
nonexecuting `disabled=yes` RouterOS7 B review commands, narrow /32
peer routes, no default route, no private key, no push and no actual
network action. The temporary sample package was discarded. The
actual external Site B PUBLIC key has NOT been received.

## Separation of dev simulation from actual live acceptance

MUST remain false until independently verified: actual Site B device
identity and RouterOS patch, genuine B public key, approved A public
endpoint and UDP listener, peer handshake, actual overlapping route
inventory, managed private worker route and trusted OLT last hop,
owner-side recovery/rollback and actual signed Tenant Admin MFA.
The earlier owner-reported C320 banner/fingerprint is a NETWORK
observation only, not trusted physical OLT key; the RouterOS7 gateway
host key previously differed from the Mac's existing pinned key.
No actual OLT login or physical adoption can be asserted from this
key-generation milestone.

## Operator SOP (after separate security review)

To inspect Site A DEV public key, without copying a private key:

```sh
ssh ipat-lab
python3 /home/openai/.cache/ipat/r915-preview/site_a_keypair.py \
 --show-public \
 --owner-only-folder /home/openai/.local/share/ipat/r916-dev-keys \
 --site-slug dev01-lab
```

Do not deploy this DEV key to real routers without verified production
vault/backup, IP/site identity, firewall and change controls. On the
future authorized tenant dashboard, export only Site A PUBLIC key,
exact independently verified endpoint, narrowed Site A tunnel /32
and approved port. Router B's operator generates/retains B PRIVATE
key, gives IPAT only B PUBLIC key and locally applies independently
reviewed B config after rollback readiness. Private-connected sites
must opt out of unnecessary VPN.

## Actual restricted Site A VPS dashboard rollout / rollback evidence

Reviewed app source SHA `63c42e333e1351b477a073ef5983ffdb100ee1a2`
was transferred from the authorized Mac using a SHA256-verified Git
bundle into a separate new nonroot owner-only VPS checkout at
`/home/openai/.cache/ipat/r916-preview/src`, preserving the canonical
VPS workspace. Pinned single-job, low-priority OFFLINE Rust build and
three real `site_a_pairing_lab` unit tests PASSED; binary SHA256:
`3fb2d5369a1e42b35a05ba4c128ab0b8f8414c1997f02c7a1085741587db46da`.
This source changed no real router/OLT/ONT infrastructure.

New dedicated PRIVATE DEV-only user unit:
`deploy/scripts/lab/r916/ipat-r911-preview.service`
SHA256 `b0bf33b1012dba4ff8fb967cc9b2b6dda3131e58f824d770e7ffe92dc6b2c0f8`.
It updates ONLY the existing `127.0.0.1:3002` nonroot private LAB
listener, binding a dev-only Site A key-folder environment path while
retaining `NoNewPrivileges`, read-only host filesystem, 256MiB and
20% CPU ceilings. The original :3000 API remains unchanged. Do NOT
expose the new key route via a public ingress: it is a private DEV
public-key interface, not tenant-authenticated production API.

The FIRST actual VPS HTTP smoke attempt revealed a SMOKE-HARNESS
failure, not a Rust handler failure: it truncated the ~19KiB HTML
and ~23KiB JS at 10KiB before checking for new UI controls.
The strict user-service rollback restored the R9.15 user unit; after
repeated short test restarts hit systemd start limiting, the owner
nonroot manager's failed state was reset and the prior private
service independently verified ACTIVE, with old :3000 HTTP200.
The harness was corrected to bounded 32768-byte reads and regression
checks now guard full HTML+JS size. The deployment rollback was
hardened to reset the user unit failed state and reuse only the
independently SHA-verified previous unit backup.

Final versioned HTTP smoke SHA256:
`2d549259a4efb3b3aa0e519979aa229965e316dcd8d2b45646c7e572f9e86ad7`.
Versioned guarded one-shot nonroot upgrade:
`deploy/scripts/lab/r916/deploy_private_preview.sh`, SHA256
`ffe1e77b97c7d8fb4fa5822f053ba23cf7381cfc377aae940be29b613209398e`.
This script was hash-verified and ACTUALLY EXECUTED on owner VPS,
result `R916_DEV_SITE_A_PUBLIC_ONLY_PRIVATE_VPS_PREVIEW_PASS`.
It confirmed actual dev Site A public HTTP GET, synthetic B public
POST reviewed narrow addresses and exactly 3 disabled RouterOS7
lines, wrong key/unknown secret field denied, missing Origin denied,
fake real business endpoints HTTP401, private listener, and untouched
original :3000 HTTP200. A separate new Mac temporary SSH local
forward also fetched the real new dashboard, read the actual VPS
Site A DEV public key and POSTed an ephemeral SYNTHETIC B public key;
all no-push, no listener, no-adoption assertions PASSED. The Mac
forward was closed, and no router peer or OLT was contacted.

Owner-only previous user-unit rollback remains at:
`/home/openai/.cache/ipat/r916-preview/rollback-unit.service`.
Nonroot safe rollback (only if necessary):

```sh
ssh ipat-lab
cp -p /home/openai/.cache/ipat/r916-preview/rollback-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
```

Actual DEV panel Mac access (keep Mac SSH tunnel terminal open):

```sh
ssh -N -L 3302:127.0.0.1:3002 ipat-lab
# Mac browser: http://127.0.0.1:3302/lab/device-workbench
```

The user manager previously reported Linger=no; the private DEV
preview is not promised HA after ALL SSH user sessions end. Releasing
this module to actual tenants requires real IdP/MFA, tenant isolation,
secret vault/backup and signed change approval, not a change to
these developer demo flags.

## Commercial release exclusions and confirmed CI

The exact R9.16 application SHA `63c42e3` achieved 4/4 GitHub
independent CI success, run `36427327886`, after correcting the
initial Rust test fixture shadowing failure at previous SHA 201f980.
Actual owner VPS test deployment and separate operator Mac tunnel
passed from that exact application binary.

This source demonstrates genuine local DEV cryptography and a real
private Rust HTTP B public-key reconciliation workflow, NOT an active
commercial central WireGuard service. Required for a later production
site onboarding: independently reviewed site/POP ownership, signed
MFA/maker-checker, real tenant database persistence and isolation,
real per-tenant key vault with separate process identity and encrypted
restore validation, verified endpoint/UDP listener, true route/VLAN
inventory and local site B return-path recovery. IPsec/L2 overlay
remain separate adapters to implement and test. For current DEV-01
C320, the first option remains direct-private if and only if its
management last hop is independently proven secure, plus trusted
OLT console fingerprint and a restricted verified on-device account.

## CI accounting external blocker — do not mislabel as application failure

The last independent app-source GitHub CI run `36427327886` for exact
SHA `63c42e3` was 4/4 successful. Subsequent docs-only/versioned
harness SHA `11bf48b` workflow `36428063835` did not start ANY job
steps: all GitHub check-run annotations reported account billing or
Actions spending-limit restrictions. This is not a code-test result
and the newest commit has NOT passed hosted CI. Separately, actual
owner VPS repeated pinned full locked control-api Rust test suite:
40/40 passed; `cargo fmt --all -- --check` also passed. Current owner
VPS nonroot :3002 and old :3000 responded normally. PR #117 must
remain draft until account billing is restored and new full CI passes.
The developer-only operational proof must never be promoted to
production merely because local tests pass.
