#!/usr/bin/env python3
"""Isolated, disposable PG18 production-contract QA. NEVER a production GO gate.

Four existing exact repository tests exercise migration/RLS, WAL PITR,
controlled streaming failover, and encrypted Restic PITR. This orchestrator
does not touch a production DB or store secrets/test transcript payloads.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
SUITES = (
    ("migration_rls", "r986_pg18_docker_repro.sh",
     "IPAT_R986_PG18_DOCKER",
     "R986_DISPOSABLE_PG18_FULL_SECURITY_CHAIN_AND_TENANT_BRANDING=PASS"),
    ("basebackup_pitr", "r988_pg18_wal_pitr_docker.sh",
     "IPAT_R988_PG18_PITR_DOCKER",
     "R988_POSTGRES18_BASEBACKUP_VERIFY_WAL_NAMED_POINT_PITR=PASS"),
    ("streaming_ha", "r989_pg18_streaming_failover_docker.sh",
     "IPAT_R989_PG18_STREAMING_DOCKER",
     "R989_POSTGRES18_STREAMING_STANDBY_CONTROLLED_PROMOTION=PASS"),
    ("encrypted_pitr", "r990_pg18_restic_encrypted_pitr.sh",
     "IPAT_R990_RESTIC_PITR_DOCKER",
     "R990_RESTIC_ENCRYPTED_PG18_BASE_WAL_PITR=PASS"),
)
UNSAFE_ENV = (
    "PGHOST", "PGPORT", "PGDATABASE", "PGUSER", "PGPASSWORD",
    "PGSERVICE", "PGSERVICEFILE", "RESTIC_REPOSITORY",
    "RESTIC_PASSWORD", "RESTIC_PASSWORD_FILE", "DOCKER_HOST",
    "DOCKER_CONTEXT", "DOCKER_TLS_VERIFY", "DOCKER_CERT_PATH",
    "KUBECONFIG",
)


def command(*args: str, timeout: int = 20) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, text=True, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, timeout=timeout, check=False)


def safe_environment(environ: dict[str, str], *, uid: int) -> None:
    if uid == 0:
        raise ValueError("root execution is forbidden")
    present = [k for k in UNSAFE_ENV if environ.get(k)]
    if present:
        # Avoid printing values; names suffice to safely remediate.
        raise ValueError("non-isolated environment keys: " + ",".join(present))
    if any(environ.get(k) == "YES" for _, _, k, _ in SUITES):
        raise ValueError("caller must not preconfigure test opt-ins")


def validate_runner(repo: Path) -> dict[str, str]:
    if repo.is_symlink() or not repo.is_dir():
        raise ValueError("invalid repository root")
    version = command("git", "-C", str(repo), "rev-parse", "HEAD")
    if version.returncode or len(version.stdout.strip()) != 40:
        raise ValueError("source commit unavailable")
    hashes = {}
    for _, script, _, _ in SUITES:
        file = repo / "deploy/db/tests" / script
        if file.is_symlink() or not file.is_file():
            raise ValueError("missing or symlinked QA script")
        hashes[script] = hashlib.sha256(file.read_bytes()).hexdigest()
    return {"source_commit": version.stdout.strip(), "sha256_scripts": hashes}


def ensure_local_docker() -> None:
    inspect = command("docker", "context", "inspect",
                      "--format", "{{.Endpoints.docker.Host}}")
    if inspect.returncode:
        raise ValueError("Docker context unavailable")
    socket = inspect.stdout.strip()
    if not socket.startswith("unix://"):
        raise ValueError("Docker must be a local Unix socket, not a remote production host")
    info = command("docker", "info", "--format", "{{.OSType}}")
    if info.returncode or info.stdout.strip() != "linux":
        raise ValueError("local Linux Docker engine unavailable")
    restic = command("restic", "version")
    if restic.returncode:
        raise ValueError("Restic unavailable")


def run_suite(repo: Path, name: str, script: str, opt_in: str,
              success_marker: str, base_env: dict[str, str]) -> dict:
    scoped = dict(base_env)
    scoped[opt_in] = "YES"
    result = subprocess.run(
        ["bash", str(repo / "deploy/db/tests" / script)],
        env=scoped, cwd=repo, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        timeout=600, check=False,
    )
    # Never persist raw output (including synthetic fixture details).
    if result.returncode != 0 or success_marker not in result.stdout:
        raise ValueError(f"{name} failed (exit={result.returncode}, marker_verified={success_marker in result.stdout})")
    return {"name": name, "result": "PASS", "isolated_synthetic": True}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true",
                        help="Explicitly run four isolated disposable PostgreSQL18 suites")
    args = parser.parse_args()
    if not args.run:
        parser.error("explicit --run is required; no implicit production action")
    try:
        safe_environment(dict(os.environ), uid=os.geteuid())
        source = validate_runner(ROOT)
        ensure_local_docker()
        records = []
        for name, script, opt_in, marker in SUITES:
            records.append(run_suite(ROOT, name, script, opt_in, marker, dict(os.environ)))
        print(json.dumps({
            "schema": "ipat.r1038.pg18-disposable-acceptance.v1",
            "observed_at_utc": datetime.now(timezone.utc).isoformat(),
            **source,
            "tests": records,
            "full_host_restore_verified": False,
            "production_postgresql_ha_pitr_verified": False,
            "production_mfa_tls_verified": False,
            "public_go": False,
        }, sort_keys=True))
        return 0
    except (ValueError, OSError, subprocess.TimeoutExpired) as error:
        print(json.dumps({"ok": False, "reason": str(error), "public_go": False}))
        return 4


if __name__ == "__main__":
    sys.exit(main())
