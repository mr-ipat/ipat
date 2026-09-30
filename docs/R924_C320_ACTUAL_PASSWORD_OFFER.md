# R9.24 — Actual physical C320 authentication-method discovery

**Verified new evidence, 29 September 2026:** In ONE bounded
credential-free nonroot IPAT VPS SSH negotiation targeting the same
owner-authorized private C320, the existing exact-device
`ssh-rsa`/`aes128-cbc`/`diffie-hellman-group14-sha256` compatibility
again reached real server host-key and SSH user-authentication
negotiation. OpenSSH's actual server method advertisement for the
owner-reported test username was **`password` only** in that
session. No password, SSH client private key or OLT command was
submitted. This observation applies to that username and server
configuration at that time; it does not establish whether another
username or a reconfigured actual firmware might allow public key.

**Important architectural correction:** existing R7.9 collector
`ssh-strict-pinned-publickey-legacy-rsa-cbc-group14-sha256` intentionally
requires `PreferredAuthentications=publickey` and disables passwords.
It therefore CANNOT be assumed to authenticate as the observed test
user with the current actual `password`-only offer. Do not incorrectly
report this as the C320 rejecting a password, as none was provided,
or as evidence that every account supports only password forever.

## Narrow route to a real safe first read

1. Real on-site operator obtains true current C320 host RSA PUBLIC key
   from an independently authenticated chassis console. Compare it
   with prior network key and create private exact-target known_hosts
   via `deploy/scripts/lab/r920/prepare_pin.py`; same-network TOFU or
   keyscan alone is insufficient. The authorized owner Mac private
   C320 intake folder contains ONLY `plan.json`, with no verified
   OOB RSA pin or approved restricted account.
2. Using trusted console and actual firmware documentation, establish
   a DEDICATED minimally privileged C320 read-only operator account
   and independently test inability to configure, reboot or perform
   provisioning. Determine whether that account actually offers
   `publickey` OR `password`. Do not change the active SSH daemon or
   assume key-based support from generic manuals.
3. If its actual account only offers password, the *first* approved
   read must be a site-operator attended, interactive SSH session
   with password entered only at the verified OpenSSH prompt, never
   the existing key-only collector, Git, logs, CLI args or plaintext
   automations. Do not use the shared default privileged test user
   as a long-running commercial collector. Credentialed operations
   also require site console recovery, real POP isolation, measured
   pre/post customer-impact baseline, true signed tenant MFA and
   separately authorized reviewer, not merely a shell flag.
4. Restrict the initial command to approved exact-firmware `show card`
   only after verified account privilege and source trust. Capture
   private, owner-only evidence and normalize with Rust `olt-evidence`.
   Review actual slot/firmware compatibility before the second
   `show version-running` or any alarm/ONT diagnostics. Full
   automated password-capable worker requires an isolated credential
   vault, scoped rotated credential, tenant-protected dispatch and
   independent immutable audit; such production worker is NOT built
   or authorized by R9.24.

Current **physical OLT configuration commands executed = 0;
authenticated OLT read commands = 0; credentials sent = 0**.
Only actual client-side compatibility and noauth method discovery
have been carried out. The condition-based R9.23 *UNEXECUTED*
configuration proposals should not be applied because no actual
server-side defect was demonstrated by the successful handshake.

A small offline `deploy/scripts/lab/r924/auth_capability.py` now
models real authentication-method evidence independent of device
identity, checks password-vs-key-only compatibility explicitly and
never opens a socket or enables adoption. The private Rust lab
capability GET similarly advertises the actual user-specific
password-only result, with real login/worker/adoption FALSE and
hardware action HTTP POST 403. This is not proof of actual login.

## Actual private runtime verification

Protected exact application source SHA
`c21ff86f0abb3b7ab845ad2aa962798eacfc1c71` was independently
cloned via verified Git bundle into a separate nonroot owner-VPS
folder. Mac and actual VPS combined safety/unit suite 62/62 PASS;
actual VPS Rust fmt, full `cargo test --locked --offline -p control-api`
and locked offline build PASS. Built binary SHA256
`b894db4b79abcbf8befbaf62625b8e4ec49145699f79c5046bd466b02bf2d0e5`.
A checksum-pinned opt-in nonroot private user service upgrade of
ONLY `127.0.0.1:3002` successfully deployed the observed test
username `password`-only server offer as Rust readiness metadata;
actual HTTP smoke PASSED, all eight physical action POSTs still
return HTTP403, and original `127.0.0.1:3000/healthz` remained
HTTP200. No real OLT password, CLI, SSH server configuration,
firmware or ONT state was touched by the deployment. The former
private user service is backed up under
`/home/openai/.cache/ipat/r924-release/rollback-user-unit.service`.
R9.24 deployment script
`deploy/scripts/lab/r924/deploy_private_auth_method_preview.sh`
SHA256 `2a888899f44faab732b768b4280b1e998f1c4d0f3c4a4a3257c8b346ecaad9d7`;
unit SHA256 `7e51794041be571536562f7e7f407d5f9958b0fce21906a5aa4a3b88935c0b75`;
actual HTTP smoke SHA256
`f4cb63bbd28c824fb0b2f701d74084a0403acfe3e44840ec744913f54ef68b14`.

Rollback only the PRIVATE owner user service if needed:

```sh
ssh ipat-lab
cp -p /home/openai/.cache/ipat/r924-release/rollback-user-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
```
