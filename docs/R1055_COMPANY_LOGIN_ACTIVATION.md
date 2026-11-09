# R10.55 — Private-first commercial company login activation

**Status: NOT ACTIVE / PUBLIC PRODUCTION NO_GO.** Root-only environment staging and actual human MFA must not be misrepresented by source tests or open TCP ports.

## Secure metadata review on authorized VPS

Use your Mac Terminal to connect to the actual Linux VPS. First verify that the reviewed script hash matches the published milestone hash, then use an interactive sudo prompt to install a root-owned copy before running it.

    ssh -tt ipat-lab

Once logged into VPS, use the exact verified R10.55 hash from the release record:

    src=/home/openai/.local/share/ipat/release-operations/r1055-company-login-activation-audit.py
    expected=c421ab6b591b1c4273c1dc7b4f0ef7fd342db10daf09dd22955db06cdf966a7d
    test "$(sha256sum "$src" | cut -d" " -f1)" = "$expected"
    sudo install -o root -g root -m 0500 "$src" /root/ipat-r1055-company-login-activation-audit.py
    test "$(sudo sha256sum /root/ipat-r1055-company-login-activation-audit.py | cut -d" " -f1)" = "$expected"
    sudo python3 /root/ipat-r1055-company-login-activation-audit.py

Compare the root-owned installed hash with expected before running Python. This read-only script checks metadata and processes only; never reads/prints secrets. Two human company MFA and browser login are **always NOT VERIFIED** by this script. Never run user-writable code directly through sudo.

## Minimum actual activation conditions

1. Approve real external confidential OIDC/IdP (Keycloak is only a proposed candidate). Validate CA-trusted issuer discovery, independently vetted signing key and kid, confidential client ID and secret in restricted root/issuer-owned files; enroll two real company users in MFA and confirm fresh signed MFA assertion.
2. Provision a trusted private HTTPS hostname and approved CA for Platform Owner and each company, with exact Host/SNI and secure same-origin OIDC callback. Public DNS and final domain may follow later, but true TLS cannot be postponed for a functioning secure login. Never claim a test hostname or raw TCP reachability is identity authority.
3. Use legitimate operator root authorization and existing reviewed R9.85/R9.92/R9.98 guarded service activators, restricted PostgreSQL peer identities and rollback. Do not fake the trusted HTTPS environment flag. No root access or secrets should be sent to ChatGPT.
4. Perform real two-human-company browser MFA acceptance: login, exact dashboard/role menus, own data, foreign Host/cookie/CSRF denial, direct forbidden APIs, Site/POP separation, device/subscriber isolation, revocation, tenant suspension and logout. Record redacted HTTP evidence; never record cookies/tokens.
5. Production public GO also requires independent off-host PostgreSQL PITR/HA and remaining PRD/device interoperability checks.

## Verified, November 9, 2026

Owner Ubuntu26 VPS: PostgreSQL18/ipat online, commercial service units inactive, no TCP 3003/3005/3006/443 listener; private owner preview 3002 HTTP200. Real VPS isolated PG18 and Rust/Axum synthetic tenant isolation PASS, PR #249 and #250 four CI jobs PASS. Actual human OIDC/MFA company login remains NOT TESTED/NOT ACTIVE.
