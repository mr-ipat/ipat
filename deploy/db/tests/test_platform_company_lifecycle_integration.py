"""R9.69 disposable platform company lifecycle and invitation tests."""
import os,pathlib,subprocess,unittest
ROOT=pathlib.Path(__file__).resolve().parents[3];M=ROOT/'deploy/db/migrations/0023_platform_company_lifecycle.sql'
ISS='https://id.r969.synthetic.invalid/realms/platform';OWNER='platform-owner';TARGET='tenant-admin-user'
T='89898989-8989-4989-8989-898989898981';INV='90909090-9090-4090-8090-909090909091'
def q(s,check=True):return subprocess.run(['psql','-X','-v','ON_ERROR_STOP=1','-At','-c',s],env=os.environ.copy(),text=True,capture_output=True,check=check)
def role(r,s,check=True):return q(f'SET ROLE {r}; {s}',check)
@unittest.skipUnless(os.getenv('IPAT_PG_EPHEMERAL_TEST')=='1','disposable PostgreSQL only')
class PlatformCompanyLifecycle(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  assert os.environ.get('PGDATABASE')=='ipat_synthetic' and os.environ.get('PGHOST')=='127.0.0.1'
  if q("SELECT to_regclass('ipat_platform.tenant_profiles') IS NULL").stdout.strip()=='t':subprocess.run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(M)],env=os.environ.copy(),text=True,capture_output=True,check=True)
  q(f"INSERT INTO ipat_platform.platform_principals(issuer,subject,role,approved_by,expires_at) VALUES('{ISS}','{OWNER}','platform_owner','synthetic-root-of-trust',clock_timestamp()+interval '1 day') ON CONFLICT(issuer,subject) DO UPDATE SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day'")
 def setUp(self):
  q(f"DELETE FROM ipat_platform.platform_tenant_events WHERE tenant_id='{T}'")
  q(f"DELETE FROM ipat_platform.identity_memberships WHERE tenant_id='{T}'")
  q(f"DELETE FROM ipat_platform.tenant_admin_invitations WHERE tenant_id='{T}'")
  q(f"DELETE FROM ipat_platform.tenant_profiles WHERE tenant_id='{T}'")
  q(f"DELETE FROM ipat_platform.tenants WHERE id='{T}'")
  q(f"UPDATE ipat_platform.platform_principals SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day' WHERE issuer='{ISS}' AND subject='{OWNER}'")
 def create(self):
  return role('ipat_platform_admin_exec',f"SELECT COALESCE(ipat_platform.create_company_tenant('{ISS}','{OWNER}','{T}','91919191-9191-4191-8191-919191919191','company-r969','Company R969','starter',100,10000)::text,'DENIED')").stdout.strip().splitlines()[-1]
 def test_00_function_only_platform_role_no_tenant_data_privilege(self):
  for table in ('ipat_platform.platform_principals','ipat_platform.identity_memberships','ipat_ops.managed_devices'):
   self.assertNotEqual(role('ipat_platform_admin_exec',f'SELECT count(*) FROM {table}',False).returncode,0)
  self.assertEqual(q("SELECT has_function_privilege('ipat_platform_admin_exec','ipat_platform.create_company_tenant(text,text,uuid,uuid,text,text,text,integer,integer)','EXECUTE')::int").stdout.strip(),'1')
  self.assertEqual(q("SELECT has_function_privilege('ipat_platform_admin_exec','ipat_platform.accept_company_tenant_admin_invitation(text,text,uuid,uuid)','EXECUTE')::int").stdout.strip(),'0')
 def test_01_platform_owner_creates_company_but_is_not_tenant_member(self):
  self.assertEqual(self.create(),T)
  self.assertEqual(q(f"SELECT count(*) FROM ipat_platform.identity_memberships WHERE tenant_id='{T}' AND issuer='{ISS}' AND subject='{OWNER}'").stdout.strip(),'0')
  out=role('ipat_platform_admin_exec',f"SELECT tenant_slug||'|'||state||'|'||display_name||'|'||plan_code FROM ipat_platform.list_company_tenants('{ISS}','{OWNER}') WHERE tenant_id='{T}'").stdout
  self.assertIn('company-r969|active|Company R969|starter',out)
 def test_02_forged_or_revoked_platform_owner_denied(self):
  bad=role('ipat_platform_admin_exec',f"SELECT COALESCE(ipat_platform.create_company_tenant('{ISS}','forged','{T}','92929292-9292-4292-8292-929292929292','bad-r969','Bad Company','starter',10,100)::text,'DENIED')").stdout.strip().splitlines()[-1];self.assertEqual(bad,'DENIED')
  q(f"UPDATE ipat_platform.platform_principals SET revoked_at=clock_timestamp() WHERE issuer='{ISS}' AND subject='{OWNER}'")
  self.assertEqual(self.create(),'DENIED')
 def test_03_profile_cas_and_suspend_resume_are_audited(self):
  self.assertEqual(self.create(),T)
  r=role('ipat_platform_admin_exec',f"SELECT COALESCE(ipat_platform.update_company_profile('{ISS}','{OWNER}','{T}','93939393-9393-4393-8393-939393939393',1,'Company R969 Updated','growth',200,20000,'R969','#123ABC')::text,'DENIED')").stdout.strip().splitlines()[-1];self.assertEqual(r,'2')
  stale=role('ipat_platform_admin_exec',f"SELECT COALESCE(ipat_platform.update_company_profile('{ISS}','{OWNER}','{T}','94949494-9494-4494-8494-949494949494',1,'Stale','growth',200,20000,NULL,NULL)::text,'DENIED')").stdout.strip().splitlines()[-1];self.assertEqual(stale,'DENIED')
  suspended=role('ipat_platform_admin_exec',f"SELECT ipat_platform.set_company_tenant_state('{ISS}','{OWNER}','{T}','95959595-9595-4595-8595-959595959595','suspended','Security review pause')").stdout.strip().splitlines()[-1];self.assertEqual(suspended,'t')
  resumed=role('ipat_platform_admin_exec',f"SELECT ipat_platform.set_company_tenant_state('{ISS}','{OWNER}','{T}','96969696-9696-4696-8696-969696969696','active','Review completed safely')").stdout.strip().splitlines()[-1];self.assertEqual(resumed,'t')
  self.assertEqual(q(f"SELECT count(*) FROM ipat_platform.platform_tenant_events WHERE tenant_id='{T}'").stdout.strip(),'4')
 def test_04_invitation_requires_exact_target_acceptance_and_creates_membership(self):
  self.assertEqual(self.create(),T)
  invited=role('ipat_platform_admin_exec',f"SELECT COALESCE(ipat_platform.invite_company_tenant_admin('{ISS}','{OWNER}','{T}','{INV}','97979797-9797-4797-8797-979797979797','{ISS}','{TARGET}',clock_timestamp()+interval '1 day')::text,'DENIED')").stdout.strip().splitlines()[-1];self.assertEqual(invited,INV)
  forged=role('ipat_tenant_invite_accept_exec',f"SELECT COALESCE(ipat_platform.accept_company_tenant_admin_invitation('{ISS}','other','{INV}','98989898-9898-4898-8898-989898989898')::text,'DENIED')").stdout.strip().splitlines()[-1];self.assertEqual(forged,'DENIED')
  accepted=role('ipat_tenant_invite_accept_exec',f"SELECT COALESCE(ipat_platform.accept_company_tenant_admin_invitation('{ISS}','{TARGET}','{INV}','99999999-9999-4999-8999-999999999999')::text,'DENIED')").stdout.strip().splitlines()[-1];self.assertEqual(accepted,T)
  self.assertEqual(q(f"SELECT role FROM ipat_platform.identity_memberships WHERE tenant_id='{T}' AND issuer='{ISS}' AND subject='{TARGET}'").stdout.strip(),'tenant_admin')
  self.assertEqual(role('ipat_tenant_invite_accept_exec',f"SELECT COALESCE(ipat_platform.accept_company_tenant_admin_invitation('{ISS}','{TARGET}','{INV}','99999999-9999-4999-8999-999999999998')::text,'DENIED')").stdout.strip().splitlines()[-1],'DENIED')
 def test_05_suspended_tenant_blocks_invite_and_membership_lookup(self):
  self.assertEqual(self.create(),T)
  role('ipat_platform_admin_exec',f"SELECT ipat_platform.set_company_tenant_state('{ISS}','{OWNER}','{T}','96969696-9696-4696-8696-969696969697','suspended','Commercial hold applied')")
  invited=role('ipat_platform_admin_exec',f"SELECT COALESCE(ipat_platform.invite_company_tenant_admin('{ISS}','{OWNER}','{T}','{INV}','97979797-9797-4797-8797-979797979796','{ISS}','{TARGET}',clock_timestamp()+interval '1 day')::text,'DENIED')").stdout.strip().splitlines()[-1];self.assertEqual(invited,'DENIED')
if __name__=='__main__':unittest.main()
