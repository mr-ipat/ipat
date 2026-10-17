#!/usr/bin/env python3
"""Non-mutating, non-privileged production foundation visibility check.

No authentication, secrets, database queries, service starts, or public listeners.
This is NOT a production release gate.
"""
import argparse
import json
import subprocess
import urllib.error
import urllib.request


def command_output(argv, timeout=5):
    try:
        result = subprocess.run(argv, text=True, capture_output=True,
                                timeout=timeout, check=False)
        return result.returncode, result.stdout
    except (OSError, subprocess.TimeoutExpired):
        return 127, ""


def probe():
    code, output = command_output(["pg_lsclusters", "--no-header"])
    postgres = code == 0 and any(
        len(fields := line.split()) >= 4 and fields[:2] == ["18", "ipat"]
        and fields[3] == "online" for line in output.splitlines()
    )
    try:
        request = urllib.request.Request(
            "http://127.0.0.1:3002/healthz", method="GET"
        )
        with urllib.request.urlopen(request, timeout=3) as response:
            private_health = response.status == 200
    except (OSError, urllib.error.URLError, ValueError):
        private_health = False
    code, output = command_output(["ss", "-H", "-ltn"])
    listeners = output if code == 0 else ""
    def listening(port):
        return any(
            line.split()[-2].rsplit(":", 1)[-1] == str(port)
            for line in listeners.splitlines() if len(line.split()) >= 5
        )
    return {
        "postgres18_ipat_online": postgres,
        "owner_private_health_http200": private_health,
        "public_tcp443_absent": code == 0 and not listening(443),
        "commercial_api_ports_absent": code == 0 and
            all(not listening(p) for p in (3003, 3005, 3006)),
        "verification_scope": "read_only_nonroot_no_production_release",
        "public_go": False,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    result = probe()
    print(json.dumps(result, sort_keys=True, indent=2 if args.json else None))
    return 0 if all(result[key] for key in (
        "postgres18_ipat_online", "owner_private_health_http200",
        "public_tcp443_absent", "commercial_api_ports_absent")) else 2


if __name__ == "__main__":
    raise SystemExit(main())
