"""R9.65 live disposable PostgreSQL tenant Device metadata CRUD safety suite.

Requires R9.57/R9.58/R9.64 ordered synthetic migration suite first.
No production database/device/credential/network endpoint is contacted.
"""
import os
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA, TB, sql, run
from test_identity_memberships_integration import ISSUER, SUBJECT
from test_managed_device_registry_integration import ISSUER_B, SUBJECT_B
from test_scoped_identity_lookup_integration import ISSUER as NOC_ISSUER, SUBJECT as NOC_SUBJECT

ROOT=Path(__file__).resolve().parents[3]
MIGRATION=ROOT/'deploy/db/migrations/0020_managed_device_metadata_crud.sql'
A='96500000-0000-4000-8000-000000000001'
B='96500000-0000-4000-8000-000000000002'
REQ_A='96510000-0000-4000-8000-000000000001'
REQ_B='96510000-0000-4000-8000-000000000002'
ADMIN='ipat_managed_registry_exec'

def role(q,who=ADMIN,expect=True):
    return sql(f'SET ROLE {who}; {q}',expect=expect)
def value(q,who=ADMIN):
    return role(q,who).stdout.strip().splitlines()[-1]
def register(dev=A,req=REQ_A,tenant=TA,issuer=ISSUER,subject=SUBJECT,site='pop-a',secret=None):
    ref='NULL' if secret is None else "'"+secret+"'"
    return value(f"""SELECT COALESCE(ipat_platform.register_managed_device(
      '{issuer}','{subject}','{tenant}','{dev}','{req}',
      '{site}','Fixture saved before physical enrollment','olt','ZTE','C320',
      'ssh','olt.fixture.invalid',22,{ref})::text,'DENIED')""")
def edit(dev=A,rev=1,tenant=TA,issuer=ISSUER,subject=SUBJECT,name='Edited without CLI',site='pop-a',model='C320',host='olt.fixture.invalid',port=22):
    m='NULL' if model is None else "'"+model+"'"
    h='NULL' if host is None else "'"+host+"'"
    p='NULL' if port is None else str(port)
    return value(f"""SELECT COALESCE(ipat_platform.edit_managed_device_metadata(
        '{issuer}','{subject}','{tenant}','{dev}',{rev},
        '{name}','{site}',{m},{h},{p})::text,'DENIED')""")
def archive(dev=A,rev=1,tenant=TA,issuer=ISSUER,subject=SUBJECT):
    return value(f"SELECT ipat_platform.archive_managed_device_metadata("
      f"'{issuer}','{subject}','{tenant}','{dev}',{rev})")
def detail(dev=A,tenant=TA,issuer=ISSUER,subject=SUBJECT):
    return role(f"SELECT id::text,metadata_revision,lifecycle_state,display_name,"
      f"management_host,management_port,pop_id,archived_site_instance_id,archived_site_name "
      f"FROM ipat_platform.get_managed_device_metadata("
      f"'{issuer}','{subject}','{tenant}','{dev}')").stdout

def listed_admin(tenant=TA,issuer=ISSUER,subject=SUBJECT,archived=False,after_time='NULL',after_id='NULL'):
    a='true' if archived else 'false'
    return role(f"""SELECT id::text,lifecycle_state,metadata_revision FROM
      ipat_platform.list_managed_devices_admin('{issuer}','{subject}','{tenant}',
      {a},{after_time},{after_id})""").stdout

