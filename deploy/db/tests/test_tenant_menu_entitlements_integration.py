"""R9.70 real disposable PostgreSQL current role→visible menu proof."""
import os,subprocess,unittest

def q(s,ok=True):
 return subprocess.run(['psql','-X','-A','-t','-v','ON_ERROR_STOP=1','-c',s],text=True,capture_output=True,check=ok)
T='76767676-7676-4676-8676-767676767671'
I='https://id.r970.synthetic.invalid/realms/ipat'
@unittest.skipUnless(os.getenv('IPAT_PG_EPHEMERAL_TEST')=='1','disposable only')
class MenuEntitlements(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  assert os.getenv('PGDATABASE')=='ipat_synthetic' and os.getenv('PGHOST')=='127.0.0.1'
  assert q("SELECT to_regprocedure('ipat_platform.tenant_admin_ui_capability(text,text,uuid)') IS NOT NULL").stdout.strip()=='t'
  q(f"INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES('{T}','r970-tenant')")
  q(f"INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at) VALUES('{T}','{I}','admin','tenant_admin','synthetic-reviewer',now()+interval '1 day'),('{T}','{I}','noc','noc_engineer','synthetic-reviewer',now()+interval '1 day')")
 def cap(self,subject='admin',issuer=I,tenant=T):
  return q(f"SET ROLE ipat_tenant_api_login;SELECT ipat_platform.tenant_admin_ui_capability('{issuer}','{subject}','{tenant}')").stdout.strip().splitlines()[-1]
 def test_01_current_tenant_admin_only_and_cross_tenant_denied(self):
  self.assertEqual(self.cap(),'t')
  self.assertEqual(self.cap('noc'),'f')
  self.assertEqual(self.cap('admin',tenant='22222222-2222-4222-8222-222222222222'),'f')
  self.assertEqual(self.cap('admin',issuer='https://forged.invalid'),'f')
 def test_02_role_is_function_only_and_oidc_issuer_cannot_infer_menus(self):
  self.assertEqual(q("SELECT has_function_privilege('ipat_tenant_api_login','ipat_platform.tenant_admin_ui_capability(text,text,uuid)','EXECUTE')::int").stdout.strip(),'1')
  self.assertEqual(q("SELECT has_function_privilege('ipat_oidc_session_issuer_login','ipat_platform.tenant_admin_ui_capability(text,text,uuid)','EXECUTE')::int").stdout.strip(),'0')
  self.assertNotEqual(q('SET ROLE ipat_tenant_api_login;SELECT count(*) FROM ipat_platform.identity_memberships',False).returncode,0)
 def test_03_revoked_or_suspended_tenant_immediately_hides_menus(self):
  q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() WHERE tenant_id='{T}' AND subject='admin'")
  self.assertEqual(self.cap(),'f')
  q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL WHERE tenant_id='{T}' AND subject='admin'")
  self.assertEqual(self.cap(),'t')
  q(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{T}'")
  self.assertEqual(self.cap(),'f')
