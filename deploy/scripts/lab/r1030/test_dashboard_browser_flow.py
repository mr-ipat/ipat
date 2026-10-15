#!/usr/bin/env python3
"""Ephemeral HTTP + actual headless Chrome test of tenant navigation/retry UI.
This is a synthetic browser test, never an authenticated production session.
"""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import json
import os
import shutil
import subprocess
import tempfile
import threading
import unittest
from urllib.parse import urlsplit,parse_qs

ROOT = Path(__file__).resolve().parents[4]
WEB = ROOT / "web/console/tenant"
CHROME = next((c for c in (
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/usr/bin/google-chrome", "/usr/bin/chromium",
) if Path(c).exists()), None)

PRELUDE = """<script>
Object.defineProperty(document,'cookie',{configurable:true,
  get:()=> '__Host-ipat_csrf=synthetic_browser_only'});
</script>"""

SCENARIO = """
<script>
(async()=>{
 const sleep=t=>new Promise(resolve=>setTimeout(resolve,t));
 const done=(x)=>{document.body.setAttribute('data-r1030-acceptance',x)};
 for(let i=0;i<90;i++){
  if(!document.querySelector('[data-page=devices]').hidden)break;
  await sleep(50);
 }
 const nav=document.querySelector('[data-page=devices]');
 if(nav.hidden){done('NO_AUTHORIZED_NAV');return}
 nav.click();
 for(let i=0;i<90;i++){
  if(document.querySelector('#device-site').options.length>1)break;
  await sleep(50);
 }
 if(nav.getAttribute('aria-current')!=='page'){done('NO_ACTIVE_NAV');return}
 if(document.activeElement!==document.querySelector('[data-panel=devices] h2')){done('NO_HEADING_FOCUS');return}
 const form=document.querySelector('#device-create');
 form.elements.display_name.value='Lab-safe synthetic example';
 form.elements.pop_id.value='SITE-R1030';
 form.elements.intended_model.value='C320';
 form.elements.management_host.value='olt.lab.invalid';
 form.elements.management_port.value='321';
 form.requestSubmit();
 for(let i=0;i<90;i++){
  if(document.querySelector('#notice').textContent.includes('HTTP 503'))break;
  await sleep(50);
 }
 if(!document.querySelector('#notice').textContent.includes('HTTP 503')){done('FIRST_REJECTION_MISSING');return}
 form.requestSubmit();
 for(let i=0;i<90;i++){
  if(document.querySelector('#notice').textContent.includes('Device saved as metadata'))break;
  await sleep(50);
 }
 if(!document.querySelector('#notice').textContent.includes('Device saved as metadata')){done('SECOND_SAVE_MISSING');return}
 done('PASS');
})();
</script>
"""


SCENARIO_SITE = """
<script>
(async()=>{
 const sleep=t=>new Promise(resolve=>setTimeout(resolve,t));
 const done=x=>document.body.setAttribute('data-r1030-site-resume',x);
 for(let i=0;i<80;i++){
  if(!document.querySelector('[data-page=devices]').hidden)break;
  await sleep(50);
 }
 document.querySelector('[data-page=devices]').click();
 for(let i=0;i<80;i++){
  if(document.querySelector('#device-site-note').textContent.startsWith('No Sites'))break;
  await sleep(50);
 }
 if(!document.querySelector('#device-site-note').textContent.startsWith('No Sites')){done('NO_EMPTY_STATE');return}
 document.querySelector('#create-site-link').click();
 if(document.querySelector('[data-page=sites]').getAttribute('aria-current')!=='page'){done('NOT_SITES');return}
 const f=document.querySelector('#site-create');
 f.elements.code.value='SITE-R1030';
 f.elements.display_name.value='Fresh operator site';
 f.requestSubmit();
 for(let i=0;i<100;i++){
  if(document.querySelector('[data-page=devices]').getAttribute('aria-current')==='page'
   && document.querySelector('#device-site').value==='SITE-R1030')break;
  await sleep(50);
 }
 if(document.querySelector('#device-site').value!=='SITE-R1030'){done('SITE_NOT_SELECTED');return}
 if(document.querySelector('[data-page=devices]').getAttribute('aria-current')!=='page'){done('DID_NOT_RETURN');return}
 done('PASS');
})();
</script>
"""


