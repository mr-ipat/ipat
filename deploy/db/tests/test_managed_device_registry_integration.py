"""R9.57 disposable-only production-path metadata registry and privilege tests.
Run after identity (0003/0004), domain/verification migrations (0012-0015).
No real hosts, credentials, network probes, or live migrations are used.
"""
import os
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA, TB, run, sql
from test_identity_memberships_integration import ISSUER, SUBJECT
from test_scoped_identity_lookup_integration import ISSUER as NOC_ISSUER, SUBJECT as NOC_SUBJECT

ROOT = Path(__file__).resolve().parents[3]
MIGRATION = ROOT / 'deploy/db/migrations/0016_managed_device_registry.sql'
A = '77777777-7777-4777-8777-777777777771'
B = '77777777-7777-4777-8777-777777777772'
REQ_A = '88888888-8888-4888-8888-888888888881'
REQ_B = '88888888-8888-4888-8888-888888888882'
ISSUER_B = 'https://synthetic.registry.invalid/tenant-b'
SUBJECT_B = 'tenant-b-owner'
FN = 'ipat_platform.register_managed_device'
ARGS = '(text,text,uuid,uuid,uuid,text,text,text,text,text,text,text,integer,text)'

def qrole(statement, role='ipat_managed_registry_exec', expect=True):
    return sql(f'SET ROLE {role}; {statement}', expect=expect)

def reg(tenant=TA, issuer=ISSUER, subject=SUBJECT, dev=A, request=REQ_A,
        name='C320 offline fixture', host='olt.example.invalid', ref=None):
    r = 'NULL' if ref is None else "'" + ref.replace("'", "''") + "'"
    return qrole(f"""SELECT COALESCE({FN}(
      '{issuer}','{subject}','{tenant}','{dev}','{request}',
      'pop-a','{name}','olt','ZTE','C320','ssh','{host}',22,{r})::text,'DENIED');""").stdout.strip().splitlines()[-1]

def listed(issuer=ISSUER, subject=SUBJECT, tenant=TA, pop=None):
    p = 'NULL' if pop is None else f"'{pop}'"
    return qrole(f"SELECT id::text FROM ipat_platform.list_managed_devices('{issuer}','{subject}','{tenant}',{p})").stdout

