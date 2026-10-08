'use strict';
// Never infer authorization from local state or a previously successful view.
// The server re-verifies current membership and CSRF on EVERY API request.
(() => {
 const $=id=>document.getElementById(id);
 const notice=text=>{$('notice').textContent=text};
 const state={page:null,sites:[],pops:[],popNext:null,devices:[],catalog:[],next:null,editId:null,editRevision:null,pendingSite:null,nocPops:[],realNocPops:[],nocMembers:[],nocAccess:[],pendingNocRequest:null,subscribers:[],subscriberEdit:null,canReadSubscribers:false,canManageSubscribers:false,canManagePppoePlans:false,canCreatePppoePlans:false,canReviewPppoePlans:false,canArmPppoePlans:false,pendingPppoe:null};
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
 async function showDomainDns(id){
  const r=await api('/api/v1/domains/'+encodeURIComponent(id)+'/dns');
  const guide=r.guide;
  const section=$('domain-dns-guide');const rows=$('domain-dns-records');
  section.hidden=false;rows.replaceChildren();
  const line=(title,value)=>{
    const p=document.createElement('p');
    const strong=document.createElement('strong');strong.textContent=title+': ';
    const text=document.createElement('span');text.textContent=String(value);
    p.append(strong,text);rows.append(p);
  };
  line('Customer hostname',guide.hostname);
  line('Ownership DNS record',guide.verification_record_type+' '+guide.verification_record_name);
  line('Unique saved TXT value',guide.verification_value);
  $('domain-dns-status').textContent=guide.safe_to_point_now
   ?'Ownership and ingress readiness approved; review the exact routing records with your DNS operator.'
   :'WAIT: complete ownership verification and reviewed HTTPS ingress before routing the customer domain.';
  line('Requested routing mode',guide.routing_mode);
  for(const record of guide.routing_records||[])
    line('Candidate '+record.record_type+' ('+record.stage+')',record.name+' → '+record.value);
  notice('DNS instructions loaded for your own registered company domain.');
 }
 async function refreshDomains(){const r=await api('/api/v1/domains');$('domain-rows').replaceChildren();for(const d of r.domains){const tr=document.createElement('tr');cell(tr,d.hostname);cell(tr,d.verification_name?`${d.verification_name} TXT ${d.verification_value||''}`:'Awaiting instructions');cell(tr,d.activation_state);
 const actions=cell(tr,'');actions.textContent='';if(d.activation_state!=='disabled')btn(actions,'DNS instructions',async()=>{try{await showDomainDns(d.id)}catch(e){notice(e.message)}});btn(actions,'Disable',async()=>{if(!confirm(`Disable domain request ${d.hostname}?`))return;try{await api('/api/v1/domains/'+encodeURIComponent(d.id),'DELETE');notice('Domain request disabled.');await refreshDomains()}catch(e){notice(e.message)}});$('domain-rows').append(tr)}}
 function refreshDiagnosticChoices(){
  const diag=$('diagnostic-distribution');const previous=diag.value;diag.replaceChildren();
  option(diag,'Select an authorized distribution router','');
  const ids=[...new Set(state.subscribers.map(x=>x.distribution_device_id).filter(Boolean))];
  for(const id of ids){
    const d=state.devices.find(x=>x.id===id);
    option(diag,d?d.display_name+' · '+d.vendor:id,id);
  }
  if(ids.includes(previous))diag.value=previous;
 }
 function refreshSubscriberChoices(){
  if(!state.canManageSubscribers)return;
  const pop=$('subscriber-pop');pop.replaceChildren();option(pop,'Select registered POP','');
  state.pops.forEach(p=>option(pop,p.display_name+' ('+p.code+')',p.code));
  const site=$('subscriber-site');site.replaceChildren();option(site,'Select registered Site','');
  state.sites.forEach(x=>option(site,x.display_name+' ('+x.code+')',x.code));
  const distribution=$('subscriber-distribution');
  distribution.replaceChildren();option(distribution,'Select registered distribution router','');
  for(const d of state.devices.filter(x=>x.device_kind==='router')){
    const label=d.display_name+' · '+d.vendor;option(distribution,label,d.id);
  }
  const access=$('subscriber-access');access.replaceChildren();option(access,'No access device assigned','');
  for(const d of state.devices.filter(x=>['olt','ont'].includes(x.device_kind)))
    option(access,d.display_name+' · '+d.device_kind.toUpperCase()+' / '+d.vendor,d.id);
 }
 async function refreshSubscribers(){
  const r=await api('/api/v1/subscribers');state.subscribers=r.subscribers||[];
  const rows=$('subscriber-rows');rows.replaceChildren();
  for(const x of state.subscribers){
    const tr=document.createElement('tr');cell(tr,x.subscriber_id+' · '+x.display_name);cell(tr,x.pppoe_username||'—');
    cell(tr,x.pop_code+' / '+x.site_code);cell(tr,x.distribution_device_id);cell(tr,x.access_device_id||'—');
    const topo=x.topology_state==='verified'
      ?'VERIFIED · '+(x.topology_source_id||'evidence')+' · '+(x.topology_verified_epoch||'')
      :'DECLARED · not independently verified';
    cell(tr,topo);cell(tr,x.revision);
    const actions=cell(tr,'');actions.textContent='';
    if(state.canManageSubscribers){
      btn(actions,'Edit',()=>{state.subscriberEdit={id:x.subscriber_id,revision:x.revision};
        $('subscriber-id').value=x.subscriber_id;$('subscriber-id').readOnly=true;$('subscriber-name').value=x.display_name;
        $('subscriber-pppoe').value=x.pppoe_username||'';$('subscriber-pop').value=x.pop_code;$('subscriber-site').value=x.site_code;
        $('subscriber-distribution').value=x.distribution_device_id;$('subscriber-access').value=x.access_device_id||'';
        $('subscriber-ont').value=x.ont_reference||'';$('subscriber-cancel').hidden=false;
        notice('Editing Subscriber 360 metadata. Changing topology invalidates previous verification. No network action will run.');});
    }else actions.textContent='Read-only scope';
    rows.append(tr);
  }
  refreshDiagnosticChoices();
 }
 function clearSubscriberEdit(){state.subscriberEdit=null;$('subscriber-form').reset();$('subscriber-id').readOnly=false;$('subscriber-cancel').hidden=true;refreshSubscriberChoices()}
 async function loadSubscriberWorkspace(){
  if(state.canManageSubscribers){
    await refreshPops();await refreshSites();await refreshDevices();refreshSubscriberChoices();
  }
  await refreshSubscribers();
}
 async function runDiagnostic(){
  const id=$('diagnostic-distribution').value,age=Number($('diagnostic-age').value);
  if(!id||!Number.isInteger(age)||age<1||age>3600)throw Error('Select a router and valid freshness window.');
  const r=await api('/api/v1/diagnostics?distribution_device_id='+encodeURIComponent(id)+'&max_age_seconds='+age);
  $('diagnostic-result').hidden=false;$('diagnostic-title').textContent='Hypothesis: '+String(r.hypothesis).replaceAll('_',' ');
  $('diagnostic-summary').textContent='Uncertainty: '+r.uncertainty+' · topology verified: '+r.topology_verified+' · affected: '+(r.affected_subscribers?.length||0)+' · stale discarded: '+r.discarded_stale+' · operator review: '+r.requires_operator_review+' · remediation: '+r.remediation_permitted+(r.reason?' · reason: '+r.reason:'');
  const evidence=$('diagnostic-evidence');evidence.replaceChildren();
  for(const e of r.evidence||[]){const p=document.createElement('p');p.textContent=(e.subscriber_id||'path')+' · '+e.signal+' · '+e.source_id+' · '+e.observed_at_epoch+' · '+e.evidence_sha256;evidence.append(p);}
 }


 function parseStrictCsvRow(line,lineNumber){
  const out=[];let value='';let quoted=false;let afterQuote=false;
  for(let i=0;i<line.length;i++){
   const c=line[i];
   if(quoted){
    if(c==='"'&&line[i+1]==='"'){value+='"';i++}
    else if(c==='"'){quoted=false;afterQuote=true}
    else value+=c;
   }else if(afterQuote){
    if(c===','){out.push(value);value='';afterQuote=false}
    else throw Error('Invalid CSV quoting on line '+lineNumber+'.');
   }else if(c===','){out.push(value);value=''}
   else if(c==='"'){
    if(value.length)throw Error('CSV quote must begin a field on line '+lineNumber+'.');
    quoted=true;
   }else value+=c;
  }
  if(quoted)throw Error('Unclosed CSV quote on line '+lineNumber+'.');
  out.push(value);return out;
 }
 function parsePppoeOperations(raw){
  if(raw.includes('\0'))throw Error('CSV contains a control character.');
  const lines=raw.split(/\r?\n/).filter(x=>x.trim().length);
  if(lines.length<2||lines.length>129)throw Error('CSV needs one header and 1–128 operations.');
  const header=parseStrictCsvRow(lines[0],1).map(x=>x.trim());
  const expected=['subscriber_id','action','username','profile','secret_ref'];
  if(header.length!==expected.length||!header.every((x,i)=>x===expected[i]))
   throw Error('CSV header must be subscriber_id,action,username,profile,secret_ref.');
  return lines.slice(1).map((line,index)=>{
   const parts=parseStrictCsvRow(line,index+2);
   if(parts.length!==5)throw Error('CSV line '+(index+2)+' must contain exactly five fields.');
   const [subscriber_id,action,username,profile,secret_ref]=parts.map(x=>x.trim());
   if(!/^[A-Za-z0-9_.-]{1,128}$/.test(subscriber_id))throw Error('Invalid subscriber ID on CSV line '+(index+2)+'.');
   if(!['create','update','disable'].includes(action))throw Error('Invalid action on CSV line '+(index+2)+'.');
   if(action==='disable'){
    if(username||profile||secret_ref)throw Error('Disable CSV line '+(index+2)+' must leave username/profile/vault blank.');
    return {subscriber_id,action,username:null,profile:null,secret_ref:null};
   }
   if(!username||/\s/.test(username)||username.length>128)throw Error('Create/update CSV line '+(index+2)+' needs a valid username.');
   if(!/^[A-Za-z0-9_.:-]{1,64}$/.test(profile))throw Error('Create/update CSV line '+(index+2)+' needs a valid profile.');
   if(!secret_ref.startsWith('vault://tenant/')||secret_ref.length>255)throw Error('Create/update CSV line '+(index+2)+' needs a tenant Vault reference, never a password.');
   return {subscriber_id,action,username,profile,secret_ref};
  });
 }
 function refreshPppoeRouterChoices(){
  const select=$('pppoe-router');const keep=select.value;select.replaceChildren();
  option(select,'Select a registered MikroTik API-SSL router','');
  state.devices.filter(d=>d.device_kind==='router'&&d.vendor==='MikroTik'&&
    d.management_transport==='routeros_api_ssl'&&d.lifecycle_state==='SAVED')
   .forEach(d=>option(select,d.display_name+' · '+d.site,d.id));
  if([...select.options].some(o=>o.value===keep))select.value=keep;
 }
 async function refreshPppoePlans(){
  const r=await api('/api/v1/pppoe-plans');const rows=$('pppoe-plan-rows');rows.replaceChildren();
  for(const p of r.plans||[]){
   const tr=document.createElement('tr');
   const router=state.devices.find(d=>d.id===p.router_id);
   cell(tr,(router?.display_name||p.router_id)+' · '+p.pop_code+' / '+p.site_code);
   cell(tr,p.item_count);cell(tr,p.state);
   cell(tr,p.requested_by+(p.reviewed_by?' / '+p.reviewed_by:''));
   cell(tr,'rate '+p.rate_limit_per_minute+'/min · execution gate '+p.execution_allowed+
     ' · physical readback '+p.physical_readback_verified+' · generic RouterOS adapter disabled');
   const actions=cell(tr,'');actions.textContent='';
   btn(actions,'View',async()=>{try{
    const x=await api('/api/v1/pppoe-plans/'+encodeURIComponent(p.id));
    const detail=$('pppoe-plan-items');detail.replaceChildren();
    for(const item of x.items||[]){const line=document.createElement('p');
      line.textContent=item.ordinal+' · '+item.subscriber_id+' · '+item.action+
       ' · before '+(item.before_username||'—')+' · desired '+(item.desired_username||'—')+
       ' · profile '+(item.profile||'—')+' · secret reference present '+item.has_secret_ref;
      detail.append(line)}
    $('pppoe-plan-detail').hidden=false;
    notice('Dry-run detail loaded. Secret references and values are not returned.');
   }catch(e){notice(e.message)}});
   if(p.state==='awaiting_approval'&&p.can_review){
    btn(actions,'Approve dry-run',async()=>{try{await api('/api/v1/pppoe-plans/'+encodeURIComponent(p.id)+'/review','POST',{approve:true});await refreshPppoePlans();notice('Dry-run approved by Security Admin. A distinct current System Admin plus fresh physical readiness is still required.')}catch(e){notice(e.message)}});
    btn(actions,'Reject',async()=>{try{await api('/api/v1/pppoe-plans/'+encodeURIComponent(p.id)+'/review','POST',{approve:false});await refreshPppoePlans();notice('Dry-run rejected.')}catch(e){notice(e.message)}});
   }else if(p.state==='awaiting_approval')actions.append(document.createTextNode(' Awaiting current Security Admin'));
   if(p.state==='approved'&&p.can_arm){
    btn(actions,'Arm reviewed execution',async()=>{try{
      if(!confirm('Arm this approved PPPoE batch only if current physical API-SSL readback and tested recovery evidence exist? No generic RouterOS command is exposed.'))return;
      const x=await api('/api/v1/pppoe-plans/'+encodeURIComponent(p.id)+'/arm','POST',{confirm:'ARM_REVIEWED_PPPOE_EXECUTION'});
      await refreshPppoePlans();
      notice('Execution safety state armed as '+x.state+'. Worker transport remains separately restricted; no generic RouterOS command was sent by this dashboard.');
    }catch(e){notice(e.message)}});
   }
   if(p.state==='approved'||p.execution_allowed){
    btn(actions,'Execution status',async()=>{try{
      const x=await api('/api/v1/pppoe-plans/'+encodeURIComponent(p.id)+'/execution');
      const a=x.attempt;
      notice(a?('Execution '+a.state+' · pending '+a.pending_count+' · unknown '+a.unknown_count+
        ' · verified '+a.verified_count+' · physical adapter '+a.physical_execution_adapter_enabled)
        :'No execution attempt exists. Physical readiness and System Admin release are required.');
    }catch(e){notice(e.message)}});
   }
   rows.append(tr);
  }
 }
 async function loadPppoeWorkspace(){
  await refreshDevices();refreshPppoeRouterChoices();await refreshPppoePlans();
 }
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
   if(x.kind==='request'&&x.state==='PENDING'&&!x.can_review)actions.textContent='Awaiting a different administrator';
   if(x.kind==='request'&&x.state==='PENDING'&&x.can_review&&x.request_id){
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
 async function page(name){if(!['pops','sites','devices','domains','subscribers','diagnostics','pppoe','noc-access','noc'].includes(name)||$('nav-'+name)?.hidden)return;state.page=name;$('welcome').hidden=true;document.querySelectorAll('[data-panel]').forEach(p=>p.hidden=p.dataset.panel!==name);notice('Loading current tenant data…');try{if(name==='pops')await refreshPops();if(name==='sites'){await refreshPops();await refreshSites()}if(name==='devices'){await refreshPops();await refreshSites();await refreshDevices()}if(name==='domains')await refreshDomains();if(name==='subscribers')await loadSubscriberWorkspace();if(name==='diagnostics')await refreshSubscribers();if(name==='pppoe')await loadPppoeWorkspace();if(name==='noc-access'){await refreshPops();await refreshNocAccess()}if(name==='noc')await refreshNoc();notice('Data loaded. Operations remain subject to current authorization.')}catch(e){notice(e.message)}}
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
  if(hours<1||hours>2160)throw Error('Duration must be 1–2160 hours.');
  const fingerprint=JSON.stringify([member.issuer,member.subject,pop,hours]);
  if(!state.pendingNocRequest||state.pendingNocRequest.fingerprint!==fingerprint){
   state.pendingNocRequest={fingerprint,request_id:crypto.randomUUID(),
    expires_at:new Date(Date.now()+hours*3600000).toISOString()};
  }
  // Preserve both idempotency key and absolute expiry after network timeout.
  // An uncertain response can safely be retried without shifting its expiry.
  await api('/api/v1/noc-access/requests','POST',{request_id:state.pendingNocRequest.request_id,
    target_issuer:member.issuer,target_subject:member.subject,pop_code:pop,
    expires_at:state.pendingNocRequest.expires_at});
  state.pendingNocRequest=null;
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
 $('subscriber-cancel').addEventListener('click',clearSubscriberEdit);
 $('subscriber-form').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{
  const d=Object.fromEntries(new FormData(f));for(const k of ['pppoe_username','access_device_id','ont_reference'])if(!d[k])d[k]=null;
  d.expected_revision=state.subscriberEdit?.revision||0;await api('/api/v1/subscribers','POST',d);clearSubscriberEdit();await refreshSubscribers();
  notice('Subscriber 360 metadata saved. No device or PPPoE configuration changed.');
 })});
 $('pppoe-plan-form').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{
  const router_id=$('pppoe-router').value;if(!router_id)throw Error('Select a registered MikroTik API-SSL router.');
  const items=parsePppoeOperations($('pppoe-operations').value);
  const fingerprint=JSON.stringify([router_id,items]);
  if(!state.pendingPppoe||state.pendingPppoe.fingerprint!==fingerprint)
    state.pendingPppoe={fingerprint,request_id:crypto.randomUUID()};
  const r=await api('/api/v1/pppoe-plans','POST',{request_id:state.pendingPppoe.request_id,router_id,items});
  state.pendingPppoe=null;$('pppoe-operations').value='';await refreshPppoePlans();
  notice('Dry-run '+r.id+' created. A different Tenant Admin must approve it; physical execution remains disabled.');
 })});
 $('diagnostic-form').addEventListener('submit',e=>{e.preventDefault();submit(e.currentTarget,runDiagnostic)});
 $('domain-create').addEventListener('submit',e=>{e.preventDefault();const f=e.currentTarget;submit(f,async()=>{await api('/api/v1/domains','POST',Object.fromEntries(new FormData(f)));clearForm(f);await refreshDomains();notice('Domain saved as pending. Open DNS instructions for unique TXT proof and the reviewed routing plan.')})});
 $('logout').addEventListener('click',async()=>{try{await api('/api/v1/logout','POST');location.assign('/auth/oidc/start')}catch(e){notice(e.message)}});
 (async()=>{try{
  const r=await api('/api/v1/capabilities');$('identity').textContent='Authorized tenant: '+r.tenant_id;
  state.nocPops=Array.isArray(r.noc_pops)?r.noc_pops.filter(p=>typeof p==='string'):[];
  state.realNocPops=Array.isArray(r.real_noc_pops)?r.real_noc_pops.filter(p=>typeof p==='string'):[];
  $('noc-pop').replaceChildren();
  for(const p of state.realNocPops)option($('noc-pop'),'Real POP: '+p,'real:'+p);
  for(const p of state.nocPops)option($('noc-pop'),'Legacy exact Site: '+p,'site:'+p);
  if(state.nocPops.length||state.realNocPops.length)$('nav-noc').hidden=false;
  state.canReadSubscribers=r.can_read_subscribers===true;
  state.canManageSubscribers=r.can_manage_subscribers===true;
  state.canManagePppoePlans=r.can_manage_pppoe_plans===true;
  state.canCreatePppoePlans=r.can_create_pppoe_plans===true;
  state.canReviewPppoePlans=r.can_review_pppoe_plans===true;
  state.canArmPppoePlans=r.can_arm_pppoe_plans===true;
  if(state.canManagePppoePlans)$('nav-pppoe').hidden=false;
  $('pppoe-create-workspace').hidden=!state.canCreatePppoePlans;
  if(state.canReadSubscribers){$('nav-subscribers').hidden=false;$('nav-diagnostics').hidden=false}
  $('subscriber-manage').hidden=!state.canManageSubscribers;
  if(r.can_manage_sites){for(const n of ['pops','sites','devices','domains','noc-access'])$('nav-'+n).hidden=false;await loadCatalog()}
  $('logout').hidden=false;
  notice(r.can_manage_sites||state.canReadSubscribers||state.nocPops.length||state.realNocPops.length?'Tenant permissions verified. Select a permitted module.':'No operations modules assigned to your current identity.');

 }catch(e){notice(e.message);$('signin').hidden=false;$('identity').textContent='Sign-in required'}})();
})();
