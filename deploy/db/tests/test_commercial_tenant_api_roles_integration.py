"""R9.68 disposable privilege-composition tests. No HTTP/real credentials."""
import os,pathlib,subprocess,unittest
ROOT=pathlib.Path(__file__).resolve().parents[3];M=ROOT/'deploy/db/migrations/0022_commercial_tenant_api_roles.sql'
def q(s,check=True):return subprocess.run(['psql','-X','-v','ON_ERROR_STOP=1','-At','-c',s],env=os.environ.copy(),text=True,capture_output=True,check=check)
@unittest.skipUnless(os.getenv('IPAT_PG_EPHEMERAL_TEST')=='1','disposable PostgreSQL only')
class CommercialTenantApiRoles(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  assert os.environ.get('PGDATABASE')=='ipat_synthetic' and os.environ.get('PGHOST')=='127.0.0.1'
  assert q("SELECT to_regprocedure('ipat_platform.authenticate_tenant_browser_session(text,text,text,boolean)') IS NOT NULL").stdout.strip()=='t'
  if q("SELECT count(*) FROM pg_roles WHERE rolname='ipat_tenant_api_exec'").stdout.strip()=='0':
   subprocess.run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(M)],env=os.environ.copy(),text=True,capture_output=True,check=True)
 def test_00_login_roles_are_unprivileged_and_passwordless(self):
  for role in ('ipat_tenant_api_login','ipat_oidc_session_issuer_login'):
   out=q(f"SELECT rolcanlogin::int,rolsuper::int,rolbypassrls::int,rolcreaterole::int,(rolpassword IS NULL)::int FROM pg_authid WHERE rolname='{role}'").stdout.strip()
   self.assertEqual(out,'1|0|0|0|1')
 def test_01_tenant_api_inherits_auth_and_crud_functions_but_not_issue(self):
  funcs=[
   "ipat_platform.authenticate_tenant_browser_session(text,text,text,boolean)",
   "ipat_platform.create_tenant_site(text,text,uuid,text,text)",
   "ipat_platform.register_managed_device(text,text,uuid,uuid,uuid,text,text,text,text,text,text,text,integer,text)",
   "ipat_platform.edit_managed_device_metadata(text,text,uuid,uuid,bigint,text,text,text,text,integer)",
   "ipat_platform.list_tenant_domains_for_member(text,text,uuid)"]
  for f in funcs:self.assertEqual(q(f"SELECT has_function_privilege('ipat_tenant_api_login','{f}','EXECUTE')::int").stdout.strip(),'1',f)
  self.assertEqual(q("SELECT has_function_privilege('ipat_tenant_api_login','ipat_platform.issue_tenant_browser_session(text,text,uuid,uuid,uuid,text,text,timestamptz)','EXECUTE')::int").stdout.strip(),'0')
 def test_02_oidc_issuer_can_issue_but_not_auth_or_business_crud(self):
  self.assertEqual(q("SELECT has_function_privilege('ipat_oidc_session_issuer_login','ipat_platform.issue_tenant_browser_session(text,text,uuid,uuid,uuid,text,text,timestamptz)','EXECUTE')::int").stdout.strip(),'1')
  for f in ["ipat_platform.authenticate_tenant_browser_session(text,text,text,boolean)","ipat_platform.create_tenant_site(text,text,uuid,text,text)","ipat_platform.register_managed_device(text,text,uuid,uuid,uuid,text,text,text,text,text,text,text,integer,text)"]:
   self.assertEqual(q(f"SELECT has_function_privilege('ipat_oidc_session_issuer_login','{f}','EXECUTE')::int").stdout.strip(),'0',f)
 def test_03_neither_login_can_read_raw_business_identity_or_session_tables(self):
  for role in ('ipat_tenant_api_login','ipat_oidc_session_issuer_login'):
   for table in ('ipat_platform.identity_memberships','ipat_platform.tenant_domains','ipat_platform.tenant_browser_sessions','ipat_ops.managed_devices','ipat_ops.tenant_sites'):
    self.assertNotEqual(q(f'SET ROLE {role}; SELECT count(*) FROM {table}',False).returncode,0,(role,table))
if __name__=='__main__':unittest.main()
