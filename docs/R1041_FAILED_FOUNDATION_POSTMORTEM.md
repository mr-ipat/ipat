# R10.41 — Actual Ubuntu26 PostgreSQL first-install failed safe, false-success regression fixed

**Observed:** 2026-10-17 WIB. **Production PostgreSQL NOT installed at incident close.**

## Actual attempted first install and root findings

Product Owner risk acceptance and root-controlled source/SQL candidate were accepted for **local-only R9.96**. R10.40 CI 4/4 PASS. On the real VPS, root executed the exact root-owned reviewed R10.40 script. Source/dirty-tree checks, binary SHA, user/unit preparation PASSED; Ubuntu PostgreSQL18 packages were downloaded and installed. However `pg_createcluster` refused the invalid inherited Mac SSH locale `LC_CTYPE=UTF-8` (the VPS provides `C.utf8`). The preexisting R9.96 `ERR` trap called rollback then returned success instead of exiting. Consequently, after deleting the runtime identities and partial cluster, the orchestrator **incorrectly printed a success marker** and wrote `/var/lib/ipat/r996-foundation.json`. The enclosing script then also printed SUCCESS. The install was **not** successful.

Independent nonroot checks found 0 PostgreSQL18 clusters, no Unix socket, all six runtime identities absent, all 3 commercial units absent and no public listeners. Root inspection found a misleading root-owned foundation marker `recovery_gate=OWNER_ACCEPTED_UNTESTED_BOOT_RESTORE_LOCAL_ONLY`, `public_go=false`, plus orphaned `/etc/ipat` and `/var/lib/ipat` directories. The owner-private C320 preview and connector remained ACTIVE. **Never use the original marker as deployment evidence.**

## Remediation R10.41

1. R9.87 bootstrap now pins `LANG=C.UTF-8 LC_ALL=C.UTF-8` for its child process, avoiding global host locale mutation.
2. R9.96, R9.87 and R9.97 install/edge abort functions now disarm traps, perform rollback and **exit code 4** on a failed child command. Tests run the extracted production abort handlers with a deliberate `false` and prove a fake success line cannot execute.
3. Before R9.96 writes a foundation marker, it independently requires a running `18/ipat` cluster, its actual Unix socket, six production runtime users, absence of temporary migrator and required systemd units. No successful marker is possible solely on the basis of exit status from a child.
4. Original source security gates, local-only owner waiver, and R9.97 independent-restore/public ingress prohibition remain unchanged.

## Controlled incident recovery and retry

Before a retry, an authorized root operator must confirm there is **no existing valid production cluster**, no commercial runtime identities/listeners, and that residual `/etc/ipat` and `/var/lib/ipat` belong entirely to the failed R10.40 attempt. Quarantine those directories under root-only evidence (do not delete evidence or customer data), retain the accepted owner-risk document and root logs, then use a NEW clean, root-owned exact reviewed R10.41 source and verified binary. Reissue a fresh 24-hour owner risk document bound to its new commit. Re-run first install once with the original rollback implementation and newly hardened postconditions. Independently verify role peer login and no public ports before marking milestone complete.

**Actual retry pending** until the exact-head GitHub CI passes and authorized root cleanup/revalidation are complete. This document is an incident record, not a production GO statement.
