'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const source=fs.readFileSync(path.resolve(__dirname,'../../../../web/lab/c320-connection-setup.js'),'utf8');
async function simulate({password='SYNTHETIC',ownerCode='',autoVerify=false}) {
 const ids=['ipat-c320-enroll-form','ipat-c320-enroll-state','ipat-c320-enroll-result',
 'ipat-c320-enroll-button','ipat-device-type','ipat-device-model','ipat-device-protocol',
 'ipat-device-host','ipat-device-port','ipat-device-username',
 'ipat-c320-device-password','ipat-c320-bootstrap','ipat-device-name',
 'ipat-device-profile-notice','ipat-device-save-stage','ipat-device-connect-stage','ipat-device-diagnostic'];
 const elements=Object.fromEntries(ids.map(id=>[id,{
   id,value:'',textContent:'',disabled:false,hidden:false,style:{},
   listeners:{},addEventListener(kind,fn){this.listeners[kind]=fn},
   focus(){this.focused=true},
   replaceChildren(){},
   append(){},
   querySelector(){return security}
 }]));
 elements['ipat-c320-enroll-form'].requestSubmit=function(){
   this.lastSubmit=this.listeners.submit({preventDefault(){}});
 };
 const security={open:false};
 Object.assign(elements['ipat-device-type'],{value:'olt'});
 Object.assign(elements['ipat-device-model'],{value:'zte_c320_lab'});
 Object.assign(elements['ipat-device-protocol'],{value:'ssh'});
 Object.assign(elements['ipat-device-host'],{value:'10.10.13.233'});
 Object.assign(elements['ipat-device-port'],{value:'321'});
 Object.assign(elements['ipat-device-username'],{value:'zte'});
 Object.assign(elements['ipat-device-name'],{value:'ZTE C320 Lab'});
 elements['ipat-c320-device-password'].value=password;
 elements['ipat-c320-bootstrap'].value=ownerCode;
 let draft=0,enroll=0,network=0,events=0;
 const status={target:'DEV-01',model:'C320',production_adopted:false,
   physical_writes_enabled:false,connector_online:true,credentials_enrolled:false,draft_saved:false};
 const context={
   document:{getElementById(id){return elements[id]},createElement(){return {value:'',textContent:''}}},
   window:{dispatchEvent(){events++}},Event:class{constructor(name){this.name=name}},
   setTimeout(){return 1},clearTimeout(){},setInterval(){},
   AbortController:class{constructor(){this.signal={}} abort(){}},
   fetch:async(url,init)=>{
     if(url.endsWith('/lab/c320-owner-connection'))return{ok:true,json:async()=>status};
     if(url.endsWith('/lab/c320-owner-save-draft')){
       draft++;
       status.draft_saved=true;
       const saved=JSON.parse(init.body);
       assert.deepEqual(Object.keys(saved).sort(),
          ['device_name','device_profile','device_type','management_ip','ssh_port','username'].sort());
       assert.equal(saved.device_name,'ZTE C320 Lab');
       assert.equal(saved.management_ip,'10.10.13.233');
       return {ok:true,json:async()=>({saved:true,adoption_state:'DRAFT_SAVED_AWAITING_AUTH'})};
     }
     if(url.endsWith('/lab/c320-owner-network-probe')){
       network++;
       return {ok:true,json:async()=>({probe_stage:'TCP_REACHABLE_AUTH_NOT_TESTED'})};
     }
     if(url.endsWith('/lab/c320-owner-enroll')){
       enroll++;
       const body=JSON.parse(init.body);
       assert.equal(body.password,'SYNTHETIC');
       assert.equal(body.bootstrap_code,ownerCode);
       status.credentials_enrolled=true;
       status.device_status='CONNECTED';
       status.adoption_state='READ_ONLY_CONNECTED_LAB';
       return {ok:true,json:async()=>({enrolled_for_read:true,
          production_adopted:false,physical_writes_enabled:false,
          adoption_state:'READ_ONLY_CONNECTED_LAB',card_count:3,
          verified_at_utc:'2026-09-29T12:00:00Z'})};
     }
     throw Error('Unexpected fetch: '+url);
   },Promise,Error,Number,JSON
 };
 vm.runInNewContext(source,context);
 for(let i=0;i<8;i++)await Promise.resolve();
 assert.equal(elements['ipat-c320-enroll-button'].disabled,false);
 await elements['ipat-c320-enroll-form'].listeners.submit({preventDefault(){}});
 if(autoVerify){
   elements['ipat-c320-bootstrap'].value='SYNTHETIC_OWNER_VERIFICATION_123456789012345';
   elements['ipat-c320-bootstrap'].listeners.change();
   assert.ok(elements['ipat-c320-enroll-form'].lastSubmit,'owner input did not resume pending SSH');
   await elements['ipat-c320-enroll-form'].lastSubmit;
 }
 return {elements,security,draft,enroll,network,events};
}
test('password with no hidden owner code persists draft, explains required step, never attempts SSH',async()=>{
 const out=await simulate({});
 assert.equal(out.draft,1);
 assert.equal(out.network,1);
 assert.equal(out.enroll,0);
 assert.equal(out.security.open,true);
 assert.equal(out.elements['ipat-c320-bootstrap'].focused,true);
 assert.equal(out.elements['ipat-c320-device-password'].value,'SYNTHETIC');
 assert.match(out.elements['ipat-c320-enroll-result'].textContent,/One-Time Owner Code/);
 assert.match(out.elements['ipat-device-connect-stage'].textContent,/One-time lab verification required/);
 assert.equal(out.elements['ipat-device-save-stage'].textContent,'Saved to Device List');
 assert.ok(out.events>=1);
});
test('full verified form persists draft first and then attempts exactly one device authentication',async()=>{
 const out=await simulate({ownerCode:'SYNTHETIC_OWNER_CODE'});
 assert.equal(out.draft,1);
 assert.equal(out.network,1);
 assert.equal(out.enroll,1);
 assert.equal(out.elements['ipat-c320-device-password'].value,'');
 assert.equal(out.elements['ipat-c320-bootstrap'].value,'');
});

test('pending saved device resumes connection automatically after owner code input',async()=>{
 const out=await simulate({autoVerify:true});
 assert.equal(out.draft,2);
 assert.equal(out.network,2);
 assert.equal(out.enroll,1);
 assert.equal(out.elements['ipat-c320-device-password'].value,'');
 assert.equal(out.elements['ipat-device-save-stage'].textContent,'Saved to Device List');
 assert.equal(out.elements['ipat-device-connect-stage'].textContent,'Connected · Read-only');
});
test('real saved device list stays above optional legacy diagnostics',()=>{
 const html=fs.readFileSync(path.resolve(__dirname,'../../../../web/lab/device-workbench.html'),'utf8');
 const list=fs.readFileSync(path.resolve(__dirname,'../../../../web/lab/device-workbench.js'),'utf8');
 assert.ok(html.indexOf('id="ipat-saved-physical-rows"')>0);
 assert.ok(html.indexOf('id="ipat-saved-physical-rows"')<html.indexOf('id="ipat-lab-diagnostics"'));
 assert.match(list,/node\('ipat-saved-physical-rows'\)\.replaceChildren\(actual\)/);
 assert.match(list,/if\(savedPhysical\)/);
});
