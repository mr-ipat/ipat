# R9.22 — Physical DEV-01 SSH compatibility breakthrough (29 September 2026)

Actual owner-approved credential-free single-shot tests on the nonroot
IPAT VPS to the previously supplied PRIVATE ZTE C320 candidate SSH
port definitively separate TWO SSH compatibility problems:

1. Default modern OpenSSH initially rejected the old OLT host-key
   algorithms offered: `ssh-rsa,ssh-dss`, as owner also reproduced.
   OpenSSH's own legacy-compatibility guidance documents opt-in
   `HostKeyAlgorithms=+ssh-rsa` **for that destination only**.
2. On actual owner VPS, after opting into `ssh-rsa` ONLY, OpenSSH
   rejected all default ciphers. The server actually offered
   `aes128-cbc,3des-cbc,blowfish-cbc`; it advertised a negotiated
   key exchange `diffie-hellman-group16-sha512`. Temporarily
   permitting the least expansion needed from this offered list,
   `aes128-cbc`, fixed cipher negotiation, but the server then timed
   out BEFORE sending its RSA host-key packet / SSH NEWKEYS.
3. An independent, bounded and **credential-free** attempt setting
   `KexAlgorithms=diffie-hellman-group14-sha256`, alongside
   `HostKeyAlgorithms=ssh-rsa` and `Ciphers=aes128-cbc`, succeeded
   in selecting group14-SHA256 and reached the **actual server RSA
   host-key and SSH AUTHENTICATION-METHODS STAGE without timeout**.
   Server acceptance of `diffie-hellman-group14-sha256` was actually
   observed; the earlier group16 timeout is a likely compatibility
   or implementation issue, NOT proof of an OLT hardware fault.
   No password, SSH private client key, OLT CLI command, or device
   configuration was sent in ANY of these noauth compatibility
   checks. Observing a network host key is NOT trusted source identity.

**Operational conclusion:** client-only, explicit, exact-device
LEGACY compatibility profile works through server key exchange.
Do NOT apply global `/etc/ssh/ssh_config` changes, allow `ssh-dss`,
force public Telnet, disable host-key checking, repeatedly retry
passwords or initialize/change live OLT keys. Direct private network
remains the preferred method; WireGuard is NOT required.

New R7.9 transport profile:

`ssh-strict-pinned-publickey-legacy-rsa-cbc-group14-sha256`

It adds exactly process-scoped `HostKeyAlgorithms=ssh-rsa`,
`Ciphers=aes128-cbc`, `KexAlgorithms=diffie-hellman-group14-sha256`
to the existing private C320 `ssh` command, keeps
`StrictHostKeyChecking=yes`, a device+port-specific independently
trusted `known_hosts`, `PasswordAuthentication=no`,
`PubkeyAuthentication=publickey`, agent disabled, one bounded
`show card` first read and a non-factory restricted account.
No credentialed production command becomes enabled from this
software-only negotiation success.

## Distinct remaining actual device admission gates

The next step is NOT further cipher guessing. Independently acquire
physical DEV-01 RSA **public** host key from a trusted device console
or authenticated owner inventory and match it offline using the
R9.20 private `prepare_pin.py` tool. An owner-provided network scan
of the same port alone is insufficient. Also provision and prove a
DEDICATED read-only SSH account supported by this exact firmware,
including real key-based auth and denial of write commands;
independently verify the source POP management ACL/last-hop and
initial live customer/alarm baseline; establish true tenant
OIDC/MFA and separate reviewer approval. Only then use the
fixed-output R7.9 first `show card` read, normalize locally via
Rust `olt-evidence`, independently review exact chassis/board/
firmware output and enable only verified read-only actions through
a dedicated tenant-scoped production worker.

Do NOT pretend the owner-shared factory privileged test account
meets a restricted read-only production collector's requirements.
Any change to device-side crypto/roles or high-risk ONT/firmware
operations needs owner maintenance, rollback and pre/post baseline.

For operator diagnostics, OpenSSH's official compatibility advice:
https://www.openssh.org/legacy.html. Compatibility is an interim
solution; upgrade or reconfigure obsolete crypto when vendor and
real network maintenance acceptance allow it.

## Actual restricted owner VPS R9.22 deployment proof

Exact protected application source SHA
`03dbdc3be00f17fd5072603b9d41588cddd210e6` was copied into a
SEPARATE 0700 nonroot owner-VPS source/target directory with a
verified Git delta and no edits to the original service code.
On the actual owner VPS, `cargo fmt --all -- --check`, combined
Python R7.9 safety suite 26/26, R9.0/R9.21 suite 54/54, full pinned
locked offline `cargo test -p control-api` and `cargo build` PASSED.
Binary SHA256:
`a1b35218a26c88cdb49302ebdc3b1a7c8badca78ad77d74a508800e44ba37302`.

Opt-in nonroot protected PRIVATE loopback `:3002` R9.22 preview
actually upgraded successfully from previous R9.19 private user
service. The new reviewed unit SHA256 is
`1ffede0dda1de59be03ac47fd978167a14ed80b8b4ffa708a45440ff84374c58`;
actual bound and no-secrets HTTP smoke SHA256 is
`1ffa9c9a6e724d0e2bfd92714ea96409d5d23ad53bb06d1a3c6d05ee54800d47`;
checksum-pinned rollback/deploy script SHA256
`05e45271ea0b047bcfaffd46434361f57ebf05d64b22886e0817921055048323`.
Actual PRIVATE Rust HTTP evidence reports observed group14-SHA256,
server RSA host-key packet, SSH auth methods reached, and ZERO
actual credentials/OLT commands, OOB identity FALSE, device adopted
FALSE. All eight physical Device Manager actions remain locked:
actual POST for even `READ_CARD_INVENTORY` and firmware mutation
returns HTTP403. The older original `127.0.0.1:3000/healthz`
returned HTTP200 unchanged; no firewall, route, K3s, live OLT or
RouterOS configuration was changed. Prior private user unit was
stored owner-only at
`/home/openai/.cache/ipat/r922-release/rollback-user-unit.service`.

Restricted nonroot operator rollback if private preview is unhealthy:

```sh
ssh ipat-lab
cp -p /home/openai/.cache/ipat/r922-release/rollback-user-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
```

Actual R9.22 physical result and exact release UI are LAB/private
until real independent console key, dedicated least-privilege account,
signed tenant admin MFA, POP isolation and live baseline permit a
first authenticated read on an active subscriber distribution OLT.

The repeat single bounded NO-CREDENTIAL group14-SHA256 owner-VPS
handshake again reached SSH AUTH methods. A temporary, isolated
NETWORK-ONLY known_hosts fingerprint was consistent with the prior
independent owner-Mac/VPS network measurements; the temporary file
was destroyed, never accepted as actual verified device provenance.
Public synthetic GitHub Actions run `36507915781` independently
passed BOTH 2/2 jobs at exact sanitized snapshot `3454f57`.
This completes transport compatibility integration and public
synthetic regression, **NOT genuine physical login or adoption**.
