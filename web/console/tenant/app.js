'use strict';
// Never infer authorization from local state or a previously successful view.
// The server re-verifies current membership and CSRF on EVERY API request.
(() => {
 const $=id=>document.getElementById(id);
 const notice=text=>{$('notice').textContent=text};
 const state={page:null,sites:[],devices:[],catalog:[],next:null,editId:null,editRevision:null,pendingSite:null};
 const csrf=()=>document.cookie.split(';').map(x=>x.trim()).find(x=>x.startsWith('__Host-ipat_csrf='))?.split('=')[1];
 async function api(path,method='GET',body){
  const headers={'Accept':'application/json'};
  if(method!=='GET'){
   const token=csrf();if(!token)throw Error('CSRF cookie missing. Sign in again.');
   headers['X-IPAT-CSRF']=token;
   if(body!==undefined)headers['Content-Type']='application/json';
  }
  const r=await fetch(path,{method,credentials:'same-origin',headers,cache:'no-store',body:body===undefined?undefined:JSON.stringify(body)});
  const result=await r.json().catch(()=>({ok:false}));
  if(!r.ok||!result.ok)throw Error(r.status===401?'Session expired or your permissions changed. Sign in again.':`Request rejected (HTTP ${r.status}); refresh before retrying.`);
  return result;
 }
 const cell=(tr,value)=>{const td=document.createElement('td');td.textContent=String(value??'—');tr.append(td);return td};
 const btn=(parent,text,callback)=>{const b=document.createElement('button');b.type='button';b.textContent=text;b.addEventListener('click',callback);parent.append(b);return b};
 const option=(parent,label,value)=>{const o=document.createElement('option');o.value=value;o.textContent=label;parent.append(o)};
 function clearForm(form){form.reset()}
 function fillSiteChoices(id,selected){const select=$(id);select.replaceChildren();option(select,'Select a registered Site','');state.sites.forEach(s=>option(select,`${s.display_name} (${s.code})`,s.code));select.value=selected||''}
 async function refreshSites(after){
  // Preserve a user's current selection while a navigation-triggered fetch
  // is in flight. Creating a Site explicitly preselects the new Site.
  const selectedDevice=$('device-site').value;
  const selectedEdit=$('edit-site').value;
  const url='/api/v1/sites'+(after?'?after='+encodeURIComponent(after):'');
  const r=await api(url);
  if(!after)state.sites=[];
  state.sites.push(...r.sites);
  state.next=r.next_after||null;
  $('site-more').hidden=!state.next;
  $('site-rows').replaceChildren();
  for(const site of state.sites){
   const tr=document.createElement('tr');cell(tr,site.code);cell(tr,site.display_name);cell(tr,site.assigned_devices);cell(tr,site.revision);
   const actions=cell(tr,'');actions.textContent='';
   btn(actions,'Rename',async()=>{const name=prompt('New Site display name',site.display_name);if(name===null)return;try{await api('/api/v1/sites/'+encodeURIComponent(site.code),'PATCH',{display_name:name,expected_revision:site.revision});notice('Site renamed.');await refreshSites()}catch(e){notice(e.message)}});
   const remove=btn(actions,'Remove',async()=>{if(!confirm(`Remove unused Site ${site.code}? This does not delete physical equipment.`))return;try{await api('/api/v1/sites/'+encodeURIComponent(site.code)+'?revision='+site.revision,'DELETE');notice('Unused Site removed.');await refreshSites()}catch(e){notice(e.message)}});
   remove.disabled=site.assigned_devices>0;
   $('site-rows').append(tr);
  }
  fillSiteChoices('device-site',state.pendingSite||selectedDevice);fillSiteChoices('edit-site',selectedEdit);state.pendingSite=null;
 }
 async function loadCatalog(){const r=await api('/api/v1/device-catalog');state.catalog=r.catalog;const types=[...new Set(state.catalog.map(v=>v.type))];$('device-kind').replaceChildren();types.forEach(t=>option($('device-kind'),t.toUpperCase(),t));syncCatalog()}
 function syncCatalog(){
  const type=$('device-kind').value;const list=state.catalog.filter(x=>x.type===type);
  const vendor=$('device-vendor');const old=vendor.value;vendor.replaceChildren();list.forEach(x=>option(vendor,x.vendor,x.vendor));if(list.some(x=>x.vendor===old))vendor.value=old;
  const match=list.find(x=>x.vendor===vendor.value);const transport=$('device-transport');transport.replaceChildren();(match?.transports||[]).forEach(x=>option(transport,x.toUpperCase(),x));syncEndpoint();
 }
 function syncEndpoint(){const remote=['cwmp','usp'].includes($('device-transport').value);for(const id of ['device-host','device-port']){$(id).disabled=remote;$(id).required=!remote;if(remote)$(id).value='';else if(id==='device-port'&&!$(id).value)$(id).value='22'}}
 async function refreshDevices(){const r=await api('/api/v1/devices');state.devices=r.devices;$('device-rows').replaceChildren();for(const d of state.devices){
  const tr=document.createElement('tr');cell(tr,d.display_name);cell(tr,d.site);cell(tr,`${d.device_kind} / ${d.vendor}`);cell(tr,d.lifecycle_state==='SAVED'?'Saved — not connected':d.lifecycle_state);
  const actions=cell(tr,'');actions.textContent='';btn(actions,'View',async()=>{try{const x=(await api('/api/v1/devices/'+encodeURIComponent(d.id))).device;notice(`${x.display_name} · ${x.vendor} ${x.intended_model||''} · ${x.lifecycle_state} · Revision ${x.revision}. No physical action available.`)}catch(e){notice(e.message)}});
  const edit=btn(actions,'Edit',async()=>{try{const x=(await api('/api/v1/devices/'+encodeURIComponent(d.id))).device;state.editId=x.id;state.editRevision=x.revision;await refreshSites();const f=$('device-edit');f.elements.display_name.value=x.display_name;fillSiteChoices('edit-site',x.site);f.elements.intended_model.value=x.intended_model||'';f.elements.management_host.value=x.management_host||'';f.elements.management_port.value=x.management_port||'';$('device-edit-panel').hidden=false;notice('Editing saved metadata only.')}catch(e){notice(e.message)}});
  const del=btn(actions,'Archive',async()=>{if(!confirm(`Archive saved inventory record ${d.display_name}? This does not disconnect equipment.`))return;try{const x=(await api('/api/v1/devices/'+encodeURIComponent(d.id))).device;await api('/api/v1/devices/'+encodeURIComponent(d.id)+'?revision='+x.revision,'DELETE');notice('Saved device archived; audit retained.');await refreshDevices()}catch(e){notice(e.message)}});
  // The backend also denies Edit/Archive on live, credential-bound or firmware-pending rows.
  if(d.lifecycle_state!=='SAVED'){edit.disabled=true;del.disabled=true}
  $('device-rows').append(tr);
 }}
 async function refreshDomains(){const r=await api('/api/v1/domains');$('domain-rows').replaceChildren();for(const d of r.domains){const tr=document.createElement('tr');cell(tr,d.hostname);cell(tr,d.verification_name?`${d.verification_name} TXT ${d.verification_value||''}`:'Awaiting instructions');cell(tr,d.activation_state);
 const actions=cell(tr,'');actions.textContent='';btn(actions,'Disable',async()=>{if(!confirm(`Disable domain request ${d.hostname}?`))return;try{await api('/api/v1/domains/'+encodeURIComponent(d.id),'DELETE');notice('Domain request disabled.');await refreshDomains()}catch(e){notice(e.message)}});$('domain-rows').append(tr)}}
 async function page(name){if(!['sites','devices','domains'].includes(name)||$('nav-'+name)?.hidden)return;state.page=name;$('welcome').hidden=true;document.querySelectorAll('[data-panel]').forEach(p=>p.hidden=p.dataset.panel!==name);notice('Loading current tenant data…');try{if(name==='sites')await refreshSites();if(name==='devices'){await refreshSites();await refreshDevices()}if(name==='domains')await refreshDomains();notice('Data loaded. Operations remain subject to current authorization.')}catch(e){notice(e.message)}}
 async function submit(form,callback){const button=form.querySelector('button[type=submit],button:not([type])');if(button)button.disabled=true;try{await callback()}catch(e){notice(e.message)}finally{if(button)button.disabled=false}}
 document.querySelectorAll('.nav').forEach(b=>{b.id='nav-'+b.dataset.page;b.addEventListener('click',()=>page(b.dataset.page))});
 $('create-site-link').addEventListener('click',e=>{e.preventDefault();page('sites')});
 $('site-more').addEventListener('click',()=>refreshSites(state.next).catch(e=>notice(e.message)));
 $('device-kind').addEventListener('change',syncCatalog);$('device-vendor').addEventListener('change',syncCatalog);$('device-transport').addEventListener('change',syncEndpoint);
 $('site-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{const d=Object.fromEntries(new FormData(f));await api('/api/v1/sites','POST',d);state.pendingSite=d.code;clearForm(f);await refreshSites();notice('Site registered successfully.')})});
 $('device-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{const d=Object.fromEntries(new FormData(f));if(['cwmp','usp'].includes(d.management_transport)){d.management_host=null;d.management_port=null}else{d.management_port=Number(d.management_port)};d.request_id=crypto.randomUUID();d.intended_model=d.intended_model||null;await api('/api/v1/devices','POST',d);clearForm(f);syncCatalog();await refreshDevices();notice('Device saved as metadata. Physical connection not yet established.')})});
 $('device-edit').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{const d=Object.fromEntries(new FormData(f));d.expected_revision=state.editRevision;d.intended_model=d.intended_model||null;d.management_host=d.management_host||null;d.management_port=d.management_port?Number(d.management_port):null;await api('/api/v1/devices/'+encodeURIComponent(state.editId),'PATCH',d);$('device-edit-panel').hidden=true;state.editId=null;await refreshDevices();notice('Saved device metadata updated.')})});
 $('edit-cancel').addEventListener('click',()=>{$('device-edit-panel').hidden=true;state.editId=null});
 $('domain-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{await api('/api/v1/domains','POST',Object.fromEntries(new FormData(f)));clearForm(f);await refreshDomains();notice('Domain requested, not activated. Review the verification record.')})});
 $('logout').addEventListener('click',async()=>{try{await api('/api/v1/logout','POST');location.assign('/auth/oidc/start')}catch(e){notice(e.message)}});
 (async()=>{try{
  const r=await api('/api/v1/capabilities');$('identity').textContent='Authorized tenant: '+r.tenant_id;
  if(r.can_manage_sites){for(const n of ['sites','devices','domains'])$('nav-'+n).hidden=false;$('logout').hidden=false;await loadCatalog();notice('Tenant permissions verified. Select a module.')}else{notice('No tenant administration modules assigned to your current identity.');$('logout').hidden=false}
 }catch(e){notice(e.message);$('signin').hidden=false;$('identity').textContent='Sign-in required'}})();
})();
