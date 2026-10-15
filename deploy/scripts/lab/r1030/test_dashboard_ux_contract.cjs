'use strict';
const fs=require('node:fs');
const assert=require('node:assert/strict');
const path=require('node:path');
const root=path.resolve(__dirname,'../../../..');
const asset=(p)=>fs.readFileSync(path.join(root,p),'utf8');
const tenant=asset('web/console/tenant/dashboard.html');
const tenantCss=asset('web/console/tenant/style.css');
const tenantJs=asset('web/console/tenant/app.js');
const platform=asset('web/console/platform/dashboard.html');
const platformCss=asset('web/console/platform/style.css');
const api=asset('apps/control-api/src/commercial_tenant_api.rs');
const db=asset('deploy/db/migrations/0020_managed_device_metadata_crud.sql');
for(const [name,html,css] of [
 ['Tenant',tenant,tenantCss],['Platform',platform,platformCss],
]){
 assert.match(html, /class="skip-link" href="#main-content"/,name+' skip link');
 if(name==='Tenant'){
  assert.match(html, /<section class="content" id="main-content" tabindex="-1"/,name+' bypass nav');
  assert.match(html, /<h1>Company operations<\/h1>/);
} else {
  assert.match(html, /<main id="main-content" tabindex="-1"/,name+' skip destination');
}
 assert.match(css, /:focus-visible/,name+' keyboard focus');
 assert.match(css, /prefers-reduced-motion:reduce/,name+' reduced motion');
 assert.match(css, /@media\(max-width:/,name+' responsive rules');
}
assert.match(platform, /aria-label="Reserved companies"/);
assert.match(platform, /aria-label="Activation queue"/);
assert.match(tenant, /id="device-site-note" role="status" aria-live="polite"/);
assert.match(tenant, /id="create-site-link">Create or manage Sites/);
assert.match(tenantJs, /state\.pendingDevice=\{fingerprint,request_id:crypto\.randomUUID\(\)\}/);
assert.match(tenantJs, /d\.request_id=state\.pendingDevice\.request_id/);
assert.match(tenantJs, /await refreshDevices\(\)/);
assert.match(tenantJs, /!savedDeviceInCurrentList\(saved\.id\)/);
assert.match(tenantJs, /state\.pendingDevice=null;clearForm/);
assert.match(tenantJs, /retry unchanged Save with the same request ID/);
assert.match(tenantJs, /state\.returnToDevicesAfterSite=true/);
assert.match(tenantJs, /await page\('devices'\)/);
assert.match(tenantJs, /heading\.focus\(\{preventScroll:true\}\)/);
assert.match(tenantJs, /setAttribute\('aria-current','page'\)/);
assert.match(tenantJs, /removeAttribute\('aria-current'\)/);
assert.match(tenantJs, /if\(state\.pageGeneration===generation\)/);
assert.match(tenantJs, /A distinct Security Admin must review it/);
assert.doesNotMatch(tenantJs,/A different Tenant Admin must approve it; physical execution remains disabled/);
assert.match(api,/register_managed_device\(/);
assert.match(db,/pg_advisory_xact_lock/);
assert.match(db,/WHERE tenant_id=p_tenant AND request_id=p_request_id/);
assert.doesNotMatch(tenantJs,/\.innerHTML\s*=/);
for(const id of ['pops','sites','devices','domains','pppoe']){
 assert(tenant.includes('data-page="'+id+'" hidden'),'deny-default nav: '+id);
}
console.log('R1030_TENANT_PLATFORM_WCAG_FLOW_IDEMPOTENCY_ROLE_CONTRACT=PASS');
