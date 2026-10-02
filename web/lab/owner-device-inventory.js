'use strict';
/* R9.61: real, persistent owner-VPS metadata CRUD + real C320 live connection.
 * No cookies, passwords, arbitrary network commands, client-side fake states
 * or browser-persisted mock rows. Production tenant registry is separate. */
(() => {
  const el = id => document.getElementById(id);
  const rows = el('ipat-owner-device-rows');
  if (!rows) return;
  const summary = el('ipat-owner-device-summary');
  const notice = el('ipat-owner-device-notice');
  const detail = el('ipat-owner-device-detail');
  const form = el('ipat-owner-device-form');
  const filter = el('ipat-owner-device-filter');
  const protocol = el('ipat-owner-protocol');
  let items = [];
  let pops = [];
  let vendorCatalog = [];
  let mastersReady = false;
  let editing = null;
  let pending = false;
  const api = '/lab/owner/devices';
  function report(message) { notice.textContent = message; }
  function field(id) { return el('ipat-owner-' + id); }
  function td(tr, value) {
    const node = document.createElement('td');
    node.textContent = String(value);
    tr.append(node);
    return node;
  }
  function button(parent, title, handler, disabled=false) {
    const node = document.createElement('button');
    node.type = 'button'; node.className = 'secondary';
    node.textContent = title; node.disabled = disabled;
    if (!disabled) node.addEventListener('click', handler);
    parent.append(node);
    return node;
  }
  async function call(path, options={}) {
    const abort = new AbortController();
    const timeout = setTimeout(() => abort.abort(), 8000);
    try {
      const response = await fetch(path, {
        credentials: 'omit', cache: 'no-store', ...options,
        headers: options.method ? {
          'Content-Type':'application/json','X-IPAT-Owner-Private':'1'
        } : {}, signal:abort.signal
      });
      const result = await response.json();
      if (!response.ok) throw Error(result.error || 'HTTP ' + response.status);
      return result;
    } finally { clearTimeout(timeout); }
  }
  function option(value, label){
    const o=document.createElement('option');o.value=value;o.textContent=label;return o;
  }
  function populatePops(wanted=''){
    const menu=field('pop');menu.replaceChildren(option('','Select a registered POP / Site'));
    for(const p of pops)menu.append(option(p.code,p.display_name+' ('+p.code+')'));
    if(pops.some(p=>p.code===wanted))menu.value=wanted;
    el('ipat-owner-pop-empty-hint').textContent=pops.length
      ?'Choose an existing registered Site/POP. Manage sites in the separate Sites & POPs module.'
      :'No Sites/POPs registered. Create your first site in the independent Sites & POPs module.';
  }
  function compatibleKinds(){
    return vendorCatalog.flatMap(v=>v.kinds.map(k=>({vendor:v.id,name:v.name,...k})));
  }
  function populateProtocols(wanted=''){
    const allowed=compatibleKinds().find(k=>k.vendor===field('vendor').value && k.kind===field('kind').value);
    protocol.replaceChildren(option('','Select a management protocol'));
    for(const id of allowed?.protocols || []){
      const name=({ssh:'SSH',snmp:'SNMP', 'api-ssl':'RouterOS API-SSL',
        https:'HTTPS',cwmp:'TR-069/CWMP',usp:'TR-369/USP'})[id]||id;
      protocol.append(option(id,name));
    }
    protocol.value=allowed?.protocols.includes(wanted)?wanted:(allowed?.protocols[0]||'');
    el('ipat-owner-vendor-capability').textContent=allowed
      ? (allowed.status==='PARTIAL_EXACT_PILOT'
         ?'PARTIAL PHYSICAL PILOT — '+allowed.scope
         :'INVENTORY CANDIDATE · NOT YET CONNECTABLE — '+allowed.scope)
      :'Select a listed vendor. Generic vendor names do not establish compatibility.';
    syncProtocol();
  }
  function populateVendors(wanted='',protocolWanted=''){
    const available=compatibleKinds().filter(k=>k.kind===field('kind').value);
    const menu=field('vendor');menu.replaceChildren(option('','Select a catalogued vendor'));
    for(const entry of available){
      menu.append(option(entry.vendor,entry.name+' · '+(entry.status==='PARTIAL_EXACT_PILOT'
        ?'C320: limited physical pilot':'Unverified inventory candidate')));
    }
    menu.value=available.some(v=>v.vendor===wanted)?wanted:(available[0]?.vendor||'');
    populateProtocols(protocolWanted);
  }
  async function loadMasters(){
    const [siteResult, vendorResult] = await Promise.all([
      call('/lab/owner/pops'),call('/lab/owner/device-catalog')
    ]);
    if(siteResult.persistent!==true || siteResult.owner_private_lab_only!==true
        || !Array.isArray(siteResult.pops) || vendorResult.owner_private_lab_only!==true
        || vendorResult.arbitrary_vendor_allowed!==false || !Array.isArray(vendorResult.vendors))
      throw Error('Master catalog failed verification');
    pops=siteResult.pops;vendorCatalog=vendorResult.vendors;
    mastersReady=true;
    populatePops(field('pop').value);
    populateVendors(field('vendor').value,protocol.value);
  }
  function hideForms() { form.hidden = true; detail.hidden = true; editing = null; }
  function render() {
    rows.replaceChildren();
    const query = filter.value.trim().toLowerCase();
    const shown = items.filter(d => [d.display_name,d.id,d.vendor,d.exact_model,d.pop_id]
      .some(value=>String(value).toLowerCase().includes(query)));
    if (!shown.length) {
      const tr=document.createElement('tr');
      const cell=td(tr, 'No matching devices. Add one above or clear the search.');
      cell.colSpan=5;rows.append(tr);return;
    }
    for (const item of shown) {
      const tr=document.createElement('tr');
      const title=td(tr,'');
      const bold=document.createElement('strong');
      bold.textContent=item.display_name;
      const small=document.createElement('small');
      small.textContent=item.id;
      title.append(bold,small);
      td(tr,item.device_kind.toUpperCase()+' · '+item.vendor+' '+item.exact_model);
      td(tr,item.pop_id);
      td(tr,item.linked_live_connector ? item.connection_state+' · REAL C320'
        : 'SAVED · NOT CONNECTED');
      const controls=document.createElement('td');
      const group=document.createElement('div');
      group.className='ipat-owner-device-actions';
      button(group,'View',()=>view(item.id));
      button(group,item.linked_live_connector?'Edit label':'Edit',()=>edit(item.id));
      const del=button(group,'Remove',()=>remove(item.id),item.linked_live_connector);
      if(item.linked_live_connector)del.title='Active C320: separate connector revocation is required before removal.';
      controls.append(group);tr.append(controls);rows.append(tr);
    }
  }
  async function load() {
    if (pending) return;
    try {
      const payload=await call(api);
      if (!payload.owner_private_lab_only || payload.persistent !== true
          || payload.production_tenant_registry !== false || !Array.isArray(payload.devices))
        throw Error('Registry returned an unexpected response');
      items=payload.devices;
      const linked=items.find(item=>item.id==='DEV-01');
      summary.textContent=items.length+' managed records · '+(linked
        ? 'Live C320: '+linked.connection_state : 'Live C320 status unknown')
        +' · Durable private owner inventory';
      render();
    } catch(error) {
      summary.textContent='Inventory unavailable: '+error.message;
      items=[];rows.replaceChildren();
    }
  }
  function syncProtocol() {
    const needs=protocol.value!=='cwmp' && protocol.value!=='usp';
    for (const wrapper of document.querySelectorAll('.ipat-owner-address')) {
      wrapper.hidden=!needs;
    }
    field('host').required=needs;
    field('port').required=needs;
  }
  function newDevice() {
    hideForms();editing={id:null};
    if(!mastersReady){report('Loading registered sites and vendor catalog. Please retry.');return;}
    form.reset();field('pop').required=true;populatePops(pops[0]?.code||'');
    field('kind').value='olt';populateVendors('ZTE','ssh');
    field('port').value='22';field('name').readOnly=false;
    if(!pops.length){
      report('No registered Sites/POPs yet. Open Sites & POPs to create the first site, then return to Add Device.');
      window.location.assign('/lab/sites?return=device');
      return;
    }
    for (const group of document.querySelectorAll('.ipat-owner-extra'))group.hidden=false;
    el('ipat-owner-device-form-title').textContent='Add a device';
    el('ipat-owner-device-save').textContent='Save device';
    el('ipat-owner-device-form-hint').textContent='Metadata only; no connection attempt or device command. Do not enter credentials.';
    syncProtocol();form.hidden=false;field('name').focus();
  }
  async function view(id) {
    if (pending) return;
    try {
      const response=await call(api+'/'+encodeURIComponent(id));
      const d=response.device;
      hideForms();detail.replaceChildren();
      const h=document.createElement('h3');h.textContent=d.display_name;
      const dl=document.createElement('dl');
      const props={
        'Device ID':d.id,'Type':d.device_kind,'Vendor':d.vendor,'Model':d.exact_model,
        'POP / site':d.pop_id,'Transport':d.management_protocol,
        'Management host':d.management_host || (d.linked_live_connector ? 'Restricted connector configuration' : 'Agent-initiated'),
        'Management port':d.management_port || 'Not applicable',
        'Connection':d.connection_state,'Revision':d.revision,
        'Last verified UTC':d.last_verified_at_utc || 'Not verified',
        'Physical changes': 'Disabled in this private release'
      };
      for(const [key,value] of Object.entries(props)){
        const dt=document.createElement('dt');dt.textContent=key;
        const dd=document.createElement('dd');dd.textContent=String(value);
        dl.append(dt,dd);
      }
      detail.append(h,dl);detail.hidden=false;
      report('Viewing persisted device '+d.id+'. Connection status is read from the actual connector when available.');
    } catch(error){report('Unable to load device: '+error.message);}
  }
  async function edit(id) {
    if (pending) return;
    try {
      const response=await call(api+'/'+encodeURIComponent(id));
      const d=response.device;
      hideForms();editing=d;form.reset();
      field('name').value=d.display_name;
      if(d.linked_live_connector){
        for (const group of document.querySelectorAll('.ipat-owner-extra'))group.hidden=true;
        field('pop').closest('.ipat-owner-site-field').hidden=false;
        populatePops(d.pop_id==='UNASSIGNED'?'':d.pop_id);
        field('pop').required=false;
        el('ipat-owner-device-form-hint').textContent='Connected C320: display label and registered POP only. Management address, port and credentials are locked. Do not register a duplicate C320.';
        el('ipat-owner-device-form-title').textContent='Edit connected C320 display label';
      } else {
        for (const group of document.querySelectorAll('.ipat-owner-extra'))group.hidden=false;
        populatePops(d.pop_id);field('pop').required=true;
        field('kind').value=d.device_kind;populateVendors(d.vendor,d.management_protocol);
        field('model').value=d.exact_model;
        field('host').value=d.management_host || '';
        field('port').value=d.management_port || '';
        syncProtocol();
        el('ipat-owner-device-form-title').textContent='Edit '+d.id;
        el('ipat-owner-device-form-hint').textContent='Editing a saved device changes its durable metadata only. It does not connect or reconfigure the physical device.';
      }
      el('ipat-owner-device-save').textContent='Save changes';
      form.hidden=false;field('name').focus();
    } catch(error){report('Unable to edit device: '+error.message);}
  }
  function recordFields() {
    const endpoint=protocol.value !== 'cwmp' && protocol.value !== 'usp';
    return {display_name:field('name').value.trim(),pop_id:field('pop').value,
      device_kind:field('kind').value,vendor:field('vendor').value,
      exact_model:field('model').value.trim(),management_protocol:protocol.value,
      management_host:endpoint ? field('host').value.trim() : null,
      management_port:endpoint ? Number(field('port').value) : null};
  }
  async function save(e) {
    e.preventDefault();if(pending || !form.reportValidity())return;
    pending=true;el('ipat-owner-device-save').disabled=true;
    try {
      const linked=editing && editing.linked_live_connector;
      if(!mastersReady)throw Error('Registered POP/vendor catalog is unavailable');
      if(!linked && !pops.some(p=>p.code===field('pop').value))
        throw Error('Select a registered POP / Site, or create one first');
      const data=linked ? {expected_revision:editing.revision,display_name:field('name').value.trim(),
          ...(field('pop').value?{pop_id:field('pop').value}:{clear_pop:true})}
        : { ...recordFields(),...(editing?.id ? {expected_revision:editing.revision}:{}) };
      const endpoint=editing?.id ? api+'/'+encodeURIComponent(editing.id) : api;
      const result=await call(endpoint,{method:editing?.id?'PUT':'POST',body:JSON.stringify(data)});
      const wasEditing=Boolean(editing?.id);
      hideForms();pending=false;
      report((wasEditing ? 'Device changes saved' : 'New device saved')
        +' durably on the owner VPS. No physical device command was sent.');
      await load();
      const id=result.device?.id || result.id;
      if(id)await view(id);
    } catch(error){report('Save failed: '+error.message+'; check the fields and reload before retrying.');}
    finally{pending=false;el('ipat-owner-device-save').disabled=false;}
  }
  async function remove(id) {
    if (pending) return;
    const d=items.find(item=>item.id===id);
    if(!d)return;
    if(d.linked_live_connector){
      report('Connected C320 cannot be removed from inventory while the sealed connector holds live credentials. Separate owner-approved connector revocation is required.');
      return;
    }
    if(!window.confirm('Remove '+d.display_name+' ('+id+') from the saved inventory? This deletes local metadata only; no device command will run.'))return;
    pending=true;
    try {
      const result=await call(api+'/'+encodeURIComponent(id),{
        method:'DELETE',body:JSON.stringify({expected_revision:d.revision,confirm:'REMOVE '+id})
      });
      if(result.removed !== true)throw Error('No successful removal acknowledgement');
      hideForms();pending=false;
      report('Removed '+id+' durably. Audit retained; physical device was not modified.');
      await load();
    } catch(error){report('Remove failed: '+error.message+'; refresh before retrying.');}
    finally{pending=false;}
  }
  el('ipat-owner-device-add').addEventListener('click',newDevice);
  el('ipat-owner-device-refresh').addEventListener('click',async()=>{
    try{await loadMasters();await load();}catch(error){report('Unable to refresh master lists: '+error.message);}
  });
  field('kind').addEventListener('change',()=>populateVendors());
  field('vendor').addEventListener('change',()=>populateProtocols());
  el('ipat-owner-device-cancel').addEventListener('click',hideForms);
  filter.addEventListener('input',render);
  protocol.addEventListener('change',syncProtocol);
  form.addEventListener('submit',save);
  window.addEventListener('ipat-device-connection-changed',load);
  loadMasters().then(async()=>{
    await load();
    if(new URLSearchParams(window.location.search).get('new-device')==='1'){
      newDevice();
      form.scrollIntoView({block:'nearest'});
    }
  }).catch(error=>{
    mastersReady=false;report('Site/vendor catalog unavailable: '+error.message);
    summary.textContent='Device master catalog unavailable. Add/Edit is disabled until it loads.';
  });
  setInterval(load,15000);
})();
