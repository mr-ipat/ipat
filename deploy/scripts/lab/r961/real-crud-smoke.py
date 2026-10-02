#!/usr/bin/env python3
"""Non-secret synthetic metadata-only CRUD over ACTUAL owner-private VPS HTTP."""
import hashlib,json,os,stat,subprocess,time,urllib.request,urllib.error
if os.environ.get('IPAT_R961_OWNER_APPROVES_SYNTHETIC_TEST') != 'YES':
    raise SystemExit('R961 explicit owner-only synthetic mutation consent required')
base='http://127.0.0.1:3002'
folder='/home/openai/.local/share/ipat/r961-device-inventory'
file=folder+'/devices.json'
secret='/home/openai/.local/share/ipat/r945-connection/device.fernet'
def request(method,path,body=None,authorized=True):
    headers={'Host':'127.0.0.1:3002'}
    if body is not None:
        headers['Content-Type']='application/json'
        if authorized:
            headers['Origin']='http://127.0.0.1:3002'
            headers['X-IPAT-Owner-Private']='1'
    raw=None if body is None else json.dumps(body).encode()
    req=urllib.request.Request(base+path,data=raw,method=method,headers=headers)
    try:
        with urllib.request.urlopen(req,timeout=6) as resp:
            return resp.status,json.load(resp)
    except urllib.error.HTTPError as e:
        content=e.read()
        try:payload=json.loads(content)
        except (ValueError,UnicodeDecodeError):payload={'http_framework_reject':e.code}
        return e.code,payload

def running(service):
    return subprocess.check_output(['systemctl','--user','show',service,'-p','MainPID','--value'],text=True).strip()

secret_before=hashlib.sha256(open(secret,'rb').read()).digest()
pid_before=running('ipat-r945-connector.service')
_,initial=request('GET','/lab/owner/devices')
assert initial['persistent'] and initial['owner_private_lab_only']
assert len([d for d in initial['devices'] if d['id']=='DEV-01' and d['connection_state']=='CONNECTED'])==1
before=initial['audit_count']
example={
'display_name':'QA-R961-Metadata-Test','pop_id':'QA-PRIVATE','device_kind':'olt',
'vendor':'C-DATA','exact_model':'MODEL-UNVERIFIED','management_protocol':'ssh',
'management_host':'192.0.2.161','management_port':22}
code,denied=request('POST','/lab/owner/devices',example,False)
assert code==403,(code,denied)
code,denied=request('DELETE','/lab/owner/devices/DEV-01',
    {'expected_revision':0,'confirm':'REMOVE DEV-01'})
assert code==409,(code,denied)
code,created=request('POST','/lab/owner/devices',example)
assert code==201,(code,created)
d=created['device'];test_id=d['id'];rev=d['revision'];created_flag=True
try:
    assert d['connection_state']=='SAVED_NOT_CONNECTED' and d['linked_live_connector'] is False
    assert created['durable'] is True and created['device_command_sent'] is False
    code,duplicate=request('POST','/lab/owner/devices',example)
    assert code==409,(code,duplicate)
    code,details=request('GET','/lab/owner/devices/'+test_id)
    assert code==200 and details['device']['display_name']==example['display_name']
    changed=dict(example,display_name='QA-R961-Renamed',management_host='192.0.2.162',expected_revision=rev)
    code,updated=request('PUT','/lab/owner/devices/'+test_id,changed)
    assert code==200,(code,updated)
    assert updated['device']['display_name']=='QA-R961-Renamed'
    assert updated['device']['management_host']=='192.0.2.162'
    rev=updated['device']['revision']
    code,conflict=request('PUT','/lab/owner/devices/'+test_id,changed)
    assert code==409,(code,conflict)
    assert conflict['error']=='STALE_REVISION'
    assert stat.S_IMODE(os.stat(folder).st_mode)==0o700
    assert stat.S_IMODE(os.stat(file).st_mode)==0o600
    with open(file,'rb') as stream: owner_data=json.load(stream)
    assert all('password' not in json.dumps(e).lower() for e in owner_data['devices'])
    assert len(owner_data['audit'])>=before+2
    print('R961_ACTUAL_HTTP_CREATE_VIEW_EDIT_DUPLICATE_REVISION_DENIAL=PASS',flush=True)
    subprocess.run(['systemctl','--user','restart','ipat-r911-preview.service'],check=True,timeout=25)
    online=False
    for _ in range(25):
        try:
            code,ready=request('GET','/lab/owner/devices')
            if code==200:
                online=True;break
        except Exception:pass
        time.sleep(.35)
    assert online,'private dashboard failed to reload after user-service restart'
    assert any(d['id']==test_id and d['display_name']=='QA-R961-Renamed' and d['revision']==rev for d in ready['devices'])
    assert any(d['id']=='DEV-01' and d['connection_state']=='CONNECTED' for d in ready['devices'])
    print('R961_REAL_USER_SERVICE_RESTART_AND_PERSISTENCE=PASS',flush=True)
finally:
    if created_flag:
        code,removed=request('DELETE','/lab/owner/devices/'+test_id,
             {'expected_revision':rev,'confirm':'REMOVE '+test_id})
        assert code==200,(code,removed)
        assert removed['removed'] is True and removed['device_command_sent'] is False
_,last=request('GET','/lab/owner/devices')
assert not any(d['id']==test_id for d in last['devices'])
assert len([d for d in last['devices'] if d['id']=='DEV-01' and d['connection_state']=='CONNECTED'])==1
assert last['audit_count']==before+3,(before,last['audit_count'])
assert running('ipat-r945-connector.service')==pid_before
assert hashlib.sha256(open(secret,'rb').read()).digest()==secret_before
assert subprocess.check_output(['systemctl','--user','is-active','ipat-r911-preview.service'],text=True).strip()=='active'
print('R961_ACTUAL_HTTP_DELETE_AUDIT_NO_SECRETS_C320_UNTOUCHED=PASS',flush=True)
