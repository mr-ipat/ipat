"""R10.05 disposable PostgreSQL Subscriber360/topology/diagnostic authorization."""
import os
import subprocess
import unittest

T1="a3a3a3a3-a3a3-43a3-83a3-a3a3a3a3a3a1"
T2="b3b3b3b3-b3b3-43b3-83b3-b3b3b3b3b3b2"
ISS="https://r1005.synthetic.invalid/realms/tenant"
ADMIN1="r1005-admin-one"
ADMIN2="r1005-admin-two"
HELP1="r1005-helpdesk-one"
HELP_NO_SCOPE="r1005-helpdesk-none"
ROUTER1="c3c3c3c3-c3c3-43c3-83c3-c3c3c3c3c3c1"
OLT1="d3d3d3d3-d3d3-43d3-83d3-d3d3d3d3d3d1"
ROUTER1B="c3c3c3c3-c3c3-43c3-83c3-c3c3c3c3c3c2"
OLT1B="d3d3d3d3-d3d3-43d3-83d3-d3d3d3d3d3d2"
ROUTER2="e3e3e3e3-e3e3-43e3-83e3-e3e3e3e3e3e2"
SHA="1"*64

def sql(stmt, role=None, check=True):
    prefix=f"SET ROLE {role};" if role else ""
    return subprocess.run(
        ["psql","-X","-A","-q","-t","-v","ON_ERROR_STOP=1","-c",prefix+stmt],
        text=True,capture_output=True,check=check)

