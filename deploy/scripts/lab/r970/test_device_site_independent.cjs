'use strict';
// Read-only UI contract: Device registration must select a Site, not a POP.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const page=fs.readFileSync(path.resolve(__dirname,'../../../../web/console/tenant/dashboard.html'),'utf8');
const device=page.split('<section data-panel="devices" hidden>')[1].split('<section data-panel="branding" hidden>')[0];
assert.match(device, /<label>Site <select name="pop_id" id="device-site" required>/);
assert.match(device, /id="create-site-link">Create or manage Sites<\/a>/);
assert.match(device, /POPs are independent and cannot be entered as Sites/);
assert.doesNotMatch(device, /<label>Site \/ POP /);
console.log('R1027_DEVICE_REQUIRES_EXISTING_SITE_NOT_POP=PASS');
