"""R10.11: exact Host-bound platform-session SQL HTTP gateway acceptance (disposable only)."""
import hashlib
import os
import subprocess
import unittest

ISSUER = "https://identity.r1011-wrapper.invalid/realm/platform"
MAKER = "owner-r1011-wrapper-maker"
CHECKER = "owner-r1011-wrapper-checker"
HOST = "platform-r1011-wrapper.net"
TENANT = "b1111111-1111-4111-8111-111111111121"
RESERVATION = "b1111111-1111-4111-8111-111111111122"
REQUEST = "b1111111-1111-4111-8111-111111111123"
DOMAIN = "b1111111-1111-4111-8111-111111111124"
COOKIE = "r1011wrappercookie" * 4
CSRF = "r1011wrappercsrf" * 4

def sha(value):
    return hashlib.sha256(value.encode()).hexdigest()

def sql(statement, role=None):
    prefix = f"SET ROLE {role};" if role else ""
    p = subprocess.run(
        ["psql", "-X", "-A", "-q", "-t", "-v", "ON_ERROR_STOP=1", "-c", prefix + statement],
        capture_output=True, text=True,
    )
    if p.returncode:
        raise AssertionError(f"PostgreSQL failure for role {role or 'bootstrap'}: {p.stderr[:1200]}")
    return p.stdout.strip()

@unittest.skipUnless(os.environ.get("IPAT_PG_EPHEMERAL_TEST") == "1", "disposable only")
class PlatformSessionActivation(unittest.TestCase):
    def test_exact_host_session_can_reserve_and_request_without_direct_table_rights(self):
        self.assertEqual(os.environ.get("PGDATABASE"), "ipat_synthetic")
        sql(f"""INSERT INTO ipat_platform.platform_principals
              (issuer,subject,role,approved_by,expires_at) VALUES
              ('{ISSUER}','{MAKER}','platform_owner','independent',now()+interval '1 day'),
              ('{ISSUER}','{CHECKER}','platform_owner','independent',now()+interval '1 day');
              INSERT INTO ipat_platform.platform_console_hosts
              (hostname,verified_at,tls_ready_at)
              VALUES('{HOST}',now()-interval '1 day',now()-interval '1 hour');""")
        issued = sql(f"""SELECT ipat_platform.issue_platform_browser_session(
                '{ISSUER}','{MAKER}','{HOST}',
                'b1111111-1111-4111-8111-111111111111',
                '{sha(COOKIE)}','{sha(CSRF)}',clock_timestamp()+interval '8 minutes')""",
                "ipat_platform_session_issuer_login")
        self.assertEqual(issued, "b1111111-1111-4111-8111-111111111111")
        reserved = sql(f"""SELECT ipat_platform.reserve_tenant_from_platform_session(
                '{sha(COOKIE)}','{HOST}','{sha(CSRF)}','{RESERVATION}',
                '{TENANT}','r1011-wrapper-company')""",
                "ipat_platform_session_api_login")
        self.assertEqual(reserved, TENANT)
        expiry = sql("SELECT to_char((clock_timestamp()+interval '30 days') AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')")
        requested = sql(f"""SELECT ipat_platform.request_company_activation_from_platform_session(
                '{sha(COOKIE)}','{HOST}','{sha(CSRF)}','{REQUEST}','{TENANT}','{DOMAIN}',
                'portal.r1011-wrapper.net','a_record',
                'https://identity.r1011-wrapper.net/realm/customer','initial-admin-r1011-wrapper',
                '{expiry}'::timestamptz,'{'e'*64}')""",
                "ipat_platform_session_api_login")
        self.assertEqual(requested, REQUEST, "Host-bound API wrapper must not silently reject valid reserved tenant activation")
        denied = sql(f"""SELECT ipat_platform.request_company_activation_from_platform_session(
                '{sha(COOKIE)}','wrong-r1011-wrapper.net','{sha(CSRF)}',
                'b1111111-1111-4111-8111-111111111125','{TENANT}',
                'b1111111-1111-4111-8111-111111111126','wrong.r1011-wrapper.net','a_record',
                'https://identity.r1011-wrapper.net/realm/customer','initial-admin-r1011-wrapper',
                '{expiry}'::timestamptz,'{'e'*64}')""",
                "ipat_platform_session_api_login")
        self.assertEqual(denied, "")

if __name__ == "__main__":
    unittest.main()
