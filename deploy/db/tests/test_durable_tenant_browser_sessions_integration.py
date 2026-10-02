"""R9.67 disposable PostgreSQL durable Host-bound session integration.

No real IdP, password, tenant, domain, cookie or production DB is touched.
This test proves the shared durable session DB boundary AFTER a caller has
already verified a signed OIDC ID/access token pair + nonce + fresh MFA.
"""
import os, pathlib, subprocess, unittest
ROOT=pathlib.Path(__file__).resolve().parents[3]
M=ROOT/'deploy/db/migrations/0021_durable_tenant_browser_sessions.sql'
TA='67676767-6767-4767-8767-676767676761'
TB='67676767-6767-4767-8767-676767676762'
DA='68686868-6868-4868-8868-686868686861'
DB='68686868-6868-4868-8868-686868686862'
DP='68686868-6868-4868-8868-686868686863'
ISS='https://id.r967.synthetic.invalid/realms/ipat'
SUB='tenant-a-operator'
SUBB='tenant-b-operator'
COOKIE='a'*64
CSRF='b'*64

def psql(q,check=True):
    return subprocess.run(['psql','-X','-v','ON_ERROR_STOP=1','-At','-c',q],env=os.environ.copy(),text=True,capture_output=True,check=check)
def file(path):
    return subprocess.run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(path)],env=os.environ.copy(),text=True,capture_output=True,check=True)
def role(name,q,check=True):
    return psql(f'SET ROLE {name}; {q}',check)
def issue(cookie=COOKIE,csrf=CSRF,tenant=TA,domain=DA,subject=SUB,sid='69696969-6969-4969-8969-696969696961',minutes=10):
    q=f"""SELECT COALESCE(ipat_platform.issue_tenant_browser_session(
      '{ISS}','{subject}','{tenant}','{domain}','{sid}','{cookie}','{csrf}',
      clock_timestamp()+interval '{minutes} minutes')::text,'DENIED')"""
    return role('ipat_browser_session_issue_exec',q).stdout.strip().splitlines()[-1]
def auth(cookie=COOKIE,host='a.r967.invalid',csrf=None,mutation=False):
    c='NULL' if csrf is None else f"'{csrf}'"
    q=f"""SELECT tenant_id::text||'|'||domain_id::text||'|'||issuer||'|'||subject
      FROM ipat_platform.authenticate_tenant_browser_session('{cookie}','{host}',{c},{str(mutation).lower()})"""
    return [x for x in role('ipat_browser_session_auth_exec',q).stdout.splitlines() if x and x!='SET']

