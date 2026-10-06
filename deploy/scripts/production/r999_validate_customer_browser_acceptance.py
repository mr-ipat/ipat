#!/usr/bin/env python3
"""Validate exact customer human-MFA/browser isolation evidence.

This does not execute a browser or authenticate a user. It validates a reviewed,
root-controlled acceptance artifact produced by an independent real-browser run.
"""
from __future__ import annotations
import argparse, json, re, stat, uuid
from datetime import datetime, timezone, timedelta
from pathlib import Path

SAFE = re.compile(r"^[A-Za-z0-9_.:@/-]{3,180}$")
HOST = re.compile(r"^[a-z0-9](?:[a-z0-9.-]{1,251}[a-z0-9])$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
REQUIRED_BOOL = (
    "real_human_mfa_browser_login_verified",
    "unauthorized_menu_hidden_verified",
    "unauthorized_direct_url_denied_verified",
    "unauthorized_api_denied_verified",
    "wrong_host_session_replay_denied",
    "cross_tenant_api_denied",
    "logout_revocation_verified",
    "stale_session_denied",
)

def secure_json(path: Path, uid: int = 0) -> dict:
    if not path.is_absolute() or path.is_symlink():
        raise ValueError("evidence path must be absolute non-symlink")
    st = path.stat()
    if (
        not stat.S_ISREG(st.st_mode)
        or st.st_uid != uid
        or stat.S_IMODE(st.st_mode) != 0o600
        or st.st_nlink != 1
    ):
        raise ValueError("evidence file must be owner-only 0600 single-link")
    if not 80 <= st.st_size <= 32768:
        raise ValueError("evidence size invalid")
    try:
        data = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as e:
        raise ValueError("invalid JSON") from e
    if not isinstance(data, dict):
        raise ValueError("evidence root must be object")
    return data

def exact_host(value: str) -> str:
    if (
        not isinstance(value, str)
        or value != value.lower()
        or len(value) > 253
        or "." not in value
        or ":" in value
    ):
        raise ValueError("invalid exact hostname")
    if value.endswith((".local", ".localhost", ".invalid", ".test", ".example")) or not HOST.fullmatch(value):
        raise ValueError("invalid exact hostname")
    for label in value.split("."):
        if not 1 <= len(label) <= 63 or label.startswith("-") or label.endswith("-"):
            raise ValueError("invalid hostname label")
    return value

def uid(value: str) -> str:
    try:
        return str(uuid.UUID(str(value)))
    except ValueError as e:
        raise ValueError("invalid UUID") from e

def validate(
    activation: dict,
    acceptance: dict,
    domain_id: str,
    hostname: str,
    now: datetime | None = None,
) -> dict:
    domain_id = uid(domain_id)
    hostname = exact_host(hostname)
    if activation.get("schema") != "ipat.customer-domain-activation.v1":
        raise ValueError("activation marker schema mismatch")
    if activation.get("domain_id") != domain_id or activation.get("hostname") != hostname:
        raise ValueError("activation marker exact customer mismatch")
    if (
        activation.get("domain_activation_state") != "active"
        or activation.get("customer_release_go") is not False
    ):
        raise ValueError("customer domain is not at the pre-release active boundary")
    if activation.get("tenant_human_mfa_browser_acceptance") != "REQUIRED":
        raise ValueError("activation marker browser gate mismatch")

    if acceptance.get("schema") != "ipat.customer-human-mfa-acceptance.v1":
        raise ValueError("browser acceptance schema mismatch")
    if acceptance.get("domain_id") != domain_id or acceptance.get("hostname") != hostname:
        raise ValueError("browser acceptance exact customer mismatch")
    tenant = uid(acceptance.get("tenant_id", ""))
    control = uid(acceptance.get("control_tenant_id", ""))
    if tenant == control:
        raise ValueError("cross-tenant control must be distinct")
    control_host = exact_host(str(acceptance.get("control_hostname", "")))
    if control_host == hostname:
        raise ValueError("cross-tenant control hostname must be distinct")
    for key in REQUIRED_BOOL:
        if acceptance.get(key) is not True:
            raise ValueError(f"{key} not accepted")

    maker = str(acceptance.get("prepared_by", ""))
    checker = str(acceptance.get("approved_by", ""))
    evidence = str(acceptance.get("evidence_id", ""))
    artifact = str(acceptance.get("artifact_sha256", ""))
    if (
        not SAFE.fullmatch(maker)
        or not SAFE.fullmatch(checker)
        or not SAFE.fullmatch(evidence)
        or maker == checker
    ):
        raise ValueError("invalid maker/checker evidence")
    if not HEX64.fullmatch(artifact) or artifact == "0" * 64:
        raise ValueError("invalid external browser artifact SHA256")
    try:
        reviewed = datetime.fromisoformat(
            str(acceptance.get("reviewed_at_utc", "")).replace("Z", "+00:00")
        )
    except ValueError as e:
        raise ValueError("invalid review timestamp") from e
    if reviewed.tzinfo is None:
        raise ValueError("review timestamp missing timezone")
    current = (now or datetime.now(timezone.utc)).astimezone(timezone.utc)
    reviewed = reviewed.astimezone(timezone.utc)
    if reviewed > current + timedelta(minutes=5) or current - reviewed > timedelta(hours=72):
        raise ValueError("customer browser acceptance stale or future-dated")

    return {
        "domain_id": domain_id,
        "hostname": hostname,
        "tenant_id": tenant,
        "control_tenant_id": control,
        "control_hostname": control_host,
        "evidence_id": evidence,
        "artifact_sha256": artifact,
        "customer_release_go": True,
    }

def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--activation", type=Path, required=True)
    p.add_argument("--acceptance", type=Path, required=True)
    p.add_argument("--domain-id", required=True)
    p.add_argument("--hostname", required=True)
    a = p.parse_args()
    try:
        out = validate(
            secure_json(a.activation),
            secure_json(a.acceptance),
            a.domain_id,
            a.hostname,
        )
    except (OSError, ValueError) as e:
        print(json.dumps({"ok": False, "error": str(e), "customer_release_go": False}, sort_keys=True))
        return 4
    print(json.dumps({"ok": True, **out}, sort_keys=True))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
