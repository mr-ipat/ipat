"""R9.80: synthetic-only platform-only Host-bound durable session tests."""
import os
import subprocess
import unittest

ISS = "https://r980.synthetic.invalid/realm/ipat"
HOST = "admin.r980.synthetic.invalid"
OTHER = "admin.elsewhere.synthetic.invalid"
SID = "80808080-8080-4080-8080-808080808081"
REQ = "80808080-8080-4080-8080-808080808082"
TID = "80808080-8080-4080-8080-808080808083"
COOKIE = 'a' * 64
CSRF = 'b' * 64


def sql(s, role=None, check=True):
    return subprocess.run(
        ["psql", "-X", "-A", "-q", "-t", "-v", "ON_ERROR_STOP=1",
         "-c", ("SET ROLE " + role + ";" if role else "") + s],
        text=True, capture_output=True, check=check)


def issue(subject='owner', host=HOST, role='ipat_platform_session_issuer_login',
          session=SID, cookie=COOKIE, csrf=CSRF):
    return sql(
        "SELECT ipat_platform.issue_platform_browser_session("
        f"'{ISS}','{subject}','{host}','{session}','{cookie}','{csrf}',"
        "now()+interval '8 minutes')", role).stdout.strip()


def auth(cookie=COOKIE, host=HOST, csrf=None, mutate=False,
         role='ipat_platform_session_api_login'):
    digest = ("NULL" if csrf is None else f"'{csrf}'")
    return sql(
        "SELECT issuer||'#'||subject FROM "
        "ipat_platform.authenticate_platform_browser_session("
        f"'{cookie}','{host}',{digest},{str(mutate).lower()})",
        role).stdout.strip()


def reserve(cookie=COOKIE, host=HOST, csrf=CSRF, req=REQ, tenant=TID,
            slug='r980-synthetic'):
    return sql(
        "SELECT ipat_platform.reserve_tenant_from_platform_session("
        f"'{cookie}','{host}','{csrf}','{req}','{tenant}','{slug}')",
        "ipat_platform_session_api_login").stdout.strip()


@unittest.skipUnless(os.getenv("IPAT_PG_EPHEMERAL_TEST") == '1',
                     "disposable DB only")
