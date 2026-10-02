'use strict';
if(process.env.IPAT_R962_APPROVE_OWNER_BROWSER_QA!=='YES' || !process.env.IPAT_R962_TEMP_PUBLIC_IPV4){
 console.error('R962 requires explicit owner test opt-in and public IPv4 input.');process.exit(2);
}
const puppeteer=require('puppeteer');const assert=require('node:assert/strict');
(async()=>{
const chrome=await puppeteer.launch({headless:true,
 executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
 userDataDir:'/private/tmp/ipat-r962-qa-chrome-profile',
 args:['--no-first-run','--disable-background-networking','--disable-extensions']});
const page=await chrome.newPage();page.setDefaultTimeout(15000);
let pop=null,device=null;
const qaCode='QA-R962-SITE';
const root='http://127.0.0.1:3002';
async function json(url,options){const res=await fetch(root+url,options);return [res.status,await res.json()];}
page.on('dialog',d=>d.accept());
try{
 await page.goto(root+'/lab/device-workbench#managed-devices',{waitUntil:'domcontentloaded'});
 await page.waitForFunction(()=>document.getElementById('ipat-owner-pops-notice').textContent.includes('registered'));
 let status=await page.$eval('#ipat-owner-pops-notice',x=>x.textContent);
 console.log('R962_REAL_CHROME_INITIAL_POP_MASTER='+status);
 // When the real owner POP list is empty, Add must suggest Create POP,
 // rather than accepting a free-text fake POP-A.
 await page.click('#ipat-owner-device-add');
 const popOpened=await page.$eval('#ipat-owner-pop-form',x=>!x.hidden);
 if(status.startsWith('No sites'))assert.equal(popOpened,true);
 if(!popOpened)await page.click('#ipat-owner-pop-create');
 await page.waitForSelector('#ipat-owner-pop-form:not([hidden])');
 await page.$eval('#ipat-owner-pop-code',(x,v)=>x.value=v,qaCode);
 await page.$eval('#ipat-owner-pop-name',x=>x.value='QA site for master selector');
 await page.click('#ipat-owner-pop-save');
 await page.waitForFunction(()=>document.querySelector('#ipat-owner-pops-notice').textContent.includes('registered POP'));
 const site=await page.evaluate(code=>{
  const selector=document.querySelector('#ipat-owner-pop');
  const opts=Array.from(selector.options).map(x=>({value:x.value,text:x.textContent}));
  return {present:opts.some(o=>o.value===code),isSelect:selector.tagName==='SELECT'};
 },qaCode);
 assert.equal(site.isSelect,true);assert.equal(site.present,true);pop=qaCode;
 console.log('R962_REAL_CHROME_EMPTY_POP_GUIDED_CREATE_AND_SELECTOR=PASS');
 // The vendor picker changes by device type; only planned vendors appear,
 // and a note explicitly denies blanket compatibility.
 await page.waitForSelector('#ipat-owner-device-form:not([hidden])');
 await page.select('#ipat-owner-kind','router');
 const routerOptions=await page.$eval('#ipat-owner-vendor',n=>Array.from(n.options).map(v=>v.value).filter(Boolean));
 assert.deepEqual(routerOptions,['MikroTik']);
 await page.select('#ipat-owner-kind','olt');
 const oltOptions=await page.$eval('#ipat-owner-vendor',n=>Array.from(n.options).map(v=>v.value).filter(Boolean));
 assert.deepEqual(oltOptions,['ZTE','C-DATA']);
 await page.select('#ipat-owner-vendor','C-DATA');
 const statusVendor=await page.$eval('#ipat-owner-vendor-capability',n=>n.textContent);
 assert.match(statusVendor,/INVENTORY CANDIDATE/);
 const protocols=await page.$eval('#ipat-owner-protocol',n=>Array.from(n.options).map(o=>o.value).filter(Boolean));
 assert.deepEqual(protocols,['ssh','snmp']);
 console.log('R962_REAL_CHROME_VENDOR_KIND_PROTOCOL_AND_SUPPORT_STATUS=PASS');
 await page.$eval('#ipat-owner-name',x=>x.value='QA-R962-OLT-METADATA');
 await page.$eval('#ipat-owner-model',x=>x.value='UNVERIFIED-C-DATA-MODEL');
 await page.$eval('#ipat-owner-host',x=>x.value='192.0.2.162');
 await page.$eval('#ipat-owner-port',x=>x.value='22');
 await page.click('#ipat-owner-device-save');
 await page.waitForFunction(()=>Array.from(document.querySelectorAll('#ipat-owner-device-rows tr')).some(r=>r.textContent.includes('QA-R962-OLT-METADATA')));
 device=await page.evaluate(()=>Array.from(document.querySelectorAll('#ipat-owner-device-rows tr'))
  .find(r=>r.textContent.includes('QA-R962-OLT-METADATA'))?.querySelector('small')?.textContent);
 assert.match(device,/^OWN-\d{6}$/);
 const entry=await page.evaluate(()=>Array.from(document.querySelectorAll('#ipat-owner-device-rows tr')).find(r=>r.textContent.includes('QA-R962-OLT-METADATA'))?.textContent);
 assert.match(entry,/SAVED · NOT CONNECTED/);assert.match(entry,/QA-R962-SITE/);
 console.log('R962_REAL_CHROME_SAVE_DURABLE_METADATA_AT_PREEXISTING_POP=PASS');
 // A still-assigned site cannot be removed through browser or backend.
 await page.evaluate(code=>{
  const row=Array.from(document.querySelectorAll('#ipat-owner-pop-rows tr'))
     .find(x=>x.textContent.includes(code));
  Array.from(row.querySelectorAll('button')).find(x=>x.textContent==='Remove').click();
 },qaCode);
 await page.waitForFunction(()=>document.querySelector('#ipat-owner-device-notice').textContent.includes('POP_HAS_ASSIGNED_DEVICES'));
 console.log('R962_REAL_CHROME_ASSIGNED_POP_DELETE_DENIED=PASS');
 // Remove only our metadata record; existing DEV-01 must remain untouched.
 await page.evaluate(()=>{
  const row=Array.from(document.querySelectorAll('#ipat-owner-device-rows tr')).find(x=>x.textContent.includes('QA-R962-OLT-METADATA'));
  Array.from(row.querySelectorAll('button')).find(x=>x.textContent==='Remove').click();
 });
 await page.waitForFunction(()=>!Array.from(document.querySelectorAll('#ipat-owner-device-rows tr')).some(r=>r.textContent.includes('QA-R962-OLT-METADATA')));
 device=null;
 await page.evaluate(code=>{
  const row=Array.from(document.querySelectorAll('#ipat-owner-pop-rows tr')).find(x=>x.textContent.includes(code));
  Array.from(row.querySelectorAll('button')).find(x=>x.textContent==='Remove').click();
 },qaCode);
 await page.waitForFunction(code=>!document.querySelector('#ipat-owner-pops-notice').textContent.includes('registered POP')
    && !Array.from(document.querySelectorAll('#ipat-owner-pop-rows tr')).some(x=>x.textContent.includes(code)),{},qaCode);
 pop=null;
 assert.match(await page.$eval('#ipat-owner-device-rows',x=>x.textContent),/DEV-01/);
 console.log('R962_REAL_CHROME_CLEANUP_AND_LINKED_C320_RETAINED=PASS');
 // Actual private admin page must remain clear about readiness. Stage ONLY
 // observed temporary public IPv4; do not infer proof of ipat.id ownership.
 await page.goto(root+'/lab/platform-admin/domains',{waitUntil:'domcontentloaded'});
 await page.waitForFunction(()=>document.querySelector('#ipat-domain-profile-status').textContent.includes('Configuration loaded'));
 assert.match(await page.$eval('#ipat-domain-gates',x=>x.textContent),/HTTPS: not ready/);
 await page.$eval('#ipat-domain-public-ip',x=>x.value=process.env.IPAT_R962_TEMP_PUBLIC_IPV4);
 await page.click('#ipat-domain-profile-save');
 await page.waitForFunction(()=>document.querySelector('#ipat-domain-profile-status').textContent.includes('Draft saved'));
 await page.reload({waitUntil:'domcontentloaded'});
 await page.waitForFunction(()=>document.querySelector('#ipat-domain-profile-status').textContent.includes('Configuration loaded'));
 assert.equal(await page.$eval('#ipat-domain-public-ip',x=>x.value),process.env.IPAT_R962_TEMP_PUBLIC_IPV4);
 assert.match(await page.$eval('#ipat-domain-gates',x=>x.textContent),/HTTPS: not ready/);
 console.log('R962_REAL_CHROME_PLATFORM_TEMP_IP_DRAFT_PERSISTS_NOT_PUBLIC=PASS');
}finally{
 // Cleanup only QA-created metadata and POP using protected owner APIs.
 // The requested operator-admin *temporary public IPv4 plan* is intentional
 // and retained, but this script NEVER changes actual routing/HTTPS.
 const cleanupPage=await chrome.newPage();
 try{
 await cleanupPage.goto(root+'/lab/device-workbench',{waitUntil:'domcontentloaded'});
 const result=await cleanupPage.evaluate(async({id,code})=>{
  const hdr={'Content-Type':'application/json','X-IPAT-Owner-Private':'1'};
  const all=await fetch('/lab/owner/devices').then(r=>r.json());
  const d=all.devices.find(x=>x.id===id);
  if(d)await fetch('/lab/owner/devices/'+encodeURIComponent(id),{method:'DELETE',headers:hdr,
    body:JSON.stringify({expected_revision:d.revision,confirm:'REMOVE '+id})});
  const sites=await fetch('/lab/owner/pops').then(r=>r.json());
  const p=sites.pops.find(x=>x.code===code);
  if(p)return (await fetch('/lab/owner/pops/'+encodeURIComponent(code),{method:'DELETE',headers:hdr,
    body:JSON.stringify({expected_revision:p.revision,confirm:'REMOVE POP '+code})})).status;
  return 'NO_TEST_RECORD';
 },{id:device,code:pop});
 console.log('R962_QA_CLEANUP_CHECK',result);
 }catch(e){console.log('R962_QA_CLEANUP_FAILED',String(e))}
 await cleanupPage.close();await chrome.close();
}
})().catch(e=>{console.error('R962_BROWSER_FAILURE',e.message);process.exitCode=1;});
