'use strict';
// EXPLICIT OPT-IN: owner-private QA temporarily changes C320 inventory SITE metadata.
if(process.env.IPAT_R963_APPROVE_OWNER_SITE_QA!=='YES'){
 console.error('Explicit opt-in required to run temporary owner-private Site/C320 metadata QA.');process.exit(2);
}
// Mutates only a synthetic Site and existing live C320's inventory POP relation.
// The OLT connector, credentials, transport and physical CLI never change.
const puppeteer=require('puppeteer');const assert=require('node:assert/strict');
(async()=>{
 const chrome=await puppeteer.launch({headless:true,
  executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  userDataDir:'/private/tmp/ipat-r963-chrome-private-qa',
  args:['--no-first-run','--disable-extensions','--disable-background-networking']});
 const page=await chrome.newPage();page.setDefaultTimeout(16000);
 const base='http://127.0.0.1:3002';const qaCode='QA-R963-INDEPENDENT';
 let qaSite=null,original=null,linkedAssigned=false;
 page.on('dialog',dialog=>dialog.accept());
 async function fetchJson(page,path,opts){return page.evaluate(async({path,opts})=>{
   const r=await fetch(path,opts);return {status:r.status,body:await r.json()};
  },{path,opts});}
 try{
  await page.goto(base+'/lab/device-workbench#managed-devices',{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>document.querySelector('#ipat-owner-device-summary').textContent.includes('managed records'));
  original=await fetchJson(page,'/lab/owner/devices');
  assert.equal(original.status,200);
  const live=original.body.devices.find(d=>d.id==='DEV-01');
  assert.equal(live?.connection_state,'CONNECTED');
  const sites=await fetchJson(page,'/lab/owner/pops');
  assert.equal(sites.status,200);
  assert(!sites.body.pops.some(p=>p.code===qaCode));
  assert.equal(sites.body.pops.length,0,'test expects owner site master initially empty');
  assert.equal(live.pop_id,'UNASSIGNED','do not replace an owner-assigned live POP');
  // Empty master must redirect to completely independent Site manager.
  await Promise.all([
    page.waitForNavigation({waitUntil:'domcontentloaded'}),
    page.click('#ipat-owner-device-add')
  ]);
  assert.match(page.url(),/\/lab\/sites\?return=device/);
  await page.waitForSelector('#ipat-sites-form-panel:not([hidden])');
  assert.equal(await page.$eval('#ipat-sites-form-heading',x=>x.textContent),'Create Site / POP');
  assert.equal(await page.$('#ipat-owner-device-form'),null);
  console.log('R963_CHROME_EMPTY_DEVICE_SITE_REDIRECT_TO_INDEPENDENT_PAGE=PASS');
  await page.$eval('#ipat-sites-code',(n,v)=>n.value=v,qaCode);
  await page.$eval('#ipat-sites-name',n=>n.value='QA standalone POP');
  await Promise.all([
    page.waitForNavigation({waitUntil:'domcontentloaded'}),
    page.click('#ipat-sites-save')
  ]);
  qaSite=qaCode;
  await page.waitForSelector('#ipat-owner-device-form:not([hidden])');
  const selected=await page.$eval('#ipat-owner-pop',node=>({tag:node.tagName,value:node.value,
    names:Array.from(node.options).map(n=>n.value)}));
  assert.equal(selected.tag,'SELECT');
  assert(selected.names.includes(qaCode));
  assert.equal(selected.value,qaCode);
  assert.equal(await page.$('#ipat-owner-pop-form'),null);
  console.log('R963_CHROME_INDEPENDENT_SITE_CREATE_RETURNS_WITH_REGISTERED_DEVICE_SELECTOR=PASS');
  await page.click('#ipat-owner-device-cancel');
  // Save metadata-only relation on the actual connected C320, then unassign.
  await page.evaluate(()=>{
    const row=Array.from(document.querySelectorAll('#ipat-owner-device-rows tr')).find(x=>x.textContent.includes('DEV-01'));
    if(!row)throw Error('Real C320 row unavailable');
    Array.from(row.querySelectorAll('button')).find(b=>b.textContent==='Edit label').click();
  });
  await page.waitForSelector('#ipat-owner-device-form:not([hidden])');
  await page.select('#ipat-owner-pop',qaCode);
  await page.click('#ipat-owner-device-save');
  await page.waitForFunction(async code=>{const r=await fetch('/lab/owner/pops/'+encodeURIComponent(code));if(!r.ok)return false;const d=await r.json();return d.pop.assigned_device_count===1;},{},qaCode);
  linkedAssigned=true;
  let pop=await fetchJson(page,'/lab/owner/pops/'+qaCode);
  assert.equal(pop.status,200);
  assert.equal(pop.body.pop.assigned_device_count,1);
  assert.equal(pop.body.pop.linked_c320_assigned,true);
  assert.equal(pop.body.pop.can_remove,false);
  const denied=await fetchJson(page,'/lab/owner/pops/'+qaCode,{method:'DELETE',
    headers:{'Content-Type':'application/json','X-IPAT-Owner-Private':'1'},
    body:JSON.stringify({expected_revision:1,confirm:'REMOVE POP '+qaCode})});
  assert.equal(denied.status,409);assert.equal(denied.body.error,'POP_HAS_ASSIGNED_DEVICES');
  console.log('R963_CHROME_LIVE_C320_SITE_ASSIGNMENT_AND_BACKEND_DELETE_GUARD=PASS');
  await page.goto(base+'/lab/sites',{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>document.querySelector('#ipat-sites-count').textContent.includes('registered'));
  const attached=await page.evaluate(code=>{
    const row=Array.from(document.querySelectorAll('#ipat-sites-rows tr')).find(r=>r.textContent.includes(code));
    return row?{text:row.textContent,removeDisabled:Array.from(row.querySelectorAll('button')).find(b=>b.textContent==='Remove').disabled}:null;
  },qaCode);
  assert.equal(attached?.removeDisabled,true);assert.match(attached.text,/Assigned · protected/);
  await page.evaluate(code=>{
    const row=Array.from(document.querySelectorAll('#ipat-sites-rows tr')).find(r=>r.textContent.includes(code));
    Array.from(row.querySelectorAll('button')).find(b=>b.textContent==='Edit').click();
  },qaCode);
  await page.waitForSelector('#ipat-sites-form-panel:not([hidden])');
  assert.equal(await page.$eval('#ipat-sites-code',n=>n.disabled),true);
  await page.$eval('#ipat-sites-name',n=>n.value='QA standalone POP renamed');
  await page.click('#ipat-sites-save');
  await page.waitForFunction(()=>Array.from(document.querySelectorAll('#ipat-sites-rows tr'))
     .some(n=>n.textContent.includes('QA standalone POP renamed')));
  pop=await fetchJson(page,'/lab/owner/pops/'+qaCode);
  assert.equal(pop.body.pop.revision,2);assert.equal(pop.body.pop.assigned_device_count,1);
  console.log('R963_CHROME_STANDALONE_SITE_RENAME_PRESERVES_LIVE_ASSIGNMENT=PASS');
  await page.goto(base+'/lab/device-workbench#managed-devices',{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>document.querySelector('#ipat-owner-device-summary').textContent.includes('managed records'));
  await page.evaluate(()=>{
    const row=Array.from(document.querySelectorAll('#ipat-owner-device-rows tr')).find(x=>x.textContent.includes('DEV-01'));
    Array.from(row.querySelectorAll('button')).find(b=>b.textContent==='Edit label').click();
  });
  await page.waitForSelector('#ipat-owner-device-form:not([hidden])');
  await page.select('#ipat-owner-pop','');
  await page.click('#ipat-owner-device-save');
  await page.waitForFunction(async code=>{const r=await fetch('/lab/owner/pops/'+encodeURIComponent(code));if(!r.ok)return false;const d=await r.json();return d.pop.assigned_device_count===0;},{},qaCode);
  linkedAssigned=false;
  const after=await fetchJson(page,'/lab/owner/devices');
  const active=after.body.devices.find(d=>d.id==='DEV-01');
  assert.equal(active.connection_state,'CONNECTED');
  assert.equal(active.display_name,live.display_name);
  assert.equal(active.pop_id,'UNASSIGNED');
  pop=await fetchJson(page,'/lab/owner/pops/'+qaCode);
  assert.equal(pop.body.pop.assigned_device_count,0);
  console.log('R963_CHROME_EXPLICIT_LIVE_C320_SITE_UNASSIGN_UNCHANGED_CONNECTOR_STATUS=PASS');
  await page.goto(base+'/lab/sites',{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>document.querySelector('#ipat-sites-count').textContent.includes('registered'));
  await page.evaluate(code=>{
    const row=Array.from(document.querySelectorAll('#ipat-sites-rows tr')).find(r=>r.textContent.includes(code));
    const button=Array.from(row.querySelectorAll('button')).find(b=>b.textContent==='Remove');
    if(button.disabled)throw Error('Site incorrectly retained assigned status');
    button.click();
  },qaCode);
  await page.waitForFunction(code=>!Array.from(document.querySelectorAll('#ipat-sites-rows tr'))
    .some(row=>row.textContent.includes(code)),{},qaCode);
  qaSite=null;
  console.log('R963_CHROME_INDEPENDENT_SITE_REMOVAL_AND_QA_CLEANUP=PASS');
 }finally{
  // Fail-closed safety cleanup, modifying only this test's metadata; never
  // issue any physical connector or CLI command.
  if(linkedAssigned||qaSite){
   try{
    await page.goto(base+'/lab/sites',{waitUntil:'domcontentloaded'});
    const summary=await page.evaluate(async({code,restore})=>{
      const hdr={'Content-Type':'application/json','X-IPAT-Owner-Private':'1'};
      const dev=await fetch('/lab/owner/devices').then(r=>r.json());
      const active=dev.devices.find(d=>d.id==='DEV-01');
      if(active.pop_id===code){
        const body={expected_revision:active.revision,display_name:active.display_name,clear_pop:true};
        const r=await fetch('/lab/owner/devices/DEV-01',{method:'PUT',headers:hdr,body:JSON.stringify(body)});
        if(!r.ok)throw Error('Unable to clean QA C320 metadata assignment');
      }
      const sites=await fetch('/lab/owner/pops').then(r=>r.json());
      const item=sites.pops.find(x=>x.code===code);
      if(item){
       const r=await fetch('/lab/owner/pops/'+encodeURIComponent(code),{method:'DELETE',headers:hdr,
         body:JSON.stringify({expected_revision:item.revision,confirm:'REMOVE POP '+code})});
       if(!r.ok)throw Error('Unable to remove QA Site');
      }
      return 'QA_CLEAN';
    },{code:qaCode,restore:linkedAssigned});
    console.log('R963_FAIL_SAFE_QA_METADATA_CLEANUP',summary);
   }catch(e){console.error('R963_MANUAL_QA_METADATA_REVIEW_NEEDED',String(e));process.exitCode=1;}
  }
  await chrome.close();
 }
})().catch(e=>{console.error('R963_REAL_BROWSER_QA_FAIL',e.stack||e.message);process.exitCode=1;});
