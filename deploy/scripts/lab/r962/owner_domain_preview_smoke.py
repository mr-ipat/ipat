#!/usr/bin/env python3
"""Opt-in safe, temporary private-only DNS preview, never issue TXT/routing."""
import os,json,urllib.request,urllib.error
if os.environ.get('IPAT_R962_APPROVE_PRIVATE_DNS_PREVIEW')!='YES':
    raise SystemExit('Explicit owner-private synthetic draft opt-in required')
base='http://127.0.0.1:3002/lab/owner/platform-domains'
headers={'Origin':'http://127.0.0.1:3002','X-IPAT-Owner-Private':'1','Content-Type':'application/json'}
def request(method,path,data=None):
    req=urllib.request.Request(base+path,method=method,headers=headers if data is not None else {},
        data=json.dumps(data).encode() if data is not None else None)
    with urllib.request.urlopen(req,timeout=6) as reply:return json.load(reply)
initial=request('GET','')
assert initial['persisted'] is True and initial['platform']['temporary_public_ipv4']=='202.162.204.121'
assert initial['platform']['hostname'] is None and initial['platform']['public_https_ready'] is False
assert initial['platform']['safe_to_point_now'] is False
name='qa-r962-preview.ipat.id'
assert not any(d['hostname']==name for d in initial['drafts'])
created=None
try:
    result=request('POST','/drafts',{'tenant_reference':'qa-r962','kind':'custom_domain','custom_hostname':name})
    assert result['state']=='PLANNED_NOT_ROUTED' and result['public_route_changed'] is False
    created=result['draft']
    info=request('GET','/drafts/'+created['id']+'/preview')
    assert info['hostname']==name and info['suggested_a_record']['value']=='202.162.204.121'
    assert info['ownership_txt_name']=='_ipat-verify.'+name
    assert info['ownership_txt_value'] is None
    assert info['dns_ownership_verified'] is False and info['tls_ready'] is False
    assert info['safe_to_point_now'] is False and info['public_route_changed'] is False
    print('R962_REAL_PRIVATE_ADMIN_DOMAIN_DRAFT_A_AND_TXT_NAME_PREVIEW=PASS')
finally:
    if created:
        removed=request('DELETE','/drafts/'+created['id'],{
           'expected_revision':created['revision'],'confirm':'REMOVE DRAFT '+created['id']})
        assert removed['removed'] is True and removed['public_route_changed'] is False
final=request('GET','')
assert not any(d['hostname']==name for d in final['drafts'])
assert final['platform']['temporary_public_ipv4']=='202.162.204.121'
assert final['platform']['safe_to_point_now'] is False
assert final['audit_count']==initial['audit_count']+2
print('R962_REAL_PRIVATE_ADMIN_DOMAIN_DRAFT_CLEANUP_NO_PUBLIC_CHANGE=PASS',final['audit_count'])
