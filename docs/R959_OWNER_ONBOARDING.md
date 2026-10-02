# R9.59 — Unblock first real C320 owner enrollment without exposing credentials

**Live observed before release:** 2026-10-02, current private Mac SSH tunnel to VPS 127.0.0.1:3002 returned HTTP 200 for dashboard/workbench/owner status, direct fixed SSH TCP probe `TCP_REACHABLE_AUTH_NOT_TESTED`; private C320 connector reported `draft_saved=true`, `credentials_enrolled=false`, `device_status=PENDING`, `production_adopted=false`, `physical_writes_enabled=false`. Owner-Mac SSH alias works; remote owner-code file exists mode 0600. Mac clipboard **at observation time** did not match the private file (later clipboard changes cannot establish whether an earlier pbcopy ever succeeded). Neither token contents nor device password were disclosed or stored in project evidence. Owner previously provided lab Telnet port :323; the **currently deployed** pinned read-only connector only supports separately observed SSH :321. Do not silently change protocol or call a reachable port authenticated.

## Usable owner procedure, no secret in chat

1. On authorized Mac, keep the existing SSH local-forward open, visit `http://127.0.0.1:3002/lab/device-workbench`. The physical-device row is **already saved as draft**. Do not add a duplicate or enter a password into the synthetic demo form in advanced diagnostics.
2. In first physical ZTE C320 form keep the exact tested pinned profile (SSH, port 321, dedicated username shown by dashboard). Enter the actual device password only in the private browser's SSH Password field. If the OLT has only Telnet port 323 enabled at present, no current authenticated SSH driver can be inferred: the owner must independently configure/verify SSH according to the device's exact firmware, or IPAT must implement and separately qualify a lab-only Telnet adapter; neither protocol shares a port by assumption.
3. Expand the now-default-open `LANGKAH WAJIB` section and use **Copy Command**. This puts the *command text* onto the clipboard; run it in your own Mac Terminal. It uses zsh `pipefail` so a failed SSH operation cannot print success. If the terminal prints `IPAT: KODE TERSALIN`, it transferred the output to clipboard without echoing secret content. Return immediately to private browser; **do not copy any other text**. Click One-Time Owner Code input and press Command+V. This code is not the OLT password and should never be sent through ChatGPT, shell history as plaintext, issue trackers or GitHub.
4. The page now detects mistakenly pasted command text, displays the saved-draft state after a refresh, and labels the action **Lanjutkan Koneksi SSH**. Enter both owner code + SSH password and click it. A real read-only authenticated card test must pass before `CONNECTED`; wrong code/password or untested CLI prompts stay visibly pending with a bounded error. Actual config, ONT provisioning, flashing, live production adoption, public tenant domain and real MFA remain independent unavailable release gates.

## Verification

```
python3 -m unittest discover deploy/scripts/lab/r959 -p test_owner_enrollment_ux.py -v
node deploy/scripts/lab/r947/test_add_device.cjs
python3 -m unittest discover deploy/scripts/lab/r83 -p test_r83_contract.py -v
node --check web/lab/c320-connection-setup.js
cargo test --locked -p control-api
```

CI must also pass the full locked workspace and disposable PostgreSQL/restore/K3s suite. A separate rootless rollback-capable private :3002 binary rollout is permitted only after reviewing the exact source SHA and the already-running canary's user-owned systemd definition. Keep previous binary intact and restart it on smoke-test failure. Never open public :443, change root firewall, alter physical OLT, copy secret into output, or modify production database as part of this UX fix.
