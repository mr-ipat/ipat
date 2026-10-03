# R9.75 — Platform-curated vendor catalog and guarded registration

The company Device Add experience obtains type, vendor, and permitted metadata protocols from a restricted PostgreSQL platform catalog. A metadata candidate means only that IPAT can persist the selected metadata; it NEVER proves exact physical model/firmware/protocol interoperability.

Operator flow: create a separate Site/POP first, choose a listed device type then vendor then allowed protocol. The UI displays the selected protocol's qualification limitations. The authenticated tenant API retrieves the same current catalog, and a separate BEFORE INSERT guard independently denies missing or disabled catalog entries, including legacy SQL callers. A disabled row preserves old saved inventory, but prevents new metadata registration.

Catalog administration is an independent Platform Owner privilege, not a tenant-admin ability. No Platform Owner catalog-editing HTTP endpoint is mounted without independently verified real MFA, strict review and audit. No hardware connection, firmware update, vendor support attestation or device management credentials are added by this work.

Reproduction command: IPAT_R975_SYNTHETIC_DOCKER=YES bash deploy/db/tests/r975_docker_repro.sh. It runs canonical selected dependency migrations through 0027 inside a disposable no-published-port PostgreSQL16 container with 22 role/guard/disabled/revocation regression tests. Axum+real PostgreSQL acceptance is separately required before source merge; actual customer/public and hardware acceptance remain independent blockers.
