#!/usr/bin/env python3
"""Read-only cutover inventory via verified owner SSH and private VPS HTTP.

This is NOT public deployment, a provider console test, a TLS check for zone
ownership, or permission to open privileged services. No secrets are read.
"""
from __future__ import annotations
import json, socket, subprocess, urllib.error, urllib.request
from datetime import datetime, timezone

BASE = 'http://127.0.0.1:3002'
COMMAND = '''printf 'OS='; . /etc/os-release; echo "$ID:$VERSION_ID";
printf 'SERVICES='; systemctl --user is-active ipat-r911-preview.service ipat-r945-connector.service | tr '\\n' ','; echo;
printf 'PRIVILEGED_ADMIN='; if sudo -n true >/dev/null 2>&1; then echo VERIFIED_NOINPUT_SUDO; else echo NOT_AVAILABLE; fi;
printf 'PORT80='; ss -H -lnt '( sport = :80 )' | wc -l;
printf 'PORT443='; ss -H -lnt '( sport = :443 )' | wc -l;
printf 'PRIVATE_APP='; ss -H -lnt '( sport = :3002 )' | awk '{print $4}';
printf 'PRIVATE_C320='; python3 -c 'import json,urllib.request; d=json.load(urllib.request.urlopen("http://127.0.0.1:3002/lab/owner/devices",timeout=3)); v=d.get("devices",[]); print("CONNECTED" if any(x.get("id")=="DEV-01" and x.get("connection_state")=="CONNECTED" for x in v) else "NOT_CONNECTED")' 2>/dev/null || echo UNVERIFIED;
python3 -c 'import json,urllib.request; p=json.load(urllib.request.urlopen("http://127.0.0.1:3002/lab/owner/platform-domains",timeout=3)).get("platform",{}); print("PRIVATE_DOMAIN_IP="+str(p.get("temporary_public_ipv4") or "")); print("PRIVATE_DOMAIN_HTTPS="+str(p.get("public_https_ready") is True)); print("PRIVATE_DOMAIN_SAFE_TO_POINT="+str(p.get("safe_to_point_now") is True))' 2>/dev/null || true;'''

def remote():
    proc = subprocess.run(['ssh','-T','-o','BatchMode=yes','-o','ConnectTimeout=7',
       '-o','StrictHostKeyChecking=yes','ipat-lab',COMMAND],capture_output=True,
       text=True,timeout=12,check=False)
    if proc.returncode:
        return {'available':False,'error':'SSH_NONINTERACTIVE_PRECHECK_FAILED'}
    pairs={}
    for line in proc.stdout.splitlines():
        if '=' in line:
            key,value=line.split('=',1)
            if key in {'OS','SERVICES','PRIVILEGED_ADMIN','PORT80','PORT443','PRIVATE_APP','PRIVATE_C320','PRIVATE_DOMAIN_IP','PRIVATE_DOMAIN_HTTPS','PRIVATE_DOMAIN_SAFE_TO_POINT'}:
                pairs[key]=value.strip()
    return {'available':True,**pairs}

def api(path):
    try:
        with urllib.request.urlopen(BASE+path,timeout=5) as resp:
            return json.load(resp)
    except (urllib.error.URLError,TimeoutError,ValueError):
        return None

def tcp_connect(ip:str,port:int):
    if not ip:return False
    try:
        with socket.create_connection((ip,port),timeout=3):return True
    except (OSError,ValueError):return False

def snapshot(live,domain,devices):
    platform=domain.get('platform',{}) if isinstance(domain,dict) else {}
    # Independently SSH-verified VPS loopback is authoritative. The Mac's
    # optional localhost tunnel may be absent; it is NOT device health evidence.
    remote_state=live.get('PRIVATE_C320') if live.get('available') else None
    connected=remote_state=='CONNECTED'
    observation=('VPS_LOOPBACK_CONNECTED' if remote_state=='CONNECTED' else
        'VPS_LOOPBACK_NOT_CONNECTED' if remote_state=='NOT_CONNECTED' else
        'UNVERIFIED_VPS_LOOPBACK')
    remote_ip=live.get('PRIVATE_DOMAIN_IP') if live.get('available') else None
    ip=remote_ip if remote_ip else platform.get('temporary_public_ipv4')
    https=(live.get('PRIVATE_DOMAIN_HTTPS')=='True' if 'PRIVATE_DOMAIN_HTTPS' in live
           else platform.get('public_https_ready') is True)
    safe=(live.get('PRIVATE_DOMAIN_SAFE_TO_POINT')=='True' if 'PRIVATE_DOMAIN_SAFE_TO_POINT' in live
          else platform.get('safe_to_point_now') is True)
    domain_source=('VPS_LOOPBACK' if 'PRIVATE_DOMAIN_HTTPS' in live else
                   'MAC_TUNNEL' if isinstance(domain,dict) else 'UNVERIFIED')
    # A configured IP/HTTPS flag does not prove owner DNS/TLS authorization.
    gates={
        'private_c320_live_read_only': connected,
        'public_tcp_443_from_owner_mac':tcp_connect(ip,443),
        'local_port_443_listening':live.get('PORT443') not in ('0',None,'') if live.get('available') else False,
        'real_privileged_deployment_identity':live.get('PRIVILEGED_ADMIN')=='VERIFIED_NOINPUT_SUDO',
        'public_domain_runtime_attests_https':https,
        'public_domain_runtime_attests_safe_to_point':safe,
        'platform_auth_and_verified_tenant_membership':'UNVERIFIED',
        'actual_dns_zone_ownership_and_txt':'UNVERIFIED',
        'verified_provider_rescue_console_and_host_restore':'UNVERIFIED',
        'commercial_postgresql_runtime_and_pitr':'UNVERIFIED',
        'high_risk_olt_ont_write_recovery':'UNTESTED'
    }
    return {'reported_at_utc':datetime.now(timezone.utc).isoformat(),
      'evaluated_scope':'READ_ONLY_EXISTING_OWNER_MAC_AND_PRIVATE_VPS',
      'public_target_ipv4_configured':bool(ip),'private_services':live.get('SERVICES'),
      'private_c320_observation':observation,'private_domain_plan_source':domain_source,
      'mac_private_tunnel_responsive':isinstance(devices,dict),
      'os':live.get('OS'),'public_go':False,'production_status':'BLOCKED',
      'gates':gates,'no_host_changes_made':True,
      'next_required_human_evidence':['Owner independently demonstrates provider rescue-console root login.',
        'Owner verifies a complete independently restorable full-host backup and controlled rollback.',
        'Owner confirms platform-owned DNS zone/certificate authorization; existing SSH hostname unchanged.'],
      'next_software_milestones':['Deploy independent real OIDC/MFA and tenant PostgreSQL runtime.',
        'Install trusted dedicated public TLS ingress with reviewed IPv4/IPv6 exposure and external tests.',
        'Mount verified platform and tenant admin domain workflow with DNS TXT/ACME/controller reconciliation.']}

def main():
    live=remote();domain=api('/lab/owner/platform-domains');devices=api('/lab/owner/devices')
    report=snapshot(live,domain,devices)
    print(json.dumps(report,indent=2,sort_keys=True))
    return 3  # A read-only report never authorizes public deployment.

if __name__=='__main__':raise SystemExit(main())
