'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const base=path.resolve(__dirname,'../../../..');
const code=fs.readFileSync(path.join(base,'web/lab/device-status-indicators.js'),'utf8');
const html=fs.readFileSync(path.join(base,'web/lab/device-workbench.html'),'utf8');
const js=fs.readFileSync(path.join(base,'web/lab/device-workbench.js'),'utf8');
const css=fs.readFileSync(path.join(base,'web/lab/device-workbench.css'),'utf8');
const rust=fs.readFileSync(path.join(base,'apps/control-api/src/c320_live_lab.rs'),'utf8');
const daemon=fs.readFileSync(path.join(base,'deploy/scripts/lab/r945/persistent_c320_connector.py'),'utf8');
function statusResult(device_status,options={}) {
  return {
    target:'DEV-01',vendor:'ZTE',model:'C320',
    connector_online:true,credentials_enrolled:true,production_adopted:false,
    physical_writes_enabled:false,device_status,
    last_verified_at_utc:new Date().toISOString(),...options
  };
}
async function runCase(body) {
  const badges=[{textContent:'',className:'',setAttribute(){},title:''},
                {textContent:'',className:'',setAttribute(){},title:''}];
  const detail={textContent:''};
  let fetchCalls=0;
  const controller={signal:{},abort(){}};
  const context={
    document:{
      querySelectorAll(query) {
        assert.equal(query,'[data-ipat-device-status="DEV-01"]');
        return badges;
      },
      getElementById(id){return id==='ipat-c320-signal-detail'?detail:null}
    },
    window:{addEventListener(){}},
    fetch:async()=>{fetchCalls++;return {ok:true,json:async()=>body}},
    AbortController:class {constructor(){return controller}},
    setTimeout:()=>1,clearTimeout(){},setInterval:()=>1,
    Date,Number,Error,Promise
  };
  vm.runInNewContext(code,context,{filename:'device-status-indicators.js'});
  for(let i=0;i<8;i++) await Promise.resolve();
  assert.equal(fetchCalls,1);
  return {badges,detail};
}
test('connected is green only with an actual recent verified time',async()=>{
  const {badges}=await runCase(statusResult('CONNECTED'));
  assert.match(badges[0].className,/--connected$/);
  assert.equal(badges[0].className,badges[1].className);
});
test('disconnected indicates known earlier physical connection loss',async()=>{
  const {badges}=await runCase(statusResult('DISCONNECTED',{last_verified_at_utc:''}));
  assert.match(badges[0].className,/--disconnected$/);
});
test('pending on initial enrollment is amber',async()=>{
  const {badges}=await runCase(statusResult('PENDING',{
    credentials_enrolled:false,last_verified_at_utc:''
  }));
  assert.match(badges[0].className,/--pending$/);
});
test('unknown remains grey for unavailable connector',async()=>{
  const {badges}=await runCase(statusResult('UNKNOWN',{
    connector_online:false,credentials_enrolled:false,last_verified_at_utc:''
  }));
  assert.match(badges[0].className,/--unknown$/);
});
test('stale timestamp can never produce a green status',async()=>{
  const stale=new Date(Date.now()-700000).toISOString();
  const {badges}=await runCase(statusResult('CONNECTED',{last_verified_at_utc:stale}));
  assert.match(badges[0].className,/--unknown$/);
});
test('existing C320 list and panel actually include same dynamic status',()=>{
  assert.match(html,/data-ipat-device-status="DEV-01"/);
  assert.match(html,/script src="\/lab\/device-status-indicators\.js"/);
  assert.match(js,/data-ipat-device-status/);
  assert.match(js,/ipat-device-list-rendered/);
  for(const key of ['connected','disconnected','pending','unknown']) {
    assert.match(css,new RegExp('\\.ipat-device-signal--'+key+'\\{'));
  }
  assert.match(rust,/"device_status":status/);
  assert.match(rust,/actual_olt_connectivity_verified/);
  assert.match(rust,/if physically_fresh \{"READ_ONLY_CONNECTED_LAB"\}/);
  assert.match(daemon,/ever_verified_since_start/);
});