SCENARIO_DENIED = """
<script>
(async()=>{
 const sleep=t=>new Promise(resolve=>setTimeout(resolve,t));
 for(let i=0;i<90;i++){
  if(document.querySelector('#notice').textContent.includes('No operations modules assigned'))break;
  await sleep(50);
 }
 const visible=[...document.querySelectorAll('.nav')].filter(n=>!n.hidden);
 document.body.setAttribute('data-r1030-denied',
  visible.length===0 && !document.querySelector('#welcome').hidden ? 'PASS':'FAIL');
})();
</script>
"""

SCENARIO_MOBILE = """
<script>
(async()=>{
 const sleep=t=>new Promise(resolve=>setTimeout(resolve,t));
 for(let i=0;i<80;i++){
  if(!document.querySelector('[data-page=devices]').hidden)break;
  await sleep(50);
 }
 document.querySelector('[data-page=devices]').click();
 await sleep(250);
 const viewport=document.documentElement.clientWidth;
 const width=document.documentElement.scrollWidth;
 const table=document.querySelector('[data-panel=devices] .table-wrap');
 const mobile=window.innerWidth<=500;
 const wrapped=table && getComputedStyle(table).overflowX==='auto';
 document.body.setAttribute('data-r1030-mobile',
  mobile&&wrapped&&width<=viewport+2?'PASS':'FAIL_'+mobile+'_'+width+'_'+viewport);
})();
</script>
"""


SCENARIO_RECONCILE = """
<script>
(async()=>{
 const sleep=t=>new Promise(resolve=>setTimeout(resolve,t));
 const done=x=>document.body.setAttribute('data-r1030-reconcile',x);
 for(let i=0;i<80;i++){
  if(!document.querySelector('[data-page=devices]').hidden)break;
  await sleep(50);
 }
 document.querySelector('[data-page=devices]').click();
 for(let i=0;i<80;i++){
  if(document.querySelector('#device-site').options.length>1)break;
  await sleep(50);
 }
 const form=document.querySelector('#device-create');
 form.elements.display_name.value='Safe reconciliation fixture';
 form.elements.pop_id.value='SITE-R1030';
 form.elements.intended_model.value='C320';
 form.elements.management_host.value='olt.lab.invalid';
 form.elements.management_port.value='321';
 form.requestSubmit();
 for(let i=0;i<80;i++){
  if(document.querySelector('#notice').textContent.includes('inventory list could not be refreshed'))break;
  await sleep(50);
 }
 if(!document.querySelector('#notice').textContent.includes('inventory list could not be refreshed')){done('MISSING_LIST_ERROR');return}
 if(form.elements.display_name.value!=='Safe reconciliation fixture'){done('LOST_FORM');return}
 form.requestSubmit();
 for(let i=0;i<80;i++){
  if(document.querySelector('#notice').textContent.includes('confirmed in the inventory list'))break;
  await sleep(50);
 }
 if(!document.querySelector('#notice').textContent.includes('confirmed in the inventory list')){done('MISSING_RECONCILIATION');return}
 done('PASS');
})();
</script>
"""

