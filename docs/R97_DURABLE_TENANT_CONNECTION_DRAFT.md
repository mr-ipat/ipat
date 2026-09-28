# R9.7 — Durable tenant network method drafts, nonexecutable

Date: 2026-09-28 Asia/Jakarta. STATUS: proposed disposable-only
migration `0010_lab_connection_drafts.sql` and CI test, pending
independent full GitHub checks; NEVER migrate customer database.

The owner requests tenant-admin-controlled tunneling and nonintrusive
real ZTE C320 adoption. This release adds the NEXT safe state boundary:
tenant-admin-proposed method/gateway choices are persisted with strict
tenant+candidate+POP and real SQL membership checks, exactly one
immutable draft per candidate and same-transaction immutable audit.
Only enums for direct_secure, WireGuard and IPsec plus gateway class
are permitted. No IP, VLAN, password, SSH, private WireGuard key,
peer endpoint or route is stored; there is NO activator or tunnel job.
RouterOS6+built-in-WireGuard and public Telnet methods are not allowed.

A separately owned no-login SECURITY DEFINER function grants EXECUTE
only to its own dedicated proposal role. Existing restricted read
role can list only own current tenant-admin drafts. No general runtime
role, public SQL client or NOC receives raw table grants. SQL caller
identity is still untrusted absent genuine IdP/MFA and sealed opaque
session; accordingly no real HTTP endpoint is mounted for this function.

CI adds a real disposable PostgreSQL 16 ordered migration/test AFTER
R9.2, requiring previously seeded two synthetic companies and signed
identity fixture. It tests SQL role isolation, invalid methods,
wrong tenant/POP/actor, idempotence, append-only audit and immutable
provisioning_enabled=false. A local skip WITHOUT disposable PG is NOT
a positive integration result. Do not merge until all CI jobs pass.

NEXT: bind a REAL confidential IdP MFA browser tenant admin session to
separate SQL EXECUTE-only PostgreSQL role (still unmounted until live
IdP); then separately design owner-approved WireGuard/IPsec site
transport templates and staged console-backed rollback under manual
independent activation. Actual C320 authenticated TC-OLT-01 remains
BLOCKED by public plaintext Telnet, unknown local isolation and actual
worker reachability. No device, firmware, ONT or PPPoE configuration
changes are permitted by this migration.
