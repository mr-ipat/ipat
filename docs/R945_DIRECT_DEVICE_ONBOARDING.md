# R9.45 — Direct-first device onboarding without repeated local terminal authentication

Date: 2026-09-29. Aligned with `IPAT_PROJECT_BRIEF.md`, latest `docs/PRD.md`, `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/DEVICE_MATRIX.md`, `docs/DECISIONS.md` and `docs/PROJECT_STATUS.md`.

## Product logic

1. An operator selects a device and management path. If the management address is reachable **FROM THE VPS OR THE CORRECT PRIVATE POP WORKER**, connect directly using a verified supported management protocol. Do **not** force a tunnel merely because a management address is private.
2. When the device is not reachable, select a site gateway, create and verify a scoped management-only WireGuard tunnel (or a separately reviewed IPsec adapter), measure reachability and authorize the route. A tunnel is a transport, NOT an authorization mechanism and should never automatically expose production/customer subnets.
3. Enter credentials **once in an authenticated GUI**. The server stores them in an encrypted secret store outside the database/repository, has a bounded background health collector and returns timestamped real data. Device inventory should distinguish `DRAFT`, `UNREACHABLE`, `CONNECTED_READ_ONLY`, `ADOPTED_MANAGED`, and `DEGRADED`; never label a network-connected device as fully managed without authentication + actual inventory and verified operation scopes.
4. Each supported device adapter has typed, vendor- and firmware-specific READ operations. Actual configuration changes use explicit separate permissions, audited commands, bounded per-device locks, native backups and tested rollback. No universal arbitrary CLI/HTTP relay.
5. The full product requires tenant OIDC+MFA, RBAC/ABAC denial in both UI and backend, POP scope, secrets vault with separate decryption key and rotation, SSH host-key OOB verification (or per-protocol TLS/SNMPv3 identity), device inventory PostgreSQL RLS, K3s worker queue and immutable audit. Source-only lab shortcuts must not be published as production access.

## Implemented in R9.45 *first-device private laboratory slice*

- New `deploy/scripts/lab/r945/persistent_c320_connector.py`: fixed owner-lab ZTE C320 private management IP `10.10.13.233`, SSH port `321`, previously owner-network-observed strict SSH RSA pin, existing firmware-observed read commands and no write commands. Linux nonroot user systemd service (deployment script to be validated separately) remains running without owner Mac TTY; direct VPS network TCP to this port was **actually checked reachable**. It uses local Unix socket mode 0600 under owner 0700 path; Linux SO_PEERCRED disallows other UIDs; only allowlisted `STATUS`, `ENROLL`, `REFRESH`, `CARDS`, `FIRMWARE`. Background `show card` health check every five minutes while enrolled, one physical read at a time.
- One-time owner bootstrap code separate from device password; generated randomly at VPS deployment outside Git, exposed only by user-controlled Mac clipboard through an authorized encrypted Mac→VPS SSH session; secret file mode 0600. Browser enrollment uses the **private SSH-forwarded** `127.0.0.1:3002` laboratory dashboard with exact Host + Origin + CSRF-style header, tight JSON body and no arbitrary device address/CLI/user-specified protocol. Invalid attempts are bounded; token is consumed after successful actual SSH authentication and card-read verification. Credential is persisted only AFTER that actual device response, encrypted using `cryptography.Fernet` with separately permission-restricted key file in an owner-private directory. NOTE: both key and ciphertext on one VPS account are an interim LAB measure, **NOT the final separately managed commercial vault**. Do not display or commit real password or bootstrap token.
- Added strict private Rust `POST /lab/c320-owner-enroll`, `GET /lab/c320-owner-connection` with allowlisted responses, separate from public SaaS. New browser `c320-connection-setup.js` + enrollment form inside operator GUI, no credentials in browser storage and immediate form clearing; prior read buttons can use the persistent Unix connector without operator Terminal or five-read/15-minute process expiration. Owner input of device password on GUI once is unavoidable and intentional; no repeating local CLI authentication. Direct initial connect is not production device adoption.
- Legacy in-memory fake candidate/simulation panels are moved under closed-by-default 'Pengaturan lanjutan dan diagnostik pengembangan' so user sees the actual operational controls first; no old demo backend APIs were globally removed, preserving regression while we replace them.
- Initially **only the physically evidenced fixed C320 target** may use this lab enrollment; other OLTs, ONTs and routers require new typed adapters and per-tenant provisioning. IPAT supports WireGuard/IPsec transport as architectural choices, but **R9.45 does not claim a working generalized tunnel provisioning API or production device registry**.
- Credential enrollment on this private lab route uses a one-time owner bootstrap instead of actual tenant login because the :3002 preview explicitly has no validated OIDC session; it must remain loopback-only, not be proxied publicly. Replace it with real tenant OIDC+MFA/ABAC before any commercial multi-user use. Do not assume private browser Host or a valid tunnel implies admin privileges.

## Acceptance matrix

| Priority | Acceptance | Evidence required |
|---|---|---|
| MUST | VPS can connect to physical C320 management address on SSH port 321 | Actual non-mutating socket connection |
| MUST | Persistent user service boots without temporary owner-TTY dependency | Actual systemd service status and Unix status response |
| MUST | One-time enrollment from **private** dashboard actually validates SSH and stores ciphertext only on successful physical card read | Fresh owner browser activation; no real credential in logs/chat |
| MUST | After enrollment and across connector restart, dashboard reads C320 without prompting again | Fresh post-restart real timestamped card+PON data |
| MUST | Connector refuses arbitrary IP/CLI/other UID, browser origin forgery and physical firmware/ONU mutations | Programmatic negative tests plus deployed HTTP |
| MUST | Old :3000 continues running and previous :3002 has a rollback image | Before/after HTTP and pinned unit/binary hashes |
| SHOULD | Generic typed per-tenant registry, one-click direct-first or verified management tunnel workflow, scoped secret vault | Tenant auth, POP scope and actual physical device read evidence |
| SHOULD | Live per-ONU optics/traffic and firmware-specific board detail after real adapter evidence | Physical firmware CLI and redacted validated parser |
| LATER | C-DATA/MikroTik/ONT heterogeneous worker adapters and high-risk tenant write approvals | Physical matrix + vendor-native rollback evidence |

**Do not claim R9.45 has passed a test or has been deployed unless later PROJECT_STATUS entries contain actual tool evidence.** The R9.44 GUI is currently the last independently verified private runtime. Genuine physical SSH321 TCP reachability by itself is NOT proof of correct SSH password, fresh inventory or adopted device.
