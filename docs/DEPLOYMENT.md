# IPAT — Deployment & Operations Runbook Specification v0.1

**Status:** v0.1 cluster/database procedures remain **PLANNED**. A guarded Stage-1 Ubuntu lab compiler bootstrap helper is now prepared in `../deploy/scripts/lab/` but requires a one-time local owner sudo prompt before execution. No Ansible, Terraform, Helm, Kubernetes manifests, certificates, cluster, or measured restore are delivered yet. This file must be converted into executable, reviewed runbooks during implementation.

## 1. Target environments and prerequisites

- **Lab OS:** Ubuntu Server 26.04 LTS. **Provisional sizing:** 16 vCPU, 64 GiB-class RAM, ~1 TB NVMe SSD + private network and external backups. Not an HA system. All-in-one heavy test 16 vCPU/128 GB/~2 TB is provisional, not obligatory. VPS virtual CPUs can differ from dedicated CPU.
- **Dev path:** local Compose for PostgreSQL, queue, IdP, emulator and Rust services, if chosen; use synthetic secrets. Version-pin images, Compose and PostgreSQL release. K3s test in isolated lab before production.
- **Cluster path:** separate K3s control plane/node joins and external stateful PostgreSQL HA plane before production. Define private VLAN/VPN, time sync/DNS, firewall, network policy, egress allowlist, observability/backup endpoints, trusted ingress/certificates and independent out-of-band rescue.
- **Preflight:** provider allows required UDP/TCP tunnels and persistent disks; capacity and IOPS adequate under measured workload; domain ownership actually verified; storage offsite and data residency reviewed; all secrets generated/stored outside Git.

## 2. Infrastructure repository plan (to implement, not present today)

| Planned path | Responsibility | Required verification |
|---|---|---|
| `deploy/ansible/playbooks/bootstrap-ubuntu.yml` | OS hardening, users, time/NTP, firewall, container/K3s deps | Idempotent repeated dry-run and restart; no secret console logging |
| `deploy/ansible/playbooks/k3s-server.yml` | K3s initial control plane, approved datastore mode | Bootstrap success and readiness; recovery and version pin |
| `deploy/ansible/playbooks/k3s-agent.yml` | Heterogeneous node join with secret vault | Node labeled, schedulable, correct requests/limits |
| `deploy/ansible/playbooks/k3s-drain-remove.yml` | Cordoned safe drain/removal | No ongoing write jobs, graceful lease handoff, stateful data safe |
| `deploy/terraform/providers/` | Supported VPS APIs, network and block storage | Plan reviewed, reproducible import/destroy protection |
| `deploy/helm/ipat/` | Axum/CWMP/USP/worker Helm chart | SecurityContext, network policy, readiness/drain, requests/limits |
| `deploy/gitops/` | Version-pinned cluster desired state | Protected PR approval, drift detection and rollback rehearsal |
| `deploy/scripts/backup-restore/` | PostgreSQL WAL/PITR and restore verification | Isolated restore and checksum/RPO-RTO report |
| `deploy/compose/` | Laboratory local services | End-to-end synthetic demo and no production creds |

## 3. Safe planned bootstrap/join/validate/drain/remove flow

**Bootstrap:** (a) approve architecture/ADR secrets and provider; (b) provision Ubuntu and patch with tested versions; (c) separate OS/service accounts and network rules; (d) configure private remote management, time and certificates; (e) bootstrap K3s control-plane per selected HA plan; (f) pin and apply GitOps/Helm with default-deny policies; (g) deploy monitoring, audit sink and secret manager; (h) initialize DB migrations using dedicated migration identity, never runtime account.

**Join:** (a) register node inventory/capacity and failure domain; (b) permit private K3s traffic; (c) use *short-lived externally managed* K3s join credential; (d) verify node is Ready/allocatable and label architecture/role; (e) validate worker constraints, capacity policy, device network reachability and service authorization; (f) add worker budget gradually then record queue age and real CPU/IO curves.

**Validate:** run health/readiness, app/worker logs, policy tenant-negative tests, queue worker lease crash replay and safe dry-run, metrics/alert/rate-limit checks, K3s scheduling and node failure network/CPU/IO observations. With node B, compare workloads and throughput *as measured*; no target exact equal CPU.

**Drain:** pause/limit new risky provisioning, verify in-flight jobs and device locks, enforce lease expiry/fencing and safe reconciliation after cutover; `kubectl cordon NODE` then, **only after verifying persistent data workloads and disruption budgets**, use an appropriate `kubectl drain NODE --ignore-daemonsets` option set. `--delete-emptydir-data` can destroy ephemeral state and must not be blindly used. Validate no dangling writes before removing hardware.

**Remove:** confirm no remaining workloads/local volumes/secrets, revoke join credentials and host/service access, remove node safely, scrub media following policy and update asset registry; platform/backup/DB state must not depend silently on the removed worker.

**Rollback:** GitOps revert tested manifests, application migration compatibility or forward-only recovery documented; if a job sent unknown router commands, **do not retry blindly**. Isolate queue, read actual device state and follow controlled rollback plan.

## 4. PostgreSQL backup/restore and HA requirements

**S1:** backup selected PostgreSQL lab tables/database; restore into a separately isolated instance and verify tenant counts, FK constraints, selected sample hash and app smoke tests. Retain job/audit recovery integrity. Capture start/end, DB version, data size, backup checksum, restore verification result, log redacted. Producing backup file without restore is **not** passed AC-07.

**Commercial:** PostgreSQL primary+standby design, WAL archiving/PITR, encrypted external independent backup, retention/immutability, restore into clean staging, failover drills and approved measured RPO/RTO; database replicas are **not** assumed to shard write throughput. Test tenant isolation in backup exports/restore and protect cross-tenant physical backup. Select operator/tool and control-plane approach in ADR-010.

**Emergency:** service/DB outage → determine if data-plane degraded vs stateful unavailable; temporarily stop risky writes to avoid unknown effects; avoid two active primaries (split brain); restore from tested point; reconcile outbox/jobs vs actual network state after DB recovery; keep tenant impact/audit evidence.

## 5. Test gates and operator handoff

- Every runbook becomes executable only after code review, placeholder-free secrets and tested rollback; support `--check`/dry run when tool permits.
- Required observability: request/trace ID, CWMP/USP sessions, permission-denied alerts, oldest queue age, retries/DLQ, router timeouts, lease/fencing conflicts, per-node CPU steal/memory/IO, database WAL/archive lag, backups success **and restore recency**.
- CI/integration and operator physical lab reports are separate. Document node type, installed version, exact test time, all observed results and deviation from acceptance gates. If provider/system unavailable, mark `BLOCKED` in `PROJECT_STATUS.md`.

**Never commit**: live credentials, TLS private keys, K3s join tokens, actual private device configs, raw ONT credentials, PPPoE secrets or patient/subscriber identifiers. All credentials in docs are explanatory placeholders only.

**Restricted lab stage 1:** [reviewable helper and no-snapshot safety plan](../deploy/scripts/lab/README.md). Read-only checks and verified user-only Rust installation are independent of privileged package/SSH/network changes.
