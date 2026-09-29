// R9.47 Add Device acceptance contract: no faux multi-vendor adoption.
'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const base=path.resolve(__dirname,'../../../..');
const html=fs.readFileSync(path.join(base,'web/lab/device-workbench.html'),'utf8');
const ui=fs.readFileSync(path.join(base,'web/lab/c320-connection-setup.js'),'utf8');
const api=fs.readFileSync(path.join(base,'apps/control-api/src/c320_live_lab.rs'),'utf8');
const css=fs.readFileSync(path.join(base,'web/lab/device-workbench.css'),'utf8');
test('Add Device exposes coherent international network-management fields',()=>{
 for(const id of ['ipat-device-name','ipat-device-type','ipat-device-model','ipat-device-protocol','ipat-device-host','ipat-device-port','ipat-device-username','ipat-c320-device-password']){
   assert.match(html,new RegExp('id="'+id+'"'));
   assert.match(ui,new RegExp("'"+id+"'"));
 }
 for(const label of ['Add Device','Device Name','Device Type','Vendor / Model',
   'Management Protocol','Management IP','SSH Port','Username','Password','Connect &amp; Save'])
   assert.ok(html.includes(label),label);
 assert.match(css,/\.ipat-device-form-grid\{/);
 assert.match(css,/@media\(max-width:700px\)/);
});
test('unsupported vendor and network targets must never accidentally enroll',()=>{
 for(const type of ['olt','ont','router','other'])assert.match(ui,new RegExp(type+':\\['));
 assert.match(ui,/exactProfile/);
 assert.match(ui,/\!available \|\| \!defaultC320/);
 assert.match(ui,/Adapter not available yet/);
 assert.match(api,/input\.device_type != "olt"/);
 assert.match(api,/input\.device_name\.len\(\) > 64/);
 assert.match(api,/input\.ssh_port != 321/);
 assert.match(api,/input\.management_ip !=/);
 assert.match(api,/TARGET_NOT_ALLOWLISTED/);
 assert.doesNotMatch(ui,/innerHTML|localStorage|sessionStorage|eval\(/);
});
test('one-time owner verification remains separate from main operator form',()=>{
 assert.match(html,/<details class="ipat-device-security">/);
 assert.match(html,/One-Time Owner Code/);
 assert.match(ui,/bootstrap_code:bootstrap/);
 assert.match(ui,/password\.value='';owner\.value='';/);
 assert.match(ui,/verified\.enrolled_for_read!==true/);
});
