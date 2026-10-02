# R9.72 — Commercial NOC read-only POP-scoped inventory

Canonical source: GitHub main R9.71 (446e5b8). Earlier local R9.54–R9.56 experiment is NOT merged: newer canonical migrations already own numbers 0013–0016.

Under an independently signed, current Host-bound tenant browser session, the backend derives current exact NOC POP grants through a sealed restricted SQL function. NOC sees only the NOC Inventory module and chooses among granted POP codes. GET-only Site/Device routes independently verify Host session and exact POP. No NOC mutation controls or physical commands are rendered; existing admin-only CRUD SQL policy remains independent.

Session and scoped SQL do not reveal credentials, endpoint addresses or privileged capability. Missing/revoked/expired/suspended memberships, forged POP or cross-host cookie replay must fail closed.

Reproduction: use opt-in IPAT_R972_SYNTHETIC_DOCKER=YES with deploy/db/tests/r972_docker_repro.sh on an isolated Docker computer. It starts a disposable no-public-port PostgreSQL16 instance and tests login-role split, current admin rights, NOC grants and negative cases. Axum+real ephemeral PostgreSQL integration is separately required. This source is NOT a public deployment or physical equipment adoption.
