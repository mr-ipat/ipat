"""Non-mutating source contracts for commercial operator flow and high-risk roles."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
RUST = ROOT / 'apps/control-api/src/commercial_tenant_api.rs'


class CommercialAcceptance(unittest.TestCase):
    def test_legacy_pppoe_tenant_admin_is_denied_without_unreachable_success(self):
        source = RUST.read_text()
        start = source.index('async fn r1007_real_pg_axum_pppoe_batch_dry_run_never_executes_routeros()')
        end = source.index('async fn r1009_real_pg_axum_pppoe_role_separation_hides_and_denies_wrong_actions()', start)
        block = source[start:end]
        self.assertIn('assert_eq!(created.status(), StatusCode::FORBIDDEN)', block)
        self.assertIn('assert!(no_execute)', block)
        self.assertNotIn('let created_json', block)
        self.assertNotIn('role_split_ready', block)
        # Allowed return statements are solely the early disposable-fixture gates.
        self.assertEqual(block.count('return;'), 2)
        self.assertIn('Provisioning Officer', source)

    def test_device_form_reuses_site_master_and_does_not_offer_pop_as_site(self):
        page = (ROOT / 'web/console/tenant/dashboard.html').read_text()
        form = page.split('<section data-panel="devices" hidden>')[1].split('<section data-panel="branding" hidden>')[0]
        self.assertIn('<label>Site <select name="pop_id" id="device-site" required>', form)
        self.assertIn('POPs are independent and cannot be entered as Sites.', form)
        self.assertNotIn('<label>Site / POP', form)
        self.assertIn('id="create-site-link"', form)


if __name__ == '__main__':
    unittest.main()
