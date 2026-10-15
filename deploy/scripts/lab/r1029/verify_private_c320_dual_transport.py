#!/usr/bin/env python3
"""Strict, read-only live acceptance of the existing loopback IPAT C320 panel.

Run on the authorized operator Mac with the SSH tunnel bound to 127.0.0.1:3002,
or on the IPAT VPS. Never accepts arbitrary hosts, device commands or secrets.
"""
from __future__ import annotations

from datetime import datetime, timezone, timedelta
import json
import sys
import urllib.request

BASE = "http://127.0.0.1:3002"
MAX_BYTES = 4096


def live_fetch(path: str) -> bytes:
    if path not in (
        "/lab/device-workbench",
        "/lab/c320-owner-network-probe",
        "/lab/c320-owner-connection",
    ):
        raise ValueError("unapproved diagnostic path")
    request = urllib.request.Request(BASE + path, headers={"Cache-Control": "no-store"})
    with urllib.request.urlopen(request, timeout=7) as response:
        if response.status != 200:
            raise ValueError("private dashboard HTTP status != 200")
        limit = 256_000 if path == "/lab/device-workbench" else MAX_BYTES
        payload = response.read(limit + 1)
        if len(payload) > limit:
            raise ValueError("response exceeds bounded limit")
        return payload


def verify(fetch=live_fetch, now=None) -> tuple[str, ...]:
    now = now or datetime.now(timezone.utc)
    html = fetch("/lab/device-workbench")
    if b'id="ipat-c320-check-network"' not in html:
        raise ValueError("new private network-check button not deployed")
    diagnostics = json.loads(fetch("/lab/c320-owner-network-probe"))
    if diagnostics.get("target") != "DEV-01":
        raise ValueError("unexpected device ID")
    for protocol, port, signature in (
        ("ssh", 321, "server_banner_observed"),
        ("telnet", 323, "negotiation_observed"),
    ):
        result = diagnostics.get(protocol)
        if not isinstance(result, dict) or result.get("port") != port:
            raise ValueError(f"{protocol} target contract mismatch")
        if (result.get("tcp_reachable") is not True
                or result.get("probe_stage") != "TCP_REACHABLE_AUTH_NOT_TESTED"
                or result.get(signature) is not True):
            raise ValueError(f"{protocol} physical transport proof missing")
    for name in (
        "ssh_authentication_verified",
        "telnet_authentication_verified",
        "device_identity_verified_by_network_probe",
        "production_adopted",
        "physical_writes_enabled",
    ):
        if diagnostics.get(name) is not False:
            raise ValueError("unsafe network probe elevation claim")
    connection = json.loads(fetch("/lab/c320-owner-connection"))
    if (connection.get("target") != "DEV-01"
            or connection.get("device_status") != "CONNECTED"
            or connection.get("connector_online") is not True
            or connection.get("credentials_enrolled") is not True
            or connection.get("production_adopted") is not False
            or connection.get("physical_writes_enabled") is not False):
        raise ValueError("private authenticated read-only connector not connected")
    stamp = connection.get("last_verified_at_utc")
    if not isinstance(stamp, str):
        raise ValueError("missing authenticated read timestamp")
    measured = datetime.fromisoformat(stamp.replace("Z", "+00:00"))
    if measured.tzinfo is None or not (timedelta(seconds=-30)
                                      <= now - measured <= timedelta(minutes=6)):
        raise ValueError("C320 authenticated read is stale or clock invalid")
    return (
        "R1029_PRIVATE_DASHBOARD_BUTTON=PASS",
        "R1029_C320_SSH321_TCP_BANNER=PASS",
        "R1029_C320_TELNET323_TCP_IAC=PASS",
        "R1029_C320_AUTHENTICATED_READ_ONLY_CONNECTOR=PASS",
        "R1029_PRODUCTION_ADOPTED=FALSE",
        "R1029_PUBLIC_GO=FALSE",
    )


if __name__ == "__main__":
    try:
        print("\n".join(verify()))
    except (OSError, ValueError, KeyError, UnicodeError, json.JSONDecodeError) as err:
        print(f"R1029_PRIVATE_C320_ACCEPTANCE_FAILED={type(err).__name__}", file=sys.stderr)
        sys.exit(1)
