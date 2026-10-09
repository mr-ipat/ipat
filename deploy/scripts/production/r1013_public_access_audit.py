#!/usr/bin/env python3
"""Read-only, fail-closed actual VPS/public IP access diagnostic.

This does not approve public launch or modify hosts, DNS, firewall, DB or devices.
Human rescue, recovery, PITR, IdP MFA and external browser acceptance remain
independent mandatory release gates regardless of this report.
"""
from __future__ import annotations

import argparse
import json
import subprocess

import r982_temporary_ip_tls_preflight as tls_preflight

SERVICE_UNITS = {
    "nginx": "nginx.service",
    "postgresql": "postgresql.service",
    "k3s": "k3s.service",
    "platform_api": "ipat-platform-api.service",
    "platform_oidc": "ipat-platform-oidc.service",
    "tenant_api": "ipat-tenant-api.service",
}
SSH_ARGS = [
    "ssh", "-T", "-o", "BatchMode=yes",
    "-o", "StrictHostKeyChecking=yes", "-o", "ConnectTimeout=7",
    "-o", "ConnectionAttempts=1", "ipat-lab",
]
# Only exact, static allowlisted units and marker paths are inspected.
READONLY = """for name in nginx postgresql k3s platform_api platform_oidc tenant_api; do
case "$name" in
  nginx) unit=nginx.service ;;
  postgresql) unit=postgresql.service ;;
  k3s) unit=k3s.service ;;
  platform_api) unit=ipat-platform-api.service ;;
  platform_oidc) unit=ipat-platform-oidc.service ;;
  tenant_api) unit=ipat-tenant-api.service ;;
esac
state=$(systemctl is-active "$unit" 2>/dev/null || true)
case "$state" in active|inactive|failed|activating|deactivating) ;; *) state=unknown ;; esac
printf 'UNIT_%s=%s\\n' "$name" "$state"
done
for item in foundation edge; do
case "$item" in
 foundation) path=/var/lib/ipat/r996-foundation.json ;;
 edge) path=/var/lib/ipat/r997-platform-edge.json ;;
esac
if test -f "$path" && test ! -L "$path"; then
  printf 'MARKER_%s=PRESENT\\n' "$item"
else
  printf 'MARKER_%s=NOT_OBSERVED\\n' "$item"
fi
done"""


def services_observation() -> dict:
    units = {name: "unknown" for name in SERVICE_UNITS}
    result = {"ssh_unit_check_succeeded": False, "services": units,
              "foundation_marker": "NOT_OBSERVED", "edge_marker": "NOT_OBSERVED"}
    try:
        proc = subprocess.run(
            [*SSH_ARGS, READONLY], capture_output=True, text=True,
            check=False, timeout=13,
        )
    except (OSError, subprocess.TimeoutExpired):
        return result
    if proc.returncode != 0:
        return result
    result["ssh_unit_check_succeeded"] = True
    for line in proc.stdout.splitlines():
        key, sep, value = line.partition("=")
        if not sep:
            continue
        if key.startswith("UNIT_") and key[5:] in units:
            if value in {"active", "inactive", "failed", "activating", "deactivating", "unknown"}:
                units[key[5:]] = value
        elif key == "MARKER_foundation" and value in {"PRESENT", "NOT_OBSERVED"}:
            result["foundation_marker"] = value
        elif key == "MARKER_edge" and value in {"PRESENT", "NOT_OBSERVED"}:
            result["edge_marker"] = value
    return result


def evaluate(preflight: dict, observation: dict) -> dict:
    units = observation.get("services") or {}
    checks = {
        "ssh_verified": preflight.get("private_vps_ssh_observed") is True
            and observation.get("ssh_unit_check_succeeded") is True,
        "configured_ip_matches": preflight.get("target_matches_existing_private_owner_plan") is True,
        "vps_https_listener": preflight.get("vps_443_listener_observed") is True,
        "external_https_reachable": preflight.get("internet_443_reachable") is True,
        "public_ca_trusted_ip_san": preflight.get("browser_ca_trusted_exact_ipv4_san_tls") is True,
        "foundation_marker_observed": observation.get("foundation_marker") == "PRESENT",
        "edge_marker_observed": observation.get("edge_marker") == "PRESENT",
        "nginx_running": units.get("nginx") == "active",
        "postgresql_running": units.get("postgresql") == "active",
        "platform_api_running": units.get("platform_api") == "active",
        "platform_oidc_running": units.get("platform_oidc") == "active",
        "tenant_api_running": units.get("tenant_api") == "active",
    }
    blockers = [name for name, ok in checks.items() if not ok]
    # Staging can use no K3s yet, but K3s/HA remains a mandatory full PRD gate.
    external = [
        "independent_provider_rescue_root_login",
        "offhost_full_host_restore_and_rollback",
        "independent_production_postgresql_pitr_and_ha",
        "real_operator_oidc_mfa_and_revocation",
        "external_wrong_host_sni_and_cross_tenant_browser_tests",
        "reviewed_public_ipv4_ipv6_exposure",
    ]
    return {
        "schema": "ipat.public-access-readonly.v1",
        "observed_at_utc": preflight.get("observed_at_utc"),
        "target_ip": preflight.get("target_ip"),
        "scope": "LIVE_READ_ONLY_NOT_DEPLOYMENT_APPROVAL",
        "checks": checks,
        "service_states": {name: units.get(name, "unknown") for name in SERVICE_UNITS},
        "missing_check_ids": blockers,
        "independent_evidence_required": external,
        "k3s_state": units.get("k3s", "unknown"),
        "public_go": False,
        "production_status": "NO_GO",
        "mutations_attempted": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ip", required=True, help="Reviewed literal public IPv4")
    args = parser.parse_args()
    # Do not SSH or initiate connections for non-public or malformed targets.
    if not tls_preflight.eligible(args.ip):
        parser.error("eligible exact global IPv4 required")
    try:
        report = tls_preflight.report(args.ip)
        observed = services_observation()
    except (OSError, ValueError, subprocess.SubprocessError):
        # Never turn an inability to inspect production into a positive result.
        report = {"target_ip": args.ip}
        observed = {}
    print(json.dumps(evaluate(report, observed), sort_keys=True, indent=2))
    return 3  # Always nonzero until separate independently approved release acceptance.


if __name__ == "__main__":
    raise SystemExit(main())
