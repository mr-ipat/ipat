"""R10.33: real headless Chrome with isolated, synthetic tenant service faults.

No company credentials, devices, DNS, or commercial runtime are touched.
"""
import subprocess
import tempfile
import threading
import unittest
from http.server import ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit
import sys

R1030=Path(__file__).resolve().parents[1]/"r1030"
sys.path.insert(0,str(R1030))
from test_dashboard_browser_flow import CHROME, Handler, WEB, PRELUDE

SCENARIO = """
<script>
(async()=>{
 const sleep=t=>new Promise(done=>setTimeout(done,t));
 for(let i=0;i<120;i++){
  const x=document.querySelector('#notice').textContent;
  if(x.includes('Device catalog unavailable')||x.includes('Company branding temporarily unavailable')
      ||x.includes('HTTP 401'))break;
  await sleep(50);
 }
 const m=new URL(location.href).searchParams.get('fault');
 const visible=id=>!document.getElementById('nav-'+id).hidden;
 const signinVisible=!document.getElementById('signin').hidden;
 const identity=document.getElementById('identity').textContent;
 const notice=document.getElementById('notice').textContent;
 let ok=false;
 if(m==='catalog'){
  ok=!visible('devices')&&visible('sites')&&visible('pops')&&!signinVisible
     &&identity.includes('Authorized tenant')&&notice.includes('Device catalog unavailable');
 }else if(m==='branding'){
  ok=visible('devices')&&!visible('branding')&&!signinVisible
    &&identity.includes('Authorized tenant')&&notice.includes('Company branding temporarily unavailable');
 }else if(m==='denied'){
  ok=[...document.querySelectorAll('.nav')].every(b=>b.hidden)
    &&signinVisible&&identity==='Sign-in required';
 }
 document.body.dataset.r1033Fault=ok?'PASS':'FAIL_'+m+'_'+identity;
})();
</script>
"""

class FaultHandler(Handler):
    fault = "catalog"
    def do_GET(self):
        route=urlsplit(self.path).path
        if route=="/dashboard":
            html=(WEB/"dashboard.html").read_text()
            html=html.replace("<head>","<head>"+PRELUDE,1)
            self.send(200,html+SCENARIO,"text/html; charset=utf-8")
            return
        if ((self.fault=="catalog" and route=="/api/v1/device-catalog")
          or (self.fault=="branding" and route=="/api/v1/branding")
          or (self.fault=="denied" and route=="/api/v1/capabilities")):
            status=401 if self.fault=="denied" else 503
            self.send(status,'{"ok":false}')
            return
        super().do_GET()

@unittest.skipUnless(CHROME, "Chrome not installed; CI must provide real Chrome")
class TenantDegradedServices(unittest.TestCase):
    def check(self, fault):
        FaultHandler.fault=fault
        FaultHandler.requests=[]
        FaultHandler.site_mode=False
        FaultHandler.reconcile_mode=False
        FaultHandler.stale_mode=False
        FaultHandler.denied_mode=False
        server=ThreadingHTTPServer(("127.0.0.1",0),FaultHandler)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            with tempfile.TemporaryDirectory(prefix="ipat-r1033-") as directory:
                args=[CHROME,"--headless=new","--no-first-run","--disable-gpu",
                    "--user-data-dir="+directory,"--virtual-time-budget=12000",
                    "--dump-dom",f"http://127.0.0.1:{server.server_port}/dashboard?fault={fault}"]
                p=subprocess.run(args,capture_output=True,text=True,timeout=40)
            self.assertEqual(p.returncode,0,p.stderr[-650:])
            self.assertIn('data-r1033-fault="PASS"',p.stdout)
            self.assertEqual(FaultHandler.requests,[])
        finally:
            server.shutdown()
            server.server_close()
    def test_catalog_503_disables_only_devices(self): self.check("catalog")
    def test_branding_503_preserves_authorized_modules(self): self.check("branding")
    def test_capability_401_hides_all_modules(self): self.check("denied")

if __name__=="__main__":
    unittest.main()