@unittest.skipUnless(os.getenv('IPAT_PG_EPHEMERAL_TEST')=='1','disposable PostgreSQL only')
class DurableTenantBrowserSessions(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
    assert os.environ.get('PGDATABASE')=='ipat_synthetic'
    assert os.environ.get('PGHOST')=='127.0.0.1'
    assert os.environ.get('IPAT_PG_SYNTHETIC_PASSWORD')=='local_ci_synthetic_only'
    assert psql("SELECT to_regclass('ipat_platform.tenant_domains') IS NOT NULL").stdout.strip()=='t'
    assert psql("SELECT to_regclass('ipat_platform.identity_memberships') IS NOT NULL").stdout.strip()=='t'
    if psql("SELECT to_regclass('ipat_platform.tenant_browser_sessions') IS NULL").stdout.strip()=='t': file(M)
    psql(f"""
      INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
       ('{TA}','r967-a','active'),('{TB}','r967-b','active')
      ON CONFLICT(id) DO UPDATE SET state='active';
      INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at)
      VALUES
       ('{TA}','{ISS}','{SUB}','tenant_admin','synthetic-security-review',clock_timestamp()+interval '1 day'),
       ('{TB}','{ISS}','{SUBB}','tenant_admin','synthetic-security-review',clock_timestamp()+interval '1 day')
      ON CONFLICT(tenant_id,issuer,subject,role) DO UPDATE
       SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day';
      INSERT INTO ipat_platform.tenant_domains(
       id,tenant_id,hostname,domain_type,verification_state,verification_method,
       verified_at,routing_mode,verification_name,verification_value,
       requested_by_issuer,requested_by_subject,requested_at,
       activation_state,ownership_verified_at,routing_ready_at,tls_ready_at,activated_at)
      VALUES
       ('{DA}','{TA}','a.r967.invalid','custom_domain','verified','dns_txt',clock_timestamp(),
        'a_record','_ipat-verify.a.r967.invalid','ipat-domain=68686868-6868-4868-8868-686868686861',
        '{ISS}','{SUB}',clock_timestamp(),'active',clock_timestamp(),clock_timestamp(),clock_timestamp(),clock_timestamp()),
       ('{DB}','{TB}','b.r967.invalid','custom_domain','verified','dns_txt',clock_timestamp(),
        'a_record','_ipat-verify.b.r967.invalid','ipat-domain=68686868-6868-4868-8868-686868686862',
        '{ISS}','{SUBB}',clock_timestamp(),'active',clock_timestamp(),clock_timestamp(),clock_timestamp(),clock_timestamp()),
       ('{DP}','{TA}','pending.r967.invalid','custom_domain','verified','dns_txt',clock_timestamp(),
        'a_record','_ipat-verify.pending.r967.invalid','ipat-domain=68686868-6868-4868-8868-686868686863',
        '{ISS}','{SUB}',clock_timestamp(),'ownership_verified',clock_timestamp(),NULL,NULL,NULL)
      ON CONFLICT(id) DO UPDATE SET
       verification_state=excluded.verification_state,verified_at=excluded.verified_at,disabled_at=NULL,
       activation_state=excluded.activation_state,ownership_verified_at=excluded.ownership_verified_at,
       routing_ready_at=excluded.routing_ready_at,tls_ready_at=excluded.tls_ready_at,activated_at=excluded.activated_at;
    """)
 @classmethod
 def tearDownClass(cls):
    # shared CI DB: only remove R9.67 synthetic sessions/fixtures, never schemas.
    psql(f"DELETE FROM ipat_platform.tenant_browser_sessions WHERE tenant_id IN ('{TA}','{TB}')")

 def setUp(self):
    psql(f"DELETE FROM ipat_platform.tenant_browser_sessions WHERE tenant_id IN ('{TA}','{TB}')")
    psql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
    psql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day' WHERE issuer='{ISS}' AND subject IN ('{SUB}','{SUBB}')")
    psql(f"UPDATE ipat_platform.tenant_domains SET disabled_at=NULL WHERE id IN ('{DA}','{DB}','{DP}')")

 def test_00_roles_are_function_only_force_rls_and_no_login(self):
    for r in ['ipat_browser_session_owner','ipat_browser_session_issue_exec','ipat_browser_session_auth_exec']:
      self.assertEqual(psql(f"SELECT rolcanlogin::int,rolsuper::int,rolbypassrls::int FROM pg_roles WHERE rolname='{r}'").stdout.strip(),'0|0|0')
    self.assertEqual(psql("SELECT relrowsecurity::int,relforcerowsecurity::int FROM pg_class WHERE oid='ipat_platform.tenant_browser_sessions'::regclass").stdout.strip(),'1|1')
    self.assertNotEqual(role('ipat_browser_session_issue_exec','SELECT * FROM ipat_platform.tenant_browser_sessions',False).returncode,0)
    self.assertNotEqual(role('ipat_browser_session_auth_exec','SELECT * FROM ipat_platform.identity_memberships',False).returncode,0)

 def test_01_only_fully_active_domain_resolves_and_issues(self):
    self.assertEqual(role('ipat_domain_reader_login',"SELECT count(*) FROM ipat_platform.resolve_active_tenant_domain('pending.r967.invalid')").stdout.strip().splitlines()[-1],'0')
    self.assertEqual(role('ipat_browser_session_issue_exec',"SELECT count(*) FROM ipat_platform.resolve_active_tenant_domain_binding('pending.r967.invalid')").stdout.strip().splitlines()[-1],'0')
    self.assertNotEqual(issue(domain=DP,sid='69696969-6969-4969-8969-696969696962'),'69696969-6969-4969-8969-696969696962')
    self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')

 def test_02_host_is_tenant_selector_and_cross_host_replay_denied(self):
    self.assertNotEqual(issue(tenant=TA,domain=DB,sid='69696969-6969-4969-8969-696969696963'),'69696969-6969-4969-8969-696969696963')
    self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')
    own=auth(); self.assertEqual(len(own),1); self.assertTrue(own[0].startswith(TA+'|'+DA+'|'))
    self.assertEqual(auth(host='b.r967.invalid'),[])
    self.assertEqual(auth(host='A.R967.INVALID'),[]) # Rust canonicalizes before SQL; raw DB does not.

 def test_03_mutation_requires_csrf_and_read_does_not(self):
    self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')
    self.assertEqual(len(auth()),1)
    self.assertEqual(auth(mutation=True),[])
    self.assertEqual(auth(csrf='c'*64,mutation=True),[])
    self.assertEqual(len(auth(csrf=CSRF,mutation=True)),1)

 def test_04_revocation_membership_and_tenant_state_fail_immediately(self):
    self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')
    self.assertEqual(len(auth()),1)
    self.assertEqual(role('ipat_browser_session_auth_exec',f"SELECT ipat_platform.revoke_tenant_browser_session('{COOKIE}','a.r967.invalid')").stdout.strip().splitlines()[-1],'t')
    self.assertEqual(auth(),[])
    self.setUp(); self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')
    psql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() WHERE tenant_id='{TA}' AND issuer='{ISS}' AND subject='{SUB}'")
    self.assertEqual(auth(),[])
    self.setUp(); self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')
    psql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{TA}'")
    self.assertEqual(auth(),[])

 def test_05_idle_expiry_absolute_expiry_and_limits_fail_closed(self):
    self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')
    psql(f"UPDATE ipat_platform.tenant_browser_sessions SET last_seen_at=clock_timestamp()-interval '6 minutes' WHERE cookie_sha256='{COOKIE}'")
    self.assertEqual(auth(),[])
    self.setUp(); self.assertEqual(issue(minutes=16,sid='69696969-6969-4969-8969-696969696964'),'DENIED')
    for i in range(8):
      self.assertNotEqual(issue(cookie=f'{i:064x}',csrf=f'{i+16:064x}',sid=f'69696969-6969-4969-8969-{i+100:012d}'),'DENIED')
    self.assertEqual(issue(cookie='f'*64,csrf='e'*64,sid='69696969-6969-4969-8969-696969696999'),'DENIED')

 def test_06_cookie_and_csrf_hashes_only_plaintext_not_present(self):
    self.assertEqual(issue(),'69696969-6969-4969-8969-696969696961')
    row=psql(f"SELECT cookie_sha256||'|'||csrf_sha256 FROM ipat_platform.tenant_browser_sessions WHERE id='69696969-6969-4969-8969-696969696961'").stdout.strip()
    self.assertEqual(row,COOKIE+'|'+CSRF)
    # The SQL contract receives digests, never original 43-char browser secrets.
    self.assertNotIn('__Host-ipat_session',row)

if __name__=='__main__': unittest.main()
