'use strict';
/* Owner-private staging: NEVER claim this is an authenticated public admin. */
(()=>{
 const el=id=>document.getElementById(id);
 let state=null; let busy=false;
 const api='/lab/owner/platform-domains';
 async function call(path,options={}){
   const controller=new AbortController();const timeout=setTimeout(()=>controller.abort(),9000);
   try{
     const response=await fetch(path,{credentials:'omit',cache:'no-store',...options,
       headers:options.method?{'Content-Type':'application/json','X-IPAT-Owner-Private':'1'}:{},
       signal:controller.signal});
     const body=await response.json();
     if(!response.ok)throw Error(body.error||'HTTP '+response.status);
     return body;
   }finally{clearTimeout(timeout);}
 }
 function add(parent,tag,content){const node=document.createElement(tag);node.textContent=String(content);parent.append(node);return node;}
 function action(parent,title,handler){const b=add(parent,'button',title);b.type='button';b.className='secondary';b.addEventListener('click',handler);return b;}
 function render(){
   if(!state)return;
   const profile=state.platform;
   el('ipat-domain-platform-host').value=profile.hostname||'';
   el('ipat-domain-public-ip').value=profile.temporary_public_ipv4||'';
   el('ipat-domain-gates').textContent='Public access: '+profile.access_mode+' · HTTPS: '+(profile.public_https_ready?'ready':'not ready')
       +' · Verified ownership: '+(profile.dns_ownership_verified?'yes':'no')+' · Tenant routing: '+(profile.ingress_tenant_routing_ready?'ready':'not ready');
   el('ipat-domain-ingress-status').textContent=profile.ingress_tenant_routing_ready?'VERIFIED':'NOT DEPLOYED';
   el('ipat-domain-tls-status').textContent=profile.public_https_ready?'READY':'NOT READY';
   el('ipat-domain-dns-status').textContent=profile.dns_ownership_verified?'VERIFIED':'NOT VERIFIED';
   const table=el('ipat-domain-draft-rows');table.replaceChildren();
   if(!state.drafts.length){const tr=document.createElement('tr');const td=add(tr,'td','No domain plans registered.');td.colSpan=5;table.append(tr);}
   for(const d of state.drafts){
     const tr=document.createElement('tr');
     for(const value of [d.hostname,d.tenant_reference,d.kind.replaceAll('_',' '),'PLANNED · NOT ROUTED'])add(tr,'td',value);
     const td=document.createElement('td');action(td,'Preview DNS',()=>preview(d));
     action(td,'Remove',()=>remove(d));tr.append(td);table.append(tr);
   }
 }
 async function load(){
   try{const next=await call(api);
     if(next.owner_private_lab_only!==true||next.persisted!==true||next.platform?.safe_to_point_now!==false
        ||next.platform?.runtime_changes_applied!==false||!Array.isArray(next.drafts))throw Error('Invalid owner-only domain API');
     state=next;render();el('ipat-domain-profile-status').textContent='Configuration loaded from the private owner VPS. Public changes have NOT been applied.';
   }catch(e){el('ipat-domain-profile-status').textContent='Cannot load domain settings: '+e.message;}
 }
 async function stage(e){
   e.preventDefault();if(busy||!state)return;
   const name=el('ipat-domain-platform-host').value.trim().toLowerCase()||null;
   const ip=el('ipat-domain-public-ip').value.trim()||null;
   if(!window.confirm('Save a PRIVATE planning configuration only? This does not open public access, issue certificates, change DNS or change the server firewall.'))return;
   busy=true;el('ipat-domain-profile-save').disabled=true;
   try{
     const next=await call(api,{method:'PUT',body:JSON.stringify({
       expected_revision:state.platform.revision,platform_hostname:name,public_ipv4:ip,
       confirm:'STAGE_DOMAIN_CONFIG'
     })});
     if(next.platform?.runtime_changes_applied!==false)throw Error('Unexpected activation response');
     state=next;render();el('ipat-domain-profile-status').textContent='Draft saved. No public routing or HTTPS change has been performed.';
   }catch(e){el('ipat-domain-profile-status').textContent='Cannot save: '+e.message;}
   finally{busy=false;el('ipat-domain-profile-save').disabled=false;}
 }
 async function request(e){
   e.preventDefault();if(busy)return;
   const kind=el('ipat-domain-kind').value;
   const tenant=el('ipat-domain-tenant-ref').value.trim().toLowerCase();
   const custom=kind==='custom_domain'?el('ipat-domain-custom-host').value.trim().toLowerCase():null;
   busy=true;el('ipat-domain-request-save').disabled=true;
   try{
     const result=await call(api+'/drafts',{method:'POST',body:JSON.stringify({tenant_reference:tenant,kind,custom_hostname:custom})});
     if(result.state!=='PLANNED_NOT_ROUTED'||result.public_route_changed!==false)throw Error('Unexpected domain state');
     await load();
     el('ipat-domain-request-status').textContent='Planning request saved. No tenant authorization, DNS proof, TLS or public route has been granted.';
     el('ipat-domain-preview').hidden=true;
     el('ipat-domain-request-form').reset();toggleKind();
   }catch(e){el('ipat-domain-request-status').textContent='Cannot create request: '+e.message;}
   finally{busy=false;el('ipat-domain-request-save').disabled=false;}
 }
 function toggleKind(){
   const isCustom=el('ipat-domain-kind').value==='custom_domain';
   el('ipat-domain-custom-wrap').hidden=!isCustom;
   el('ipat-domain-custom-host').required=isCustom;
 }
 async function preview(d){
   try{const data=await call(api+'/drafts/'+encodeURIComponent(d.id)+'/preview');
     if(data.safe_to_point_now!==false||data.public_route_changed!==false||data.ownership_txt_value!==null)throw Error('Invalid domain readiness');
     const view=el('ipat-domain-preview');view.hidden=false;
     view.textContent=['PREVIEW ONLY — DO NOT POINT DNS YET','Hostname: '+data.hostname,
       'Proposed A record: '+(data.suggested_a_record.value||'Public IPv4 not configured'),
       'Required future TXT name: '+data.ownership_txt_name,
       'Verification TXT token: NOT ISSUED — pending authorized tenant ownership workflow',
       'DNS verified: NO · TLS ready: NO · Routing active: NO',data.warning].join('\n');
   }catch(e){el('ipat-domain-request-status').textContent='Preview failed: '+e.message;}
 }
 async function remove(d){
   if(!window.confirm('Remove the unactivated planning record '+d.hostname+'? This will not touch DNS or tenants.'))return;
   try{const result=await call(api+'/drafts/'+encodeURIComponent(d.id),{method:'DELETE',
       body:JSON.stringify({expected_revision:d.revision,confirm:'REMOVE DRAFT '+d.id})});
     if(result.removed!==true||result.public_route_changed!==false)throw Error('Unexpected removal response');
     await load();el('ipat-domain-preview').hidden=true;
   }catch(e){el('ipat-domain-request-status').textContent='Cannot remove draft: '+e.message;}
 }
 el('ipat-domain-profile-form').addEventListener('submit',stage);
 el('ipat-domain-request-form').addEventListener('submit',request);
 el('ipat-domain-kind').addEventListener('change',toggleKind);
 el('ipat-domain-refresh').addEventListener('click',load);
 toggleKind();load();
})();
