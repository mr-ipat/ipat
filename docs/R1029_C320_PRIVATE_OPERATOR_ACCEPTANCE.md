# R10.29 — Private C320 network acceptance

The accepted mode is an authorized operator Mac with an SSH tunnel to the existing owner-private IPAT dashboard, or a direct console on the VPS. The page is `http://127.0.0.1:3002/lab/device-workbench`. Choose **Periksa jaringan SSH & Telnet** to check only the registered lab ZTE C320 ports; the network test does not use passwords or send any CLI commands.

To rerun the verified acceptance from the repository root, with the local tunnel already active:

```bash
python3 deploy/scripts/lab/r1029/verify_private_c320_dual_transport.py
python3 -m unittest discover -s deploy/scripts/lab/r1029 -p 'test_*.py' -v
```

Expected: SSH321 TCP and SSH identification PASS; Telnet323 TCP and Telnet IAC PASS; existing enrolled read-only C320 connector CONNECTED with a fresh verification timestamp. The script fails closed for missing transport, stale connector read, unexpected device identity, or an API claiming physical writes or production adoption. It uses fixed loopback endpoints only. It is not a public SaaS smoke test. Production PostgreSQL, recovery attestation, MFA and TLS have separate acceptance gates.
