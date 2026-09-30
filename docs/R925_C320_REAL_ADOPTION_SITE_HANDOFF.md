# R9.25 — Physical ZTE C320 first authenticated read: precise site handoff

**Actual state:** DEV-01 is still `OBSERVED_NOT_ADOPTED`. The authorized
owner VPS has already verified a working SSH TRANSPORT combination:
`HostKeyAlgorithms=ssh-rsa`, `Ciphers=aes128-cbc`,
`KexAlgorithms=diffie-hellman-group14-sha256` on the existing PRIVATE
management route. The same server advertised `password` as the only
SSH authentication method for the OWNER-REPORTED TEST ACCOUNT during
one bounded NO-CREDENTIAL test. This is NOT a successful login and
not evidence that a dedicated restricted account is available.

## Evidence verified versus still missing

Verified: server SSH banner, legacy algorithm offers, actual successful
SSH RSA host-key exchange, method advertisement, stable network-only
RSA fingerprint across the owner Mac and VPS; private :3002 Rust
readiness dashboard and hosted synthetic CI tested separately.
**None of those network observations independently proves physical
chassis ownership.** The authorized owner's local IPAT private intake
currently contains only `plan.json`; independently sourced chassis
RSA public key, approved dedicated restricted account, actual device
firmware and actual pre/post subscriber baseline have NOT been supplied.

## Required technician action: ONE independently authenticated console visit

Through an owner-authenticated **physical console / separately trusted
vendor management console** for the real DEV-01 chassis, confirm
chassis identity, actual firmware, security role and service baseline.
If that firmware supports the read-only commands in the current safe
operator mode, run individually:

```text
show ssh
show card
show version-running
```

The first should be recorded even when SSH already works. The latter
two should be read only in modes verified to be safe for the actual
firmware, under the agreed low-impact change window. They are
**site-operator instructions, NOT claims that IPAT executed them**.
Do NOT paste entire running configuration, password hashes, subscriber
serial lists or private keys into GitHub/ChatGPT; retain real captures
inside a protected 0700 owner folder with 0600 files and provide only
redacted allowed fields and SHA-256 of original private evidence.

Separately obtain or independently attest the true chassis RSA
PUBLIC host key through that trusted console or independently signed
asset inventory. If firmware has no safe RSA export command, a trusted
on-site technician must independently verify the network endpoint
against the device's own management identity using a separate approved
management channel. Do not create an invented RSA key, copy a
network-only `ssh-keyscan` result into a fake console file or mark
`out_of_band_host_key_verified=true` merely because two network
clients observed the same server fingerprint.

The on-site operator must establish a DEDICATED minimally privileged
read-only account, confirm its AUTHENTICATION TYPE on the REAL
firmware, and independently verify that it cannot modify VLAN/PON,
provision/reboot, write SSH config or initiate firmware updates.
The existing default test login is not a proven least-privilege
commercial collector credential. Restrict remote source to the
approved private IPAT gateway, measure live CPU/alarms/ONT/PPPoE
baseline, and preserve working local console recovery.

## After proof is available: first restricted SSH login

With a genuine console RSA key, prepare the locked exact-target pin
using the already-tested `deploy/scripts/lab/r920/prepare_pin.py`.
For a restricted account **actually confirmed to offer `password`**,
a site-authorized owner may open ONE attended interactive terminal:

```sh
ssh -F /dev/null -tt -p 321 \
  -o HostKeyAlgorithms=ssh-rsa \
  -o Ciphers=aes128-cbc \
  -o KexAlgorithms=diffie-hellman-group14-sha256 \
  -o StrictHostKeyChecking=yes \
  -o UserKnownHostsFile="$HOME/private-c320-pin/known_hosts" \
  -o GlobalKnownHostsFile=/dev/null \
  -o PreferredAuthentications=password \
  -o PubkeyAuthentication=no \
  -o KbdInteractiveAuthentication=no \
  -o IdentityAgent=none \
  -o NumberOfPasswordPrompts=1 \
  -o ConnectionAttempts=1 \
  -o ConnectTimeout=8 \
  RESTRICTED_READONLY_OPERATOR@PRIVATE_MANAGEMENT_IP
```

Do NOT type or store the password before the OpenSSH prompt on the
verified host. Run only one authorized `show card` after checking
operator privilege, session audit and live baseline. Stop on
privilege escalation, unknown prompt, chassis mismatch, fault alarm,
subscriber disruption or unsupported firmware syntax. The
`r79/c320-ssh-readonly.py` current collector is KEY-ONLY and MUST NOT
be used against the observed default password-only account; it is
still explicitly locked to isolated-lab policy, not approved live
production worker.

After the owner-only first capture, normalize strictly through the
existing Rust `olt-evidence` command using 0700 input/output folders
and 0600 files, review exact cards/firmware with the independent
site reviewer, then implement and validate a genuine signed-MFA
multi-tenant production worker. No LAB readiness flag, manual CLI
snapshot or synthetic public CI result can independently mark
`ADOPTED_READ_ONLY` without that real authorization and audit chain.

## R9.25 correction to vendor SSH troubleshooting

Historical ZTE C320 manuals present SSHv2 `show ssh` examples with
`SSH init server key` values `not initialized` **and** `disable`.
Those labels alone do not show that a functioning SSHv2 server has
no RSA host key. Actual owner VPS already received an RSA host-key
packet from this target through group14-SHA256. R9.25 changes offline
`show ssh` triage to `SSHV2_HOST_KEY_INITIALIZATION_FIELD_AMBIGUOUS`
for both reported strings and suppresses even an indirect server
key-generation recommendation. Do not run `ssh server generate-key`,
change SSH version, reboot or reset live service solely on a historic
example. Any proposed real server config change requires an observed
real firmware-specific fault plus independently approved recovery.

**Physical commands IPAT executed in R9.25: 0.**
**Actual authenticated device CLI captures: 0.**
**OLT SSH server/config/ONT/firmware changes: 0.**
