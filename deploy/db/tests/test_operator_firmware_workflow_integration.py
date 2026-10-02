"""R9.58: real synthetic PG checks of operator-only firmware orchestration.
No binary uploads, live roles, real DNS, remote connections or OLT actions.
Runs in CI ONLY after R9.57 migration/tests with fixed disposable DB.
"""
import os
from pathlib import Path
import unittest
from test_postgres_rls_integration import TA,TB,sql,run
from test_identity_memberships_integration import ISSUER,SUBJECT

ROOT=Path(__file__).resolve().parents[3]
MIG=ROOT/'deploy/db/migrations/0017_operator_firmware_workflow.sql'
REVIEW_ISS='https://synthetic.firmware.invalid/reviewer'
REVIEW_SUB='independent-firmware-reviewer'
SHA='a'*64
ART='f1111111-1111-4111-8111-111111111111'
ART_B='f2222222-2222-4222-8222-222222222222'

def quote(s):return "'"+s.replace("'","''")+"'"
def role(sqlstmt,who='ipat_fw_api_execute',expect=True):
    return sql(f'SET ROLE {who}; {sqlstmt}',expect=expect)
def result(sqlstmt,who='ipat_fw_api_execute'):
    return role(sqlstmt,who).stdout.strip().splitlines()[-1]
def stage(tenant=TA,issuer=ISSUER,subject=SUBJECT,artifact=ART,model='C320',sha=SHA):
    return result(f"""SELECT COALESCE(ipat_platform.stage_firmware_artifact(
       {quote(issuer)},{quote(subject)},'{tenant}','{artifact}',
       'ZTE',{quote(model)},'SYNTHETIC-NOT-A-VENDOR-RELEASE',1024,'{sha}',
       'artifact://tenant/{tenant}/firmware/{artifact}',
       'synthetic-release-for-negative-tests')::text,'DENIED')""")
def register(dev,request):
    return result(f"""SELECT COALESCE(ipat_platform.register_managed_device(
      '{ISSUER}','{SUBJECT}','{TA}','{dev}','{request}',
      'pop-a','Synthetic ZTE laboratory fixture','olt','ZTE','C320','ssh',
      'olt.example.invalid',22,NULL)::text,'DENIED')""",'ipat_managed_registry_exec')
def propose(device,change,request,window,artifact=ART,tenant=TA,issuer=ISSUER,subject=SUBJECT):
    st,en=window
    return result(f"""SELECT COALESCE(ipat_platform.propose_firmware_change(
      '{issuer}','{subject}','{tenant}','{change}','{device}','{artifact}',
      '{request}','Synthetic planned maintenance only',{quote(st)}::timestamptz,
      {quote(en)}::timestamptz)::text,'DENIED')""")
def attest(change,kind,sha=SHA,tenant=TA,observed_vendor='NULL',observed_model='NULL',observed_version='NULL'):
    return result(f"""SELECT ipat_platform.attest_firmware_evidence(
      '{tenant}','{change}','{kind}','{sha}',{observed_vendor},{observed_model},{observed_version})::int""",
      'ipat_fw_attestor_execute')
def review(change,issuer=REVIEW_ISS,subject=REVIEW_SUB,approve=True,tenant=TA):
    val='true' if approve else 'false'
    return result(f"""SELECT COALESCE(ipat_platform.review_firmware_change(
      '{issuer}','{subject}','{tenant}','{change}',{val}),'DENIED')""")
def execution(change,issuer=ISSUER,subject=SUBJECT,tenant=TA):
    return result(f"""SELECT COALESCE(ipat_platform.request_firmware_execution(
      '{issuer}','{subject}','{tenant}','{change}'),'DENIED')""")
def all_attest(change):
    for kind in ('VENDOR_RELEASE','DEVICE_IDENTITY','BACKUP_RESTORE','IMPACT_BASELINE','RECOVERY_PATH','ADAPTER_QUALIFIED'):
        vendor="'ZTE'" if kind=='DEVICE_IDENTITY' else 'NULL'
        model="'C320'" if kind=='DEVICE_IDENTITY' else 'NULL'
        version="'SYNTHETIC-RUNNING'" if kind=='DEVICE_IDENTITY' else 'NULL'
        assert attest(change,kind,observed_vendor=vendor,observed_model=model,observed_version=version)=='1'

