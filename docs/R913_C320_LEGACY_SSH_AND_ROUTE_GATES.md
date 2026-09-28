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

## Physical no-auth connectivity via ephemeral owner Mac SSH relay

Actual, separately bounded owner Mac test also successfully established
an OWNER-ONLY SSH encrypted reverse AF_UNIX socket (inside a 0700
nonroot VPS folder), forwarding precisely ONE private SSH TCP endpoint.
From the actual VPS, a single read-only socket client RECEIVED 20
inbound bytes containing the same UNTRUSTED `ZTE_SSH.1.0` banner.
ZERO credentials, ZERO outbound SSH commands, ZERO OLT config changes.
The relay was closed and its remote socket removed and independently
verified absent. It is NOT a durable VPN, site-managed tunnel,
verified management LAN or approval to log into the distribution OLT.

The reproducible guarded helper is
`deploy/scripts/lab/r913/temporary_mac_relay_noauth.py`. It defaults
to NO action, only accepts exact RFC1918 IPv4 and port, requires
nonroot operator and explicit per-run opt-in, checks a 0700 owner-only
remote Unix socket parent, relies on existing pinned VPS SSH trust,
and destroys the randomly named socket after precisely one inbound
banner observation. It never stores credentials or enables any OLT
commands. Its five mock tests cover strict SSH/reverse-socket options,
public address denial, no opt-in denial, single read/cleanup and
fail-closed cleanup failure. Actual bounded script test PASS on the
owner Mac/VPS with repeat 20-byte untrusted banner, zero credentials,
and verified socket cleanup. Do NOT run this as continuous monitoring.

Operator-only one-time reproduction, ONLY for explicitly authorized
transport tests and never for credentials:

```sh
IPAT_R913_APPROVE_ONESHOT_PRIVATE_SSH_RELAY=YES \
  python3 deploy/scripts/lab/r913/temporary_mac_relay_noauth.py \
    --probe --private-ipv4 OWNER_APPROVED_PRIVATE_IP --port APPROVED_SSH_PORT
```

The dashboard now documents the historical test and its closure,
while still reporting the actual long-lived VPS worker path as
DEFAULT_ROUTE_ONLY, last-hop trust FALSE, physical adoption FALSE.
For actual onboarding use a separately approved durable site tunnel,
independently pinned OLT key and dedicated read-only account.

## Actual owner VPS R9.13 private physical-pending dashboard rollout

The reviewed final R9.13 source SHA `0d0af601ff2e0cea1723bf3caaf32cbbd3195679`
was transferred as a verified SHA-256 Git delta bundle into a separate
nonroot VPS checkout, WITHOUT modifying canonical VPS main.
Its source-only delta bundle SHA256 is
`eb89a9d14e64a832a12b4018806c4d78a257c54f600366c9dbc6355504e8c99d`.
Pinned low-priority locked offline build produced binary SHA256
`672caf4e756853d0226e93b7cf712f8d229af170cc7450a348ae94b9ee86aeac`.
The R9.13 nonroot reviewed user unit is identical in safety controls
to previous R9.12, but points only to the separately compiled R9.13
binary; unit SHA256 is
`2f9d254675c9080b94287035a0722e1e3c7b41f233fce679bfd50b11b52d72c1`.
The prior running unit SHA was independently checked and backed up
owner-only at `/home/openai/.cache/ipat/r913-preview/rollback-unit.service`.

Actual on-VPS constrained loopback `127.0.0.1:3002` service restart and
versioned HTTP smoke PASS: R9.12 synthetic tunnel POST remains safely
BLOCKED, historical physical observation shows DEV-01 pending and
owner-Mac temporary relay OBSERVED AND CLOSED, fake real business API
HTTP401, original :3000 HTTP200 throughout. A separate fresh Mac SSH
local-forward loaded actual dashboard HTML/JS and read the matching
historical physical evidence without any OLT network operation.
Only owner nonroot systemd user service changed; no SSH root policy,
firewall, K3s, permanent VPN, customer router or OLT/ONT config changed.
The user manager still has `Linger=no`; no HA/runtime-outliving-user
claim applies to this private LAB preview. Operational rollback:

```sh
ssh ipat-lab
cp -p /home/openai/.cache/ipat/r913-preview/rollback-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user restart ipat-r911-preview.service
```

The next actual stage requires trusted out-of-band RSA fingerprint,
restricted public-key account, explicit isolated site route and
independently approved traffic/ONT baseline; neither optional legacy
SSH compatibility nor ephemeral relay constitutes physical adoption.
