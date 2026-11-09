#!/usr/bin/env python3
"""Non-mutating VPS commercial login readiness audit.

Never reads credentials or configuration contents, enumerates customer hosts,
prints tenant IDs, or claims MFA success. Unknown metadata is NOT a failure
and must not be mistaken for missing configuration.
"""
import json
import os
import pathlib
import re
import stat
import subprocess

ROOT = pathlib.Path("/etc/ipat")
SERVICES = ("ipat-platform-api.service", "ipat-platform-oidc.service",
            "ipat-tenant-api.service")
PORTS = (443, 3003, 3005, 3006)


def command(*args):
    try:
        result = subprocess.run(args, capture_output=True, text=True, timeout=4, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return result.stdout.strip() if result.returncode == 0 else None


def protected(path):
    """Metadata only. NO file reads, no symlink traversal, no secret output."""
    try:
        st = path.lstat()
    except (OSError, ValueError):
        return "missing"
    if not stat.S_ISREG(st.st_mode) or st.st_nlink != 1:
        return "unsafe"
    if st.st_uid != 0 or stat.S_IMODE(st.st_mode) != 0o600:
        return "unsafe"
    return "protected"


def inspect():
    privileged = os.geteuid() == 0
    pg = command("pg_lsclusters", "--no-header")
    pg18_online = bool(pg and any(
        len(cols := line.split()) >= 4 and cols[0:2] == ["18", "ipat"] and cols[3] == "online"
        for line in pg.splitlines()))
    states = {}
    for service in SERVICES:
        states[service.removesuffix(".service")] = command(
            "systemctl", "is-active", service) == "active"
    socket = command("ss", "-H", "-ltn")
    listening = set()
    if socket is not None:
        for line in socket.splitlines():
            cols = line.split()
            if len(cols) > 3:
                p = cols[3].rsplit(":", 1)[-1]
                if p.isdecimal():
                    listening.add(int(p))
    if privileged:
        try:
            oidc_dir = ROOT / "tenant-oidc"
            # Do not follow paths across an untrusted parent.
            safe_parent = (ROOT.lstat().st_uid == 0 and
                           stat.S_ISDIR(ROOT.lstat().st_mode) and
                           stat.S_IMODE(ROOT.lstat().st_mode) & 0o022 == 0)
            safe_oidc_dir = oidc_dir.is_dir() and not oidc_dir.is_symlink() and (
                oidc_dir.lstat().st_uid == 0 and
                stat.S_IMODE(oidc_dir.lstat().st_mode) & 0o022 == 0)
            if safe_parent and safe_oidc_dir:
                instances = [protected(f) for f in oidc_dir.iterdir()
                             if re.fullmatch(r"[a-z0-9][a-z0-9-]{0,47}\.env", f.name)]
                oidc_protected = sum(v == "protected" for v in instances)
                oidc_unsafe = sum(v != "protected" for v in instances)
            else:
                oidc_protected = 0
                oidc_unsafe = -1
            envs = {name: protected(ROOT / f"{name}.env") if safe_parent else "unsafe"
                    for name in ("platform-api", "platform-oidc", "tenant-api")}
        except (OSError, ValueError):
            envs = {name: "unknown" for name in ("platform-api", "platform-oidc", "tenant-api")}
            oidc_protected, oidc_unsafe = 0, -1
    else:
        envs = {name: "unknown_requires_root"
                for name in ("platform-api", "platform-oidc", "tenant-api")}
        oidc_protected, oidc_unsafe = None, None
    return {
        "scope": "read_only_metadata_only_no_customer_secrets",
        "postgres18_ipat_online": pg18_online,
        "commercial_services_active": states,
        "listeners": {str(p): p in listening if socket is not None else None for p in PORTS},
        "protected_environment_files": envs,
        "protected_tenant_oidc_instances": oidc_protected,
        "unsafe_tenant_oidc_instances": oidc_unsafe,
        "root_metadata_inspection": privileged,
        "oidc_human_mfa_accepted": False,
        "two_company_real_browser_login_accepted": False,
        "public_production_go": False,
    }


if __name__ == "__main__":
    print(json.dumps(inspect(), sort_keys=True, indent=2))
