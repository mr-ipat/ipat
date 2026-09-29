# R9.23 ZTE C320 — actual configuration ledger and controlled site change

Date: 29 September 2026. Target: owner-reported live subscriber-serving
ZTE C320 DEV-01. Current physical state is **OBSERVED_NOT_ADOPTED**.

## Every actual physical-device operation as of R9.23

**ACTUAL OLT CONFIGURATION COMMANDS EXECUTED: ZERO.**
**ACTUAL AUTHENTICATED OLT READ COMMANDS EXECUTED: ZERO.**
**ACTUAL PASSWORDS/SSH PRIVATE CLIENT KEYS SENT BY IPAT: ZERO.**
**ACTUAL OLT FIRMWARE/VLAN/PON/ONT/REBOOT CHANGES: ZERO.**

The owner reported a plain OpenSSH error: host key algorithms offered
`ssh-rsa,ssh-dss`. A bounded authorized owner-VPS credential-free SSH
comparison obtained real server offers `aes128-cbc,3des-cbc,blowfish-cbc`.
RSA+aes128-CBC and an initially selected group16-SHA512 KEX still
stalled before the host key. Changing ONLY the local SSH client KEX
choice to `diffie-hellman-group14-sha256` plus the selected RSA/CBC
options successfully obtained the network server RSA host key and
entered the SSH authentication-method stage. A bounded network-only
continuity check matched the previously observed owner Mac/VPS RSA
fingerprint. The matching fingerprint is STILL NOT an independently
trusted physical chassis-console identity; all temporary network
pins were discarded. No OLT configuration has been touched.

## Commands actually used in testing

All SSH probes have **no password or authorized login**, and use
strictly bounded process-local options on the authorized private
VPS. Their diagnostic purpose was equivalent to the following
representative command shape (the implemented probe additionally
uses explicit no-agent/isolated temporary known_hosts settings):

```sh
ssh -F /dev/null -n -T -p <PRIVATE_OLT_SSH_PORT> \
  -o HostKeyAlgorithms=ssh-rsa \
  -o Ciphers=aes128-cbc \
  -o KexAlgorithms=diffie-hellman-group14-sha256 \
  -o PreferredAuthentications=none \
  -o PasswordAuthentication=no \
  -o PubkeyAuthentication=no \
  -o KbdInteractiveAuthentication=no \
  -o ConnectTimeout=6 \
  -o ConnectionAttempts=1 \
  -o IdentityAgent=none \
  -o GlobalKnownHostsFile=/dev/null \
  -o UserKnownHostsFile=<EMPTY_TEMP_NETWORK_OBSERVATION_ONLY> \
  <CANDIDATE_USERNAME>@<PRIVATE_OLT_IP>
```

`StrictHostKeyChecking=accept-new` was used ONLY in the isolated
no-credential TEMP network-observation session; this does NOT satisfy
the production collector, which instead mandates `yes` with an
independently trusted chassis-public RSA key. No global system SSH
policy change was installed.

## First safe site-console read and owner-approved change policy

The current next site operation is **read only** `show ssh` if its
actual firmware/current privilege mode supports it. Owner should
use a separately authenticated local chassis console, record actual
firmware, service CPU/alarm/ONT/PPPoE baseline and verify the console
recovery path. Store ONLY the bounded `show ssh` status in an owner
0700 directory/0600 file outside Git. No credential or full running
configuration should be sent through public chat or CI.

R9.23 `deploy/scripts/lab/r923/plan_ssh_remediation.py` takes that
actual trusted console status and emits a non-executable JSON decision:

- If the actual SSHv2 server is ENABLED and has a reported usable
  host key: **NO OLT SERVER CONFIG CHANGE**. Use the working strictly
  local group14/RSA/aes128-CBC profile and an independently pinned
  physical RSA key plus a tested restricted read-only account.
- If actual output reports SSH server DISABLED, it MAY propose
  `ssh server enable` as a **NOT EXECUTED**, firmware-conditional
  site-engineer change after an independently approved maintenance
  window and console rollback. This is NOT current device evidence.
- If actual output reports SSHv1, it MAY propose
  `ssh server version 2` under the same safeguards after exact real
  firmware verification. NEVER infer from a historical manual alone.
- If SSHv2 reports an uninitialized key, **DO NOT automatically
  generate or rotate server keys**. Some historical ZTE command
  references specify `ssh server generate-key` only for SSHv1; an
  actual vendor firmware support check and independently reviewed
  maintenance plan are necessary before any device-side change.
  The observed network SSH key exchange previously succeeded on
  group14, so do NOT assume the live device lacks a host key.

There is no generic safe rollback for unverified device firmware,
previously shared services and SSH management dependencies. Capture
PRE-CHANGE running status, ensure secure local console recovery and
approve a firmware-specific inverse action with site operator BEFORE
any change. Abort on changed RSA identity, abnormal CPU/alarms,
subscriber drops, loss of console or nonmatching firmware output.

## Actual FIRST read / adoption gate after independent approval

1. Acquire independently trusted exact physical chassis RSA public
   key (not network keyscan), compare offline and create exact-target
   0600 pinned known_hosts using R9.20.
2. Verify a dedicated **non-factory restricted read-only** SSH
   account/key plus genuine POP-isolated management route, baseline,
   signed tenant MFA and independently authorized first-read change.
3. Use `ssh-strict-pinned-publickey-legacy-rsa-cbc-group14-sha256`
   first-read worker candidate only for **one** bounded `show card`;
   store private 0600 evidence and normalize via `olt-evidence`.
4. After independent reviewer validates actual chassis/firmware,
   authorize `show version-running` only if exact firmware supports
   that command in the lowest restricted operator mode. Disable any
   untested or write operations. Update DEVICE_MATRIX/PROJECT_STATUS
   only when the physical evidence was actually collected.

The R9.22 PRIVATE dashboard reports actual no-credential SSH
transport success but all eight hardware action POSTs still return
HTTP403, with zero device dispatch; device health is NOT_MEASURED.

Operator may preview the **unapplied** local site decision once
trusted console data is actually present:

```sh
python3 deploy/scripts/lab/r923/plan_ssh_remediation.py \
  --trusted-console-show-ssh-output \
    "$HOME/private-c320-ssh-status/show-ssh.txt" \
  --owner-attests-independent-console-source
```

This command sends **zero** packets to the OLT and never changes
SSH settings. No current evidence authorizes running
`ssh server enable`, `ssh server version 2`,
`ssh server generate-key`, any provisioning, reboot or upgrade on
the live distribution chassis.
