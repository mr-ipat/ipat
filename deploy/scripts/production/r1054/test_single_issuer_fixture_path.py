"""One writer of the synthetic two-company Axum fixture per disposable DB."""
from pathlib import Path
import unittest
SOURCE=Path(__file__).resolve().parents[4]/"apps/control-api/src/commercial_tenant_api.rs"

class TestSingleFixturePath(unittest.TestCase):
    def test_two_company_matrix_runs_only_through_mandatory_r970_ci_test(self):
        s=SOURCE.read_text()
        self.assertEqual(s.count("r1053_two_company_host_cookie_matrix().await;"),1)
        self.assertIn("async fn r1053_two_company_host_cookie_matrix()",s)
        self.assertNotIn("async fn r1053_real_pg_two_company_host_cookie_isolation_and_revocation()",s)
        r970=s.index("async fn r970_real_pg_tenant_dashboard_current_role_catalog_and_logout()")
        invocation=s.index("r1053_two_company_host_cookie_matrix().await;")
        r968=s.index("async fn r968_real_pg_durable_host_session_to_site_device_domain_http()")
        self.assertLess(r970,invocation)
        self.assertLess(invocation,r968)

if __name__=="__main__":
    unittest.main()
