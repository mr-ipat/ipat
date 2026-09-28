# R9.14 — Durable independently reviewed physical site evidence (lab-only)

Date: 2026-09-28. Physical owner-reported DEV-01 ZTE C320 remains
NOT ADOPTED; there has never been an approved authenticated read.
A LIVE distribution OLT cannot be guaranteed unaffected by a login;
its operational acceptance requires owner-reviewed exact blast radius,
no-change command allowlist, on-site recovery and observed baselines.

## Increment actually implemented

Migration `deploy/db/migrations/0011_lab_physical_site_evidence.sql`
is a new append-only, time-bounded metadata table; no IP, hostname,
secret, private key, plaintext evidence or management worker dispatch.
A separately owned SECURITY DEFINER SQL function checks approved
ACTIVE own tenant, approved exact candidate, independent approved
security_admin reviewer (not original applicant), immutable request ID,
strict SHA-256 evidence digest, and short validity. A separate
EXECUTE-only role can submit, while an isolated existing restricted
read role may see only its own active tenant's 6 evidence gates:

- Trusted out-of-band fingerprint confirmation (not TOFU).
- Independently isolated management last hop.
- Restricted device public-key-only account.
- Exact firmware support for explicitly permitted read-only command.
- Owner-approved baseline + abort thresholds for live distribution.
- Dedicated worker's actual authenticated private route.

Each latest gate is true only while fresh; later BLOCKED verdict revokes
its gate. `physical_worker_enabled` is ALWAYS FALSE even when synthetic
lab evidence claims all six; this schema is NEVER authority to touch
real C320. Textual issuer/subject submitted to SQL are NOT real
authentication; caller needs genuine independently pinned OIDC MFA
before any actual endpoint is mounted. Actual site evidence digest and
trusted identity have NOT been provided by the owner; no attestation
here should be treated as physical truth. The migration is tested only
on disposable PostgreSQL. NEVER apply it to customer DB without
staged DR, migration, release/permission and isolated physical review.

## Operational critical path still blocked

1. Owner/site trusted console independently verifies existing RSA
   fingerprint before SSH account use.
2. Site verifies actual private management VLAN, restricted gateway
   ingress and independent console/router recovery (not simply RFC1918).
3. Install an approved permanent tenant/site tunnel through an
   independent maker/checker-controlled deployment workflow.
4. Owner provisions a genuinely restricted OLT account and proves
   single read-only operation supported on its exact firmware.
5. Capture per-OLT/control-plane/ONT/PPPoE before-after baselines,
   approved low-rate one-command maintenance protocol and abort.
6. Bind real live OIDC/MFA/tenant-scoped backend role to site evidence,
   separately authorize exactly one hardware read with lease and
   audit before setting any operational adoption status.

No physical OLT read, tunnel activation, firmware, router, ONT or
PPPoE change is authorized by R9.14. Do not upgrade observed SSH
transport alone into an adoption claim.

## Operator-safe independent console RSA verification tool

To make the next missing physical gate objectively testable, R9.14
adds `deploy/scripts/lab/r914/verify_owner_console_host_key.py`.
It checks one LOCAL owner-only 0600 `ssh-rsa` public-key file exported
through the owner's INDEPENDENT, already-trusted physical OLT console
(or an independently authenticated site asset system). It never fetches
a network key; observed `ssh-keyscan`, Telnet banners, and private SSH
key files are not acceptable as the independently trusted source.
A local `ssh-keygen -lf` comparison tests whether the independently
provided RSA key fingerprint equals the previously observed network
fingerprint. Missing/invalid file, unexpected key type, symlink,
shared file or untrusted provenance assertion fails closed. Even a
MATCH explicitly records `independent_reviewer_approval_recorded=false`
and `device_adopted=false`: operator assertion is not independent
security-admin approval or physical model/firmware verification.
It sends ZERO packets or credentials. No actual trusted-console key
was provided; four offline mock tests passed, but live OOB match
remains NOT RUN.

Run ONLY after copying the device's own SSH HOST RSA PUBLIC KEY
from an independently trusted console into a non-repository 0600
local file on the owner's machine. NEVER place a private key or
factory account password into this file or the repository:

```sh
chmod 600 /your/private/folder/device-console-host-rsa.pub
python3 deploy/scripts/lab/r914/verify_owner_console_host_key.py \
  --trusted-console-rsa-public-key \
  /your/private/folder/device-console-host-rsa.pub \
  --owner-attests-independent-console-source
```

Review the result and its independent console provenance with a
separate authorized security administrator before any live access.
A matching SSH RSA fingerprint does NOT prove an account is readonly
or the network management VLAN is isolated.
