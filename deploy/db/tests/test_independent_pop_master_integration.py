"""R9.76: synthetic PostgreSQL only. Separate POP, explicit Site FK and legacy NOC."""
import os, subprocess, unittest

T="76767676-7676-4767-8767-767676767671"
OTHER="76767676-7676-4767-8767-767676767672"
ISS="https://pop.r976.synthetic.invalid"
def q(statement,role=None,check=True):
    sql=("SET ROLE "+role+";") if role else ""
    return subprocess.run(
        ["psql","-X","-q","-A","-t","-v","ON_ERROR_STOP=1","-c",sql+statement],
        capture_output=True,text=True,check=check)
def f(fn,params):
    return q("SELECT COALESCE("+fn+"("+params+")::text,'DENIED')",
             role="ipat_tenant_api_login").stdout.strip()
def create(code="DC-R976",tenant=T,subject="admin"):
    return f("ipat_platform.create_tenant_pop",
       f"'{ISS}','{subject}','{tenant}','{code}','Synthetic POP'")
def listp(tenant=T,subject="admin"):
    return q("SELECT code||':'||assigned_sites FROM ipat_platform.list_tenant_pops("
       f"'{ISS}','{subject}','{tenant}',NULL)",role="ipat_tenant_api_login").stdout.strip()
