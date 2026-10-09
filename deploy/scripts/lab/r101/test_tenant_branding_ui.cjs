'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const html=fs.readFileSync('web/console/tenant/dashboard.html','utf8');
const js=fs.readFileSync('web/console/tenant/app.js','utf8');
const css=fs.readFileSync('web/console/tenant/style.css','utf8');
const api=fs.readFileSync('apps/control-api/src/commercial_tenant_api.rs','utf8');
const migration=fs.readFileSync('deploy/db/migrations/0042_tenant_safe_branding.sql','utf8');
assert(html.includes('data-page="branding" hidden'));
assert(html.includes('id="brand-display-name"'));
assert(html.includes('id="brand-mark-text"'));
assert(html.includes('id="brand-accent"'));
assert(!html.includes('logo_url'));
assert(!html.includes('type="color"'));
assert(js.includes("api('/api/v1/branding'"));
assert(js.includes("api('/api/v1/branding','PATCH'"));
assert(js.includes("$('brand-name').textContent=b.display_name"));
assert(js.includes("$('brand-mark').textContent=b.mark_text"));
assert(js.includes('document.body.dataset.accent=b.accent_token'));
assert(!js.includes("$('brand-name').innerHTML"));
assert(!js.includes("$('brand-mark').innerHTML"));
assert(js.includes("if(r.can_manage_branding)$('nav-branding').hidden=false"));
assert(js.includes('pendingBranding'));
assert(js.includes('request_id:crypto.randomUUID()'));
for(const token of ['slate','blue','indigo','emerald','amber','rose']){
  assert(css.includes('body[data-accent="'+token+'"]'),token);
  assert(migration.includes("'"+token+"'"),token);
}
assert(api.includes('get_tenant_branding_for_member'));
assert(api.includes('set_tenant_branding'));
assert(api.includes('"can_manage_branding":admin'));
assert(api.includes('valid_accent_token'));
assert(migration.includes("m.role='tenant_admin'"));
assert(migration.includes("m.role IN ('tenant_admin','system_admin'"));
for(const role of ['tenant_admin','system_admin','security_admin','noc_manager',
                   'noc_engineer','provisioning_officer','helpdesk','field_technician','auditor']){
  assert(migration.includes("'"+role+"'"),role);
}
assert(migration.includes('tenant_branding_events'));
assert(migration.includes('pg_advisory_xact_lock'));
console.log('R101_TENANT_SAFE_BRANDING_UI_CONTRACT=PASS');