@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1","disposable DB only")
class Subscriber360(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE")=="ipat_synthetic"
        sql(f"""
        INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES
          ('{T1}','r1005-one','active'),('{T2}','r1005-two','active');
        INSERT INTO ipat_platform.identity_memberships(
          tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
          ('{T1}','{ISS}','{ADMIN1}','tenant_admin','independent',now()+interval '1 day'),
          ('{T2}','{ISS}','{ADMIN2}','tenant_admin','independent',now()+interval '1 day'),
          ('{T1}','{ISS}','{HELP1}','helpdesk','independent',now()+interval '1 day'),
          ('{T1}','{ISS}','{HELP_NO_SCOPE}','helpdesk','independent',now()+interval '1 day');
        INSERT INTO ipat_platform.identity_pop_grants(
          tenant_id,issuer,subject,role,pop_id)
        VALUES('{T1}','{ISS}','{HELP1}','helpdesk','SITE-A');
        """)
        for tenant,subject,pop,site in [
            (T1,ADMIN1,"POP-A","SITE-A"),
            (T1,ADMIN1,"POP-A2","SITE-A2"),
            (T2,ADMIN2,"POP-B","SITE-B"),
        ]:
            assert sql(
                f"SELECT ipat_platform.create_tenant_pop('{ISS}','{subject}','{tenant}','{pop}','{pop}');"
            ).stdout.strip()==pop
            assert sql(
                f"SELECT ipat_platform.create_tenant_site('{ISS}','{subject}','{tenant}','{site}','{site}');"
            ).stdout.strip()==site
            assert sql(
                f"SELECT ipat_platform.assign_tenant_site_to_pop('{ISS}','{subject}','{tenant}','{site}','{pop}',1);"
            ).stdout.strip()=="2"

        def dev(tenant,subject,did,site,kind,vendor,transport,port):
            return sql(
             "SELECT ipat_platform.register_managed_device("
             f"'{ISS}','{subject}','{tenant}','{did}',gen_random_uuid(),"
             f"'{site}','{kind}-{did[-4:]}','{kind}','{vendor}',NULL,"
             f"'{transport}','device.r1005.invalid',{port},NULL);").stdout.strip()
        assert dev(T1,ADMIN1,ROUTER1,"SITE-A","router","MikroTik","routeros_api_ssl",8729)==ROUTER1
        assert dev(T1,ADMIN1,OLT1,"SITE-A","olt","ZTE","ssh",22)==OLT1
        assert dev(T1,ADMIN1,ROUTER1B,"SITE-A2","router","MikroTik","routeros_api_ssl",8729)==ROUTER1B
        assert dev(T1,ADMIN1,OLT1B,"SITE-A2","olt","ZTE","ssh",22)==OLT1B
        assert dev(T2,ADMIN2,ROUTER2,"SITE-B","router","MikroTik","routeros_api_ssl",8729)==ROUTER2

    def upsert(self, subscriber, name, pop, site, router, access, ont, revision, actor=ADMIN1):
        access_sql="NULL" if access is None else f"'{access}'"
        ont_sql="NULL" if ont is None else f"'{ont}'"
        return sql(
            "SELECT ipat_platform.upsert_subscriber360("
            f"'{ISS}','{actor}','{T1}','{subscriber}','{name}',NULL,"
            f"'{pop}','{site}','{router}',{access_sql},{ont_sql},{revision})",
            "ipat_tenant_api_login").stdout.strip()

    def test_01_exact_topology_and_cas_cross_site_denied(self):
        self.assertEqual(self.upsert(
            "SUB-001","Customer One","POP-A","SITE-A",ROUTER1,OLT1,"1/1/1:1",0),"1")
        self.assertEqual(self.upsert(
            "SUB-002","Customer Two","POP-A2","SITE-A2",ROUTER1B,OLT1B,"1/1/2:1",0),"1")
        self.assertEqual(self.upsert(
            "SUB-X","Wrong Access Site","POP-A","SITE-A",ROUTER1,OLT1B,None,0),"")
        self.assertEqual(self.upsert(
            "SUB-X","Wrong Router Site","POP-A","SITE-A",ROUTER1B,None,None,0),"")
        self.assertEqual(self.upsert(
            "SUB-001","Customer One","POP-A","SITE-A",ROUTER1,OLT1,"1/1/1:1",0),"")
        self.assertEqual(self.upsert(
            "SUB-001","Customer One Renamed","POP-A","SITE-A",ROUTER1,OLT1,"1/1/1:1",1),"2")
        row=sql(
          f"SELECT subscriber_id||'|'||pop_code||'|'||site_code||'|'||revision||'|'||topology_state "
          f"FROM ipat_platform.list_subscriber360('{ISS}','{ADMIN1}','{T1}') "
          "WHERE subscriber_id='SUB-001'",
          "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(row,"SUB-001|POP-A|SITE-A|2|declared")

    def test_02_scoped_read_capability_and_raw_table_denial(self):
        admin=sql(
            f"SELECT can_read::int||'|'||can_manage::int FROM "
            f"ipat_platform.subscriber_diagnostic_ui_capabilities('{ISS}','{ADMIN1}','{T1}')",
            "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(admin,"1|1")
        helpdesk=sql(
            f"SELECT can_read::int||'|'||can_manage::int FROM "
            f"ipat_platform.subscriber_diagnostic_ui_capabilities('{ISS}','{HELP1}','{T1}')",
            "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(helpdesk,"1|0")
        none=sql(
            f"SELECT can_read::int||'|'||can_manage::int FROM "
            f"ipat_platform.subscriber_diagnostic_ui_capabilities('{ISS}','{HELP_NO_SCOPE}','{T1}')",
            "ipat_tenant_api_login").stdout.strip()
        self.assertEqual(none,"0|0")
        scoped=sql(
            f"SELECT subscriber_id FROM ipat_platform.list_subscriber360("
            f"'{ISS}','{HELP1}','{T1}') ORDER BY 1",
            "ipat_tenant_api_login").stdout.strip().splitlines()
        self.assertEqual(scoped,["SUB-001"])
        self.assertEqual(sql(
            f"SELECT subscriber_id FROM ipat_platform.list_subscriber360("
            f"'{ISS}','{ADMIN2}','{T1}')",
            "ipat_tenant_api_login").stdout.strip(),"")
        for role in ("ipat_tenant_api_login","ipat_app_runtime"):
            denied=sql("SELECT count(*) FROM ipat_ops.subscribers",role,check=False)
            self.assertNotEqual(denied.returncode,0)

    def test_03_topology_verification_is_separate_and_invalidated_on_change(self):
        signature=(
          "ipat_platform.record_subscriber_topology_verification("
          "uuid,text,text,text,uuid,uuid,text,text,timestamptz,text)")
        for role in ("ipat_tenant_api_login","ipat_diag_ingest_exec"):
            self.assertEqual(sql(
                f"SELECT has_function_privilege('{role}','{signature}','EXECUTE')::int"
            ).stdout.strip(),"0")
        exact=sql(
          "SELECT ipat_platform.record_subscriber_topology_verification("
          f"'{T1}','SUB-001','POP-A','SITE-A','{ROUTER1}','{OLT1}',"
          f"'1/1/1:1','topology.worker:01',now(),'{SHA}')",
          "ipat_topology_verify_exec").stdout.strip()
        self.assertEqual(exact,"t")
        self.assertEqual(sql(
          f"SELECT topology_state||'|'||topology_source_id FROM "
          f"ipat_platform.list_subscriber360('{ISS}','{ADMIN1}','{T1}') "
          "WHERE subscriber_id='SUB-001'",
          "ipat_tenant_api_login").stdout.strip(),"verified|topology.worker:01")
        # Non-topology edit preserves exact evidence.
        self.assertEqual(self.upsert(
            "SUB-001","Name Only","POP-A","SITE-A",ROUTER1,OLT1,"1/1/1:1",2),"3")
        self.assertEqual(sql(
          f"SELECT topology_state FROM ipat_platform.list_subscriber360("
          f"'{ISS}','{ADMIN1}','{T1}') WHERE subscriber_id='SUB-001'",
          "ipat_tenant_api_login").stdout.strip(),"verified")
        # Any topology tuple edit invalidates prior verification.
        self.assertEqual(self.upsert(
            "SUB-001","Name Only","POP-A","SITE-A",ROUTER1,OLT1,"1/1/1:2",3),"4")
        self.assertEqual(sql(
          f"SELECT topology_state||'|'||coalesce(topology_source_id,'-') FROM "
          f"ipat_platform.list_subscriber360('{ISS}','{ADMIN1}','{T1}') "
          "WHERE subscriber_id='SUB-001'",
          "ipat_tenant_api_login").stdout.strip(),"declared|-")
        stale=sql(
          "SELECT ipat_platform.record_subscriber_topology_verification("
          f"'{T1}','SUB-001','POP-A','SITE-A','{ROUTER1}','{OLT1}',"
          f"'1/1/1:2','topology.worker:old',now()-interval '1 hour','{SHA}')",
          "ipat_topology_verify_exec").stdout.strip()
        self.assertEqual(stale,"f")
        wrong=sql(
          "SELECT ipat_platform.record_subscriber_topology_verification("
          f"'{T1}','SUB-001','POP-A','SITE-A','{ROUTER1}','{OLT1B}',"
          f"'1/1/1:2','topology.worker:01',now(),'{SHA}')",
          "ipat_topology_verify_exec").stdout.strip()
        self.assertEqual(wrong,"f")

    def test_04_ingest_is_separate_bounded_and_signal_shape_strict(self):
        # Re-verify SUB-001's current tuple for later diagnostics.
        self.assertEqual(sql(
          "SELECT ipat_platform.record_subscriber_topology_verification("
          f"'{T1}','SUB-001','POP-A','SITE-A','{ROUTER1}','{OLT1}',"
          f"'1/1/1:2','topology.worker:02',now(),'{SHA}')",
          "ipat_topology_verify_exec").stdout.strip(),"t")
        base=("SELECT ipat_platform.record_diagnostic_observation("
              f"'{T1}','{ROUTER1}',")
        accepted=[
          ("NULL","distribution_uplink_healthy"),
          ("'SUB-001'","ont_optical_normal"),
          ("'SUB-001'","pppoe_authentication_rejected"),
          ("'SUB-001'","cwmp_inform_missing"),
        ]
        for sub,signal in accepted:
            q=base+f"{sub},'{signal}','telemetry.worker:01',now(),'{SHA}')"
            self.assertRegex(sql(q,"ipat_diag_ingest_exec").stdout.strip(),r"^\d+$")
        # A path signal must not masquerade as subscriber evidence.
        self.assertEqual(sql(
            base+f"'SUB-001','distribution_uplink_down','telemetry.worker:01',now(),'{SHA}')",
            "ipat_diag_ingest_exec").stdout.strip(),"")
        # Missing-CWMP without an exact subscriber remains non-actionable.
        self.assertEqual(sql(
            base+f"NULL,'cwmp_inform_missing','telemetry.worker:01',now(),'{SHA}')",
            "ipat_diag_ingest_exec").stdout.strip(),"")
        # Neighbor-health evidence is also subscriber-bound, never anonymous.
        self.assertEqual(sql(
            base+f"NULL,'neighbor_ont_healthy','telemetry.worker:01',now(),'{SHA}')",
            "ipat_diag_ingest_exec").stdout.strip(),"")
        forbidden=sql(
            base+f"'SUB-001','ont_optical_los','evil',now(),'{SHA}')",
            "ipat_tenant_api_login",check=False)
        self.assertNotEqual(forbidden.returncode,0)
        stale=base+f"'SUB-001','ont_optical_los','telemetry.worker:01',now()-interval '25 hours','{SHA}')"
        self.assertEqual(sql(stale,"ipat_diag_ingest_exec").stdout.strip(),"")
        cross=("SELECT ipat_platform.record_diagnostic_observation("
               f"'{T1}','{ROUTER1}','SUB-002','ont_optical_los','telemetry.worker:01',now(),'{SHA}')")
        self.assertEqual(sql(cross,"ipat_diag_ingest_exec").stdout.strip(),"")

    def test_05_observation_read_is_scope_filtered_hash_only(self):
        admin_rows=sql(
          "SELECT coalesce(subscriber_id,'-')||'|'||signal||'|'||source_id||'|'||evidence_sha256 "
          "FROM ipat_platform.list_diagnostic_observations("
          f"'{ISS}','{ADMIN1}','{T1}','{ROUTER1}') ORDER BY signal",
          "ipat_tenant_api_login").stdout.strip().splitlines()
        self.assertEqual(len(admin_rows),4)
        self.assertTrue(all(x.endswith("|"+SHA) for x in admin_rows))
        help_rows=sql(
          "SELECT coalesce(subscriber_id,'-')||'|'||signal FROM "
          "ipat_platform.list_diagnostic_observations("
          f"'{ISS}','{HELP1}','{T1}','{ROUTER1}') ORDER BY signal",
          "ipat_tenant_api_login").stdout.strip().splitlines()
        self.assertEqual(len(help_rows),4)
        self.assertEqual(sql(
          "SELECT signal FROM ipat_platform.list_diagnostic_observations("
          f"'{ISS}','{HELP_NO_SCOPE}','{T1}','{ROUTER1}')",
          "ipat_tenant_api_login").stdout.strip(),"")
        self.assertEqual(sql("SELECT has_function_privilege("
          "'ipat_tenant_api_login',"
          "'ipat_platform.record_diagnostic_observation(uuid,uuid,text,text,text,timestamptz,text)',"
          "'EXECUTE')::int").stdout.strip(),"0")

if __name__=="__main__":
    unittest.main()
