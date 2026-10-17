#!/usr/bin/env python3
"""Read-only host inspection; never asserts identity or production acceptance."""
import json
import subprocess
import urllib.request

def run(*argv):
    try:
        p = subprocess.run(argv, capture_output=True, text=True, timeout=5, check=False)
        return p.returncode, p.stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        return 127, ""

def report():
    _, units = run("systemctl", "is-active", "ipat-tenant-api.service",
                   "ipat-platform-api.service", "ipat-platform-oidc.service")
    states = units.splitlines()
    db_code, db = run("pg_lsclusters", "--no-header")
    pg = db_code == 0 and any(
        v[:2] == ["18", "ipat"] and len(v) > 3 and v[3] == "online"
        for line in db.splitlines() if len(v := line.split()) > 3
    )
    code, sock = run("ss", "-H", "-ltn")
    listening = {x.split()[-2].rsplit(":", 1)[-1] for x in sock.splitlines()
                 if len(x.split()) >= 5} if code == 0 else set()
    return {
        "postgres18_ipat_online": pg,
        "tenant_api_active": len(states) > 0 and states[0] == "active",
        "platform_api_active": len(states) > 1 and states[1] == "active",
        "platform_oidc_active": len(states) > 2 and states[2] == "active",
        "tenant_api_port_3003_listening": "3003" in listening,
        "public_https_port_443_listening": "443" in listening,
        "human_oidc_mfa_verified": False,
        "customer_login_verified": False,
        "public_production_go": False,
        "scope": "nonroot_read_only_configuration_no_secrets",
    }

if __name__ == "__main__":
    print(json.dumps(report(), indent=2, sort_keys=True))
