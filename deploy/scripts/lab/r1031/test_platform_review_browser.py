"""Real local headless Chrome with synthetic, in-memory Platform Owner API only."""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import subprocess
import tempfile
import threading
import unittest

ROOT=Path(__file__).resolve().parents[4]
WEB=ROOT/"web/console/platform"
CHROME=next((str(p) for p in (
 Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
 Path("/usr/bin/google-chrome"),Path("/usr/bin/chromium")
) if p.exists()),None)

SCRIPT="""
<script>
Object.defineProperty(document,'cookie',{configurable:true,
 get:()=> '__Host-ipat_platform_csrf=synthetic'});
window.confirm=()=>location.search.includes('approve=1');
(async()=>{
 const sleep=t=>new Promise(done=>setTimeout(done,t));
 for(let i=0;i<90;i++){
  if(document.querySelector('#activation-rows button'))break;
  await sleep(50);
 }
 const btn=document.querySelector('#activation-rows button');
 if(!btn){document.body.dataset.r1031Review='NO_BUTTON';return;}
 btn.click();
 await sleep(350);
 document.body.dataset.r1031Review=location.search.includes('approve=1')
  ?(document.querySelector('#notice').textContent.includes('activation approved')?'PASS':'FAILED_POST')
  :(!btn.disabled?'PASS':'FAILED_CANCEL');
})();
</script>
"""

class Handler(BaseHTTPRequestHandler):
    review_count=0
    review_payloads=[]
    def log_message(self,*_): pass
    def send(self,code,data,kind='application/json'):
        self.send_response(code)
        self.send_header('Content-Type',kind)
        self.send_header('Cache-Control','no-store')
        self.end_headers()
        self.wfile.write(data.encode())
    def do_GET(self):
        route=self.path.split('?')[0]
        if route=='/platform':
            page=(WEB/'dashboard.html').read_text()
            self.send(200,page.replace('<head>','<head>'+SCRIPT,1),'text/html')
            return
        if route=='/platform/assets/app.js':
            self.send(200,(WEB/'app.js').read_text(),'text/javascript')
            return
        if route=='/platform/assets/style.css':
            self.send(200,(WEB/'style.css').read_text(),'text/css')
            return
        if route=='/api/v1/platform/reservations':
            self.send(200,json.dumps({'ok':True,'reservations':[]}))
            return
        if route=='/api/v1/platform/activation-requests':
            rows=[] if self.review_count else [{
                'request_id':'00000000-0000-4000-8000-000000000011',
                'tenant_slug':'synthetic-company','tenant_id':'00000000-0000-4000-8000-000000000012',
                'hostname':'synthetic.lab.invalid','admin_subject':'none','admin_issuer':'synthetic',
                'state':'REQUESTED'
            }]
            self.send(200,json.dumps({'ok':True,'requests':rows}))
            return
        self.send(404,'{"ok":false}')
    def do_POST(self):
        path=self.path.split('?')[0]
        size=int(self.headers.get('Content-Length','0'))
        payload=json.loads(self.rfile.read(size) or b'{}')
        if path.endswith('/review') and path.startswith('/api/v1/platform/activation-requests/'):
            self.__class__.review_count+=1
            self.__class__.review_payloads.append(payload)
            self.send(200,'{"ok":true}')
            return
        self.send(404,'{"ok":false}')

@unittest.skipUnless(CHROME,"Chrome not present: CI Node contract must still run")
class ReviewBrowser(unittest.TestCase):
    def run_case(self,approve):
        Handler.review_count=0;Handler.review_payloads=[]
        server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            with tempfile.TemporaryDirectory(prefix='ipat-r1031-browser-') as profile:
                proc=subprocess.run([CHROME,'--headless=new','--no-first-run',
                    '--disable-gpu','--user-data-dir='+profile,'--virtual-time-budget=11000',
                    '--dump-dom',f'http://127.0.0.1:{server.server_port}/platform?approve={int(approve)}'],
                    capture_output=True,text=True,timeout=35)
            self.assertEqual(proc.returncode,0,proc.stderr[-700:])
            self.assertIn('data-r1031-review="PASS"',proc.stdout)
            self.assertEqual(Handler.review_count,int(approve))
            if approve:self.assertEqual(Handler.review_payloads,[{'approve':True}])
        finally:
            server.shutdown();server.server_close()
    def test_cancel_does_not_post(self): self.run_case(False)
    def test_confirm_posts_one_review(self): self.run_case(True)

if __name__=='__main__':
    unittest.main()