class Handler(BaseHTTPRequestHandler):
    requests = []
    site_mode = False
    site_created = False
    denied_mode = False
    mobile_mode = False
    reconcile_mode = False
    list_failed = False
    def log_message(self, *_):
        pass
    def send(self, code, body, content_type="application/json"):
        self.send_response(code)
        self.send_header("Content-Type", content_type)
        self.send_header("Cache-Control","no-store")
        self.end_headers()
        self.wfile.write(body.encode())
    def do_GET(self):
        path=urlsplit(self.path).path
        if path=="/dashboard":
            html=(WEB/"dashboard.html").read_text()
            html=html.replace("<head>", "<head>"+PRELUDE, 1)
            self.send(200,html+(SCENARIO_DENIED if self.denied_mode else SCENARIO_RECONCILE if self.reconcile_mode else SCENARIO_MOBILE if self.mobile_mode else SCENARIO_SITE if self.site_mode else SCENARIO), "text/html; charset=utf-8")
            return
        for route,name,kind in (
            ("/dashboard/assets/app.js","app.js","text/javascript"),
            ("/dashboard/assets/style.css","style.css","text/css")):
            if path==route:
                self.send(200,(WEB/name).read_text(),kind);return
        responses={
            "/api/v1/capabilities": {"ok":True,"tenant_id":"00000000-0000-4000-8000-000000000010","can_manage_sites":not self.denied_mode},
            "/api/v1/branding": {"ok":True,"branding":{"display_name":"Synthetic Company","mark_text":"SC","accent_token":"slate","revision":1}},
            "/api/v1/device-catalog": {"ok":True,"catalog_semantics":"platform_curated_metadata_only_not_verified_physical_support",
              "catalog":[{"type":"olt","vendor":"ZTE","transports":["ssh"],"protocols":[{"transport":"ssh","description":"Inventory candidate only."}]}]},
            "/api/v1/pops": {"ok":True,"pops":[],"next_after":None},
            "/api/v1/sites": {"ok":True,"sites":([{"code":"SITE-R1030","display_name":"Test site","assigned_devices":0,"revision":1,"parent_pop_code":None}] if not self.site_mode or self.site_created else []),"next_after":None},
            "/api/v1/devices": {"ok":True,"devices":[]},
        }
        if path=="/api/v1/devices" and self.reconcile_mode and len(self.requests)==1 and not self.list_failed:
            self.__class__.list_failed=True
            self.send(503,'{"ok":false}')
            return
        self.send(200,json.dumps(responses.get(path,{"ok":False})))
    def do_POST(self):
        path=urlsplit(self.path).path
        size=int(self.headers.get("Content-Length","0"))
        body=json.loads(self.rfile.read(size) or b"{}")
        if path=="/api/v1/sites" and self.site_mode:
            self.__class__.site_created=True
            self.send(201,json.dumps({"ok":True,"code":"SITE-R1030"}))
            return
        if path=="/api/v1/devices":
            self.__class__.requests.append(body)
            if len(self.requests)==1 and not self.reconcile_mode:
                self.send(503,json.dumps({"ok":False}))
            else:
                self.send(201,json.dumps({"ok":True,"id":"00000000-0000-4000-8000-000000000012","lifecycle_state":"SAVED"}))
            return
        self.send(404,'{"ok":false}')

