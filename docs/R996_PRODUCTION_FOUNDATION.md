# R9.96 — Reviewed first-install production foundation orchestrator

R9.96 reduces the production bootstrap to one first-install dependency sequence without exposing the server publicly.

## Scope

The orchestrator prepares, in this exact order:

1. R9.85 Platform Owner runtime binary/users/systemd, services disabled.
2. R9.92 commercial tenant runtime users/systemd, services disabled.
3. R9.94 customer ingress-verifier user/binary/systemd, timer disabled.
4. R9.95 DNS ownership-verifier user/systemd, service disabled.
5. R9.87 PostgreSQL18 cluster and restricted local Unix-socket peer identities.

It does **not** invoke the R9.84 public-IP HTTPS edge or R9.93 customer HTTPS ingress. It does not configure firewall, DNS, physical devices or public application listeners.

## Recovery gate

Before the first mutation, the root-only script requires an absolute root-owned 0600 single-link recovery attestation. The attestation must be no older than 14 days and must identify the exact source commit. It records previously reviewed evidence that provider rescue/root console, independent full-host restore and a separate backup destination have been verified.

The attestation is a deployment gate, not independent evidence by itself. Production PostgreSQL PITR remains deliberately false at foundation time because the real production cluster does not exist until this foundation is created. Public activation must remain blocked until a real off-host encrypted base/WAL restore, pg_verifybackup, named PITR, RLS validation and measured RPO/RTO have subsequently passed.

## First-install and rollback contract

The orchestrator refuses a mixed installation: any existing IPAT runtime identity, managed unit/binary, PostgreSQL cluster, or listener on managed ports requires a separate upgrade/recovery review.

If any preparation/bootstrap step fails, rollback disables application/verifier units, drops only the newly created 18/ipat cluster, removes the first-install IPAT runtime units/binaries/config directories and deletes only the newly created IPAT service identities. PostgreSQL packages may remain installed, but there is no surviving IPAT database cluster or public service.

On success it writes a root-only `/var/lib/ipat/r996-foundation.json` that records source/binary hashes, local PostgreSQL-only status, disabled application services and `public_go=false`.

## Current acceptance

Source tests verify ordering, pre-mutation recovery/source/listener checks, no public ingress/firewall/device call, fail-closed attestation semantics and rollback coverage. The current owner VPS still lacks noninteractive root access, so the live foundation has not been executed. That is an infrastructure acceptance blocker, not permission to bypass root/recovery controls.


## R10.20 — strict ownership of first-install rollback trees

The R9.96 first-install rollback includes recursive removal of three IPAT-owned OIDC/domain runtime-state directories. Earlier preflight checked individual unit/binary paths but not these directories and could erase pre-existing state on a failed retry. R10.20 adds fail-closed `r1014_require_absent` checks for `/opt/ipat`, `/etc/ipat`, `/var/lib/ipat`, `/var/lib/ipat-platform-oidc`, `/var/lib/ipat-tenant-oidc`, and `/var/lib/ipat-domain-ownership-verifier` before rollback is armed. Pre-existing directories, ordinary files or symlinks (including dangling) require separate recovery/migration review: never auto-clear these paths just to rerun the installer. Disposable tests prove rejection preserves existing directory contents. This does not authorize root installation without genuine provider rescue and independently tested full-host restore evidence.


## R10.21 — trusted, root-owned source and executable mandatory

For any root-executed R9.96 foundation, use a fresh, reviewed **root-owned** checkout located under root-owned, non-group/other-writable ancestor directories. Every file/dir in the checkout including .git must be root-owned and free of group/other write; symlinks and unsupported inode types are refused. Stage the exact checksum-verified `control-api` ELF64 binary likewise in a root-owned, trusted absolute path with only one hardlink. Git SHA/clean state and expected binary SHA256 remain separately required. Paths in `/home/openai/ipat-release-src-*` and `/home/openai/.local/share/ipat/release-candidates` are **source/build candidates**, not root-approved inputs. Never `chown -R` a live user development directory as a shortcut; use a separately reviewed root-only deployment source and preserve rollback/independent recovery gates. No public listener or production database installation is implied.
