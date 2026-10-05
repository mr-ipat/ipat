#!/usr/bin/env python3
"""Read-only exact temporary VPS IP HTTPS SAN preflight. ALWAYS production NO_GO.

It never generates certificates, changes DNS/firewall or uses private credentials.
Provider console, independently recovered host/PITR and real IdP MFA need their
own evidence. A positive TLS check alone never authorizes public deployment.
"""
import argparse
import importlib.util
import ipaddress
import json
import socket
import ssl
from datetime import datetime, timezone
from pathlib import Path

HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("r963_preflight",HERE/"r963_public_cutover_preflight.py")
r963=importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(r963)

def eligible(ip):
    try:
        value=ipaddress.IPv4Address(ip)
    except ipaddress.AddressValueError:
        return False
    # Only a globally routable EXACT IPv4 address, not lab, RFC-documentation,
    # private, loopback, shared, benchmark, multicast or reserved space.
    return value.is_global

def trusted_ip_san_tls(ip, port=443, timeout=3):
    """Public CA trust and certificate SAN IP matching via Python TLS."""
    try:
        context=ssl.create_default_context()
        context.check_hostname=True
        context.verify_mode=ssl.CERT_REQUIRED
        with socket.create_connection((ip,port),timeout=timeout) as raw:
            with context.wrap_socket(raw,server_hostname=ip) as tls:
                certificate=tls.getpeercert()
                san=certificate.get("subjectAltName",())
                return any(kind=="IP Address" and value==ip for kind,value in san)
    except (OSError, ssl.SSLError,ValueError):
        return False

def report(ip, remote=None, tcp=None, tls=None):
    if not eligible(ip):
        raise ValueError("IP must be an eligible exact public IPv4")
    live=r963.remote() if remote is None else remote
    configured=live.get("PRIVATE_DOMAIN_IP") if live.get("available") else None
    exact=bool(configured and configured==ip)
    listener=bool(live.get("available") and live.get("PORT443") not in ("0",None,""))
    reachable=bool(r963.tcp_connect(ip,443) if tcp is None else tcp) if exact else False
    trusted=bool(trusted_ip_san_tls(ip) if tls is None else tls) if exact and reachable else False
    return {
        "schema":"ipat.temporary-public-ip-tls.v1",
        "observed_at_utc":datetime.now(timezone.utc).isoformat(),
        "target_ip":ip,
        "target_matches_existing_private_owner_plan":exact,
        "private_vps_ssh_observed":bool(live.get("available")),
        "vps_443_listener_observed":listener,
        "internet_443_reachable":reachable,
        "browser_ca_trusted_exact_ipv4_san_tls":trusted,
        "provider_rescue_full_host_pitr_real_mfa_external_gates":"REQUIRE_INDEPENDENT_PROOF",
        "access_mode":"TEMPORARY_PUBLIC_IPV4_ADMIN_NOT_UNOWNED_BRAND_DOMAIN",
        "no_firewall_dns_device_or_host_mutations":True,
        "production_status":"BLOCKED",
        "public_go":False,
    }

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--ip",required=True,help="Exact reviewed IPv4 from existing owner plan")
    args=p.parse_args()
    try:
        print(json.dumps(report(args.ip),indent=2,sort_keys=True))
    except ValueError as e:
        p.error(str(e))
    return 3

if __name__=="__main__":
    raise SystemExit(main())
