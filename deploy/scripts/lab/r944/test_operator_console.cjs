// Node built-in regression: operator GUI must use existing protected read routes
// and may never expose arbitrary physical CLI, device writes, or fake live status.
'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const base=path.resolve(__dirname,'..','..','..','..');
const html=fs.readFileSync(path.join(base,'web/lab/device-workbench.html'),'utf8');
const js=fs.readFileSync(path.join(base,'web/lab/c320-operator-console.js'),'utf8');
const api=fs.readFileSync(path.join(base,'apps/control-api/src/device_workbench_lab.rs'),'utf8');
const physical=fs.readFileSync(path.join(base,'apps/control-api/src/c320_live_lab.rs'),'utf8');
const snapshot=fs.readFileSync(path.join(base,'apps/control-api/src/c320_actions_lab.rs'),'utf8');

test('operational console is integrated with an actually served lab module',()=>{
  assert.match(html, /id="ipat-c320-console"/);
  assert.match(html, /script src="\/lab\/c320-operator-console\.js" defer/);
  assert.match(api,/C320_OPERATOR_JS: &str = include_str!/);
  assert.match(api,/\.route\("\/lab\/c320-operator-console\.js", get\(operator_console_js\)\)/);
  for(const id of ['configured','online','offline','unconfigured','cards','firmware',
    'connectivity','observation-stamp','console-status','write-guard']){
    assert.match(html,new RegExp('id="ipat-c320-'+id+'"'));
  }
});
test('GUI only enables previously implemented strictly scoped physical reads',()=>{
  for(const route of ['c320-owner-connection','c320-owner-live-refresh',
                      'c320-owner-live-cards','c320-owner-live-firmware']){
    assert.match(js,new RegExp('/lab/'+route));
  }
  for(const route of ['c320-owner-agent-state','c320-owner-live-refresh',
                      'c320-owner-live-cards','c320-owner-live-firmware']){
    assert.match(physical,new RegExp('/lab/'+route));
  }
  assert.match(html, /<option value="other" disabled>/);
  assert.doesNotMatch(js, /show (?:run|card)|conf(?:igure)?\s+t|vlan\s+\d+/i);
  assert.doesNotMatch(js, /localStorage|sessionStorage|innerHTML|eval\(/);
});
test('manual snapshot is not falsely presented as live evidence',()=>{
  assert.match(js,/snapshot_is_live !== false/);
  assert.match(js,/owner-connection/);
  assert.match(js,/\!state.ready/);
  assert.match(js,/source !== 'VERIFIED_LOCAL_OWNER_AGENT_LAB_ONLY'/);
  assert.match(html,/TIDAK/);
});
test('dated 72-ONU table has no serial numbers or fabricated live per-ONU state',()=>{
  assert.match(html,/id="ipat-c320-onu-rows"/);
  assert.match(html,/id="ipat-c320-onu-search"/);
  assert.match(js,/v\.manual_onu_ids/);
  assert.match(js,/manual_onu_rows_are_live === false/);
  assert.match(js,/Pembacaan total secara live TIDAK memperbarui status per baris/);
  assert.match(snapshot,/"manual_onu_ids":\[/);
  assert.match(snapshot,/"manual_onu_rows_are_live":false/);
  assert.match(snapshot,/"serial_numbers_disclosed":false/);
  assert.doesNotMatch(js,/ZTEGC[0-9A-F]{8}/);
});

test('one-time browser connection enrollment is wired to server-side fixed SSH adapter',()=>{
  const connect=fs.readFileSync(path.join(base,'web/lab/c320-connection-setup.js'),'utf8');
  const daemon=fs.readFileSync(path.join(base,'deploy/scripts/lab/r945/persistent_c320_connector.py'),'utf8');
  assert.match(html,/id="ipat-c320-enroll-form"/);
  assert.match(html,/script src="\/lab\/c320-connection-setup\.js" defer/);
  assert.match(api,/C320_CONNECT_JS: &str = include_str!/);
  assert.match(api,/\.route\("\/lab\/c320-connection-setup\.js", get\(connection_setup_js\)\)/);
  assert.match(connect,/\/lab\/c320-owner-enroll/);
  assert.match(connect,/\/lab\/c320-owner-connection/);
  assert.doesNotMatch(connect,/localStorage|sessionStorage|innerHTML/);
  assert.match(daemon,/Fernet\.generate_key\(\)/);
  assert.match(daemon,/cryptography\.fernet/);
  assert.match(daemon,/TOKEN_HASH/);
  assert.match(daemon,/socket\.SO_PEERCRED/);
  assert.doesNotMatch(daemon,/StrictHostKeyChecking=no|sshpass/);
  assert.match(html,/<details class="ipat-lab-diagnostics"/);
});