@unittest.skipUnless(CHROME, "Headless Chrome not installed; separate GitHub source/CI required")
class DashboardChromeTest(unittest.TestCase):
    def test_saved_device_reconciles_failed_list_without_new_request_id(self):
        Handler.requests=[]
        Handler.reconcile_mode=True
        Handler.list_failed=False
        Handler.site_mode=False
        Handler.denied_mode=False
        server=ThreadingHTTPServer(("127.0.0.1",0),Handler)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            with tempfile.TemporaryDirectory(prefix="ipat-r1030-reconcile-") as directory:
                proc=subprocess.run([CHROME,"--headless=new","--no-first-run",
                  "--disable-gpu","--user-data-dir="+directory,
                  "--virtual-time-budget=12000","--dump-dom",
                  f"http://127.0.0.1:{server.server_port}/dashboard"],
                  capture_output=True,text=True,timeout=35)
            self.assertEqual(proc.returncode,0,proc.stderr[-700:])
            self.assertIn('data-r1030-reconcile="PASS"',proc.stdout)
            self.assertTrue(Handler.list_failed)
            self.assertEqual(len(Handler.requests),2)
            self.assertEqual(Handler.requests[0]["request_id"],Handler.requests[1]["request_id"])
            print("R1030_HEADLESS_CHROME_POST_SUCCEEDED_LIST_FAILED_SAFE_RETRY=PASS")
        finally:
            Handler.reconcile_mode=False
            server.shutdown()
            server.server_close()

    def test_mobile_viewport_has_no_document_horizontal_overflow(self):
        Handler.requests=[]
        Handler.mobile_mode=True
        Handler.denied_mode=False
        Handler.site_mode=False
        server=ThreadingHTTPServer(("127.0.0.1",0),Handler)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            with tempfile.TemporaryDirectory(prefix="ipat-r1030-mobile-") as directory:
                proc=subprocess.run([CHROME,"--headless=new","--no-first-run",
                  "--disable-gpu","--user-data-dir="+directory,
                  "--window-size=390,850","--force-device-scale-factor=1",
                  "--virtual-time-budget=10000","--dump-dom",
                  f"http://127.0.0.1:{server.server_port}/dashboard"],
                  capture_output=True,text=True,timeout=35)
            self.assertEqual(proc.returncode,0,proc.stderr[-700:])
            self.assertIn('data-r1030-mobile="PASS"',proc.stdout)
            print("R1030_HEADLESS_CHROME_COMPACT_LAYOUT_OVERFLOW=PASS")
        finally:
            Handler.mobile_mode=False
            server.shutdown()
            server.server_close()

    def test_deny_by_default_menu_stays_hidden_without_capability(self):
        Handler.requests=[]
        Handler.denied_mode=True
        Handler.site_mode=False
        server=ThreadingHTTPServer(("127.0.0.1",0),Handler)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            with tempfile.TemporaryDirectory(prefix="ipat-r1030-no-role-") as directory:
                proc=subprocess.run([CHROME,"--headless=new","--no-first-run",
                  "--disable-gpu","--user-data-dir="+directory,
                  "--virtual-time-budget=10000","--dump-dom",
                  f"http://127.0.0.1:{server.server_port}/dashboard"],
                  capture_output=True,text=True,timeout=35)
            self.assertEqual(proc.returncode,0,proc.stderr[-700:])
            self.assertIn('data-r1030-denied="PASS"',proc.stdout)
            self.assertEqual(len(Handler.requests),0)
            print("R1030_HEADLESS_CHROME_DEFAULT_DENY_MENU=PASS")
        finally:
            Handler.denied_mode=False
            server.shutdown()
            server.server_close()

    def test_empty_site_guidance_returns_to_saved_device_form(self):
        Handler.requests=[]
        Handler.site_mode=True
        Handler.site_created=False
        server=ThreadingHTTPServer(("127.0.0.1",0),Handler)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            with tempfile.TemporaryDirectory(prefix="ipat-r1030-site-") as directory:
                proc=subprocess.run([CHROME,"--headless=new","--no-first-run",
                  "--disable-gpu","--user-data-dir="+directory,
                  "--virtual-time-budget=12000","--dump-dom",
                  f"http://127.0.0.1:{server.server_port}/dashboard?scenario=site"],
                  capture_output=True,text=True,timeout=35)
            self.assertEqual(proc.returncode,0,proc.stderr[-700:])
            self.assertIn('data-r1030-site-resume="PASS"',proc.stdout)
            self.assertTrue(Handler.site_created)
            print("R1030_HEADLESS_CHROME_EMPTY_SITE_REGISTER_RESUME=PASS")
        finally:
            Handler.site_mode=False
            server.shutdown()
            server.server_close()

    def test_single_device_intent_reuses_idempotency_id_after_timeout(self):
        Handler.requests=[]
        Handler.site_mode=False
        Handler.site_created=False
        server=ThreadingHTTPServer(("127.0.0.1",0),Handler)
        thread=threading.Thread(target=server.serve_forever,daemon=True)
        thread.start()
        try:
            with tempfile.TemporaryDirectory(prefix="ipat-r1030-ui-") as directory:
                args=[CHROME,"--headless=new","--no-first-run","--no-default-browser-check",
                  "--disable-gpu","--user-data-dir="+directory,"--virtual-time-budget=12000",
                  "--dump-dom",f"http://127.0.0.1:{server.server_port}/dashboard"]
                proc=subprocess.run(args,capture_output=True,text=True,timeout=40)
            self.assertEqual(proc.returncode,0,proc.stderr[-700:])
            self.assertIn('data-r1030-acceptance="PASS"',proc.stdout)
            self.assertEqual(len(Handler.requests),2)
            self.assertEqual(Handler.requests[0]["request_id"],Handler.requests[1]["request_id"])
            self.assertEqual(Handler.requests[0]["pop_id"],"SITE-R1030")
            self.assertEqual(Handler.requests[1]["management_port"],321)
            print("R1030_HEADLESS_CHROME_NAV_FOCUS_RETRY_IDEMPOTENCY=PASS")
        finally:
            server.shutdown()
            server.server_close()

if __name__ == "__main__":
    unittest.main()
