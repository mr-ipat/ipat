"""R10.51: source contract for one-to-one module ownership and deny-by-default menus."""
from collections import Counter
from html.parser import HTMLParser
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[4]
MODULE_CAPS = {
    "pops": "can_manage_pops",
    "sites": "can_manage_sites",
    "devices": "can_manage_devices",
    "domains": "can_manage_domains",
    "branding": "can_manage_branding",
    "subscribers": "can_read_subscribers",
    "diagnostics": "can_read_diagnostics",
    "pppoe": "can_manage_pppoe_plans",
    "noc-access": "can_manage_noc_access",
    "noc": "can_read_noc",
}

class Controls(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids=[]
        self.menus=[]
        self.panels=[]
        self.forms=[]
    def handle_starttag(self, tag, attrs):
        a=dict(attrs)
        if "id" in a:self.ids.append(a["id"])
        if tag=="button" and "nav" in a.get("class","").split():
            self.menus.append(a)
        if tag=="section" and "data-panel" in a:
            self.panels.append(a)
        if tag=="form":self.forms.append(a)

class ModuleBoundary(unittest.TestCase):
    def test_one_navigation_one_panel_and_unique_controls(self):
        html=(ROOT/"web/console/tenant/dashboard.html").read_text()
        p=Controls();p.feed(html)
        for label,values in (("id",p.ids),("nav", [x["data-page"] for x in p.menus]),
                             ("panel",[x["data-panel"] for x in p.panels]),
                             ("form",[x.get("id") for x in p.forms])):
            duplicates=[k for k,v in Counter(values).items() if v>1]
            self.assertEqual(duplicates, [], f"duplicate {label}: {duplicates}")
        self.assertEqual(set(x["data-page"] for x in p.menus),set(MODULE_CAPS))
        self.assertEqual(set(x["data-panel"] for x in p.panels),set(MODULE_CAPS))
        self.assertTrue(all("hidden" in x for x in p.menus))
        self.assertTrue(all("hidden" in x for x in p.panels))

    def test_each_module_uses_own_server_issued_capability(self):
        js=(ROOT/"web/console/tenant/app.js").read_text()
        api=(ROOT/"apps/control-api/src/commercial_tenant_api.rs").read_text()
        for menu,cap in MODULE_CAPS.items():
            self.assertIn(f'"{cap}"',api,cap)
            if menu in {"pppoe","noc","branding","subscribers"}:
                # Different scopes / current role authority, checked separately.
                continue
            needle=f"if(r.{cap}===true)"
            self.assertIn(needle,js,f"{menu} must not inherit an unrelated grant")
        self.assertIn("if(r.can_manage_noc_access===true)",js)
        self.assertIn("if(r.can_read_diagnostics===true)",js)
        self.assertIn("'create-site-link').hidden=r.can_manage_sites!==true",js)
        self.assertIn("'manage-pops-link').hidden=r.can_manage_pops!==true",js)
        self.assertNotIn("for(const n of ['pops','sites','domains','noc-access'])",js)

if __name__=="__main__":
    unittest.main()
