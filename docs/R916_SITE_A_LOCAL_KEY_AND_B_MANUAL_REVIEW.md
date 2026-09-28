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
