"""R9.71 real disposable PostgreSQL one-time Host/PKCE pending proof."""
import os, subprocess, unittest, concurrent.futures

def q(sql,ok=True):
 r=subprocess.run(['psql','-X','-q','-A','-t','-v','ON_ERROR_STOP=1','-c',sql],capture_output=True,text=True,check=False)
 if ok and r.returncode: raise RuntimeError('SQL_ERROR '+r.stderr[:1200])
 return r

def issue(hash,proof,nonce,host='tenant.r971.invalid'):
 return q(f"SET ROLE ipat_oidc_session_issuer_login;SELECT ipat_platform.begin_oidc_pending('{hash}','{proof}','{nonce}','{host}')").stdout.strip().splitlines()[-1]
def consume(hash,proof,host='tenant.r971.invalid'):
 return q(f"SET ROLE ipat_oidc_session_issuer_login;SELECT nonce FROM ipat_platform.consume_oidc_pending('{hash}','{proof}','{host}')").stdout.strip().replace('SET\n','')
@unittest.skipUnless(os.getenv('IPAT_PG_EPHEMERAL_TEST')=='1','disposable only')
class Pending(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  assert os.getenv('PGHOST')=='127.0.0.1' and os.getenv('PGDATABASE')=='ipat_synthetic'
  assert q("SELECT to_regclass('ipat_platform.oidc_pending_states') IS NOT NULL").stdout.strip()=='t'
 def test_01_unprivileged_role_has_functions_not_table(self):
  self.assertEqual(q("SELECT has_function_privilege('ipat_oidc_session_issuer_login','ipat_platform.begin_oidc_pending(text,text,text,text)','EXECUTE')::int").stdout.strip(),'1')
  self.assertEqual(q("SELECT has_function_privilege('ipat_tenant_api_login','ipat_platform.consume_oidc_pending(text,text,text)','EXECUTE')::int").stdout.strip(),'0')
  self.assertNotEqual(q('SET ROLE ipat_oidc_session_issuer_login;SELECT * FROM ipat_platform.oidc_pending_states',False).returncode,0)
  self.assertEqual(q("SELECT relrowsecurity::int,relforcerowsecurity::int FROM pg_class WHERE oid='ipat_platform.oidc_pending_states'::regclass").stdout.strip(),'1|1')
 def test_02_single_host_exact_proof_consumption(self):
  h='a'*64;p='V'*43;n='N'*43
  self.assertEqual(issue(h,p,n),'t')
  self.assertEqual(issue(h,p,n),'f')
  self.assertEqual(consume(h,'X'*43),'')
  self.assertEqual(consume(h,p,'foreign.r971.invalid'),'')
  self.assertEqual(consume(h,p),n)
  self.assertEqual(consume(h,p),'')
 def test_03_concurrent_callback_one_winner(self):
  h='b'*64;p='Q'*43;n='B'*43
  self.assertEqual(issue(h,p,n),'t')
  with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
   outcomes=list(pool.map(lambda _:consume(h,p),range(5)))
  self.assertEqual(outcomes.count(n),1)
  self.assertEqual(outcomes.count(''),4)
 def test_04_expired_or_invalid_never_issues(self):
  h='c'*64;p='P'*43;n='C'*43
  self.assertEqual(issue('NOT_SHA',p,n),'f')
  self.assertEqual(issue(h,'VERIFIER-PLAINTEXT',n),'f')
  self.assertEqual(issue(h,p,n),'t')
  q(f"UPDATE ipat_platform.oidc_pending_states SET issued_at=clock_timestamp()-interval '2 minutes', expires_at=clock_timestamp()-interval '1 second' WHERE state_sha256='{h}'")
  self.assertEqual(consume(h,p),'')
 def test_05_per_host_cap(self):
  p='P'*43;n='D'*43
  for i in range(64):
   self.assertEqual(issue(f'{i+1000:064x}',p,n,'capacity.r971.invalid'),'t')
  self.assertEqual(issue('f'*64,p,n,'capacity.r971.invalid'),'f')
  self.assertEqual(issue('e'*64,p,n,'other.r971.invalid'),'t')
if __name__=='__main__':unittest.main()
