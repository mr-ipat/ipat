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

## Actual independent nonroot VPS offline test evidence

A separately verified nonroot owner-VPS source bundle of R9.18 was
checked out in `/home/openai/.cache/ipat/r918-release/src`, with
its own isolated Rust target cache, no changes to active :3000/:3002
service units, SSH config, C320, POP gateway, ONTs or PPPoE.
An initial `--locked` attempt correctly detected that a new Rust
workspace dependency edge needed a Cargo.lock update. A controlled
OFFLINE dependency resolution from the existing lock added ONLY
`serde_json` and `sha2` to olt-core's existing lock entry. The first
compiler run found an incompatible sha2 v0.11 LowerHex formatting
assumption; revised code explicitly formats SHA-256 bytes as hex.

After fixing and formatting, ACTUAL locked offline `cargo test
--locked --offline -p olt-core` passed 10/10 strict unit/fixture
cases (3 new normalizer tests + 7 pre-existing vendor fixture tests),
and the new `olt-evidence` binary was actually built. A PRIVATE
nonroot temporary owner 0700 test directory contained entirely
SYNTHETIC sample card/version captures with 0600 permissions.
The binary emitted a strict normalized 0600 JSON file with exactly
two sample cards, matching firmware board/slot identity, SHA-256
transcript digests and ALL actual hardware/adoption/network booleans
FALSE. An existing output file was provably NOT overwritten. A
world-readable (0644) sample input was provably rejected, producing
no output. The synthetic fixture was deleted after the tests.
Compiled binary SHA256
`e426c637db7860f803bb01e617fe59eb0650b147b8a1d6ca3aba8d47aacfd266`.
This does NOT prove compatibility with exact DEV-01 firmware or a
single genuine OLT login; actual capture remains NOT RUN.

Public CI mirror push used new clean Git history but current OAuth
cannot modify `.github/workflows`. The sanctioned workflow has been
staged in `ci/github-actions.yml` and must be installed by repo owner
with authorized GitHub UI action or refreshed `workflow` OAuth scope.
Public repository source contains no real device credentials or
actual site addressing by construction; its checks are synthetic.

## Sanitized public source CI rehearsal on the actual VPS

After the public synthetic source was published, a clean public Git
bundle for exact source SHA `c281ac5` was separately cloned into
another NONROOT owner VPS folder, not the private canonical source.
Owner-only low-priority validation against that exact public snapshot
PASSED: synthetic Python 45+25+13 tests, independent public secret
and real site address guard, rustfmt pinned Rust1.98.1 and full
`cargo test --workspace --locked --offline -j 1` (178 second total
long Rust suite run). No actual OLT, real keys or real management
addresses were included. The staged standard-runners GitHub Actions
workflow remains unregistered because current OAuth cannot upload
workflow files. The public repository has enabled secret scanning
and push protection. Do not claim hosted Actions completed.
