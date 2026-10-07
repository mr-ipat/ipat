'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');
const html=fs.readFileSync('web/console/tenant/dashboard.html','utf8');
const js=fs.readFileSync('web/console/tenant/app.js','utf8');
const api=fs.readFileSync('apps/control-api/src/commercial_tenant_api.rs','utf8');
const sql=fs.readFileSync('deploy/db/migrations/0037_pppoe_batch_dry_run.sql','utf8');
assert(html.includes('data-page="pppoe" hidden'));
assert(html.includes('audited CSV plan'));
assert(html.includes('subscriber_id,action,username,profile,secret_ref'));
assert(html.includes('Never paste a PPPoE password'));
assert(js.includes('parseStrictCsvRow'));
assert(js.includes('canManagePppoePlans'));
assert(js.includes('physical execution remains disabled'));
assert(js.includes('secret reference present'));
assert(!js.includes("'/api/v1/pppoe-execute"));
assert(!api.includes('/api/v1/pppoe-execute'));
assert(api.includes('"physical_execution_available":false'));
assert(sql.includes('CHECK(execution_allowed=false)'));
assert(sql.includes('There is intentionally NO claim/lease/execute/router-write function'));
assert(!html.match(/>Execute</));
const begin=js.indexOf(' function parseStrictCsvRow');
const end=js.indexOf(' function refreshPppoeRouterChoices',begin);
assert(begin>=0&&end>begin);
const sandbox={Error};
vm.createContext(sandbox);
vm.runInContext(js.slice(begin,end)+';this.parsePppoeOperations=parsePppoeOperations;',sandbox);
const tenant='a1070000-0000-4000-8000-000000000001';
const good='subscriber_id,action,username,profile,secret_ref\n'
 +'SUB-1,update,"customer,one",default,vault://tenant/'+tenant+'/pppoe/SUB-1\n'
 +'SUB-2,disable,,,';
const parsed=sandbox.parsePppoeOperations(good);
assert.equal(parsed.length,2);
assert.equal(parsed[0].username,'customer,one');
assert.equal(parsed[1].secret_ref,null);
const bad=[
 'SUB-1,update,user,default,vault://tenant/x/pppoe/SUB-1',
 'subscriber_id,action,username,profile,secret_ref\nSUB-1,disable,user,,',
 'subscriber_id,action,username,profile,secret_ref\nSUB-1,update,user,default,password',
 'subscriber_id,action,username,profile,secret_ref\nSUB-1,update,"broken,default,vault://tenant/x/pppoe/SUB-1',
 'subscriber_id,action,username,profile,secret_ref,extra\nSUB-1,disable,,,,'
];
for(const sample of bad)assert.throws(()=>sandbox.parsePppoeOperations(sample));
console.log('R1007_PPPOE_STRICT_CSV_DRY_RUN_UI_NO_EXECUTION_CONTRACT=PASS');
