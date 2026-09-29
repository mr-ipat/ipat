# R9.26: Physical C320 acceptance package, NO false adoption

## Actually inspected owner devices (29 September 2026)

On owner Mac the canonical existing private directory
`~/.local/share/ipat/c320-private-packet` is present but contains
**only `plan.json`**; there is no actual trusted chassis RSA console
public key, no real `show ssh`, no first authenticated `show card` or
`show version-running` capture. No independent approved dedicated
read-only operator or tenant production worker has been proven.
The owner VPS nonroot physical proof folder had no evidence files;
its current private lab readiness GET remained HTTP200. Do not change
the physical device's adoption state from `OBSERVED_NOT_ADOPTED`.

## One exact command to assess the user's OWN available evidence

```sh
cd ~/Projects/ipat-current
python3 deploy/scripts/lab/r926/assess_site_packet.py \
  --private-owner-folder "$HOME/.local/share/ipat/c320-private-packet"
```

The command is entirely local: no SSH, no credentials, no OLT packets,
no writes and no device action. It prints ONLY evidence SHA256,
nonsecret recognized status flags and missing requirements, never
actual raw customer CLI, password or RSA key bytes. No owner supplied
file can set production `device_adopted=true`, even if all captures
and hashes pass local syntax checks. Actual independence of the
console source and tenant owner/reviewer must be confirmed through
separate signed policy/asset verification.

## Required PRIVATE on-site evidence, once, not repeated SSH probing

Authorized field technician at the actual chassis or through an
**independently authenticated** OLT console/asset source must capture:

- `console-host-rsa.pub`: genuine chassis RSA PUBLIC host key obtained
  independent of its existing management SSH path, single ASCII line.
- `show-ssh.txt`: only the actual firmware-supported safe `show ssh`
  status (not the full running config). Do not include credential lines.
- `cards.txt`: first approved, firmware-supported and role-restricted
  `show card` read. Its exact vendor parser compatibility is still
  not verified on this production chassis.
- `versions.txt`: a separately authorized `show version-running`
  only after first card output and true firmware CLI support are
  independently checked.

All files must be kept **outside Git** in the current owner's 0700
private folder and individually mode 0600. Do not send passwords,
private SSH keys or customer serials to ChatGPT or PUBLIC mirror.
The offline RSA check and first trusted known_hosts pin flow remains
`deploy/scripts/lab/r920/prepare_pin.py`; the actual physical C320
password-only offer for the previously tested account does not prove
that a new restricted account can authenticate successfully. The
existing key-only collector is NOT authorized for that account.

After getting the first REAL capture, run the already built, nonroot
owner-private Rust `olt-evidence` offline parser from the actual
owner-VPS build or pinned source; it refuses mismatched card/version
layouts, symlinks/weak file permissions and leaves all physical
adoption/security flags FALSE until independent review. If the
vendor's real output differs, revise the adapter only against that
privately retained firmware capture (redacted fixture for tests).

## Acceptance conditions required to mark operationally ADOPTED

The console public key is independently verified against actual
chassis identity; source management POP isolation/ACL tested;
firmware and dedicated non-factory least-privilege role verified;
actual one-command read and evidence parser successful; true OIDC
MFA Tenant Admin and distinct site reviewer have approved; exact
POP-scoped non-demo auditable worker and bounded job path have
independent negative tenant/policy tests; measured before/after live
subscriber impact is acceptable. A local capture package alone
cannot authorize production writes, alarms, ONT discovery, reboot,
PON provisioning or firmware upgrade.

## Actual action ledger

- **Actual management SSH transport**: previous authorized no-password
  VPS negotiation succeeded with RSA/AES128-CBC/group14-SHA256 and
  advertised `password` for one test account. Host key is NETWORK
  OBSERVATION ONLY, not physical provenance.
- **Actual authenticated OLT login**: NONE.
- **Actual CLI run on physical OLT by IPAT**: NONE.
- **Actual device-side configuration changes / firmware writes**:
  NONE.
- **R9.26 owner Mac offline audit**: `plan.json` only, eight clearly
  classified real evidence and production authorization gaps.
- **R9.26 implemented endpoint**: separate owner-only CLI audit, not
  a production device operator and not a live device worker.
