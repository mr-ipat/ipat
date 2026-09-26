# IPAT R6.2 — Rust RouterOS identity normalization and private evidence boundary

**Developer:** Mr. iPat
**Scope:** Offline Rust domain integration of the pre-existing, separate
owner-Mac-only R6.1 first-GET candidate. **No physical RouterOS connection or
production device enrollment was made during this milestone.**

## Delivered and strictly limited

The new `crates/routeros-core/` Rust module is a domain boundary for the
initial owner-reported DEV-08 customer router, **RB951Ui-2HnD** and
**RouterOS 7.23.7**. It is not a network driver or a claim of RouterOS
interoperability. The existing R6.1 Python probe remains the only candidate
for the very first operator-approved HTTPS network GET, with its own
permission, isolated route, account, independent CA and double opt-in.

The Rust parser `normalize_resource(..., ApprovedReadProfile::Dev08OwnerReportedRb951)`
accepts a bounded `/rest/system/resource` JSON response with exactly
one resource object, rejects duplicate fields (including duplicate
unapproved fields), multiple objects, extra-large responses, unsafe/missing
identity values and unsupported model/architecture/firmware. It extracts
only exact board name, `mipsbe` architecture and approved version;
all other raw fields, including vendor serials, IP addresses and
sensitive values, are ignored and never returned, serialized or logged.
The `UnreviewedInventory` type has deliberately private fields and
no constructor permitting tenant assignment, physical enrollment or
configuration writes.

The normalized version matches **7.23.7** or that exact version with a
recognized `(stable)` or `(long-term)` channel suffix, consistent
between the Rust parser and the guarded Python R6.1 lab script. A
different RouterOS version, development/testing channel, model or CPU
architecture is rejected. These syntactic checks do not prove that a
reported router exists or is running that version.

The separate Rust `evidence::normalize_staged_lab_evidence` only accepts
the *allowlisted, redacted* R6.1 output schema. It rejects new sensitive
fields, duplicates, missing fields, unknown methods/resources, changed
model/version and any forged value attempting to change
`physical_device_enrolled`, `tenant_binding_verified`,
`compatibility_verified`, `configuration_modified` or human review.
Time is checked only for basic syntax, **not** source authenticity.

The `routeros-lab-evidence` binary is a **local/offline** schema checker
for an already obtained private redacted artifact. It refuses missing
arguments, source-relative paths, symlinks, unexpected owner-only file-mode
permissions, open files outside a private parent directory, excessive
file size and schema violations. It prints only generic verdicts,
never private paths, values or raw router responses. A syntactically
valid file can be fabricated: CLI success always says physical read
**NOT AUTHENTICATED**, tenant **NOT ENROLLED** and compatibility
**UNVERIFIED**. No database, frontend write route, network module,
auto-retry, cross-tenant access or RouterOS configuration command is
introduced by this crate.

## Independent non-network verification

With Rust 1.98.1 already available in an authorized disposable
development environment:

```bash
cd ~/Projects/ipat-current
cargo fmt --all -- --check
cargo test --workspace --locked --offline
python3 -m unittest discover deploy/scripts/lab/r61 -p 'test_r61*.py' -v
bash deploy/scripts/lab/r62/synthetic-cross-contract.sh
```

The cross-contract script generates only fake data, deliberately adds a
fake raw serial/password/IP field to a **synthetic** response, passes
it through the prior Python allowlist, then uses the Rust CLI to
validate a mode-0600 redacted file. The test rejects a forged
tenant-enrolled artifact and a file with an added serial field.
It is executed without a router IP, account, external network or TLS
connection to customer equipment. Tests run on actual Ubuntu 26.04.1
in a separate unprivileged source checkout, not by modifying live
production services.

## Next owner-authorized physical steps, not completed

**Current blocker:** on the authorized Mac,
`~/.local/share/ipat/router-lab/` was initially absent at the
first milestone preflight. It was subsequently created with ONLY
a deliberately non-operational, mode-0600 `probe.template.json`
whose TEST-NET address and non-confirmed permission fail preflight.
There is no actual `probe.json`, usable CA or restricted netrc. Therefore there is **no independently configured private
router route, trusted dedicated lab TLS CA or locally held restricted
test account** available to the automatic workflow. Do not create
fictional config files, silently enable router `www-ssl`, guess the
management IP, bypass TLS, use an admin account or contact the active
customer router without these preparations.

After the owner independently establishes **actual** isolated,
non-disruptive equipment access and backup/recovery, refer to
[DEV-08 first HTTPS one-GET runbook](MIKROTIK_CUSTOMER_R61.md).
First run the **offline** `--preflight`. Only the operator may
opt in to the one real GET with both explicit flags after reviewing
the exact device, private management route, TLS CA and dedicated
read-only account.

If this actually yields
`~/.local/share/ipat/router-lab/DEV-08-evidence.json`, the new
Rust binary can then perform its **offline schema check** on the same
machine where its Rust source/toolchain are available:

```bash
cargo run --locked --offline -p routeros-core \
  --bin routeros-lab-evidence -- \
  --input "$HOME/.local/share/ipat/router-lab/DEV-08-evidence.json"
```

An independent operator must review the actual device's board,
firmware and tenant authorization, redacted test transcript,
non-disruption and TLS identity separately before declaring the
narrow **read-only TC-ROS-03 feature** physically tested. Further
features (wireless, VPN, subscriber, firewall, PPPoE and update)
require separate specifications, adapter implementation,
authorization/approval and physical regression. No proposed feature
should silently be displayed as an implemented permission/menu.
The main VPS K3s/PostgreSQL/nftables and shared external perimeter
remain unchanged and blocked by the earlier recovery gates.

## R6.2 reviewed feature evidence (before final docs merge)

[Feature PR #55](https://github.com/mr-ipat/ipat/pull/55)
merged to exact source SHA
`9901627a1739a4cfa785ed899fed8fb2c55580fa`.
Its actual four-job PR CI `36222753736` and independently
repeated post-feature-main CI `36222869650` both
finished all-success, including disposable real Ubuntu 26.04
K3s and isolated PostgreSQL recovery. Real actual VPS
source matched private GitHub/Mac and passed 99 Rust offline
workspace tests, 12 R6.1 Python tests and
cross-language fake-secret stripping + forged tenant denial.
Feature-source encrypted Restic snapshot `b13e3f49`
independently restored and selected root-readable config
`abaa9827` also separately restored with full pack read PASS.
This is not live router login, signed physical evidence,
production PostgreSQL recovery or Kubernetes readiness.

The authorized Mac's new owner-only
`~/.local/share/ipat/router-lab/probe.template.json`
contains **intentionally invalid** values, no actual
connection details, and no credential, CA or owner permission.
The offline real-helper preflight correctly rejected it

without any device network traffic. Do not convert the
placeholder to an approved configuration without separately
verified owner permission, private route, trusted TLS certificate,
restricted identity and backup/recovery evidence.
