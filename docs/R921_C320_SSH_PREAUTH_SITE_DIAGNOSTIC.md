# R9.21 — Narrow ZTE C320 trusted-console SSH settings triage

Status: ACTUAL owner Mac and authorized VPS both previously reached
TCP and SSH2 KEX selection through the existing DIRECT PRIVATE link,
then stalled before obtaining a host key / NEWKEYS. No real OLT
password, host console transcript or model/firmware has been read.
The actual SSH daemon state remains UNKNOWN.

A *historical* ZXA10 C320 CLI configuration guide documents the
read-only `show ssh` diagnostic, with fields including `SSH version`,
`SSH enable-flag configuration` and `SSH init server key`. Its
**example** displays `SSH init server key : not initialized`, which
is a useful diagnostic hypothesis but is NOT evidence of the user's
physical C320 configuration. Source:
https://pdfcoffee.com/zxa10-c320-configuration-manual-cli-pdf-free.html
A different actual firmware may vary in syntax or settings.

The offline `deploy/scripts/lab/r921/inspect_show_ssh.py` consumes
ONLY an owner-asserted independently trusted, small 0600 `show ssh`
transcript inside a 0700 owner-only folder outside Git. It checks
strict recognized settings, refuses malformed/duplicate values and
texts containing credential assignments, then prints a bounded
secret-free SSH diagnostic and evidence SHA256. It does not issue the
`show ssh` command; no management network request, login, restart,
server-key generation, password exchange, device configuration or
adoption is supported. Any reported status remains unverified until
the capture source and chassis identity are separately authenticated.

## Exact next on-site diagnostic

A site operator with independently authenticated OLT console access
should, **without changing any setting**, first check the live
baseline/alarms and the firmware-specific availability of `show ssh`.
If the command is supported and approved in its existing safe CLI
context, capture ONLY its output into a 0600 private file outside Git.
Do NOT capture running-config, actual passwords, user hashes or
configuration commands. Never post the raw transcript publicly.

```sh
# Owner nonroot Mac or authorized private VPS with a real trusted console capture:
mkdir -m 700 "$HOME/private-c320-ssh-status"
chmod 600 "$HOME/private-c320-ssh-status/show-ssh.txt"
python3 deploy/scripts/lab/r921/inspect_show_ssh.py \
  --trusted-console-show-ssh-output "$HOME/private-c320-ssh-status/show-ssh.txt" \
  --owner-attests-independent-console-source
```

Interpretation: `not initialized` INDICATES a possible site-side SSH
host-key issue requiring independent operational review, not proof
of the KEX failure cause. `initialized` similarly does not certify
the RSA host key, ACL, SSH KEX implementation or login privileges.
If `show ssh` is not available for the actual firmware or current
least-privilege console role, stop and use authenticated vendor
support instead. Never auto-enable SSH, initialize/change server
keys or retry the factory password on the live distribution OLT.

If site evidence is independently reviewed and shows a properly
initialized host key, export its RSA **public** material from the
trusted chassis console using the exact firmware's supported method;
`r920/prepare_pin.py` then checks it against the prior network
observation and creates an owner-only exact-target known_hosts pin.
Even that comparison does not activate remote reads without real
MFA, least-privileged account, last-hop isolation and service
baseline. See `docs/SOP_ZTE_C320_READONLY_ADOPTION.md`.
