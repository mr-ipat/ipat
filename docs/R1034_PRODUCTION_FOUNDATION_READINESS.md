# IPAT R10.34 — Actual Ubuntu 26.04 production foundation readiness

**Observed:** 16 October 2026 WIB. **Public GO: FALSE**. This report distinguishes a real nonroot host preflight from privileged disaster recovery and production activation.

## Proven native release candidate

Reviewed canonical main: `6d0562da79483c0e5df1d264d651b5673907f8dc`. Clean detached source worktree on VPS: `/home/openai/ipat-r1034-main`. Rust 1.98.1 passed `cargo fmt --check`, 4/4 R10.28 read-only physical transport unit tests and the exact main offline release build.

Compiled `control-api` SHA256: `7e041f843668e77d46c89c2fb7cc61972b794af12f3b674beb284cba3ecf91cd`.

VPS staged nonroot artifact: `/home/openai/.local/share/ipat/release-candidates/6d0562da79483c0e5df1d264d651b5673907f8dc/control-api`. Binary chmod 0500, `SOURCE_COMMIT` and SHA256SUMS verified. **The candidate is user-owned, not a reviewed root-owned production artifact. It must not be used directly for a root execution.** Production root must separately take custody, check exact Git source, ownership, file modes, manifest and digest.

## Real host observations

- Ubuntu Server 26.04 x86-64, approximately 32 GB RAM and 120 GB free disk at inspection. Seven commercial runtime users absent, commercial platform/tenant units inactive, no listeners on 80/443/5432/3003/3005/3006.
- PostgreSQL cluster utilities not yet installed; `apt-cache policy postgresql-18` reported Ubuntu candidate `18.6-0ubuntu0.26.04.1`. `apt-get -s --no-install-recommends` reported seven packages with no installation.
- Current `sudo -n` requires interactive authentication. Nonroot cannot inspect root-owned recovery attestation; previous owner-reported independent rescue/restore remains **not independently verified by this milestone**.
- Existing lab C320: actual SSH321 and Telnet323 initial network protocol replies PASS, existing separate SSH-only connector CONNECTED, fresh read-only hardware CARDS=3 and FIRMWARE rows=5, firmware mapping unreconciled. Private Device Workbench loaded in real Mac Chrome over SSH tunnel. Physical write and tenant commercial adoption remain false.

## Reproducible, read-only gate

```bash
python3 deploy/scripts/production/r1034/read_only_foundation_gate.py \
  --source /home/openai/ipat-r1034-main \
  --candidate /home/openai/.local/share/ipat/release-candidates/6d0562da79483c0e5df1d264d651b5673907f8dc \
  --commit 6d0562da79483c0e5df1d264d651b5673907f8dc \
  --binary-sha256 7e041f843668e77d46c89c2fb7cc61972b794af12f3b674beb284cba3ecf91cd
```

Executed on the actual VPS from the Mac via SSH stdin, emitted `source_and_binary_hash_verified=true`, `postgresql18_candidate_available=true`, all managed commercial ports `NOT_LISTENING`, root recovery/restore `NOT_ASSESSED`, `production_pitr_ha=NOT_VERIFIED` and `public_go=false`. Its exit code 0 means **read-only inspection completed**, NOT that commercial foundation is ready.

Mac negative/schema suite: 4/4 PASS. Actual Ubuntu26 R9.96 foundation tests: 18/18 PASS; R9.97 source activation tests 6/6 PASS. These are source/security contract tests, not proof of a successful full-host restore or physical customer enrollment.

## Production gate still required

1. Authorized root must inspect genuine provider rescue and independent offsite backup/full-host restore evidence; ensure R9.96 root-owned recovery attestation is recent, owner-only, trusted ancestry and tied to the exact intended source revision. No fabricated evidence.
2. Independently stage a **root-owned, root-controlled** clean source and hash-verified binary. The user-owned candidate may only serve as a digest-checked input.
3. Only after root recovery signoff, perform R9.96 first-install PostgreSQL18 with rollback and no public listeners; then independently prove production PITR/HA.
4. R9.97 Platform Owner OIDC/MFA and public TLS, tenant Host/SNI/custom domains and role-isolated real browser acceptance must pass before a public GO.

No firewall, PostgreSQL install, production service, OLT configuration/firmware, DNS or root-owned artefact was mutated in R10.34. No change to binding architecture or PRD.
