# R9.18 — read-only C320 evidence action and safe public CI mirror

**No hardware adoption claim.** The requested C320 default account
is a privileged factory account reported by owner on a LIVE
subscriber-distribution OLT. Historic private SSH no-auth banner and
fingerprint are repeatable, but no independent trusted-console key
has been supplied and no dedicated restricted read-only login has
been proven. No firmware/ONT/PPPoE/OLT configuration action has
been executed; do not infer interoperability from synthetic CLI text.

## Release 1: safe offline post-login C320 action normalization

`crates/olt-core/src/bin/olt-evidence.rs` extends the existing strict
Rust C320 CLI parser with a nonroot, entirely local offline operator
binary. It supports only `show card` and optional `show version-running`
**files** already captured by a separately approved/restricted SSH
read. No network, passwords, login attempts, live OLT command,
operator display of raw transcript, firmware changes or adoption.
Raw capture files must be owner-only regular 0600 and reside inside
0700 directories outside Git. Normalized JSON is created exactly
once, exclusively 0600 within a separate 0700 directory outside Git,
with SHA-256 evidence digests, parsed board/versions and explicit
`device_adopted=false`, `network_actions=0` and
`firmware_write_enabled=false`. Parsing failure writes no result.
Version-specific compatibility only becomes eligible for human
review after ACTUAL trusted capture is supplied, then requires
separate real tenant security-admin approval.

Private operator example AFTER real trusted read, with paths chosen
locally and no real IP/password in shell history:

```sh
mkdir -m 700 ~/private-c320-capture ~/private-c320-normalized
chmod 600 ~/private-c320-capture/cards.txt
cargo run --locked --offline -p olt-core --bin olt-evidence -- \
  --cards ~/private-c320-capture/cards.txt \
  --out ~/private-c320-normalized/inventory.json
# Only if independent approved second version read exists:
# --versions ~/private-c320-capture/versions.txt
```

Do NOT place real vendor transcript in chats, Git, logs or the
identity-free lab demo dashboard. Admin factory password is not an
acceptable production collector secret. This is a protective offline
normalization step, not an authorized live device action endpoint.

## GitHub Actions recovery without disclosing real management data

NEW PUBLIC snapshot: https://github.com/mr-ipat/ipat-open-ci
(verified public). It has a clean initial commit with synthetic data,
without historical operational commits/logs. The original
`mr-ipat/ipat` and all its PRs remain PRIVATE. Current GitHub CLI
authorization refuses writing `.github/workflows/ci.yml` for missing
`workflow` OAuth scope. The vetted public workflow is staged at
`ci/github-actions.yml`; copy it to `.github/workflows/ci.yml`
using an owner-authorized GitHub UI action OR grant workflow scope.
Until then, public CI is **NOT RUNNING** despite working public
repository. Never expose owner OLT network maps or real captures to
public clone, logs, Issues, Actions artifacts or repo secrets.
