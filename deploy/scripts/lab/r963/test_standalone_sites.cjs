'use strict';
const {test}=require('node:test');const assert=require('node:assert/strict');
const {readFileSync}=require('node:fs');const {resolve}=require('node:path');
const root=resolve(__dirname,'../../../..');const source=p=>readFileSync(resolve(root,p),'utf8');
const workbench=source('web/lab/device-workbench.html');
const deviceJs=source('web/lab/owner-device-inventory.js');
const sites=source('web/lab/site-manager.html');
const sitesJs=source('web/lab/site-manager.js');
const siteApi=source('apps/control-api/src/owner_device_registry_lab.rs');
const siteUi=source('apps/control-api/src/owner_site_ui_lab.rs');
const main=source('apps/control-api/src/main.rs');
test('Sites/POPs have own route, navigation, separate HTML and JavaScript',()=>{
 assert.ok(main.includes('app.merge(owner_site_ui_lab::router())'));
 for(const route of ['/lab/sites','/lab/sites.js','/lab/sites.css'])assert.ok(siteUi.includes('"'+route+'"'));
 for(const field of ['ipat-sites-rows','ipat-sites-new','ipat-sites-form','ipat-sites-code',
   'ipat-sites-name','ipat-sites-filter','ipat-sites-refresh','ipat-sites-form-panel'])
    assert.ok(sites.includes('id="'+field+'"'),field);
 assert.ok(workbench.includes('href="/lab/sites"'));
 assert.doesNotMatch(workbench,/id="ipat-owner-pop-(form|rows|create)"/);
 assert.doesNotMatch(deviceJs,/function (showPopForm|submitPop|removePop|renderPops)/);
 assert.doesNotMatch(sitesJs,/fetch\(['"]\/lab\/owner\/devices|\/lab\/device-candidates/);
 assert.doesNotMatch(sitesJs,/localStorage|sessionStorage|innerHTML|document\.cookie/);
 assert.ok(sitesJs.includes('assigned_device_count'));
 assert.ok(sitesJs.includes('return')&&sitesJs.includes('window.location.assign(optionReturn)'));
});
test('Backend site resource exposes assignment counts, guarded deletion and controlled live-site unassign',()=>{
 for(const item of ['site_view','detail_pop','assigned_device_count','linked_c320_assigned',
   'POP_HAS_ASSIGNED_DEVICES','AMBIGUOUS_POP_ASSIGNMENT','clear_pop'])
   assert.ok(siteApi.includes(item),item);
 assert.ok(siteApi.includes('get(detail_pop).put(update_pop).delete(remove_pop)'));
 assert.ok(deviceJs.includes('clear_pop:true'));
 assert.ok(deviceJs.includes('function toggleExtraControls(linked)'));
 assert.ok(deviceJs.includes('input.disabled=linked'));
 assert.ok(deviceJs.includes("field('pop').disabled=false"));
 assert.ok(deviceJs.includes('toggleExtraControls(false)'));
 assert.ok(deviceJs.includes("window.location.assign('/lab/sites?return=device')"));
});
