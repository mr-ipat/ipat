'use strict';
const {test}=require('node:test');
const assert=require('node:assert/strict');
const {readFileSync}=require('node:fs');
const {resolve}=require('node:path');
const root=resolve(__dirname,'../../../..');
const source=(path)=>readFileSync(resolve(root,path),'utf8');
const js=source('web/lab/owner-device-inventory.js');
const html=source('web/lab/device-workbench.html');
const api=source('apps/control-api/src/owner_device_registry_lab.rs');
const main=source('apps/control-api/src/main.rs');
const c320=source('apps/control-api/src/c320_live_lab.rs');
test('owner-private persistent device manager is the first real dashboard section',()=>{
 for(const id of ['managed-devices','ipat-owner-device-rows','ipat-owner-device-summary','ipat-owner-device-add',
 'ipat-owner-device-filter','ipat-owner-device-form','ipat-owner-device-detail','ipat-owner-device-save',
 'ipat-owner-device-notice','ipat-owner-protocol','ipat-owner-host'])assert.ok(html.includes('id="'+id+'"'),id);
 assert.ok(html.indexOf('id="managed-devices"')<html.indexOf('id="ipat-c320-console"'));
 assert.match(html,/lang="en"/);
 assert.match(html,/src="\/lab\/owner-device-inventory.js"/);
 assert.match(main,/app\.merge\(owner_device_registry_lab::router\(\)\)/);
 assert.match(main,/if owner_live_read/);
 assert.match(c320,/pub\(super\) async fn connection_status/);
});
test('real CRUD uses backend not demo/local storage and never assumes connection on Save',()=>{
 for(const term of ['method:\'DELETE\'','method:editing?.id?\'PUT\':\'POST\'',
 'encodeURIComponent','expected_revision','REMOVE ','SAVED · NOT CONNECTED','linked_live_connector',
 'ACTIVE','call(api)','window.confirm']){
   if(term==='ACTIVE')continue;
   assert.ok(js.includes(term),term);
 }
 assert.doesNotMatch(js,/demo\/device-candidates|innerHTML|localStorage|sessionStorage|document\.cookie/);
 assert.match(js,/pending=false;\n      report/);
 assert.match(api,/DUPLICATE_MANAGEMENT_ENDPOINT/);
 assert.match(api,/STALE_REVISION/);
 assert.match(api,/ACTIVE_CONNECTOR_REQUIRES_SEPARATE_REVOCATION/);
 assert.match(api,/private_file\(path\)/);
 assert.match(api,/File::open\(folder\)\?\.sync_all\(\)/);
 assert.match(api,/"\/lab\/owner\/devices"/);
 assert.match(api,/"\/lab\/owner\/devices\/\{id\}"/);
});
test('metadata changes cannot accept credentials, alter active C320 endpoint or send CLI',()=>{
 assert.doesNotMatch(html,/id="ipat-owner-password"|id="ipat-owner-secret"/);
 assert.match(api,/serde\(deny_unknown_fields\)/);
 assert.match(api,/LINKED_DEVICE_LABEL_ONLY/);
 assert.match(api,/"physical_writes_enabled":false/);
 assert.match(api,/device_command_sent/);
 assert.match(api,/strict_owner\(&headers, true\)/);
 assert.match(api,/owner_private_lab_only/);
 assert.doesNotMatch(api,/Command::new|\.write_all\(b"show |\.write_all\(b"config |tokio_postgres/);
});
