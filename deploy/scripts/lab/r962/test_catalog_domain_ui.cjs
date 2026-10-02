'use strict';
const {test}=require('node:test');
const assert=require('node:assert/strict');
const {readFileSync}=require('node:fs');
const {resolve}=require('node:path');
const root=resolve(__dirname,'../../../..');
const source=path=>readFileSync(resolve(root,path),'utf8');
const form=source('web/lab/device-workbench.html');
const manager=source('web/lab/owner-device-inventory.js');
const registry=source('apps/control-api/src/owner_device_registry_lab.rs');
const page=source('web/lab/platform-domain-admin.html');
const admin=source('web/lab/platform-domain-admin.js');
const backend=source('apps/control-api/src/owner_domain_control_lab.rs');
const main=source('apps/control-api/src/main.rs');
test('Device Manager form gets POP and vendor from backed master catalogs',()=>{
 assert.match(form,/<select id="ipat-owner-pop" required>/);
 assert.match(form,/<select id="ipat-owner-vendor" required>/);
 assert.doesNotMatch(form,/<input id="ipat-owner-(pop|vendor)"/);
 assert.ok(form.includes('id="ipat-owner-pop-inline"'));
 assert.doesNotMatch(form,/id="ipat-owner-pop-form"|id="ipat-owner-pop-rows"/);
 assert.ok(manager.includes("call('/lab/owner/pops'),call('/lab/owner/device-catalog')"));
 assert.ok(manager.includes("window.location.assign('/lab/sites?return=device')"));
 assert.match(manager,/populateVendors\(d\.vendor,d\.management_protocol\)/);
 assert.match(registry,/POP_MUST_BE_REGISTERED/);
 assert.match(registry,/fn permitted_vendor/);
 assert.match(registry,/"PARTIAL_EXACT_PILOT"/);
 assert.match(registry,/"INVENTORY_CANDIDATE"/);
 assert.match(registry,/linked_c320_pop/);
});
test('Owner Admin stages domain settings without asserting public readiness',()=>{
 for(const id of ['ipat-domain-profile-form','ipat-domain-platform-host','ipat-domain-public-ip',
 'ipat-domain-request-form','ipat-domain-tenant-ref','ipat-domain-kind','ipat-domain-draft-rows'])
   assert.ok(page.includes('id="'+id+'"'),id);
 assert.match(page,/<html lang="en-US">/);
 assert.match(page,/PRD PRODUCTION BLOCKER/);
 assert.match(admin,/STAGE_DOMAIN_CONFIG/);
 assert.match(admin,/PLANNED · NOT ROUTED/);
 assert.match(admin,/ownership_txt_value!==null/);
 assert.doesNotMatch(admin,/innerHTML|localStorage|sessionStorage|document\.cookie|eval\(/);
 assert.match(backend,/safe_to_point_now.*false/);
 assert.match(backend,/"public_https_ready":false/);
 assert.match(backend,/"runtime_changes_applied":false/);
 assert.match(backend,/OWNER_TUNNEL_REQUIRED/);
 assert.match(backend,/"ipat\.fadly\.id"/);
 assert.match(backend,/super::owner_device_registry_lab::private_file/);
 assert.match(main,/app\.merge\(owner_domain_control_lab::router\(\)\)/);
});
test('Backend routes do not contain device CLI dispatch',()=>{
 for(const route of ['/lab/owner/pops','/lab/owner/device-catalog'])assert.ok(registry.includes(route));
 for(const route of ['/lab/platform-admin/domains','/lab/owner/platform-domains','/lab/owner/platform-domains/drafts'])
    assert.ok(backend.includes(route));
 assert.doesNotMatch(backend,/Command::new|\.write_all\(b"show /);
});
