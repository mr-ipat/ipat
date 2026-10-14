#!/usr/bin/env bash
# Read-only commercial-runtime inventory. No sudo, no secrets, no implied GO.
set -euo pipefail
export LC_ALL=C
printf 'IPAT_R1024_READ_ONLY_RUNTIME_INVENTORY\n'
if [[ -r /etc/os-release ]]; then
  . /etc/os-release
  printf 'OS_ID=%s\nOS_VERSION_ID=%s\n' "${ID:-unknown}" "${VERSION_ID:-unknown}"
fi
for u in ipatpapi ipatpoidc ipattapi ipattoidc ipatdverify ipatdnsverify ipatpgmigrate; do
  if getent passwd "$u" >/dev/null 2>&1; then state=PRESENT; else state=ABSENT_OR_INACCESSIBLE; fi
  printf 'IDENTITY_%s=%s\n' "$u" "$state"
done
for u in ipat-platform-api.service ipat-platform-oidc.service ipat-tenant-api.service ipat-domain-ownership-verifier.service; do
  if systemctl is-active --quiet "$u" 2>/dev/null; then state=ACTIVE; else state=NOT_ACTIVE_OR_INACCESSIBLE; fi
  printf 'SERVICE_%s=%s\n' "$u" "$state"
done
if command -v ss >/dev/null 2>&1; then
  for p in 80 443 5432 3003 3005 3006; do
    if ss -H -ltn "( sport = :$p )" 2>/dev/null | grep -q .; then state=LISTENING; else state=NOT_OBSERVED; fi
    printf 'PORT_%s=%s\n' "$p" "$state"
  done
else
  printf 'PORT_INSPECTION=UNAVAILABLE\n'
fi
printf 'ATTESTATION=NOT_ASSESSED_REQUIRES_ROOT_REVIEW\n'
printf 'RECOVERY_RESTORE=NOT_ASSESSED_REQUIRES_INDEPENDENT_EVIDENCE\n'
printf 'PUBLIC_GO=NOT_DETERMINED\n'
