'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const root=path.resolve(__dirname,'../../../..');
const platform=fs.readFileSync(path.join(root,'web/console/platform/app.js'),'utf8');
const tenant=fs.readFileSync(path.join(root,'web/console/tenant/app.js'),'utf8');
const platformApi=fs.readFileSync(path.join(root,'apps/control-api/src/platform_owner_api.rs'),'utf8');
const tenantApi=fs.readFileSync(path.join(root,'apps/control-api/src/commercial_tenant_api.rs'),'utf8');
for(const [name,script] of [['Platform',platform],['Tenant',tenant]]){
 assert.match(script,/if\(!confirm\(/,name+' review cancel protection');
 assert.match(script,/reviewButtons\.forEach\(button=>button\.disabled=true\)/,name+' double-click protection');
 assert.match(script,/finally\{reviewButtons\.forEach\(button=>button\.disabled=false\)\}/,name+' recovery');
 assert.match(script,/setAttribute\('aria-label'/,name+' accessible action labels');
 assert.doesNotMatch(script,/innerHTML\s*=/);
}
assert.match(platform,/Two distinct current Platform Owners are required/);
assert.match(platform,/Approval does NOT activate DNS, TLS or customer login/);
assert.match(tenant,/No device execution is authorized/);
assert.match(tenant,/distinct System Admin and fresh physical readiness/);
assert.match(platformApi,/review/);
assert.match(tenantApi,/pppoe-plans/);
console.log('R1031_CONFIRMATION_MAKER_CHECKER_NO_DOUBLE_CLICK_UI_CONTRACT=PASS');
