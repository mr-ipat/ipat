"""R9.64 disposable-only two-phase real PostgreSQL Site/Device tenancy tests.

Actual commercial migration is NOT deployed by this test. Requires the existing
ordered disposable 0001-0017 CI, intentionally verified human mapping before
0019. No real tenant, device, IP, operator or credential may be used.
"""
import os
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA, TB, run, sql
from test_identity_memberships_integration import ISSUER, SUBJECT
from test_managed_device_registry_integration import ISSUER_B, SUBJECT_B
from test_scoped_identity_lookup_integration import ISSUER as NOC_ISSUER, SUBJECT as NOC_SUBJECT

ROOT=Path(__file__).resolve().parents[3]
STAGE=ROOT/'deploy/db/migrations/0018_tenant_sites_staged.sql'
VALIDATE=ROOT/'deploy/db/migrations/0019_tenant_sites_validate_fk.sql'
PREFLIGHT=ROOT/'deploy/db/migrations/0018_tenant_sites_backfill_preflight.sql'
SITE='ipat_platform'

def run_as_role(query, role='ipat_site_registry_exec', expect=True):
    return sql(f'SET ROLE {role}; {query}',expect=expect)

def make_site(code, name, tenant=TA, issuer=ISSUER, subject=SUBJECT):
    return run_as_role(f"SELECT COALESCE({SITE}.create_tenant_site("
      f"'{issuer}','{subject}','{tenant}','{code}','{name}'),'DENIED');")

def rename_site(code, name, revision, tenant=TA, issuer=ISSUER, subject=SUBJECT):
    return run_as_role(f"SELECT COALESCE({SITE}.rename_tenant_site("
      f"'{issuer}','{subject}','{tenant}','{code}','{name}',{revision})::text,'DENIED');")

def delete_site(code,revision,tenant=TA, issuer=ISSUER,subject=SUBJECT):
    return run_as_role(f"SELECT {SITE}.delete_tenant_site("
      f"'{issuer}','{subject}','{tenant}','{code}',{revision});")

def listing(tenant=TA,issuer=ISSUER,subject=SUBJECT,pop=None,after=None):
    p='NULL' if pop is None else "'"+pop+"'"
    a='NULL' if after is None else "'"+after+"'"
    return run_as_role(f"SELECT code,revision,assigned_devices FROM {SITE}.list_tenant_sites("
         f"'{issuer}','{subject}','{tenant}',{p},{a});").stdout.replace('SET\n','',1)

