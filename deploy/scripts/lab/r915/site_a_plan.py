#!/usr/bin/env python3
"""Pure Site A/hub to Site B/spoke topology review; never pushes config."""
import ipaddress
import json

PRIVATE = tuple(map(ipaddress.ip_network, (
    '10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16')))
REQUIRED = {'mode', 'site_a_endpoint', 'site_b_gateway', 'management_host',
            'vpn_subnet', 'site_a_networks', 'site_b_networks',
            'private_path_verified', 'site_b_recovery_verified'}

def ip(v):
    a = ipaddress.ip_address(v)
    if a.version != 4:
        raise ValueError('IPv4 only')
    return a

def private(a):
    return any(a in n for n in PRIVATE)

def review(data):
    if type(data) is not dict or set(data) != REQUIRED:
        raise ValueError('unexpected fields / secret inputs denied')
    if data['mode'] not in ('direct_private', 'wireguard', 'ipsec'):
        raise ValueError('unsupported connection mode')
    if data['site_b_gateway'] not in ('routeros7', 'linux'):
        raise ValueError('unsupported site B platform')
    a, host = ip(data['site_a_endpoint']), ip(data['management_host'])
    if not private(host):
        raise ValueError('site OLT address must be private IPv4')
    if type(data['private_path_verified']) is not bool or type(data['site_b_recovery_verified']) is not bool:
        raise ValueError('invalid evidence flags')
    nets = []
    for key in ('site_a_networks', 'site_b_networks'):
        v = data[key]
        if type(v) is not list or not 1 <= len(v) <= 24:
            raise ValueError('explicit current network lists required')
        part = [ipaddress.ip_network(n, strict=True) for n in v]
        if any(n.version != 4 for n in part):
            raise ValueError('IPv4 networks only')
        nets.append(part)
    if not any(host in n for n in nets[1]):
        raise ValueError('OLT management LAN not declared at site B')
    if private(a) and not any(a in n for n in nets[0]):
        raise ValueError('private hub must belong to a declared Site A network')
    if data['mode'] == 'wireguard' and any(x.overlaps(y) for x in nets[0] for y in nets[1]):
        raise ValueError('overlapping A/B networks need separately designed NAT; deny direct tunnel')
    vpn = ipaddress.ip_network(data['vpn_subnet'], strict=True)
    if vpn.version != 4 or vpn.prefixlen != 30 or not private(vpn.network_address):
        raise ValueError('isolated RFC1918 /30 is required')
    if host in vpn or any(vpn.overlaps(n) for part in nets for n in part):
        raise ValueError('tunnel network conflict')
    if data['mode'] == 'direct_private':
        if not private(a) or not data['private_path_verified']:
            raise ValueError('direct requires independently verified private path')
    elif data['mode'] == 'wireguard' and private(a) and not data['private_path_verified']:
        raise ValueError('outside spoke cannot reach unverified private hub address')
    x, y = list(vpn.hosts())
    flags = ['real_tenant_mfa', 'independent_approval', 'site_b_firmware_verified', 'trusted_device_key',
             'isolated_last_hop', 'verified_return_route',
             'live_distribution_baseline', 'actual_worker_path']
    if not data['site_b_recovery_verified']:
        flags.append('site_b_console_recovery')
    return {'schema': 1, 'state': 'REVIEW_ONLY',
            'site_a_role': 'CENTRAL_HUB_LISTENER',
            'site_b_role': 'SELF_CONFIGURED_SPOKE',
            'site_a_endpoint_kind': 'PRIVATE' if private(a) else 'PUBLIC',
            'link_mode': data['mode'],
            'a_tunnel_host': str(x), 'b_tunnel_host': str(y),
            'target_host_route': str(host) + '/32',
            'site_a_allowed_peer_routes': [str(y) + '/32', str(host) + '/32'],
            'site_b_allowed_peer_routes': [str(x) + '/32'],
            'site_b_configuration_owner': 'SITE_B_OPERATOR',
            'missing_independent_evidence': flags,
            'ip_reachability_actually_measured': False,
            'router_push_enabled': False, 'config_generated': False,
            'secrets_accepted': False, 'network_actions': 0,
            'device_adopted': False}

def main():
    import argparse
    import os
    import stat
    from pathlib import Path
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--local-reviewed-topology-json',type=Path,required=True)
    args=parser.parse_args()
    path=args.local_reviewed_topology_json
    try:
        info=path.lstat()
        if os.geteuid()==0 or not path.is_absolute() or not stat.S_ISREG(info.st_mode) \
            or info.st_uid!=os.getuid() or info.st_nlink!=1 \
            or stat.S_IMODE(info.st_mode)!=0o600 or info.st_size>8192:
            raise ValueError('owner-only absolute regular topology file required')
        result=review(json.loads(path.read_text()))
        print(json.dumps(result,sort_keys=True))
    except (ValueError,TypeError,KeyError,OSError):
        parser.exit(2,'DENIED: topology needs independent review; no configuration generated\n')

if __name__=='__main__':main()
