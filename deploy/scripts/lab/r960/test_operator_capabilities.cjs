'use strict';
const {test} = require('node:test');
const assert = require('node:assert/strict');
const {readFileSync} = require('node:fs');
const {resolve} = require('node:path');
const root = resolve(__dirname,'../../../..');
const src = name => readFileSync(resolve(root,name),'utf8');
const html = src('web/lab/device-workbench.html');
const js = src('web/lab/c320-action-catalog.js');
const api = src('apps/control-api/src/c320_live_lab.rs');
const server = src('apps/control-api/src/device_workbench_lab.rs');
test('operational dashboard mounts English catalog backed by real owner-private API',()=>{
 for (const id of ['c320-capabilities','ipat-capability-heading','ipat-c320-capabilities-rows',
   'ipat-c320-capabilities-state','ipat-c320-capability-detail','ipat-c320-refresh-capabilities'])
   assert.ok(html.includes('id="'+id+'"'),id);
 assert.match(html,/src="\/lab\/c320-action-catalog.js"/);
 assert.match(server,/"\/lab\/c320-action-catalog.js"/);
 assert.match(api,/"\/lab\/c320-owner-action-catalog"/);
 assert.match(api,/connection_status\(headers\)\.await\?/);
});
test('fail-closed approved reads cannot be edited via server-provided endpoints',()=>{
 assert.match(js,/Object\.freeze\(\{/);
 assert.match(js,/cards: '\/lab\/c320-owner-live-cards'/);
 assert.match(js,/firmware: '\/lab\/c320-owner-live-firmware'/);
 assert.doesNotMatch(js,/\/lab\/c320-owner-live-refresh'/);
 assert.match(js,/catalog\.connected && capability\.state === 'AVAILABLE_READ_ONLY'/);
 assert.match(js,/capability\.endpoint === endpoints\[capability\.id\]/);
 assert.match(js,/v\.physical_writes_enabled !== false/);
 assert.match(js,/button\.disabled = !executable/);
 assert.doesNotMatch(js,/eval\(|innerHTML|localStorage|sessionStorage/);
});
test('writes remain visible in private owner lab only, non-dispatchable by construction',()=>{
 for (const name of ['onu_provision','onu_deprovision','vlan_service','ont_cwmp','ont_usp',
   'reboot','config_backup','config_restore','firmware_upgrade','alarms','optical_levels','traffic'])
    assert.ok(api.includes('"id":"'+name+'"'),name);
 assert.match(api,/owner_catalog_response/);
 assert.match(api,/APPROVAL_AND_DRIVER_REQUIRED/);
 assert.match(api,/"state":"DEGRADED"/);
 assert.match(api,/fn owner_catalog_never_authorizes_an_unqualified_command/);
});