@unittest.skipUnless(os.getenv('IPAT_PG_EPHEMERAL_TEST')=='1','synthetic PostgreSQL only')
class OperatorFirmwareWorkflow(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.environ['PGDATABASE']=='ipat_synthetic'
        assert os.environ['PGHOST']=='127.0.0.1'
        assert os.environ['IPAT_PG_SYNTHETIC_PASSWORD']=='local_ci_synthetic_only'
        assert sql("SELECT to_regclass('ipat_ops.managed_devices') IS NOT NULL").stdout.strip()=='t'
        assert sql("SELECT to_regclass('ipat_ops.firmware_changes') IS NULL").stdout.strip()=='t'
        run(['psql','-X','-v','ON_ERROR_STOP=1','-f',str(MIG)])
        sql(f"""INSERT INTO ipat_platform.identity_memberships
        (tenant_id,issuer,subject,role,approved_by,expires_at)
        VALUES ('{TA}','{REVIEW_ISS}','{REVIEW_SUB}','security_admin',
           'synthetic-review-approval',statement_timestamp()+interval '1 day')""")
        assert stage()==ART
        cls.st=sql("SELECT (statement_timestamp()+interval '2 hours')::text").stdout.strip()
        cls.en=sql("SELECT (statement_timestamp()+interval '3 hours')::text").stdout.strip()

    def setUp(self):
        n=self._testMethodName.split('_')[1]
        self.dev=f'77000000-0000-4000-8000-0000000000{n}'
        self.dev_req=f'88000000-0000-4000-8000-0000000000{n}'
        self.change=f'99000000-0000-4000-8000-0000000000{n}'
        self.req=f'aa000000-0000-4000-8000-0000000000{n}'
        sql(f"UPDATE ipat_platform.tenants SET state='active' WHERE id IN ('{TA}','{TB}')")
        sql(f"""UPDATE ipat_platform.identity_memberships SET revoked_at=NULL,
          expires_at=statement_timestamp()+interval '1 day'
          WHERE (issuer='{ISSUER}' AND subject='{SUBJECT}') OR
            (issuer='{REVIEW_ISS}' AND subject='{REVIEW_SUB}')""")
        self.assertEqual(register(self.dev,self.dev_req),self.dev)
        self.window=(self.st,self.en)

    def test_00_role_and_direct_table_denial(self):
        for tab in ('firmware_artifacts','firmware_changes','firmware_evidence','firmware_change_events'):
            self.assertEqual(sql(f"""SELECT relrowsecurity::int,relforcerowsecurity::int
                FROM pg_class WHERE oid='ipat_ops.{tab}'::regclass""").stdout.strip(),'1|1')
            for who in ('ipat_fw_api_execute','ipat_fw_attestor_execute','ipat_app_runtime'):
                self.assertNotEqual(role(f'SELECT * FROM ipat_ops.{tab}',who,expect=False).returncode,0)
                self.assertNotEqual(role(f'DELETE FROM ipat_ops.{tab}',who,expect=False).returncode,0)
        self.assertEqual(sql("""SELECT
           has_function_privilege('ipat_fw_api_execute',
             'ipat_platform.attest_firmware_evidence(uuid,uuid,text,text,text,text,text)','EXECUTE')::int,
           has_function_privilege('ipat_fw_attestor_execute',
             'ipat_platform.request_firmware_execution(text,text,uuid,uuid)','EXECUTE')::int""").stdout.strip(),'0|0')

    def test_01_operator_stages_only_metadata_then_requests(self):
        self.assertEqual(stage(),ART)
        self.assertEqual(propose(self.dev,self.change,self.req,self.window),self.change)
        self.assertEqual(propose(self.dev,self.change,self.req,self.window),self.change)
        self.assertEqual(result(f"""SELECT state FROM ipat_platform.list_firmware_changes(
          '{ISSUER}','{SUBJECT}','{TA}') WHERE id='{self.change}'"""),'AWAITING_EVIDENCE')
        self.assertEqual(sql(f"SELECT count(*) FROM ipat_ops.firmware_change_events WHERE change_id='{self.change}'").stdout.strip(),'1')
        self.assertEqual(execution(self.change),'DENIED')
        self.assertEqual(review(self.change),'DENIED')
        self.assertEqual(stage(sha='F'*64),'DENIED')
        self.assertEqual(stage(model='unapproved-other-model'),'DENIED')

    def test_02_prevents_cross_tenant_model_and_double_job(self):
        self.assertEqual(propose(self.dev,self.change,self.req,self.window,tenant=TB),'DENIED')
        self.assertEqual(propose(self.dev,self.change,self.req,self.window,issuer='https://forged.invalid'),'DENIED')
        self.assertEqual(propose(self.dev,self.change,self.req,self.window),self.change)
        self.assertEqual(propose(self.dev,'99000000-0000-4000-8000-000000000099',
          'aa000000-0000-4000-8000-000000000099',self.window),'DENIED')
        # Same request ID reuses the first ID, not a caller-provided substitute.
        self.assertEqual(propose(self.dev,'99000000-0000-4000-8000-000000000098',
          self.req,self.window),self.change)
        self.assertEqual(review(self.change,issuer=ISSUER,subject=SUBJECT,approve=False),'DENIED')
        self.assertNotIn(str(self.change),result(f"""SELECT count(*)::text FROM ipat_platform.list_firmware_changes(
          '{ISSUER}','{SUBJECT}','{TB}')"""))

    def test_03_rejects_spoofed_attestation_and_unrelated_vendor(self):
        self.assertEqual(propose(self.dev,self.change,self.req,self.window),self.change)
        stmt=f"SELECT ipat_platform.attest_firmware_evidence('{TA}','{self.change}','VENDOR_RELEASE','{SHA}',NULL,NULL,NULL)"
        self.assertNotEqual(role(stmt,expect=False).returncode,0)
        self.assertEqual(attest(self.change,'VENDOR_RELEASE',sha='b'*64),'0')
        self.assertEqual(attest(self.change,'DEVICE_IDENTITY',observed_vendor="'ZTE'",
          observed_model="'wrong-model'",observed_version="'synthetic'"),'0')
        self.assertEqual(attest(self.change,'VENDOR_RELEASE'),'1')
        self.assertEqual(attest(self.change,'VENDOR_RELEASE'),'0')
        self.assertEqual(review(self.change),'DENIED')
        self.assertEqual(review(self.change,approve=False),'REJECTED')
        self.assertEqual(execution(self.change),'DENIED')
        self.assertEqual(attest(self.change,'BACKUP_RESTORE'),'0')

    def test_04_independent_checker_then_explicit_operator_click(self):
        self.assertEqual(propose(self.dev,self.change,self.req,self.window),self.change)
        all_attest(self.change)
        self.assertEqual(review(self.change,issuer=ISSUER,subject=SUBJECT),'DENIED')
        self.assertEqual(review(self.change),'APPROVED')
        self.assertEqual(review(self.change),'DENIED')
        self.assertEqual(execution(self.change,issuer=REVIEW_ISS,subject=REVIEW_SUB),'DENIED')
        self.assertEqual(execution(self.change),'EXECUTION_REQUESTED')
        self.assertEqual(execution(self.change),'DENIED')
        s=sql(f"SELECT state,execution_requested_at IS NOT NULL FROM ipat_ops.firmware_changes WHERE id='{self.change}'").stdout.strip()
        self.assertEqual(s,'EXECUTION_REQUESTED|t')
        events=sql(f"SELECT event FROM ipat_ops.firmware_change_events WHERE change_id='{self.change}' ORDER BY id").stdout.strip().splitlines()
        self.assertEqual(events,['REQUESTED']+['EVIDENCE_ATTESTED']*6+['APPROVED','EXECUTION_REQUESTED'])

    def test_05_revocation_after_review_prevents_execution(self):
        self.assertEqual(propose(self.dev,self.change,self.req,self.window),self.change)
        all_attest(self.change)
        self.assertEqual(review(self.change),'APPROVED')
        sql(f"""UPDATE ipat_platform.identity_memberships SET revoked_at=statement_timestamp()
          WHERE tenant_id='{TA}' AND issuer='{REVIEW_ISS}' AND subject='{REVIEW_SUB}'""")
        self.assertEqual(execution(self.change),'DENIED')
        sql(f"UPDATE ipat_platform.tenants SET state='suspended' WHERE id='{TA}'")
        self.assertEqual(result(f"""SELECT count(*)::text FROM ipat_platform.list_firmware_changes(
          '{ISSUER}','{SUBJECT}','{TA}')"""),'0')

if __name__=='__main__':unittest.main()