class ManagedDeviceRegistry(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get('IPAT_PG_EPHEMERAL_TEST') != '1':
            raise unittest.SkipTest('Opt-in disposable PostgreSQL only')
        assert os.environ.get('PGDATABASE') == 'ipat_synthetic'
        assert os.environ.get('PGHOST') == '127.0.0.1'
        assert os.environ.get('IPAT_PG_SYNTHETIC_PASSWORD') == 'local_ci_synthetic_only'
        assert sql("SELECT to_regprocedure('ipat_platform.lookup_active_membership(text,text,uuid,text,text)') IS NOT NULL").stdout.strip() == 't'
        assert sql("SELECT to_regclass('ipat_ops.managed_devices') IS NULL").stdout.strip() == 't'
        run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(MIGRATION)])
        # Distinct second tenant's approved admin, never a global operator.
        sql(f"""INSERT INTO ipat_platform.identity_memberships
           (tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
           ('{TB}','{ISSUER_B}','{SUBJECT_B}','tenant_admin','R957-reviewer',
            statement_timestamp()+interval '1 day')""")

    def setUp(self):
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
        sql(f"""UPDATE ipat_platform.identity_memberships
          SET revoked_at=NULL,expires_at=statement_timestamp()+interval '1 day'
          WHERE (issuer='{ISSUER}' AND subject='{SUBJECT}')
             OR (issuer='{ISSUER_B}' AND subject='{SUBJECT_B}')""")

    def test_00_owner_only_exec_no_direct_table_access(self):
        rights = sql(f"""SELECT
         has_function_privilege('ipat_managed_registry_exec','{FN}{ARGS}','EXECUTE')::int,
         has_function_privilege('ipat_app_runtime','{FN}{ARGS}','EXECUTE')::int,
         has_table_privilege('ipat_managed_registry_exec','ipat_ops.managed_devices','SELECT')::int,
         has_table_privilege('ipat_managed_registry_exec','ipat_ops.managed_devices','INSERT')::int""").stdout.strip()
        self.assertEqual(rights,'1|0|0|0')
        for tab in ('managed_devices','managed_device_audit'):
            self.assertNotEqual(qrole(f'SELECT count(*) FROM ipat_ops.{tab}',expect=False).returncode,0)
            self.assertNotEqual(qrole(f'DELETE FROM ipat_ops.{tab}',expect=False).returncode,0)
            flags=sql(f"""SELECT relrowsecurity::int,relforcerowsecurity::int
                 FROM pg_class WHERE oid='ipat_ops.{tab}'::regclass""").stdout.strip()
            self.assertEqual(flags,'1|1')

    def test_01_save_immediately_listed_as_pending_metadata_only(self):
        self.assertEqual(reg(),A)
        self.assertEqual(reg(),A)  # exact idempotent replay
        self.assertEqual(reg(dev='77777777-7777-4777-8777-777777777779'),A)  # new generated device UUID on HTTP retry
        self.assertIn(A,listed())
        self.assertNotIn(A,listed(tenant=TB))
        self.assertNotIn(A,listed(issuer='https://fake.invalid',subject=SUBJECT))
        self.assertEqual(sql(f"SELECT lifecycle_state FROM ipat_ops.managed_devices WHERE id='{A}'").stdout.strip(),'SAVED')
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_ops.managed_device_audit WHERE device_id='{A}'").stdout.strip(),'1')
        self.assertEqual(reg(name='modified on replay'),'DENIED')

    def test_02_tenant_bound_admin_and_noc_read_only_pop(self):
        self.assertEqual(reg(tenant=TB,issuer=ISSUER_B,subject=SUBJECT_B,dev=B,request=REQ_B),B)
        self.assertNotIn(B,listed())
        self.assertIn(B,listed(issuer=ISSUER_B,subject=SUBJECT_B,tenant=TB))
        self.assertEqual(reg(tenant=TB,issuer=ISSUER,subject=SUBJECT,dev=B,request=REQ_B),'DENIED')
        self.assertIn(A,listed(issuer=NOC_ISSUER,subject=NOC_SUBJECT,pop='pop-a'))
        self.assertNotIn(A,listed(issuer=NOC_ISSUER,subject=NOC_SUBJECT,pop='pop-b'))
        self.assertEqual(reg(issuer=NOC_ISSUER,subject=NOC_SUBJECT,request='88888888-8888-4888-8888-888888888889'),'DENIED')

    def test_03_revoked_or_suspended_scope_denied_without_mutation(self):
        sql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{TA}'")
        self.assertNotIn(A,listed())
        self.assertEqual(reg(),'DENIED')
        self.assertIn(B,listed(issuer=ISSUER_B,subject=SUBJECT_B,tenant=TB))
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id='{TA}'")
        sql(f"""UPDATE ipat_platform.identity_memberships SET revoked_at=statement_timestamp()
           WHERE tenant_id='{TA}' AND issuer='{ISSUER}' AND subject='{SUBJECT}'""")
        self.assertNotIn(A,listed())
        self.assertEqual(reg(),'DENIED')

    def test_04_secret_refs_are_tenant_scoped_no_plaintext_credentials(self):
        req='88888888-8888-4888-8888-888888888884'
        dev='77777777-7777-4777-8777-777777777774'
        self.assertEqual(reg(dev=dev,request=req,ref='password-in-a-db'),'DENIED')
        self.assertEqual(reg(dev=dev,request=req,ref=f'vault://tenant/{TB}/devices/olt'),'DENIED')
        self.assertEqual(reg(dev=dev,request=req,ref=f'vault://tenant/{TA}/devices/olt'),dev)
        row=sql(f"SELECT secret_ref FROM ipat_ops.managed_devices WHERE id='{dev}'").stdout.strip()
        self.assertEqual(row,f'vault://tenant/{TA}/devices/olt')
        self.assertNotIn('vault://',listed())
        self.assertNotIn('olt.example.invalid',listed())

    def test_05_backend_must_not_trust_forged_headers(self):
        # DB accepts only exact signed identity passed by trusted BFF; no
        # SQL function has an X-Tenant/X-Role argument or header parser.
        self.assertEqual(reg(issuer='https://forged.invalid',subject=SUBJECT,
          dev='77777777-7777-4777-8777-777777777775',
          request='88888888-8888-4888-8888-888888888885'),'DENIED')
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id='{TA}'")
        self.assertNotIn(B,listed())

if __name__=='__main__':
    unittest.main()