class PlatformSessions(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert os.getenv("PGDATABASE") == "ipat_synthetic"
        assert os.getenv("PGHOST") == "127.0.0.1"
        assert os.getenv("IPAT_PG_SYNTHETIC_PASSWORD") == "local_ci_synthetic_only"
        sql(f"""INSERT INTO ipat_platform.platform_principals(
            issuer,subject,role,approved_by,created_at,expires_at)
            VALUES
            ('{ISS}','owner','platform_owner','separate-reviewer',now()-interval '1 minute',now()+interval '1 day'),
            ('{ISS}','revoked','platform_owner','separate-reviewer',now()-interval '1 minute',now()+interval '1 day'),
            ('{ISS}','expired','platform_owner','separate-reviewer',now()-interval '1 day',now()-interval '1 minute')""")
        sql(f"UPDATE ipat_platform.platform_principals SET revoked_at=now() WHERE issuer='{ISS}' AND subject='revoked'")
        sql(f"""INSERT INTO ipat_platform.platform_console_hosts(
            hostname,verified_at,tls_ready_at) VALUES
            ('{HOST}',now()-interval '1 day',now()-interval '1 hour'),
            ('{OTHER}',now()-interval '1 day',now()-interval '1 hour')""")

    def test_01_issuer_only_and_host_exact(self):
        self.assertEqual(issue(subject='revoked'), "")
        self.assertEqual(issue(subject='expired'), "")
        self.assertEqual(issue(subject='missing'), "")
        self.assertEqual(issue(host="unverified.synthetic.invalid"), "")
        for forbidden in ('ipat_tenant_api_login', 'ipat_oidc_session_issuer_login'):
            with self.assertRaises(subprocess.CalledProcessError):
                issue(role=forbidden)
        self.assertEqual(issue(), SID)
        self.assertEqual(auth(), ISS+"#owner")
        self.assertEqual(auth(host=OTHER), "")
        self.assertEqual(auth(csrf='c'*64, mutate=True), "")
        self.assertEqual(auth(mutate=True), "")
        self.assertEqual(auth(csrf=CSRF, mutate=True), ISS+"#owner")

    def test_02_guarded_suspended_reservation_exact_retry(self):
        self.assertEqual(reserve(csrf='c'*64), "")
        self.assertEqual(reserve(host=OTHER), "")
        self.assertEqual(reserve(), TID)
        self.assertEqual(reserve(), TID)
        self.assertEqual(reserve(slug='replay-forged'), "")
        self.assertEqual(sql(
            f"SELECT state FROM ipat_platform.tenants WHERE id='{TID}'"
        ).stdout.strip(), "suspended")
        self.assertEqual(sql(
            f"SELECT count(*) FROM ipat_platform.identity_memberships WHERE tenant_id='{TID}'"
        ).stdout.strip(), "0")
        self.assertEqual(sql(
            f"SELECT count(*) FROM ipat_platform.tenant_domains WHERE tenant_id='{TID}'"
        ).stdout.strip(), "0")

    def test_03_current_host_and_owner_gate_every_request(self):
        listing = sql(
            "SELECT tenant_slug FROM ipat_platform.list_reservations_from_platform_session("
            f"'{COOKIE}','{HOST}')", "ipat_platform_session_api_login"
        ).stdout.strip()
        self.assertIn("r980-synthetic", listing.splitlines())
        sql(f"UPDATE ipat_platform.platform_console_hosts SET disabled_at=now() WHERE hostname='{HOST}'")
        self.assertEqual(auth(), "")
        self.assertEqual(reserve(req='80808080-8080-4080-8080-808080808084',
                                 tenant='80808080-8080-4080-8080-808080808085',
                                 slug='blocked-host'), "")
        sql(f"UPDATE ipat_platform.platform_console_hosts SET disabled_at=NULL WHERE hostname='{HOST}'")
        sql(f"UPDATE ipat_platform.platform_principals SET revoked_at=now() WHERE issuer='{ISS}' AND subject='owner'")
        self.assertEqual(auth(), "")
        self.assertEqual(issue(session='80808080-8080-4080-8080-808080808086',
                               cookie='d'*64, csrf='e'*64), "")
        sql(f"UPDATE ipat_platform.platform_principals SET revoked_at=NULL WHERE issuer='{ISS}' AND subject='owner'")

    def test_04_function_grant_and_raw_table_boundary(self):
        for role in ('ipat_tenant_api_login', 'ipat_oidc_session_issuer_login',
                     'ipat_app_runtime'):
            for function in (
                'issue_platform_browser_session(text,text,text,uuid,text,text,timestamptz)',
                'authenticate_platform_browser_session(text,text,text,boolean)',
                'reserve_tenant_from_platform_session(text,text,text,uuid,uuid,text)',
            ):
                value = sql(
                    f"SELECT has_function_privilege('{role}',"
                    f"'ipat_platform.{function}','EXECUTE')::int").stdout.strip()
                self.assertEqual(value, "0")
            denied = sql(
                "SELECT count(*) FROM ipat_platform.platform_browser_sessions",
                role, check=False)
            self.assertNotEqual(denied.returncode, 0)
        self.assertEqual(sql("SELECT has_function_privilege("
            "'ipat_platform_session_api_login',"
            "'ipat_platform.issue_platform_browser_session(text,text,text,uuid,text,text,timestamptz)',"
            "'EXECUTE')::int").stdout.strip(), "0")
        self.assertEqual(sql("SELECT has_function_privilege("
            "'ipat_platform_session_issuer_login',"
            "'ipat_platform.reserve_tenant_from_platform_session(text,text,text,uuid,uuid,text)',"
            "'EXECUTE')::int").stdout.strip(), "0")

    def test_05_csrf_required_for_logout_and_revocation_immediate(self):
        self.assertEqual(sql(
            "SELECT ipat_platform.revoke_platform_browser_session("
            f"'{COOKIE}','{HOST}','{'c'*64}')",
            'ipat_platform_session_api_login').stdout.strip(), 'f')
        self.assertEqual(auth(), ISS+'#owner')
        self.assertEqual(sql(
            "SELECT ipat_platform.revoke_platform_browser_session("
            f"'{COOKIE}','{HOST}','{CSRF}')",
            'ipat_platform_session_api_login').stdout.strip(), 't')
        self.assertEqual(auth(), "")
        self.assertEqual(reserve(req='80808080-8080-4080-8080-808080808084',
                                 tenant='80808080-8080-4080-8080-808080808085',
                                 slug='blocked-after-logout'), "")
        self.assertEqual(sql(
            "SELECT count(*) FROM ipat_platform.platform_browser_sessions WHERE "
            f"cookie_sha256='{COOKIE}'").stdout.strip(), "1")


if __name__ == '__main__':
    unittest.main()