def assign(site="SITE-R976",pop="DC-R976",rev=1,subject="admin",tenant=T):
    value="NULL" if pop is None else f"'{pop}'"
    return f("ipat_platform.assign_tenant_site_to_pop",
       f"'{ISS}','{subject}','{tenant}','{site}',{value},{rev}")

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","synthetic-only")
class IndependentPop(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        assert os.getenv("PGHOST")=="127.0.0.1"
        assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD")=="local_ci_synthetic_only"
        for tenant,slug in [(T,"r976"),(OTHER,"r976-else")]:
            q(f"INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES('{tenant}','{slug}')")
        q(f"""INSERT INTO ipat_platform.identity_memberships(
         tenant_id,issuer,subject,role,approved_by,created_at,expires_at) VALUES
         ('{T}','{ISS}','admin','tenant_admin','independent-r976',
          now()-interval '1 minute',now()+interval '2 hours'),
         ('{OTHER}','{ISS}','admin-other','tenant_admin','independent-r976',
          now()-interval '1 minute',now()+interval '2 hours'),
         ('{T}','{ISS}','noc','noc_engineer','independent-r976',
          now()-interval '1 minute',now()+interval '2 hours')""")
        q(f"""INSERT INTO ipat_ops.tenant_sites(
          tenant_id,code,display_name,created_by_issuer,created_by_subject,
          updated_by_issuer,updated_by_subject)
          VALUES('{T}','SITE-R976','Existing Site No Invented POP',
          '{ISS}','admin','{ISS}','admin')""")
        q(f"""INSERT INTO ipat_platform.identity_pop_grants(
          tenant_id,issuer,subject,role,pop_id)
          VALUES('{T}','{ISS}','noc','noc_engineer','SITE-R976')""")

    def test_00_staged_composite_fk_validates_without_fabricated_pop(self):
        self.assertEqual(q("SELECT convalidated FROM pg_constraint "
         "WHERE conname='tenant_sites_parent_tenant_pop'").stdout.strip(),"t")
        self.assertEqual(q("SELECT count(*) FROM ipat_ops.tenant_pops "
         f"WHERE tenant_id='{T}'").stdout.strip(),"0")

    def test_01_pop_and_site_are_distinct_tenant_masters(self):
        self.assertEqual(q(f"SELECT parent_pop_code IS NULL FROM ipat_ops.tenant_sites "
                           f"WHERE tenant_id='{T}' AND code='SITE-R976'").stdout.strip(),"t")
        self.assertEqual(create(),"DC-R976")
        self.assertEqual(listp(),"DC-R976:0")
        self.assertEqual(create(),"DENIED")
        self.assertEqual(create(code="../bad"),"DENIED")
        self.assertEqual(create(code="DC-R976",tenant=OTHER,subject="admin-other"),"DC-R976")
        self.assertEqual(create(code="CROSS",tenant=OTHER,subject="admin"),"DENIED")
        self.assertEqual(listp(OTHER,subject="admin"),"")

    def test_02_cross_tenant_fk_and_cas_guard_association(self):
        self.assertEqual(assign(pop="CROSS"),"DENIED")
        self.assertEqual(assign(pop="DC-R976"),"2")
        self.assertEqual(assign(pop="DC-R976",rev=1),"DENIED")
        self.assertEqual(assign(pop="DC-R976",rev=2),"2")  # idempotent exact CAS
        self.assertEqual(listp(),"DC-R976:1")
        self.assertEqual(q(f"SELECT parent_pop_code FROM ipat_ops.tenant_sites "
            f"WHERE tenant_id='{T}' AND code='SITE-R976'").stdout.strip(),"DC-R976")

    def test_03_grants_never_expand_from_legacy_site_code_to_new_pop(self):
        # The old NOC grant names the old exact Site code. The new parent
        # cannot implicitly expand that grant to other linked Sites.
        legacy=q("SELECT code FROM ipat_platform.list_tenant_sites("
          f"'{ISS}','noc','{T}','SITE-R976',NULL)",role="ipat_tenant_api_login").stdout.strip()
        self.assertEqual(legacy,"SITE-R976")
        new=q("SELECT code FROM ipat_platform.list_tenant_sites("
          f"'{ISS}','noc','{T}','DC-R976',NULL)",role="ipat_tenant_api_login").stdout.strip()
        self.assertEqual(new,"")
        self.assertEqual(q("SELECT code||':'||parent_pop_code FROM "
         f"ipat_platform.list_tenant_sites_with_parent_pop('{ISS}','admin','{T}',NULL,NULL)",
         role="ipat_tenant_api_login").stdout.strip(),"SITE-R976:DC-R976")

    def test_04_pop_delete_guard_rename_and_append_only_audit(self):
        self.assertEqual(f("ipat_platform.delete_unused_tenant_pop",
          f"'{ISS}','admin','{T}','DC-R976',1"),"false")
        self.assertEqual(f("ipat_platform.rename_tenant_pop",
          f"'{ISS}','admin','{T}','DC-R976','Renamed POP',1"),"2")
        self.assertEqual(f("ipat_platform.rename_tenant_pop",
          f"'{ISS}','admin','{T}','DC-R976','Stale',1"),"DENIED")
        self.assertEqual(assign(pop=None,rev=2),"3")
        self.assertEqual(f("ipat_platform.delete_unused_tenant_pop",
          f"'{ISS}','admin','{T}','DC-R976',2"),"true")
        self.assertEqual(create(code="DC-R976"),"DENIED")  # tombstone: no POP identity reuse
        events=q("SELECT event FROM ipat_ops.tenant_pop_events WHERE "
          f"tenant_id='{T}' AND pop_code='DC-R976' ORDER BY id").stdout.strip().splitlines()
        self.assertEqual(events,["POP_CREATED","SITE_POP_ASSIGNED",
             "POP_RENAMED","SITE_POP_ASSIGNED","POP_DELETED"])

    def test_045_atomic_site_create_explicit_pop_and_no_fake_assignment(self):
        self.assertEqual(create("POP-ATOMIC"),"POP-ATOMIC")
        good=f("ipat_platform.create_tenant_site_with_pop",
          f"'{ISS}','admin','{T}','SITE-ATOMIC','New Site','POP-ATOMIC'")
        self.assertEqual(good,"SITE-ATOMIC")
        self.assertEqual(q("SELECT parent_pop_code||':'||revision FROM "
          f"ipat_ops.tenant_sites WHERE tenant_id='{T}' AND code='SITE-ATOMIC'"
          ).stdout.strip(),"POP-ATOMIC:1")
        self.assertEqual(f("ipat_platform.create_tenant_site_with_pop",
          f"'{ISS}','admin','{T}','SITE-NOPOP','Denied','MISSING'"),"DENIED")
        self.assertEqual(q("SELECT count(*) FROM ipat_ops.tenant_sites "
           f"WHERE tenant_id='{T}' AND code='SITE-NOPOP'").stdout.strip(),"0")
        # Legacy NOC exact-site grant never automatically receives this new Site.
        self.assertEqual(q("SELECT code FROM ipat_platform.list_tenant_sites("
          f"'{ISS}','noc','{T}','POP-ATOMIC',NULL)",
          role="ipat_tenant_api_login").stdout.strip(),"")
        self.assertEqual(q("SELECT code FROM ipat_platform.list_tenant_sites_with_parent_pop("
          f"'{ISS}','admin','{T}','SITE-ATOMIC',NULL)",
          role="ipat_tenant_api_login").stdout.strip().split("|")[0],"SITE-ATOMIC")

    def test_05_default_deny_and_revocation(self):
        self.assertEqual(q("SELECT has_table_privilege('ipat_tenant_api_login',"
         "'ipat_ops.tenant_pops','SELECT')::int").stdout.strip(),"0")
        self.assertNotEqual(q("SELECT * FROM ipat_ops.tenant_pops",
                        role="ipat_tenant_api_login",check=False).returncode,0)
        self.assertEqual(create(code="NO-NOC",subject="noc"),"DENIED")
        q(f"UPDATE ipat_platform.identity_memberships SET revoked_at=now() "
          f"WHERE tenant_id='{T}' AND subject='admin'")
        self.assertEqual(create(code="NO-REVOKED"),"DENIED")
        self.assertEqual(listp(),"")

if __name__=="__main__":
    unittest.main()
