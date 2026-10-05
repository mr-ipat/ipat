"""R9.78 maker/checker typed real POP grant lifecycle. Synthetic PostgreSQL only."""
import os, subprocess, unittest
ISS="https://identity.r978.synthetic.invalid"
T1="78787878-7878-4787-8787-787878787861"
T2="78787878-7878-4787-8787-787878787862"
REQ="78787878-7878-4787-8787-787878787871"
REQ2="78787878-7878-4787-8787-787878787872"
EXP=None

def q(s,role=None,check=True):
    p=("SET ROLE "+role+";") if role else ""
    return subprocess.run(["psql","-X","-q","-A","-t","-v","ON_ERROR_STOP=1","-c",p+s],
      capture_output=True,text=True,check=check)

def request(req=REQ,actor="maker",target="noc",pop="POP-A",tenant=T1):
    return q("SELECT COALESCE(ipat_platform.request_noc_real_pop_grant("
      f"'{ISS}','{actor}','{tenant}','{req}','{ISS}','{target}','{pop}',"
      f"'{EXP}'::timestamptz)::text,'DENIED')",
      role="ipat_tenant_api_login").stdout.strip()

def review(req=REQ,checker="checker",approve=True,tenant=T1):
    return q("SELECT COALESCE(ipat_platform.review_noc_real_pop_grant("
      f"'{ISS}','{checker}','{tenant}','{req}',{'true' if approve else 'false'})"
      ",'DENIED')",role="ipat_tenant_api_login").stdout.strip()

