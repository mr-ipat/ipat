# R9.20: offline C320 host-key pin handoff from trusted local console

**Not physical adoption.** Both authorized network SSH attempts reached
KEX selection but stalled before the server host-key packet. A prior
network-observed fingerprint is not independent identity verification.
The actual trusted console public RSA host key and on-site evidence
are still missing; no live OLT authentication has been attempted by
this milestone.

The local `deploy/scripts/lab/r920/prepare_pin.py` converts a REAL
operator-supplied trusted-console RSA **PUBLIC** key into a restricted,
exact-target `known_hosts` pin only after the offline OpenSSH
fingerprint matches the stored historical network observation.
It requires nonroot owner-only 0700 input/output folders, a single
0600 non-symlink RSA public-key file outside Git, a separate initially
EMPTY 0700 output folder and explicit owner provenance assertion.
A mismatch leaves NO output and makes ZERO network requests. The
result explicitly preserves `device_adopted=false`,
`physical_read_permitted=false`, `tenant_mfa_verified=false` and
`independent_provenance_cryptographically_verified=false`: an owner
assertion and matching key alone never prove trusted provenance.

## One-time operator console evidence step (not a network scrape)

Export the host RSA **public** key from the actual ZTE chassis
console, using the vendor-approved instructions for the actual
firmware and the owner's independently authenticated console. Do
not send the default credential, a private SSH key or raw management
configuration through chat or the public repository. The tool does
not guess the vendor console command. On the owner's authorized
nonroot Mac or VPS, make two canonical private directories outside
any repository and save the exported *public* key to
`console-host-rsa.pub` with mode 0600. Confirm host origin and the
current alarms/CPU/service baseline independently first.

Example (paths/target supplied by approved site operator):

```sh
mkdir -m 700 "$HOME/private-c320-console" "$HOME/private-c320-pin"
chmod 600 "$HOME/private-c320-console/console-host-rsa.pub"
python3 deploy/scripts/lab/r920/prepare_pin.py \
  --trusted-console-rsa-public-key "$HOME/private-c320-console/console-host-rsa.pub" \
  --empty-private-output "$HOME/private-c320-pin" \
  --private-ipv4 YOUR_PRIVATE_OLT_IP \
  --ssh-port YOUR_APPROVED_SSH_PORT \
  --owner-attests-independent-console-source
```

Do NOT use a key obtained from `ssh-keyscan`, SSH TOFU, the same
untrusted path, or a copied historical lab demo to satisfy this step.
Output `known_hosts` remains 0600 and does not grant permission to
login. The additional gate checks (restricted OLT account, exact
firmware read command, final management-link isolation, live baseline,
true OIDC MFA, independent reviewer and scoped worker) are still
required before the ONE allowed physical `show card` read described
in `docs/SOP_ZTE_C320_READONLY_ADOPTION.md`.

If the real trusted key differs from the earlier observed network key,
STOP: do not install either key, do not provide a password or disable
host verification. Investigate the network path and server identity
through separately authenticated site-console access.
