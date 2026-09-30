# R9.38 — Owner-supervised C320 fast-path to usable ONT MVP

Owner objective: operate at least one REAL ONT from IPAT; minimize repeated synthetic-only work. This is an explicit split between OWNER-OPERATED isolated LAB workflow and commercial unattended adoption, not a waiver of hardware-as-live change discipline. Current exact C320 owner-VPS SSH port 321 passed new no-credential passive TCP reachability; genuine earlier R9.34 owner-approved scripted `show card` was authenticated and returned three actual cards. Do not relabel reachability as registration.

## First tangible acceptance, not another mock

`deploy/scripts/lab/r938/owner_c320_onu_first_inventory.py` is a new bounded one-time *human interactive* PRIVATE owner-VPS read. It accepts ONE temporary password via local TTY (never ChatGPT, Git, CLI arg or env), strict existing owner network-observed SSH host pin, and sends exactly three read-only C320 commands: `show gpon onu uncfg`, `show gpon onu state`, `show run interface gpon-olt_1/1/1`. Stops on unexpected prompt, timeout, unsupported/paged CLI, unrecognized actual-firmware output. The raw responses include potentially sensitive real ONT serials and must remain owner-only 0600 in private 0700 VPS directory; stdout exposes only command IDs, output shapes and bounded counts. No mutation/firmware/account/password changes, no retries, no automatic worker, no auto-adoption.

**Owner-local run only after script is source-verified in the private VPS and the owner can use the VPS TTY:**

```sh
cd /home/openai/.cache/ipat/r938-read/src
IPAT_R938_OWNER_ONE_TIME_DISCOVERY=YES python3 deploy/scripts/lab/r938/owner_c320_onu_first_inventory.py --owner-interactive-read
```

Exact private path must be confirmed by actual deploy, not inferred. The owner types opt-in phrase `ONE_OWNER_LAB_ONU_READ_NO_WRITES` then password *locally*, never sends credentials to chat. A result is NOT claimed until the real owner command and independent redacted receipt have been verified. A header-only/empty valid ONU discovery, if actual C320 reports it, does not imply a connected ONT exists: attach a single isolated ONT and repeat only via separately approved bounded read.

## Immediate engineering sequence after verified real ONU reads

1. Map actual `show gpon onu uncfg` and `show gpon onu state` raw shapes from OWNER-only source to strictly sanitized operator inventory. Reconcile `show run interface gpon-olt_1/1/1` to select a genuinely unused ONU ID, never use the provisional index in `uncfg` output as a free ID.
2. Confirm the model printed on the *physical isolated ONT* and exact matching supported `onu type`, existing VLAN/TCONT/GEM and service profiles from true read-only C320 CLI. Do not invent profiles or customer-facing VLANs. A full live model read and inventory refresh precedes any registration.
3. Present an immutable one-ONT dry-run plan: actual serial (owner-private), port and genuinely free ONU ID, model profile, existing bridge VLAN/GEM/TCONT and exact diff; confirm native recovery and rollback, tenant/operator access, second approval and explicit maintenance scope before hardware writes.
4. Build an exact-firmware one-ONT executor with no arbitrary CLI input, one device/PON lock, bounded command/reply checks, mandatory physical read-back and UNKNOWN outcome quarantine. Verify one physical ONT and restore on failure. Distinguish LAB_ONBOARDED_READ_ONLY vs PRODUCTION_ADOPTED and LAB_REGISTERED_ONT vs production capability.

The full commercial automatic `ADOPTED` requirements remain recorded in R9.35, but they must not block owner-supervised verified READ-ONLY laboratory discovery. They *do* block unattended privileged management and high-impact ONT configuration until controls are actually present. One-ONT write acceptance is a separate high-risk stage, not implied by read-only success.

STOP: R9.38 currently only ships the source-level bounded CLI collector and five synthetic/noninteractive safety tests; the owner's fresh actual read and operational ONT registration are NOT YET EXECUTED. Native restore, restricted service identity and independently verified physical console are still open prerequisites for persistent writes.

## Owner reported real manual Telnet login and immediate read-only command sequence

On 29 September 2026 owner showed a successful interactive connection to private C320 management Telnet port 323, ZTE `ZXAN product C320` banner and privileged `olt.backup#` prompt. **Owner-reported interactive evidence**, distinct from independently authenticated R9.34 SSH proof. Login showed a weak-password warning. Do not paste passwords/raw ONU serials into ChatGPT, log the full terminal publicly, change credentials yet or interpret Telnet as an approved unattended transport. This preexisting temporary Telnet is a LAB-only manual exception; final persistent collector must use separately pinned secure transport and scoped identity.

Because the owner is already at an authenticated CLI, the fastest verified actual-discovery action is an owner-manual *read-only* sequence from that SAME prompt, with no second SSH login:

```text
olt.backup#show gpon onu uncfg
olt.backup#show gpon onu state gpon-olt_1/1/1
olt.backup#show run interface gpon-olt_1/1/1
```

Run sequentially, inspect one result before the next, and STOP if a command shows an unknown/paged response. CLI patterns are reported in older C320 documentation, NOT yet independently accepted for exact owner firmware. These commands are inventory-only; `configure terminal`, `onu N type`, profile, `save`, firmware and removal commands are OUT OF SCOPE. ONT raw SN and PON config must stay private. Share only sanitized per-command success/failure, count and any non-sensitive parser/header mismatch. The `uncfg` provisional `:N` suffix is not a proof that ONU ID `N` is free. Prior private 119980-byte historical running-config already contains 171 interface-scoped ONU declarations including 72 in gpon-olt_1/1/1, so treat all legacy configuration as potentially preserved despite owner's no-live-customer statement.

The R9.38 owner-VPS tool remains a separate *SSH port321* operator-driven path and must not be described as the existing Telnet323 terminal. Do not claim that the new three-command physical read has run until sanitized success receipt or separately attested owner output is obtained.