def scopes(subject="noc",tenant=T1):
    return q("SELECT pop_code FROM ipat_platform.current_noc_real_pop_scopes("
      f"'{ISS}','{subject}','{tenant}')",role="ipat_tenant_api_login").stdout.strip().splitlines()

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","synthetic only")
class Lifecycle(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
    assert os.getenv("PGDATABASE")=="ipat_synthetic"
    assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD")=="local_ci_synthetic_only"
    global EXP
    EXP=q("SELECT (now()+interval '2 hours')::text").stdout.strip()
    q(f"INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES('{T1}','r978-a'),('{T2}','r978-b')")
    q(f"""INSERT INTO ipat_platform.identity_memberships(
      tenant_id,issuer,subject,role,approved_by,created_at,expires_at) VALUES
      ('{T1}','{ISS}','maker','tenant_admin','seed',now()-interval '1 minute',now()+interval '1 day'),
      ('{T1}','{ISS}','checker','tenant_admin','seed',now()-interval '1 minute',now()+interval '1 day'),
      ('{T1}','{ISS}','noc','noc_engineer','seed',now()-interval '1 minute',now()+interval '1 day'),
      ('{T1}','{ISS}','noc2','noc_engineer','seed',now()-interval '1 minute',now()+interval '1 day'),
      ('{T2}','{ISS}','otheradmin','tenant_admin','seed',now()-interval '1 minute',now()+interval '1 day')""")
    for tenant,code in [(T1,"POP-A"),(T1,"POP-B"),(T2,"POP-A")]:
      q(f"""INSERT INTO ipat_ops.tenant_pops(
       tenant_id,code,display_name,created_by_issuer,created_by_subject,
       updated_by_issuer,updated_by_subject) VALUES
       ('{tenant}','{code}','{code}','{ISS}','seed','{ISS}','seed')""")

 def test_01_request_is_idempotent_but_cannot_self_approve(self):
    self.assertEqual(request(),REQ)
    self.assertEqual(request(),REQ)
    # The same key with a changed expiry is NOT the same approved intent.
    changed_expiry=q("SELECT (now()+interval '3 hours')::text").stdout.strip()
    forged=q("SELECT ipat_platform.request_noc_real_pop_grant("
      f"'{ISS}','maker','{T1}','{REQ}','{ISS}','noc','POP-A',"
      f"'{changed_expiry}'::timestamptz)",role="ipat_tenant_api_login").stdout.strip()
    self.assertEqual(forged,"")
    self.assertEqual(review(checker="maker"),"DENIED")
    self.assertEqual(scopes(),[])
    self.assertEqual(q("SELECT event FROM ipat_platform.noc_real_pop_grant_events "
      f"WHERE request_id='{REQ}' ORDER BY id").stdout.strip(),"REQUESTED")

 def test_02_distinct_current_admin_approves_and_scope_appears(self):
    self.assertEqual(review(),"APPROVED")
    self.assertEqual(scopes(),["POP-A"])
    self.assertEqual(review(),"DENIED")
    events=q("SELECT event FROM ipat_platform.noc_real_pop_grant_events "
      f"WHERE request_id='{REQ}' ORDER BY id").stdout.strip().splitlines()
    self.assertEqual(events,["REQUESTED","APPROVED"])

 def test_03_active_duplicate_and_cross_tenant_or_noc_actor_denied(self):
    self.assertEqual(request(req=REQ2),"DENIED")
    self.assertEqual(request(req=REQ2,actor="noc",target="noc2",pop="POP-B"),"DENIED")
    self.assertEqual(request(req=REQ2,actor="otheradmin",target="noc2",pop="POP-A"),"DENIED")
    self.assertEqual(q("SELECT count(*) FROM ipat_platform.noc_real_pop_grant_requests "
      f"WHERE request_id='{REQ2}'").stdout.strip(),"0")

 def test_04_revoke_is_immediate_and_new_request_can_regrant(self):
    ok=q("SELECT ipat_platform.revoke_noc_real_pop_grant("
      f"'{ISS}','maker','{T1}','{ISS}','noc','POP-A')",
      role="ipat_tenant_api_login").stdout.strip()
    self.assertEqual(ok,"t")
    self.assertEqual(scopes(),[])
    self.assertEqual(request(req=REQ2),"78787878-7878-4787-8787-787878787872")
    self.assertEqual(review(req=REQ2),"APPROVED")
    self.assertEqual(scopes(),["POP-A"])

 def test_05_target_revocation_blocks_new_approval_and_current_lists_are_admin_only(self):
    req3="78787878-7878-4787-8787-787878787873"
    self.assertEqual(request(req=req3,target="noc2",pop="POP-B"),req3)
    q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=now() WHERE tenant_id='{T1}' AND subject='noc2'")
    self.assertEqual(review(req=req3),"DENIED")
    members=q("SELECT subject FROM ipat_platform.list_current_noc_members_for_admin("
      f"'{ISS}','maker','{T1}')",role="ipat_tenant_api_login").stdout.strip().splitlines()
    self.assertEqual(members,["noc"])
    self.assertNotEqual(q("SELECT count(*) FROM ipat_platform.noc_real_pop_grant_requests",
      role="ipat_tenant_api_login",check=False).returncode,0)
    self.assertEqual(q("SELECT has_function_privilege('ipat_oidc_session_issuer_login',"
      "'ipat_platform.request_noc_real_pop_grant(text,text,uuid,uuid,text,text,text,timestamptz)',"
      "'EXECUTE')::int").stdout.strip(),"0")
    # Failed approval remains auditable Pending; restore target then explicitly reject it.
    q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL WHERE tenant_id='{T1}' AND subject='noc2'")
    self.assertEqual(review(req=req3,approve=False),"REJECTED")

 def test_06_reject_is_terminal_and_checker_revocation_denies_review(self):
    q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=NULL WHERE tenant_id='{T1}' AND subject='noc2'")
    req4="78787878-7878-4787-8787-787878787874"
    self.assertEqual(request(req=req4,target="noc2",pop="POP-B"),req4)
    self.assertEqual(review(req=req4,approve=False),"REJECTED")
    self.assertEqual(review(req=req4),"DENIED")
    req5="78787878-7878-4787-8787-787878787875"
    self.assertEqual(request(req=req5,target="noc2",pop="POP-B"),req5)
    q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=now() WHERE tenant_id='{T1}' AND subject='checker'")
    self.assertEqual(review(req=req5),"DENIED")
    self.assertEqual(scopes(subject="noc2"),[])

if __name__=="__main__": unittest.main()