class TenantSiteRegistry(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get('IPAT_PG_EPHEMERAL_TEST')!='1':
            raise unittest.SkipTest('Requires explicitly opted-in synthetic disposable PostgreSQL')
        assert os.environ.get('PGDATABASE')=='ipat_synthetic'
        assert os.environ.get('PGHOST')=='127.0.0.1'
        assert os.environ.get('IPAT_PG_SYNTHETIC_PASSWORD')=='local_ci_synthetic_only'
        assert sql("SELECT to_regclass('ipat_ops.managed_devices') IS NOT NULL").stdout.strip()=='t'
        assert sql("SELECT to_regclass('ipat_ops.tenant_sites') IS NULL").stdout.strip()=='t'
        # Stable disposable fixtures, no password, real customer, default site
        # or inferred physical location. Synthetic actors were pre-approved in
        # previously executed identity/managed-registry CI fixture steps.
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
        sql(f"""UPDATE ipat_platform.identity_memberships
          SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day'
          WHERE (issuer='{ISSUER}' AND subject='{SUBJECT}') OR
            (issuer='{ISSUER_B}' AND subject='{SUBJECT_B}') OR
            (issuer='{NOC_ISSUER}' AND subject='{NOC_SUBJECT}')""")
        run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(STAGE)])
        # Nonvalidated FK MUST block NEW unknown Site codes immediately.
        assert sql("SELECT convalidated::int FROM pg_constraint WHERE "
            "conrelid='ipat_ops.managed_devices'::regclass AND "
            "conname='managed_devices_registered_site_fk'").stdout.strip()=='0'
        # The migration must refuse to self-heal old records: all mapping must
        # be explicitly reviewed and registered by the correct tenant admin.
        cls.unresolved=sql(f"SELECT tenant_id::text||'|'||pop_id FROM ("
           f"SELECT DISTINCT d.tenant_id,d.pop_id FROM ipat_ops.managed_devices d"
           f") q ORDER BY 1").stdout.strip().splitlines()
        assert cls.unresolved,'Expected pre-existing 0016 synthetic registry fixture'
        check=run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(VALIDATE)],expect=False)
        assert check.returncode!=0,'0019 must refuse legacy orphan records'
        assert 'EXPLICIT_TENANT_SITE_BACKFILL_REQUIRED' in check.stderr
        before=run(['psql','-X','-A','-t','-v','ON_ERROR_STOP=1','-f',str(PREFLIGHT)])
        assert 'missing_site_code' not in before.stdout or 'pop-a' in before.stdout
        # This is only a synthetic test fixture. In real deployment the
        # operator must independently prove each mapping, never auto-import.
        for ref in cls.unresolved:
            tenant,code=ref.split('|',1)
            actor=(ISSUER,SUBJECT) if tenant==TA else (ISSUER_B,SUBJECT_B) if tenant==TB else None
            assert actor is not None and code=='pop-a',(tenant,code)
            r=make_site(code,f'Synthetic confirmed {tenant[:6]} site',tenant,*actor)
            assert r.stdout.strip().splitlines()[-1]==code
        run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(VALIDATE)])

    def setUp(self):
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
        sql(f"""UPDATE ipat_platform.identity_memberships
         SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day'
         WHERE (issuer='{ISSUER}' AND subject='{SUBJECT}') OR
           (issuer='{ISSUER_B}' AND subject='{SUBJECT_B}') OR
           (issuer='{NOC_ISSUER}' AND subject='{NOC_SUBJECT}')""")

    def test_00_final_migration_validated_without_auto_created_unknown_site(self):
        self.assertEqual(sql("SELECT convalidated::int FROM pg_constraint WHERE "
          "conrelid='ipat_ops.managed_devices'::regclass AND "
          "conname='managed_devices_registered_site_fk'").stdout.strip(),'1')
        self.assertEqual(sql("SELECT count(*) FROM ipat_ops.managed_devices d LEFT JOIN "
          "ipat_ops.tenant_sites s ON s.tenant_id=d.tenant_id AND s.code=d.pop_id "
          "WHERE s.code IS NULL").stdout.strip(),'0')
        self.assertEqual(listing().count('pop-a'),1)
        self.assertEqual(listing(tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B).count('pop-a'),1)

    def test_01_no_public_or_runtime_table_functions_or_audit_mutation(self):
        q=sql("""SELECT
         has_function_privilege('ipat_site_registry_exec',
           'ipat_platform.create_tenant_site(text,text,uuid,text,text)','EXECUTE')::int,
         has_function_privilege('ipat_app_runtime',
           'ipat_platform.create_tenant_site(text,text,uuid,text,text)','EXECUTE')::int,
         has_table_privilege('ipat_site_registry_exec','ipat_ops.tenant_sites','SELECT')::int,
         has_table_privilege('ipat_site_registry_exec','ipat_ops.tenant_sites','INSERT')::int""")
        self.assertEqual(q.stdout.strip(),'1|0|0|0')
        for role in ['ipat_site_registry_owner','ipat_site_registry_exec']:
            self.assertEqual(sql(f"SELECT rolcanlogin::int,rolsuper::int,rolbypassrls::int FROM pg_roles WHERE rolname='{role}'").stdout.strip(),'0|0|0')
        for table in ['tenant_sites','tenant_site_events']:
            self.assertEqual(sql(f"SELECT relrowsecurity::int,relforcerowsecurity::int "
             f"FROM pg_class WHERE oid='ipat_ops.{table}'::regclass").stdout.strip(),'1|1')
            self.assertNotEqual(run_as_role(f'SELECT count(*) FROM ipat_ops.{table}',expect=False).returncode,0)
            self.assertNotEqual(run_as_role(f'DELETE FROM ipat_ops.{table}',expect=False).returncode,0)
        self.assertNotEqual(run_as_role('SELECT count(*) FROM ipat_ops.managed_devices',expect=False).returncode,0)

    def test_02_tenant_bound_duplicate_and_current_approval_enforced(self):
        self.assertEqual(make_site('SITE-A','Approved test Site').stdout.strip().splitlines()[-1],'SITE-A')
        self.assertEqual(make_site('SITE-A','Unauthorized duplicate').stdout.strip().splitlines()[-1],'DENIED')
        self.assertEqual(make_site('SITE-A','Same code other tenant',TB,ISSUER_B,SUBJECT_B).stdout.strip().splitlines()[-1],'SITE-A')
        self.assertEqual(make_site('TENANT-B-ONLY','Proven tenant B location',TB,ISSUER_B,SUBJECT_B).stdout.strip().splitlines()[-1],'TENANT-B-ONLY')
        self.assertEqual(make_site('SITE-C','Tenant B attempt',TB,ISSUER,SUBJECT).stdout.strip().splitlines()[-1],'DENIED')
        self.assertEqual(make_site('FORGED','No forged role',TA,'https://forged.invalid',SUBJECT).stdout.strip().splitlines()[-1],'DENIED')
        self.assertEqual(make_site('../ESCAPE','Invalid code').stdout.strip().splitlines()[-1],'DENIED')
        self.assertIn('SITE-A',listing())
        self.assertNotIn('SITE-A',listing(tenant=TB))
        self.assertEqual(listing(tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B).count('SITE-A'),1)
        self.assertIn('pop-a',listing(issuer=NOC_ISSUER,subject=NOC_SUBJECT,pop='pop-a'))
        self.assertNotIn('SITE-A',listing(issuer=NOC_ISSUER,subject=NOC_SUBJECT,pop='SITE-A'))
        self.assertEqual(listing(issuer=NOC_ISSUER,subject=NOC_SUBJECT,pop=None).strip(),'')

    def test_03_fk_rejects_unregistered_site_even_for_real_admin_and_cross_tenant(self):
        # Existing 0016 SECURITY DEFINER routine has NO direct access to the
        # site master, yet its INSERT is constrained by the tenant-bound FK.
        for pop in ['TENANT-B-ONLY','SITE-A']:
            res=run_as_role("SELECT COALESCE(ipat_platform.register_managed_device("
              f"'{ISSUER}','{SUBJECT}','{TA}',"
              f"'99999999-9999-4999-8999-999999999991',"
              f"'99999999-9999-4999-8999-999999999992',"
              f"'{pop}','Candidate OLT','olt','ZTE','C320','ssh',"
              "'olt.fixture.invalid',22,NULL)::text,'DENIED');",role='ipat_managed_registry_exec')
            self.assertEqual(res.stdout.strip().splitlines()[-1],
                'DENIED' if pop=='TENANT-B-ONLY' else '99999999-9999-4999-8999-999999999991')
        invalid=run_as_role("SELECT COALESCE(ipat_platform.register_managed_device("
              f"'{ISSUER}','{SUBJECT}','{TA}',"
              f"'99999999-9999-4999-8999-999999999993',"
              f"'99999999-9999-4999-8999-999999999994',"
              "'pop-b','Cross-company injection','olt','ZTE','C320','ssh',"
              "'olt.fixture.invalid',22,NULL)::text,'DENIED');",role='ipat_managed_registry_exec')
        self.assertEqual(invalid.stdout.strip().splitlines()[-1],'DENIED')
        self.assertIn('SITE-A|1|1',listing())

    def test_04_rename_cas_delete_protection_and_immutable_audit(self):
        self.assertEqual(rename_site('SITE-A','Renamed production Site',1).stdout.strip().splitlines()[-1],'2')
        self.assertEqual(rename_site('SITE-A','Stale update',1).stdout.strip().splitlines()[-1],'DENIED')
        self.assertEqual(delete_site('SITE-A',2).stdout.strip().splitlines()[-1],'f')
        self.assertIn('SITE-A|2|1',listing())
        self.assertEqual(make_site('UNUSED-01','Unused separate Site').stdout.strip().splitlines()[-1],'UNUSED-01')
        self.assertEqual(delete_site('UNUSED-01',999).stdout.strip().splitlines()[-1],'f')
        self.assertEqual(delete_site('UNUSED-01',1).stdout.strip().splitlines()[-1],'t')
        self.assertEqual(delete_site('UNUSED-01',1).stdout.strip().splitlines()[-1],'f')
        actions=sql(f"SELECT event FROM ipat_ops.tenant_site_events WHERE tenant_id='{TA}' AND site_code='UNUSED-01' ORDER BY id").stdout.strip().splitlines()
        self.assertEqual(actions,['SITE_CREATED','SITE_DELETED'])
        self.assertNotIn('UNUSED-01',listing())

    def test_05_revocation_suspension_and_keyset_cursor_are_denied_or_scoped(self):
        self.assertNotIn('pop-a',listing(after='pop-a'))
        self.assertIn('SITE-A',listing(after='SITE-9'))
        sql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{TA}'")
        self.assertEqual(listing().strip(),'')
        self.assertEqual(make_site('SHOULD-DENY','While suspended').stdout.strip().splitlines()[-1],'DENIED')
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id='{TA}'")
        sql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() "
          f"WHERE tenant_id='{TA}' AND issuer='{ISSUER}' AND subject='{SUBJECT}'")
        self.assertEqual(listing().strip(),'')
        self.assertEqual(rename_site('SITE-A','Denied after revoke',2).stdout.strip().splitlines()[-1],'DENIED')
        self.assertEqual(delete_site('SITE-A',2).stdout.strip().splitlines()[-1],'f')
        self.assertIn('SITE-A',listing(tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B))

    def test_06_concurrent_duplicate_create_is_serialized_and_audited_once(self):
        def attempt(n):
            return make_site('ONE-TIME-CODE',f'Concurrent attempt {n}').stdout.strip().splitlines()[-1]
        with ThreadPoolExecutor(max_workers=5) as executor:
            outcomes=list(executor.map(attempt,range(5)))
        self.assertEqual(outcomes.count('ONE-TIME-CODE'),1)
        self.assertEqual(outcomes.count('DENIED'),4)
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_ops.tenant_site_events WHERE "
           f"tenant_id='{TA}' AND site_code='ONE-TIME-CODE' AND event='SITE_CREATED'").stdout.strip(),'1')
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_ops.tenant_sites WHERE "
           f"tenant_id='{TA}' AND code='ONE-TIME-CODE'").stdout.strip(),'1')

if __name__=='__main__':unittest.main()
