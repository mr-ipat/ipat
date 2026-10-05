#!/usr/bin/env python3
"""Read-only owner-readiness evidence manifest. NEVER authorizes a release.

This module deliberately cannot inspect a provider's authenticated VNC console,
off-host backup keys, MFA accounts, or a real device's firmware without
separately authorized independent evidence. No credentials are accepted.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    'r963_readonly_inventory', HERE / 'r963_public_cutover_preflight.py'
)
r963 = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(r963)

HOSTNAME_RE = re.compile(
    r'(?=.{1,253}$)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+'
    r'[a-z]{2,63}\Z'
)
EVIDENCE = {
    'provider_rescue': {
        'owner_reported_ready': True,
        'required': (
            'Redacted time-stamped proof of a successful provider console '
            'administrator login and a safely planned rollback; a console '
            'button or cancelled reset alone does not qualify.'
        ),
        'verified': False,
    },
    'independent_backup': {
        'owner_reported_ready': True,
        'required': (
            'Nonsecret independent backup destination identifier and '
            'time-stamped encrypted full-host restore log PLUS real production '
            'PostgreSQL base+WAL PITR rebuild verification in a separate '
            'isolated instance. Earlier encrypted C320 CLI capture is not '
            'a full-host or device-native recovery.'
        ),
        'verified': False,
    },
    'owned_domain': {
        'owner_reported_ready': True,
        'required': (
            'Exact registered hostname, redacted registrar/authoritative-zone '
            'control proof, independently retrieved fresh challenge TXT, '
            'reviewed ingress and trusted HTTPS certificate for that hostname. '
            'Do not change current lab SSH DNS.'
        ),
        'verified': False,
    },
    'real_mfa_identity': {
        'owner_reported_ready': True,
        'required': (
            'Redacted real IdP/MFA policy and successful verified distinct '
            'Platform Owner, Tenant Admin maker/checker and NOC login audit, '
            'plus actual two-tenant negative Host/API browser tests. Never '
            'provide tokens, recovery codes, QR seeds or passwords.'
        ),
        'verified': False,
    },
    'physical_device_lab': {
        'owner_reported_ready': True,
        'required': (
            'Nonsecret model, firmware/RouterOS version, enabled management '
            'protocol and non-customer-impacting lab window for ZTE C320, '
            'C-DATA, ZTE/VSOL ONT, MikroTik. Redacted physical CWMP/USP and '
            'read-only vendor result logs before claiming interoperability. '
            'Any physical writes require separate vendor-native rollback proof.'
        ),
        'verified': False,
    },
}


def public_dns_status(domain: str) -> str:
    """Queries public DNS only for an explicitly specified syntax-safe name."""
    if not HOSTNAME_RE.fullmatch(domain):
        raise ValueError('A valid exact domain is required; no DNS request made')
    try:
        p = subprocess.run(
            ['dig', '@1.1.1.1', '+time=2', '+tries=1',
             '+noall', '+comments', '+answer', domain, 'NS'],
            capture_output=True, text=True, timeout=6, check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return 'UNVERIFIED_QUERY_UNAVAILABLE'
    if p.returncode != 0:
        return 'UNVERIFIED_QUERY_FAILED'
    match = re.search(r'\bstatus:\s*([A-Z]+)', p.stdout)
    return match.group(1) if match else 'UNVERIFIED_UNPARSEABLE'


def report(domain: str | None = None) -> dict:
    live = r963.remote()
    # VPS SSH and its private read-only HTTP facts must remain the authority:
    # a missing Mac-local :3002 tunnel is not a disconnected physical C320.
    stage = r963.snapshot(live, None, None)
    gates = stage['gates']
    public_443 = gates['public_tcp_443_from_owner_mac']
    trusted_https = gates['public_domain_runtime_attests_https']
    remote_admin = gates['real_privileged_deployment_identity']
    dns_result = public_dns_status(domain) if domain else 'UNVERIFIED_DOMAIN_NOT_SPECIFIED'
    source = subprocess.run(
        ['git', '-C', str(HERE.parents[2]), 'rev-parse', 'HEAD'],
        capture_output=True, text=True, timeout=5, check=False,
    )
    return {
        'schema': 'ipat.owner-release-evidence.v1',
        'observed_at_utc': datetime.now(timezone.utc).isoformat(),
        'source_sha': source.stdout.strip() if source.returncode == 0 else None,
        'environment': 'OWNER_MAC_TO_EXISTING_PRIVATE_VPS_READ_ONLY',
        'declared_readiness_is_not_technical_acceptance': True,
        'observed': {
            'private_ssh_verification': bool(live.get('available')),
            'server_os': live.get('OS') if live.get('available') else None,
            'private_c320_status': stage['private_c320_observation'],
            'private_service_summary': stage['private_services'],
            'public_tcp_443_reachable': bool(public_443),
            'remote_noninteractive_deployment_admin': bool(remote_admin),
            'runtime_https_ready': bool(trusted_https),
            'runtime_safe_to_point': bool(gates['public_domain_runtime_attests_safe_to_point']),
            'domain_checked': domain,
            'public_dns_lookup_status': dns_result,
        },
        'evidence_items': EVIDENCE,
        'next_gate': (
            'REVIEW_REDACTED_OWNER_EVIDENCE_AND_ISOLATED_RECOVERY_DRILLS'
            if bool(live.get('available')) else
            'RESTORE_READ_ONLY_PRIVATE_ACCESS_FOR_INDEPENDENT_INSPECTION'
        ),
        'release_decision': 'BLOCKED',
        'public_go': False,
        'no_host_or_device_changes_made': True,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        '--domain', help='Exact independently managed candidate hostname '
                         '(read-only public DNS lookup; does not prove ownership)'
    )
    args = parser.parse_args()
    try:
        print(json.dumps(report(args.domain), indent=2, sort_keys=True))
    except ValueError as e:
        parser.error(str(e))
    return 3  # never exits success as a commercial-public release oracle


if __name__ == '__main__':
    raise SystemExit(main())
