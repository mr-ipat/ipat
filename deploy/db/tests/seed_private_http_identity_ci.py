"""Seed *only* disposable CI PostgreSQL 16 for signed Rust HTTP integration.
No real credentials, tenant, DNS claims, device data or live VPS modifications.
"""
import os
from test_postgres_rls_integration import TA, TB, sql

ISSUER="https://id.example.invalid/realms/lab"
SUB="synthetic-operator"
def main():
    assert os.getenv("IPAT_PG_EPHEMERAL_TEST")=="1"
    assert os.getenv("PGHOST")=="127.0.0.1"
    assert os.getenv("PGDATABASE")=="ipat_synthetic"
    assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD")=="local_ci_synthetic_only"
    assert sql("SELECT to_regprocedure('ipat_platform.lookup_active_membership(text,text,uuid,text,text)') IS NOT NULL").stdout.strip()=="t"
    assert sql("SELECT to_regrole('ipat_lab_identity_reader') IS NULL").stdout.strip()=="t"
    sql("""CREATE ROLE ipat_lab_identity_reader LOGIN INHERIT
      NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
      PASSWORD 'local_ci_synthetic_only'""")
    sql("GRANT ipat_identity_query TO ipat_lab_identity_reader")
    assert sql("SELECT to_regrole('ipat_lab_device_registrar') IS NULL").stdout.strip()=="t"
    sql("""CREATE ROLE ipat_lab_device_registrar LOGIN INHERIT
      NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
      PASSWORD 'local_ci_synthetic_only'""")
    sql("GRANT ipat_device_registry_execute TO ipat_lab_device_registrar")
    sql(f"""INSERT INTO ipat_platform.identity_memberships
       (tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
       ('{TA}','{ISSUER}','{SUB}','noc_engineer','CI-APPROVED-A',
          statement_timestamp() + interval '1 day'),
       ('{TB}','{ISSUER}','{SUB}','helpdesk','CI-APPROVED-B',
          statement_timestamp() + interval '1 day'),
       ('{TB}','{ISSUER}','{SUB}','noc_engineer','CI-APPROVED-B-NOC',
          statement_timestamp() + interval '1 day'),
       ('{TA}','{ISSUER}','{SUB}','tenant_admin','CI-ADMIN-A',
          statement_timestamp() + interval '1 day'),
       ('{TB}','{ISSUER}','{SUB}','tenant_admin','CI-ADMIN-B',
          statement_timestamp() + interval '1 day')""")
    sql(f"""INSERT INTO ipat_platform.identity_pop_grants
       (tenant_id,issuer,subject,role,pop_id) VALUES
       ('{TA}','{ISSUER}','{SUB}','noc_engineer','pop-a'),
       ('{TB}','{ISSUER}','{SUB}','helpdesk','pop-b'),
       ('{TB}','{ISSUER}','{SUB}','noc_engineer','pop-b')""")
    for table in ("identity_memberships","identity_pop_grants","platform_principals"):
        for privilege in ("SELECT","INSERT","UPDATE","DELETE","TRUNCATE"):
            assert sql(f"SELECT has_table_privilege('ipat_lab_identity_reader','ipat_platform.{table}','{privilege}')::int").stdout.strip()=="0"
    assert sql("SELECT has_function_privilege('ipat_lab_identity_reader','ipat_platform.lookup_active_membership(text,text,uuid,text,text)','EXECUTE')::int").stdout.strip()=="1"
    for role in ("ipat_lab_identity_reader","ipat_lab_device_registrar"):
        for privilege in ("SELECT","INSERT","UPDATE","DELETE","TRUNCATE"):
            assert sql(f"""SELECT has_table_privilege(
              '{role}','ipat_ops.device_candidates','{privilege}')::int"""
              ).stdout.strip()=="0"
    write="ipat_platform.propose_lab_device_candidate(text,text,uuid,uuid,text,text,text,text,text,text)"
    read="ipat_platform.list_lab_device_candidates(text,text,uuid,text,text)"
    assert sql(f"""SELECT
      has_function_privilege('ipat_lab_device_registrar','{write}','EXECUTE')::int,
      has_function_privilege('ipat_lab_device_registrar','{read}','EXECUTE')::int,
      has_function_privilege('ipat_lab_identity_reader','{write}','EXECUTE')::int,
      has_function_privilege('ipat_lab_identity_reader','{read}','EXECUTE')::int"""
       ).stdout.strip()=="1|0|0|1"
    print("R83_DISPOSABLE_SEPARATE_READER_AND_REGISTER_WRITER_NO_DIRECT_TABLE_ACCESS")
if __name__=="__main__":
    main()
