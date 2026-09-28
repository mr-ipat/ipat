# R9.13 — DEV-01 live-C320 legacy SSH preparation and actual VPS routing evidence

Date: 2026-09-28. This milestone prepares, but DOES NOT EXECUTE,
authenticated physical adoption. The owner reports DEV-01 is LIVE
distribution and requests no disturbance to OLT, ONTs or router.

## Executed on the actual IPAT VPS (no network packets)

`ip -j -4 route get` inspected the exact owner-supplied private
candidate using the existing route table, alongside the local default
route. Actual result: `DEFAULT_ROUTE_ONLY`; there is NO independently
proven dedicated management VPN route from the present VPS worker.
The local route-table command sends no packets to the OLT and cannot
prove whether other off-path NAT/firewall changes would make it work.
Its safe sanitized report is stored as a historical observation only;
all dashboards retain DEVICE NOT ADOPTED and HEALTH NOT MEASURED.

Reproduce from restricted approved nonroot host:

```sh
python3 deploy/scripts/lab/r913/verify_restricted_private_route.py \
  --private-ipv4 EXACT_OWNER_APPROVED_PRIVATE_IPV4
```

## Implemented offline strict legacy candidate (not live login)

`deploy/scripts/lab/r79/c320-ssh-readonly.py` now supports a separate
explicit `ssh-strict-pinned-publickey-legacy-rsa-cbc` transport profile.
Only in this profile will one process use exact ssh-rsa host-key
algorithm and aes128-cbc cipher in its isolated SSH invocation. It
preserves StrictHostKeyChecking=yes, an exact PRIVATE owner-only
known_hosts pin, NO password prompts, NO ssh-agent, NO proxy, and
only the original two fixed candidate read-only commands.

The legacy profile rejects a non-RSA pinned key and the live-read
validation rejects factory/privileged usernames; the owner-shared
temporary default account remains unsuitable for the first live
production-distribution read. Do NOT assume firmware supports
noninteractive public-key SSH without a separate independent check.
Each prereq must still be independently demonstrated; a boolean
in the operator packet alone cannot prove site isolation or MFA.
No collector flag or env opt-in was used in this milestone.

## Required acceptance before one physical read

SITE-02: independently verify fingerprint on actual trusted console,
independently prove management path isolation and last-hop trust;
SITE-03: actual dedicated VPN/private worker route and strict ingress;
SITE-04: exact device-only restricted public-key account supported
by firmware with fixed read-only commands; SITE-05: genuine live
signed MFA, independent approval, per-device lock, baseline/abort.
Only then may one bounded read-only session be considered.
No physical device adoption, changes, firmware or customer impact
measurements occurred in R9.13. Existing CLI parser mock tests are
NOT physical ZTE compatibility certification.

## R9.13 one-command first-read plan (only AFTER all real gates)

The original collector also now accepts `--first-read PRIVATE_PACKET
--out EMPTY_OWNER_0700_DIR` for ONE strictly pinned SSH `show card`
invocation instead of its historical two-command offline pattern.
The output stays raw, owner-only 0600, not accepted into production
inventory before redaction and actual firmware/identity review. It
STILL requires `IPAT_R79_OPERATOR_APPROVES_REMOTE_READ=YES`, the
preexisting exact owner-only packet, dedicated nondefault publickey
identity and every prerequisite independently proven, not merely
checked by declaring booleans true. This command was NOT EXECUTED
against live DEV-01 in this milestone. Do not attempt while the
actual worker still has DEFAULT_ROUTE_ONLY or no trusted RSA pin.

## Evidence-backed physical candidate appears on the private dashboard

The LAB-only Device Manager now displays a separate, clearly marked
historical physical-observation DEV-01 row if and ONLY if the server's
strict historical evidence validates: owner reported ZTE C320,
2026-09-28 no-auth private SSH banner observed, host identity still
UNVERIFIED, POP UNKNOWN, actual VPS route DEFAULT_ROUTE_ONLY,
zero credentials/commands, no worker dispatch and no adoption.
It is not inserted into tenant SQL, is never included in the demo
candidate count, cannot be deleted or used for configuration actions,
and always displays unknown connectivity / unmeasured health.
If evidence is missing or overclaims safety the row disappears;
no target address or credentials are rendered. Real signed tenant
assignment and independent physical identity remain gated.