class ManagedDeviceMetadataCrud(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get('IPAT_PG_EPHEMERAL_TEST')!='1':
            raise unittest.SkipTest('Opt-in disposable database ONLY')
        assert os.environ.get('PGDATABASE')=='ipat_synthetic'
        assert os.environ.get('PGHOST')=='127.0.0.1'
        assert os.environ.get('IPAT_PG_SYNTHETIC_PASSWORD')=='local_ci_synthetic_only'
        assert sql("SELECT to_regclass('ipat_ops.firmware_changes') IS NOT NULL").stdout.strip()=='t'
        assert sql("SELECT convalidated::int FROM pg_constraint WHERE "
          "conname='managed_devices_registered_site_fk' AND "
          "conrelid='ipat_ops.managed_devices'::regclass").stdout.strip()=='1'
        assert sql("SELECT to_regclass('ipat_ops.managed_device_events') IS NULL").stdout.strip()=='t'
        run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(MIGRATION)])
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
        sql(f"""UPDATE ipat_platform.identity_memberships SET revoked_at=NULL,
         expires_at=clock_timestamp()+interval '1 day' WHERE
         (issuer='{ISSUER}' AND subject='{SUBJECT}') OR
         (issuer='{ISSUER_B}' AND subject='{SUBJECT_B}') OR
         (issuer='{NOC_ISSUER}' AND subject='{NOC_SUBJECT}')""")
        assert register()==A
        assert register(B,REQ_B,TB,ISSUER_B,SUBJECT_B)==B

    def setUp(self):
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
        sql(f"""UPDATE ipat_platform.identity_memberships SET revoked_at=NULL,
         expires_at=clock_timestamp()+interval '1 day' WHERE
         (issuer='{ISSUER}' AND subject='{SUBJECT}') OR
         (issuer='{ISSUER_B}' AND subject='{SUBJECT_B}') OR
         (issuer='{NOC_ISSUER}' AND subject='{NOC_SUBJECT}')""")

    def test_00_restricted_exec_roles_and_force_rls(self):
        self.assertEqual(sql("""SELECT has_function_privilege('ipat_managed_registry_exec',
          'ipat_platform.edit_managed_device_metadata(text,text,uuid,uuid,bigint,text,text,text,text,integer)',
          'EXECUTE')::int,has_function_privilege('ipat_app_runtime',
          'ipat_platform.archive_managed_device_metadata(text,text,uuid,uuid,bigint)',
          'EXECUTE')::int,has_table_privilege('ipat_managed_registry_exec',
          'ipat_ops.managed_devices','UPDATE')::int""").stdout.strip(),'1|0|0')
        self.assertEqual(sql("SELECT relrowsecurity::int,relforcerowsecurity::int FROM "
          "pg_class WHERE oid='ipat_ops.managed_device_events'::regclass").stdout.strip(),'1|1')
        for table in ('managed_devices','managed_device_events','firmware_changes'):
            self.assertNotEqual(role(f'SELECT count(*) FROM ipat_ops.{table}',expect=False).returncode,0)
        self.assertNotEqual(role('DELETE FROM ipat_ops.managed_devices',expect=False).returncode,0)

    def test_01_exact_tenant_admin_view_but_no_secret_or_cross_tenant_leak(self):
        self.assertIn(A,detail())
        self.assertIn(B,detail(dev=B,tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B))
        self.assertNotIn(A,detail(tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B))
        self.assertNotIn(B,listed_admin())
        self.assertNotIn(A,listed_admin(tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B))
        self.assertEqual(detail(issuer=NOC_ISSUER,subject=NOC_SUBJECT).replace('SET\n','').strip(),'')
        self.assertEqual(detail(issuer='https://forged.invalid').replace('SET\n','').strip(),'')
        cols=sql("SELECT pg_get_function_result('ipat_platform.get_managed_device_metadata(text,text,uuid,uuid)'::regprocedure)").stdout.lower()
        self.assertNotIn('secret_ref',cols)

    def test_02_stale_cas_edit_foreign_site_and_invalid_endpoint_fail(self):
        self.assertEqual(edit(name='Updated label & metadata'), '2')
        self.assertEqual(edit(rev=1,name='Stale label'),'DENIED')
        # Call the independent role; registry cannot mint a Site by itself.
        b_site=role(f"SELECT ipat_platform.create_tenant_site("
          f"'{ISSUER_B}','{SUBJECT_B}','{TB}','B-ONLY','Tenant B only')",who='ipat_site_registry_exec')
        self.assertIn('B-ONLY',b_site.stdout)
        self.assertEqual(edit(rev=2,site='B-ONLY'),'DENIED')
        self.assertEqual(edit(rev=2,host='root:password@invalid'),'DENIED')
        self.assertEqual(edit(rev=2,port=0),'DENIED')
        self.assertEqual(edit(rev=2,model=''),'DENIED')
        self.assertEqual(edit(rev=2,issuer=NOC_ISSUER,subject=NOC_SUBJECT),'DENIED')
        self.assertEqual(edit(rev=2,tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B),'DENIED')
        self.assertEqual(edit(rev=2,site='pop-a',name='Valid revision'),'3')
        self.assertIn('3',detail())
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_ops.managed_device_events WHERE tenant_id='{TA}' AND device_id='{A}' AND event='METADATA_EDITED'").stdout.strip(),'2')

    def test_03_rejects_secret_backed_and_active_firmware_metadata_removal(self):
        secret=f'vault://tenant/{TA}/devices/isolated'
        cred='96500000-0000-4000-8000-000000000003'
        req='96510000-0000-4000-8000-000000000003'
        self.assertEqual(register(cred,req,secret=secret),cred)
        self.assertEqual(archive(cred),'f')
        self.assertEqual(edit(dev=cred,name='Label allowed',host='other.invalid'),'DENIED')
        self.assertEqual(edit(dev=cred,name='Label allowed'),'2')
        self.assertEqual(archive(cred,2),'f')
        artifact='96520000-0000-4000-8000-000000000001'
        change='96530000-0000-4000-8000-000000000001'
        sql(f"""INSERT INTO ipat_ops.firmware_artifacts(
           tenant_id,id,vendor,exact_model,target_version,size_bytes,sha256,object_ref,
           vendor_release_ref,uploaded_by_issuer,uploaded_by_subject)
         VALUES ('{TA}','{artifact}','ZTE','C320','SYNTHETIC-UPDATE',2048,'{'a'*64}',
           'artifact://tenant/{TA}/firmware/{artifact}',
           'synthetic-firmware-integration-fixture','{ISSUER}','{SUBJECT}')""")
        sql(f"""INSERT INTO ipat_ops.firmware_changes(tenant_id,id,device_id,artifact_id,
           request_id,reason,window_start,window_end,requested_by_issuer,requested_by_subject)
          VALUES ('{TA}','{change}','{A}','{artifact}',
           '96540000-0000-4000-8000-000000000001',
           'synthetic dry run pending change',now()+interval '1 day',
           now()+interval '1 day 1 hour','{ISSUER}','{SUBJECT}')""")
        self.assertEqual(edit(rev=3,name='Blocked during pending firmware'),'DENIED')
        self.assertEqual(archive(rev=3),'f')
        self.assertEqual(sql(f"SELECT lifecycle_state FROM ipat_ops.managed_devices WHERE tenant_id='{TA}' AND id='{A}'").stdout.strip(),'SAVED')

    def test_04_parallel_compare_and_swap_allows_one_mutation(self):
        extra='96500000-0000-4000-8000-000000000004'
        req='96510000-0000-4000-8000-000000000004'
        self.assertEqual(register(extra,req),extra)
        with ThreadPoolExecutor(max_workers=5) as exe:
            results=list(exe.map(lambda i: edit(dev=extra,name=f'CAS change {i}'),range(5)))
        self.assertEqual(results.count('2'),1)
        self.assertEqual(results.count('DENIED'),4)
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_ops.managed_device_events WHERE device_id='{extra}' AND event='METADATA_EDITED'").stdout.strip(),'1')

    def test_05_archive_is_metadata_only_audited_and_never_replayed(self):
        extra='96500000-0000-4000-8000-000000000005'
        req='96510000-0000-4000-8000-000000000005'
        self.assertEqual(register(extra,req),extra)
        self.assertEqual(archive(extra,rev=1),'t')
        self.assertEqual(archive(extra,rev=1),'f')
        self.assertEqual(edit(extra,rev=2),'DENIED')
        self.assertEqual(register(extra,req),'DENIED')
        self.assertNotIn(extra,listed_admin())
        self.assertIn(extra,listed_admin(archived=True))
        self.assertIn('ARCHIVED',detail(extra))
        self.assertEqual(sql(f"SELECT lifecycle_state,metadata_revision,(archived_at IS NOT NULL)::int FROM ipat_ops.managed_devices WHERE id='{extra}'").stdout.strip(),'ARCHIVED|2|1')
        self.assertEqual(sql(f"SELECT event FROM ipat_ops.managed_device_events WHERE device_id='{extra}'").stdout.strip(),'METADATA_ARCHIVED')
        self.assertEqual(sql("SELECT count(*) FROM ipat_ops.tenant_sites WHERE tenant_id='"+TA+"' AND code='pop-a'").stdout.strip(),'1')

    def test_05b_archived_device_releases_site_with_historical_site_instance(self):
        # Correct operational flow: archiving a metadata-only candidate must
        # not permanently prevent deleting an otherwise unused Site. A stable
        # Site instance UUID + display name survive even if code is reused.
        name='RELEASABLE-SITE'
        new='96500000-0000-4000-8000-000000000009'
        req='96510000-0000-4000-8000-000000000009'
        result=role(f"SELECT ipat_platform.create_tenant_site("
          f"'{ISSUER}','{SUBJECT}','{TA}','{name}','Disposable Site for archive')",
          who='ipat_site_registry_exec')
        self.assertIn(name,result.stdout)
        before=sql(f"SELECT site_instance_id::text FROM ipat_ops.tenant_sites WHERE "
          f"tenant_id='{TA}' AND code='{name}'").stdout.strip()
        self.assertEqual(len(before),36)
        self.assertEqual(register(new,req,site=name),new)
        assigned=role(f"SELECT assigned_devices FROM ipat_platform.list_tenant_sites("
          f"'{ISSUER}','{SUBJECT}','{TA}','{name}',NULL)",who='ipat_site_registry_exec').stdout
        self.assertIn('1',assigned)
        self.assertEqual(archive(new),'t')
        self.assertEqual(sql(f"SELECT pop_id IS NULL,archived_site_code,"
          f"archived_site_instance_id::text,archived_site_name FROM "
          f"ipat_ops.managed_devices WHERE id='{new}'").stdout.strip(),
          f't|{name}|{before}|Disposable Site for archive')
        self.assertIn(name,detail(new))
        self.assertEqual(role(f"SELECT ipat_platform.delete_tenant_site("
          f"'{ISSUER}','{SUBJECT}','{TA}','{name}',1)",who='ipat_site_registry_exec').stdout.strip().splitlines()[-1],'t')
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_ops.tenant_sites WHERE "
          f"tenant_id='{TA}' AND code='{name}'").stdout.strip(),'0')
        self.assertIn('ARCHIVED',detail(new))
        self.assertIn(name,detail(new))
        self.assertEqual(sql(f"SELECT previous_site_instance_id::text,new_site IS NULL FROM "
          f"ipat_ops.managed_device_events WHERE tenant_id='{TA}' "
          f"AND device_id='{new}' AND event='METADATA_ARCHIVED'").stdout.strip(),f'{before}|t')
        result=role(f"SELECT ipat_platform.create_tenant_site("
          f"'{ISSUER}','{SUBJECT}','{TA}','{name}','New different site generation')",
          who='ipat_site_registry_exec')
        self.assertIn(name,result.stdout)
        newer=sql(f"SELECT site_instance_id::text FROM ipat_ops.tenant_sites WHERE "
          f"tenant_id='{TA}' AND code='{name}'").stdout.strip()
        self.assertNotEqual(newer,before)
        self.assertEqual(sql(f"SELECT archived_site_instance_id::text FROM "
          f"ipat_ops.managed_devices WHERE id='{new}'").stdout.strip(),before)

    def test_06_revoked_suspended_and_noc_cannot_edit_or_archive(self):
        new='96500000-0000-4000-8000-000000000006'
        req='96510000-0000-4000-8000-000000000006'
        self.assertEqual(register(new,req),new)
        self.assertEqual(edit(new,issuer=NOC_ISSUER,subject=NOC_SUBJECT),'DENIED')
        self.assertEqual(archive(new,issuer=NOC_ISSUER,subject=NOC_SUBJECT),'f')
        sql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{TA}'")
        self.assertEqual(edit(new),'DENIED')
        self.assertEqual(archive(new),'f')
        self.assertNotIn(new,listed_admin())
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id='{TA}'")
        sql(f"UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() WHERE issuer='{ISSUER}' AND subject='{SUBJECT}' AND tenant_id='{TA}'")
        self.assertEqual(edit(new),'DENIED')
        self.assertEqual(archive(new),'f')
        self.assertNotIn(new,listed_admin())

    def test_07_keyset_admin_pages_work_with_same_timestamp_and_no_duplicate(self):
        # Test synthetic rows: 102 records with equal created_at show UUID
        # cursor tie-breaking and explicit 101-row lookahead for the BFF.
        ids=sql(f"""INSERT INTO ipat_ops.managed_devices(tenant_id,id,request_id,pop_id,
          display_name,device_kind,vendor,management_transport,management_host,management_port,
          added_by_issuer,added_by_subject,created_at)
          SELECT '{TA}',gen_random_uuid(),gen_random_uuid(),'pop-a',
          'Synthetic page fixture','olt','ZTE','ssh','olt.fixture.invalid',22,
          '{ISSUER}','{SUBJECT}', '2025-01-01 UTC'::timestamptz FROM generate_series(1,102)
          RETURNING id""").stdout
        self.assertEqual(ids.count('-'),102*4)
        # Page values are restricted to the exact same tenant; both cursor
        # components must be given together or pagination fails closed.
        first=role(f"SELECT id::text FROM ipat_platform.list_managed_devices_admin("
          f"'{ISSUER}','{SUBJECT}','{TA}',false,NULL,NULL)").stdout.strip().splitlines()[1:]
        self.assertEqual(len(first),101)
        cursor=sql(f"SELECT created_at::text,id::text FROM ipat_ops.managed_devices WHERE id='{first[99]}'").stdout.strip().split('|')
        self.assertEqual(len(cursor),2)
        t,id_=cursor
        next_=role(f"SELECT id::text FROM ipat_platform.list_managed_devices_admin("
          f"'{ISSUER}','{SUBJECT}','{TA}',false,'{t}'::timestamptz,'{id_}'::uuid)").stdout.strip().splitlines()[1:]
        self.assertGreater(len(next_),0)
        self.assertTrue(set(next_).isdisjoint(first[:100]))

if __name__=='__main__':unittest.main()
