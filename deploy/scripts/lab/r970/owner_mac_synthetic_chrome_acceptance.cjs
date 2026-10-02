'use strict';
// Local synthetic Chrome acceptance ONLY. All HTTP requests are intercepted;
// never point this test at real company DNS, private VPS or a real IdP.
if(process.env.IPAT_R970_SYNTHETIC_CHROME!=='YES'){
 console.error('Set IPAT_R970_SYNTHETIC_CHROME=YES for synthetic-only browser test.');process.exit(2);
}
const assert=require('node:assert/strict'),fs=require('node:fs'),puppeteer=require('puppeteer');
const BASE='https://tenant-r970.example.invalid';
const pages={
 '/dashboard':fs.readFileSync('web/console/tenant/dashboard.html','utf8'),
 '/dashboard/assets/app.js':fs.readFileSync('web/console/tenant/app.js','utf8'),
 '/dashboard/assets/style.css':fs.readFileSync('web/console/tenant/style.css','utf8'),
};
const catalog=[
 {type:'olt',vendor:'ZTE',transports:['ssh','snmp']},
 {type:'olt',vendor:'C-DATA',transports:['ssh','snmp']},
 {type:'ont',vendor:'ZTE',transports:['cwmp','usp']},
 {type:'ont',vendor:'VSOL',transports:['cwmp','usp']},
 {type:'router',vendor:'MikroTik',transports:['ssh','snmp','routeros_api_ssl']},
];
let siteList=[],devices=[],domainList=[],reads=0,writeCount=0;
async function setup(page,admin){
 await page.setCookie({name:'__Host-ipat_csrf',value:'z'.repeat(43),url:BASE+'/',secure:true,sameSite:'Strict'});
 await page.setRequestInterception(true);
 page.on('request',async req=>{
  try{
   const u=new URL(req.url());
   if(u.origin!==BASE)throw Error('Unexpected external request: '+req.url());
   let out,status=200,type='application/json';
   const p=u.pathname, method=req.method(),body=req.postData()?JSON.parse(req.postData()):null;
   if(p in pages){out=pages[p];type=p.endsWith('.js')?'application/javascript':p.endsWith('.css')?'text/css':'text/html'}
   else if(p==='/api/v1/capabilities'){out={ok:true,tenant_id:'70707070-7070-4070-8070-707070707071',can_manage_sites:admin,can_manage_devices:admin,can_manage_domains:admin}}
   else if(!admin){status=403;out={ok:false}}
   else if(p==='/api/v1/device-catalog'){out={ok:true,catalog}}
   else if(p==='/api/v1/sites'&&method==='GET'){out={ok:true,sites:siteList,next_after:null};reads++}
   else if(p==='/api/v1/sites'&&method==='POST'){assert.strictEqual(req.headers()['x-ipat-csrf'],'z'.repeat(43));assert.strictEqual(req.headers().origin,BASE);siteList.push({...body,revision:1,assigned_devices:0});out={ok:true,code:body.code};status=201;writeCount++}
   else if(p==='/api/v1/devices'&&method==='GET'){out={ok:true,devices};reads++}
   else if(p==='/api/v1/devices'&&method==='POST'){assert.strictEqual(req.headers()['x-ipat-csrf'],'z'.repeat(43));assert.strictEqual(req.headers().origin,BASE);assert(!('secret_ref' in body));assert.equal(body.vendor,'ZTE');assert.equal(body.pop_id,'POP-R970');devices.push({...body,id:'73737373-7373-4373-8373-737373737373',site:body.pop_id,lifecycle_state:'SAVED'});out={ok:true,id:devices[0].id,lifecycle_state:'SAVED'};status=201;writeCount++}
   else if(p==='/api/v1/domains'&&method==='GET'){out={ok:true,domains:domainList};reads++}
   else if(p==='/api/v1/domains'&&method==='POST'){assert.strictEqual(req.headers()['x-ipat-csrf'],'z'.repeat(43));domainList.push({...body,id:'74747474-7474-4474-8474-747474747474',verification_name:'_ipat-verify.'+body.hostname,verification_value:'ipat-domain=synthetic-only',activation_state:'pending_dns'});out={ok:true,id:domainList[0].id};status=201;writeCount++}
   else if(p==='/favicon.ico'){out='';status=404;type='text/plain'}
   else throw Error('Unrecognized synthetic route '+method+' '+p);
   await req.respond({status,contentType:type,body:typeof out==='string'?out:JSON.stringify(out)});
  }catch(e){console.error('MOCK_API_FLOW_ERROR',e);await req.abort('failed');process.exitCode=1;}
 });
}
(async()=>{
 const browser=await puppeteer.launch({headless:true,executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',args:['--disable-extensions','--no-first-run']});
 try{
  const page=await browser.newPage();page.setDefaultTimeout(11000);await setup(page,true);
  await page.goto(BASE+'/dashboard',{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>!document.querySelector('#nav-sites').hidden);
  assert.equal(await page.$eval('#nav-devices',n=>n.hidden),false);
  await page.click('#nav-sites');await page.waitForFunction(()=>document.querySelector('#notice').textContent.includes('Data loaded'));
  await page.$eval('#site-create input[name=code]',n=>n.value='POP-R970');
  await page.$eval('#site-create input[name=display_name]',n=>n.value='Synthetic standalone POP');
  await page.click('#site-create button');
  await page.waitForFunction(()=>document.querySelector('#site-rows').textContent.includes('POP-R970'));
  console.log('R970_CHROME_STANDALONE_SITE_REGISTRATION=PASS');
  await page.click('#nav-devices');await page.waitForFunction(()=>document.querySelector('#device-site').options.length>1);
  assert.equal(await page.$eval('#device-vendor',n=>Array.from(n.options).some(o=>o.value==='Forged')),false);
  await page.$eval('#device-create input[name=display_name]',n=>n.value='Synthetic C320 metadata only');
  await page.select('#device-site','POP-R970');
  await page.$eval('#device-create input[name=intended_model]',n=>n.value='C320');
  await page.$eval('#device-create input[name=management_host]',n=>n.value='olt.synthetic.invalid');
  await page.click('#device-create button');
  try{await page.waitForFunction(()=>document.querySelector('#device-rows').textContent.includes('Saved — not connected'))}
  catch(error){console.error('R970_DEVICE_FORM_DIAGNOSTIC',await page.evaluate(()=>({notice:document.querySelector('#notice').textContent,valid:document.querySelector('#device-create').checkValidity(),invalid:Array.from(document.querySelectorAll('#device-create :invalid')).map(x=>x.name),values:Object.fromEntries(new FormData(document.querySelector('#device-create')))})),{writeCount,devicesCount:devices.length});throw error;}
  console.log('R970_CHROME_SERVER_CATALOG_AND_SITE_FIRST_SAVED_NOT_CONNECTED=PASS');
  await page.click('#nav-domains');await page.waitForFunction(()=>document.querySelector('#notice').textContent.includes('Data loaded'));
  await page.$eval('#domain-create input[name=hostname]',n=>n.value='portal.synthetic.invalid');
  await page.click('#domain-create button');
  await page.waitForFunction(()=>document.querySelector('#domain-rows').textContent.includes('ipat-domain=synthetic-only'));
  assert.equal(domainList[0].activation_state,'pending_dns');
  console.log('R970_CHROME_CUSTOM_DOMAIN_PENDING_TXT_WITHOUT_FALSE_ACTIVATION=PASS');
  const readOnly=await browser.newPage();readOnly.setDefaultTimeout(11000);await setup(readOnly,false);
  await readOnly.goto(BASE+'/dashboard',{waitUntil:'domcontentloaded'});
  await readOnly.waitForFunction(()=>document.querySelector('#notice').textContent.includes('No tenant administration modules'));
  assert.equal(await readOnly.$eval('#nav-sites',n=>n.hidden),true);
  assert.equal(await readOnly.$eval('#nav-devices',n=>n.hidden),true);
  assert.equal(await readOnly.$eval('#nav-domains',n=>n.hidden),true);
  console.log('R970_CHROME_UNAUTHORIZED_OPERATOR_MENUS_HIDDEN=PASS');
  assert.equal(writeCount,3);assert(reads>=3);console.log('R970_SYNTHETIC_CHROME_ALL_FOUR_PRODUCT_FLOWS=PASS');
 }finally{await browser.close()}
})().catch(e=>{console.error('R970_SYNTHETIC_CHROME_FAILED',e.stack||e);process.exitCode=1});
