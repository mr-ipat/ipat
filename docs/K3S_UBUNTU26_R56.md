# IPAT R5.6 — Real K3s on Disposable Ubuntu 26.04

**Developer:** Mr. iPat
**Scope:** real K3s smoke test on a separate disposable GitHub-hosted Ubuntu 26.04 runner only. No actual IPAT VPS K3s installation.

## 1. Why this advances K3s without risking the only reachable VPS

Actual Ubuntu VPS preflight continues to show Ubuntu 26.04.1, K3s inactive and public dual-stack SSH, while independent out-of-band login has not been proven. The shared externally managed allow-all group serves multiple VPSs and is not an IPAT software integration. Its rules must not be changed to unblock K3s. An encrypted, separately restored Git snapshot and selected root config are **not** complete host recovery; standalone production database PITR and private K3s cluster design are still open. These preflight blockers prevent any live VPS installation even with owner desire to advance.

Instead, GitHub Actions offers a disposable Ubuntu 26.04 x86_64 runner to test native installation on the project's *actual target OS family*. The R5.6 script lives in `deploy/scripts/lab/k3s-ubuntu26-ephemeral-ci.sh`; non-GitHub runners, mismatched OS/architecture and any pre-existing K3s installation are explicitly refused. The runner is not reused as an IPAT server. This scope differs from the previous Ubuntu 24.04 Rust/PostgreSQL jobs.

## 2. Fixed lab implementation and evidence criteria

- The comparative experiment pins upstream **K3s v1.36.4+k3s1**, a non-prerelease release dated 2026-08-27. The initially tested newer v1.37.0+k3s1 reliably booted its embedded-etcd control plane, but its packaged CoreDNS and gateway installer containers failed under the disposable Ubuntu 26.04 runner with OCI/runtime permission errors (CoreDNS last exit code 128). This does **not** establish compatibility of either version on the live VPS; a release/security review is mandatory before production selection. Download only the tagged HTTPS release binary and verify the exact upstream published amd64 SHA-256 `835873f37245fc615f547a2fe2af9402a347875f13fa64a1f136de644955ea3f` *before execution*. No mutable latest-install pipe, latest tag or package auto-upgrade.
- Run non-systemd foreground K3s with a temporary data directory, explicit single-node **embedded etcd**, disposable node RFC1918 IPv4 binding, dedicated non-default API port `16443`, and no Traefik, ServiceLB or metrics server. These flags are **CI-only**, not an approved production CNI or node design. A production node requires verified private management routes, dedicated ingress policy and a separately signed ADR-017 before any installation.
- Wait with bounded retries for the actual Kubernetes node `Ready`, confirm API is not listening on public IPv4/IPv6 wildcards, wait for CoreDNS `Available`, start a real temporary BusyBox pod, and execute internal DNS resolution. Fail fast on timeout.
- Create a real embedded-etcd snapshot inside the disposable runner and prove the file exists. This is **NOT** a snapshot restore, external backup, multi-node failover, production tenant data, full Kubernetes conformance, or an independently accessible private control plane.
- No secrets or generated Kubeconfig are uploaded as CI artifacts; ephemeral runner cleanup deletes its test path. Some ephemeral K3s privileged processes/CNI routes exist only while the runner lives. A successful K3s CI run does not prove this host shares the live VPS's network or failure characteristics.

Run the portable, **safe** syntax and refusal test locally:
```bash
bash -n deploy/scripts/lab/k3s-ubuntu26-ephemeral-ci.sh
python3 -m unittest discover deploy/scripts/lab -p 'test_k3s_ubuntu26_ephemeral_review.py' -v
# Non-GitHub machines are deliberately refused by the executable script.
```

Actual K3s root execution occurs only in the new GitHub CI job `k3s-ubuntu26-disposable`. Its PASS/FAIL evidence must be read from the real GitHub Actions run. Do not claim it ran on Ubuntu 26.04 until that job passes.

## 3. Path from real single-node lab to controlled IPAT deployment

1. **Completed independently before live root install:** actual out-of-band console login including an alternate SSH failure route; complete encrypted off-host host-state recovery and an isolated independent rebuild; effective dedicated per-IPAT dual-stack ingress inventory and external denial test without editing other VPS groups; explicit ADR-017 choice of private inter-node transport, CNI, pod/service networks and multi-server embedded-etcd quorum design. The existing read-only R5.5 readiness script intentionally stays `NO_GO` until those independent gates receive evidence.
2. **Discrete disposable Ubuntu 26.04 rehearsal:** verify signed/hash-pinned package, systemd service lifecycle on a disposable VM, nftables/CNI compatibility and default-deny across both IP families, control-plane snapshot and restore into a **second** disposable machine, clean node rejoin/failure, private-only `6443`, restricted kubelet, isolated Flannel overlay (VXLAN UDP/8472 **not public**) and traffic probes. The current CI test is a prerequisite, not that entire gate.
3. **Controlled live transaction only after actual prerequisites:** independently test the rescue channel; capture and restore encrypted full host state; set the dedicated effective ingress baseline and retest from different authorized networks; approve a signed change+rollback; install the reviewed pinned K3s binary on the actual Ubuntu 26.04 host; validate zero public API/overlay listeners, strict key-only SSH, cluster health and time-limited tested independent rollback. Never install a production control plane directly using the CI-only runner profile.
4. **Next service milestones:** separately deploy restricted loopback-only `control-api` in the private cluster, then a trusted OIDC/POP binding and PostgreSQL runtime. Do not expose synthetic-only CWMP/USP/provisioning code, execute device writes or treat a single K3s node as production HA. At least three independently recoverable control-plane hosts are needed for a conventional three-member embedded-etcd HA design; the topology and availability targets remain subject to approval and measurements.

**External technical references:** official K3s Requirements and Configuration documentation, and the fixed tagged upstream K3s GitHub release. No hosting provider integration is included in IPAT.
