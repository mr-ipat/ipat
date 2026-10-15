'use strict';
// Verifies the actual pure production predicate rather than a duplicated
// implementation. No production environment, network or device is touched.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const root=path.resolve(__dirname,'../../../..');
const src=fs.readFileSync(path.join(root,'web/console/tenant/app.js'),'utf8');
const start=src.indexOf(' function savedDeviceInCurrentList(deviceId){');
const end=src.indexOf(' async function refreshDevices(){',start);
assert(start>=0 && end>start,'production visibility predicate required');
const body=src.slice(start,end);
assert.doesNotMatch(body,/fetch|localStorage|sessionStorage|innerHTML|name|display_name|management_host/);
function check(devices,id){
 const context={state:{devices}};
 vm.createContext(context);
 return vm.runInContext(body+';savedDeviceInCurrentList',context)(id);
}
const id='00000000-0000-4000-8000-000000000032';
assert.equal(check([],id),false,'missing row is not successful Save');
assert.equal(check([{id:'00000000-0000-4000-8000-000000000033',lifecycle_state:'SAVED'}],id),false,'wrong device ID denied');
assert.equal(check([{id,lifecycle_state:'ARCHIVED'}],id),false,'archived row denied');
assert.equal(check([{id,lifecycle_state:'SAVED'}],id),true,'exact saved row accepted');
assert.equal(check([{id,lifecycle_state:'SAVED'}],null),false,'null ID denied');
assert.equal(check([{id,lifecycle_state:'SAVED'}],42),false,'numeric ID denied');
const saveStart=src.indexOf("  const saved=await api('/api/v1/devices','POST',d);");
const saveEnd=src.indexOf(" $('device-edit').addEventListener",saveStart);
assert(saveStart>0&&saveEnd>saveStart);
const flow=src.slice(saveStart,saveEnd);
const iList=flow.indexOf('await refreshDevices()');
const iVisibility=flow.indexOf('!savedDeviceInCurrentList(saved.id)');
const iClear=flow.indexOf('state.pendingDevice=null;clearForm(f)');
assert(iList>=0&&iVisibility>iList&&iClear>iVisibility,'read-after-write check must precede form/key reset');
assert.match(flow,/retry unchanged Save with the same request ID/);
assert.match(flow,/its exact ID verified in the inventory list/);
assert.doesNotMatch(flow,/fetch\(['"]https?:/);
console.log('R1032_EXACT_ID_SAVED_LIFECYCLE_READ_AFTER_WRITE_VM=PASS');
