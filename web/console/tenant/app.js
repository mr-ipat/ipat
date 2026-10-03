'use strict';
// Never infer authorization from local state or a previously successful view.
// The server re-verifies current membership and CSRF on EVERY API request.
(() => {
 const $=id=>document.getElementById(id);
 const notice=text=>{$('notice').textContent=text};
 const state={page:null,sites:[],pops:[],popNext:null,devices:[],catalog:[],next:null,editId:null,editRevision:null,pendingSite:null,nocPops:[],realNocPops:[],nocMembers:[],nocAccess:[]};
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
 function fillPopChoices(id,selected){const sel=$(id);sel.replaceChildren();option(sel,'Unassigned — no POP','');state.pops.forEach(p=>option(sel,p.display_name+' ('+p.code+')',p.code));sel.value=selected||''}
 async function refreshPops(after){
  const r=await api('/api/v1/pops'+(after?'?after='+encodeURIComponent(after):''));
  if(!after)state.pops=[];
  state.pops.push(...r.pops);state.popNext=r.next_after||null;
  $('pop-more').hidden=!state.popNext;
  $('pop-rows').replaceChildren();
  for(const p of state.pops){
   const tr=document.createElement('tr');
   cell(tr,p.code);cell(tr,p.display_name);cell(tr,p.assigned_sites);cell(tr,p.revision);
   const actions=cell(tr,'');actions.textContent='';
   btn(actions,'Rename',async()=>{
    const name=prompt('New POP display name',p.display_name);if(name===null)return;
    try{await api('/api/v1/pops/'+encodeURIComponent(p.code),'PATCH',
      {display_name:name,expected_revision:p.revision});
      await refreshPops();await refreshSites();notice('POP renamed; Site associations preserved.');
    }catch(e){notice(e.message)}
   });
   const remove=btn(actions,'Remove',async()=>{
    if(!confirm('Remove unused POP '+p.code+'? Assigned Sites are never removed.'))return;
    try{await api('/api/v1/pops/'+encodeURIComponent(p.code)+'?revision='+p.revision,'DELETE');
      await refreshPops();notice('Unused POP removed; audit retained.');
    }catch(e){notice(e.message)}
   });
   remove.disabled=p.assigned_sites>0;
   $('pop-rows').append(tr);
  }
  fillPopChoices('site-parent-pop',$('site-parent-pop').value);
 }
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
   const tr=document.createElement('tr');cell(tr,site.code);cell(tr,site.display_name);cell(tr,site.parent_pop_code||'Unassigned');cell(tr,site.assigned_devices);cell(tr,site.revision);
   const actions=cell(tr,'');actions.textContent='';
   const parent=document.createElement('select');parent.setAttribute('aria-label','Parent POP for Site '+site.code);
   option(parent,'Unassigned','');state.pops.forEach(p=>option(parent,p.display_name+' ('+p.code+')',p.code));
   parent.value=site.parent_pop_code||'';actions.append(parent);
   btn(actions,'Assign POP',async()=>{
    try{const selected=parent.value||null;
     await api('/api/v1/sites/'+encodeURIComponent(site.code)+'/pop','PATCH',
       {parent_pop_code:selected,expected_revision:site.revision});
     await refreshPops();await refreshSites();
     notice(selected?'Site assigned to registered POP.':'Site unassigned from POP; device metadata unchanged.');
    }catch(e){notice(e.message)}
   });
   btn(actions,'Rename',async()=>{const name=prompt('New Site display name',site.display_name);if(name===null)return;try{await api('/api/v1/sites/'+encodeURIComponent(site.code),'PATCH',{display_name:name,expected_revision:site.revision});notice('Site renamed.');await refreshSites()}catch(e){notice(e.message)}});
   const remove=btn(actions,'Remove',async()=>{if(!confirm(`Remove unused Site ${site.code}? This does not delete physical equipment.`))return;try{await api('/api/v1/sites/'+encodeURIComponent(site.code)+'?revision='+site.revision,'DELETE');notice('Unused Site removed.');await refreshSites()}catch(e){notice(e.message)}});
   remove.disabled=site.assigned_devices>0;
   $('site-rows').append(tr);
  }
  fillSiteChoices('device-site',state.pendingSite||selectedDevice);fillSiteChoices('edit-site',selectedEdit);state.pendingSite=null;
 }
 async function loadCatalog(){const r=await api('/api/v1/device-catalog');state.catalog=r.catalog;
  if(r.catalog_semantics!=='platform_curated_metadata_only_not_verified_physical_support')throw Error('Catalog identity not verified; registration disabled.');const types=[...new Set(state.catalog.map(v=>v.type))];$('device-kind').replaceChildren();types.forEach(t=>option($('device-kind'),t.toUpperCase(),t));syncCatalog()}
 function syncCatalog(){
  const type=$('device-kind').value;const list=state.catalog.filter(x=>x.type===type);
  const vendor=$('device-vendor');const old=vendor.value;vendor.replaceChildren();list.forEach(x=>option(vendor,x.vendor,x.vendor));if(list.some(x=>x.vendor===old))vendor.value=old;
  const match=list.find(x=>x.vendor===vendor.value);const transport=$('device-transport');transport.replaceChildren();(match?.transports||[]).forEach(x=>option(transport,x.toUpperCase(),x));syncEndpoint();
 }
 function syncEndpoint(){
  const entry=state.catalog.find(x=>x.type===$('device-kind').value&&x.vendor===$('device-vendor').value);
  const selected=(entry?.protocols||[]).find(p=>p.transport===$('device-transport').value);
  $('device-catalog-note').textContent=selected
   ?'Registration allowed for metadata only. '+selected.description+' No automatic physical connection or firmware action.'
   :'Choose a catalog-listed vendor and transport; compatibility is not established.';
  const remote=['cwmp','usp'].includes($('device-transport').value);for(const id of ['device-host','device-port']){$(id).disabled=remote;$(id).required=!remote;if(remote)$(id).value='';else if(id==='device-port'&&!$(id).value)$(id).value='22'}}
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
 function refreshNocAccessChoices(){
  const member=$('noc-access-member');member.replaceChildren();
  option(member,'Select a current NOC member','');
  state.nocMembers.forEach((m,i)=>option(member,m.subject+' · '+m.issuer,String(i)));
  const pop=$('noc-access-pop');pop.replaceChildren();
  option(pop,'Select a real POP','');
  state.pops.forEach(p=>option(pop,p.display_name+' ('+p.code+')',p.code));
 }
 async function refreshNocAccess(){
  const r=await api('/api/v1/noc-access');
  state.nocMembers=r.members||[];state.nocAccess=r.access||[];
  refreshNocAccessChoices();
  const rows=$('noc-access-rows');rows.replaceChildren();
  for(const x of state.nocAccess){
   const tr=document.createElement('tr');
   cell(tr,x.kind);cell(tr,x.target_subject+' · '+x.target_issuer);cell(tr,x.pop_code);
   cell(tr,x.state);cell(tr,x.expires_at);cell(tr,x.requested_by+(x.reviewed_by?' / '+x.reviewed_by:''));
   const actions=cell(tr,'');actions.textContent='';
   if(x.kind==='request'&&x.state==='PENDING'&&x.request_id){
    btn(actions,'Approve',async()=>{try{await api('/api/v1/noc-access/requests/'+encodeURIComponent(x.request_id)+'/review','POST',{approve:true});await refreshNocAccess();notice('NOC POP access approved.')}catch(e){notice(e.message)}});
    btn(actions,'Reject',async()=>{try{await api('/api/v1/noc-access/requests/'+encodeURIComponent(x.request_id)+'/review','POST',{approve:false});await refreshNocAccess();notice('NOC POP access rejected.')}catch(e){notice(e.message)}});
   }
   if(x.kind==='grant'&&x.state==='ACTIVE'){
    btn(actions,'Revoke',async()=>{if(!confirm('Revoke read-only POP access for '+x.target_subject+'?'))return;try{await api('/api/v1/noc-access/revoke','POST',{target_issuer:x.target_issuer,target_subject:x.target_subject,pop_code:x.pop_code});await refreshNocAccess();notice('NOC POP access revoked immediately.')}catch(e){notice(e.message)}});
   }
   rows.append(tr);
  }
 }
 async function refreshNoc(){
  const chosen=$('noc-pop').value;
  const typed=chosen.startsWith('real:');
  const legacy=chosen.startsWith('site:');
  const pop=chosen.slice(5);
  if(!((typed&&state.realNocPops.includes(pop))||
       (legacy&&state.nocPops.includes(pop)))){
   throw Error('Select a currently authorized exact Site or real POP scope.');
  }
  const base=typed?'/api/v1/noc/real':'/api/v1/noc';
  const [sites,devices]=await Promise.all([
   api(base+'/sites?pop='+encodeURIComponent(pop)),
   api(base+'/devices?pop='+encodeURIComponent(pop))
  ]);
  $('noc-site-rows').replaceChildren();
  for(const s of sites.sites){const tr=document.createElement('tr');cell(tr,s.code);cell(tr,s.display_name);cell(tr,s.assigned_devices);$('noc-site-rows').append(tr)}
  $('noc-device-rows').replaceChildren();
  for(const d of devices.devices){const tr=document.createElement('tr');cell(tr,d.display_name);cell(tr,d.site);cell(tr,d.device_kind+' / '+d.vendor);cell(tr,d.lifecycle_state==='SAVED'?'Saved — not connected':d.lifecycle_state);$('noc-device-rows').append(tr)}
 }
 async function page(name){if(!['pops','sites','devices','domains','noc-access','noc'].includes(name)||$('nav-'+name)?.hidden)return;state.page=name;$('welcome').hidden=true;document.querySelectorAll('[data-panel]').forEach(p=>p.hidden=p.dataset.panel!==name);notice('Loading current tenant data…');try{if(name==='pops')await refreshPops();if(name==='sites'){await refreshPops();await refreshSites()}if(name==='devices'){await refreshPops();await refreshSites();await refreshDevices()}if(name==='domains')await refreshDomains();if(name==='noc-access'){await refreshPops();await refreshNocAccess()}if(name==='noc')await refreshNoc();notice('Data loaded. Operations remain subject to current authorization.')}catch(e){notice(e.message)}}
 async function submit(form,callback){const button=form.querySelector('button[type=submit],button:not([type])');if(button)button.disabled=true;try{await callback()}catch(e){notice(e.message)}finally{if(button)button.disabled=false}}
 document.querySelectorAll('.nav').forEach(b=>{b.id='nav-'+b.dataset.page;b.addEventListener('click',()=>page(b.dataset.page))});
 $('create-site-link').addEventListener('click',e=>{e.preventDefault();page('sites')});
 $('manage-pops-link').addEventListener('click',e=>{e.preventDefault();page('pops')});
 $('pop-more').addEventListener('click',()=>refreshPops(state.popNext).catch(e=>notice(e.message)));
 $('pop-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{
  const d=Object.fromEntries(new FormData(f));await api('/api/v1/pops','POST',d);
  clearForm(f);await refreshPops();notice('POP registered independently. Create or associate a Site next.');
 })});
 $('site-more').addEventListener('click',()=>refreshSites(state.next).catch(e=>notice(e.message)));
 $('noc-access-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{
  const idx=Number($('noc-access-member').value);const member=state.nocMembers[idx];
  const pop=$('noc-access-pop').value;const hours=Number($('noc-access-hours').value);
  if(!member||!pop||!Number.isInteger(hours))throw Error('Choose a current NOC member, a real POP and a valid duration.');
  await api('/api/v1/noc-access/requests','POST',{request_id:crypto.randomUUID(),target_issuer:member.issuer,target_subject:member.subject,pop_code:pop,expires_in_hours:hours});
  await refreshNocAccess();notice('POP access request created. A different current Tenant Admin must approve it.');
 })});
 $('noc-pop').addEventListener('change',()=>{if(state.page==='noc')refreshNoc().catch(e=>notice(e.message))});
 $('device-kind').addEventListener('change',syncCatalog);$('device-vendor').addEventListener('change',syncCatalog);$('device-transport').addEventListener('change',syncEndpoint);
 $('site-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{const d=Object.fromEntries(new FormData(f));d.parent_pop_code=d.parent_pop_code||null;
  await api('/api/v1/sites','POST',d);state.pendingSite=d.code;clearForm(f);
  await refreshPops();await refreshSites();notice('Site registered successfully.')})});
 $('device-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{const d=Object.fromEntries(new FormData(f));if(['cwmp','usp'].includes(d.management_transport)){d.management_host=null;d.management_port=null}else{d.management_port=Number(d.management_port)};d.request_id=crypto.randomUUID();d.intended_model=d.intended_model||null;await api('/api/v1/devices','POST',d);clearForm(f);syncCatalog();await refreshDevices();notice('Device saved as metadata. Physical connection not yet established.')})});
 $('device-edit').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{const d=Object.fromEntries(new FormData(f));d.expected_revision=state.editRevision;d.intended_model=d.intended_model||null;d.management_host=d.management_host||null;d.management_port=d.management_port?Number(d.management_port):null;await api('/api/v1/devices/'+encodeURIComponent(state.editId),'PATCH',d);$('device-edit-panel').hidden=true;state.editId=null;await refreshDevices();notice('Saved device metadata updated.')})});
 $('edit-cancel').addEventListener('click',()=>{$('device-edit-panel').hidden=true;state.editId=null});
 $('domain-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{await api('/api/v1/domains','POST',Object.fromEntries(new FormData(f)));clearForm(f);await refreshDomains();notice('Domain requested, not activated. Review the verification record.')})});
 $('logout').addEventListener('click',async()=>{try{await api('/api/v1/logout','POST');location.assign('/auth/oidc/start')}catch(e){notice(e.message)}});
 (async()=>{try{
  const r=await api('/api/v1/capabilities');$('identity').textContent='Authorized tenant: '+r.tenant_id;
  state.nocPops=Array.isArray(r.noc_pops)?r.noc_pops.filter(p=>typeof p==='string'):[];
  state.realNocPops=Array.isArray(r.real_noc_pops)?r.real_noc_pops.filter(p=>typeof p==='string'):[];
  $('noc-pop').replaceChildren();
  for(const p of state.realNocPops)option($('noc-pop'),'Real POP: '+p,'real:'+p);
  for(const p of state.nocPops)option($('noc-pop'),'Legacy exact Site: '+p,'site:'+p);
  if(state.nocPops.length||state.realNocPops.length)$('nav-noc').hidden=false;
  if(r.can_manage_sites){for(const n of ['pops','sites','devices','domains','noc-access'])$('nav-'+n).hidden=false;await loadCatalog()}
  $('logout').hidden=false;
  notice(r.can_manage_sites||state.nocPops.length||state.realNocPops.length?'Tenant permissions verified. Select a permitted module.':'No operations modules assigned to your current identity.');

 }catch(e){notice(e.message);$('signin').hidden=false;$('identity').textContent='Sign-in required'}})();
})();
