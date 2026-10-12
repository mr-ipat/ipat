# R10.18 — Operator root-authentication and production foundation handoff

Date: 2026-10-12 (Asia/Jakarta). Scope: existing IPAT owner VPS (Ubuntu 26.04.1). This is a **read-only** root gate and exact-main binary release candidate, **not public deployment**.

## Why SSH write access is not enough

The trusted SSH principal is nonroot `openai` (uid 1000), member of group `sudo`. Noninteractive `sudo -n -l` returns "interactive authentication is required." The SSH key authenticates the OS user only; it is not root authorization. `/etc/systemd/system` and `/opt` require root, and production packages/PostgreSQL cluster/HTTPS ingress must not be installed through an unprivileged owner-preview process.

The user-level `ipat-r911-preview.service` remains loopback-only. It is not the signed commercial Platform API/OIDC or tenant release.

## Current exact-main deployable candidate

- Verified `main` commit: `845eeac1f039168fd08af7f860ab50413f91d85a`.
- Existing Ubuntu 26.04.1 x86_64 VPS rebuilt `control-api` with Rust 1.98.1: `cargo build --locked --offline --release -p control-api` **PASS**.
- Release SHA256: `1753d706b6d076fb67ba9509782e1e5c82a5aa673f901e9e809935bd98a66f20`; no missing dynamic libraries reported.
- Clean exact source directory: `/home/openai/ipat-release-src-r1018` (unprivileged; NOT trustable as a root-owned production execution tree).
- Exact binary: `/home/openai/.local/share/ipat/release-candidates/845eeac1f039168fd08af7f860ab50413f91d85a/control-api`; source marker and SHA256SUMS verify `control-api: OK`.
- The prior user preview keeps its independent R10.16 binary; do not stop or replace its unit while preparing production.

## One command for owner: prove interactive root access

Reviewed root-only no-mutation probe has been uploaded to:
`/home/openai/.local/share/ipat/release-operations/r1018-root-readiness.sh`.
SHA256: `8082043ed77cd597d8f21339502272ccaa2794c41ce7ca386c18a3513c13e436`.

Run on **owner Mac Terminal**:

```bash
ssh -tt ipat-lab 'sudo install -o root -g root -m 0500 /home/openai/.local/share/ipat/release-operations/r1018-root-readiness.sh /root/ipat-r1018-root-readiness.sh && sudo /root/ipat-r1018-root-readiness.sh'
```

Type the `openai` **sudo password in the VPS terminal prompt**, never send it in chat, GitHub, logs or scripts. Existing interactive sudo authentication is sufficient for this one command. Do **not** add `NOPASSWD: ALL` or disable SSH/sudo password controls.

The probe refuses execution from user-controlled file locations even under root; it only executes if copied into `/root/ipat-r1018-root-readiness.sh` and that copy is owned by root, permission 0500, hardlink count 1. It checks exact Git commit, binary digest, empty production first-install paths and port conflicts, PostgreSQL clusters and metadata of the expected root-only recovery attestation path, without changing configuration. The root-only recovery attestation is expected at `/root/ipat-evidence/r996-recovery-attestation.json` (0600, root-owned, one link). Its existence is **not independent backup evidence**, and the probe never marks `PUBLIC_GO=true`.

If sudo authentication fails, recover/authorize interactive OS administrative access through the real VPS provider console and existing approved operator accounts. Do not bypass authentication or use the private dashboard as a root management shell.

## Stage boundaries after root access

1. Independently verify provider rescue console/root login and **actual distinct-host, encrypted full-host restore** with rollback. Check external evidence; do not synthesize positive `r996` attestation booleans.
2. Before production mutation, independently review source+build provenance. Install a reviewed **root-owned** checkout/binary as appropriate. Pin exact source commit and control-api SHA256 and use the existing `r996_validate_recovery_attestation.py` gate for a root-owned 0600 attestation tied to that commit. A writable user checkout is not a trusted root script execution directory.
3. Only with approved recovery evidence and root-review, run `r996_prepare_production_foundation.sh` in its documented first-install order. Expect **application services disabled and no public listener**. Independently test actual PG18 cluster, 33 pinned migrations, Unix peer roles and rollback.
4. Set up and independently test production PostgreSQL offhost base+WAL, `pg_verifybackup`, named PITR, RLS restore, standby failover and fencing, measured RPO/RTO, maker/checker. Then and only then use `r997_activate_platform_edge.sh` for Platform API/OIDC and CA-trusted literal-IP TLS.
5. Pass real operator human MFA browser login, replay/revocation/Host and SNI tests; only then activate one verified company custom-domain OIDC/ingress through `r998`, followed by `r999` genuine hostile-browser acceptance.
6. Physical CWMP/USP, C-DATA/VSOL/ZTE/MikroTik transport, model/firmware and safe provisioning/readback/rollback plus multi-node load testing are separate MUST acceptance gates. No physical writes are implied by this root handoff.

**Release state:** built binary PASS, private preview PASS, owner root **authentication not independently demonstrated**, host full restore, actual production PITR/HA, MFA/TLS and commercial browser acceptance NOT PASS. Entire public commercial MUST PRD remains NO_GO.

## Safe test and verification

```sh
bash -n deploy/scripts/production/r1018_root_readiness.sh
python3 -m unittest discover deploy/scripts/production -p test_r1018_root_readiness.py -v
ssh ipat-lab 'cd ~/.local/share/ipat/release-candidates/845eeac1f039168fd08af7f860ab50413f91d85a && sha256sum -c SHA256SUMS'
```

The first two tests validate source/shell and refusal of running a privileged script from an untrusted location. No automated test can prove rescue-console authentication or distinct-host backup restore.

## Integration defect corrected during this handoff

R9.96 originally omitted R9.87's required child opt-in/source-root environment and would always reject before creating the PostgreSQL18 cluster. R10.18 source correction now explicitly exports both on just that subprocess invocation. The regression executes the Bash child call in an isolated temporary directory and tests both variables; no actual DB or VPS installation occurs. **After merge, build/install from the final reviewed main commit, not an earlier user-home checkout**. The pinned R10.18 root-readiness script and initial candidate SHA above remain a limited, read-only root-authentication handoff and are *not* an installer authorization for the corrected future commit.
