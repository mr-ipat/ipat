import pathlib
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[4]


class ThemeContract(unittest.TestCase):
    def test_theme_controls_and_safe_dom(self):
        for section in ("tenant", "platform"):
            base = ROOT / "web/console" / section
            html = (base / "dashboard.html").read_text()
            js = (base / "app.js").read_text()
            css = (base / "style.css").read_text()
            with self.subTest(section=section):
                self.assertEqual(html.count('id="appearance-theme"'), 1)
                for mode in ("system", "light", "dark"):
                    self.assertIn(f'<option value="{mode}">', html)
                self.assertIn("allowedThemes.has", js)
                self.assertIn("localStorage.setItem(themeKey", js)
                self.assertIn("document.documentElement.dataset.theme", js)
                self.assertNotIn("innerHTML", js[:js.index(" const themeKey=")])
                self.assertIn(':root[data-theme="dark"]', css)
                self.assertIn("prefers-color-scheme:dark", css)
                self.assertIn("focus-visible", css)
                self.assertIn("[hidden]", css)
                self.assertIn("var(--ui-bg)", css)
                self.assertEqual(subprocess.run(["node", "--check", str(base / "app.js")],
                                                capture_output=True).returncode, 0)

    def test_tenant_entitlements_still_hidden_by_default(self):
        text = (ROOT / "web/console/tenant/dashboard.html").read_text()
        import re
        menus = re.findall(r'<button[^>]*class="nav"[^>]*>', text)
        self.assertGreaterEqual(len(menus), 8)
        self.assertTrue(all(" hidden" in item for item in menus))


if __name__ == "__main__":
    unittest.main()
