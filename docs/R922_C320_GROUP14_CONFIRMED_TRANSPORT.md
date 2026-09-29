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
