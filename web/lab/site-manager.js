'use strict';
/* Independent owner-only Site/POP master UI. No device CRUD or backend device
 * requests here; assignment counts and delete guard come from the site API. */
(()=>{
 const el=id=>document.getElementById(id);
 const api='/lab/owner/pops';
 const notice=el('ipat-sites-notice');
 const form=el('ipat-sites-form');
 const filter=el('ipat-sites-filter');
 let sites=[],editing=null,busy=false;
 const backToDevice=new URLSearchParams(window.location.search).get('return')==='device';
 const optionReturn='/lab/device-workbench?new-device=1#managed-devices';
 function textNode(tag,value){const node=document.createElement(tag);node.textContent=String(value);return node;}
 async function request(path,options={}){
  const controller=new AbortController();const timeout=setTimeout(()=>controller.abort(),8000);
  try{
   const res=await fetch(path,{cache:'no-store',credentials:'omit',...options,
    headers:options.method?{'Content-Type':'application/json','X-IPAT-Owner-Private':'1'}:{},signal:controller.signal});
   const data=await res.json();
   if(!res.ok)throw Error(data.error||'HTTP '+res.status);
   return data;
  }finally{clearTimeout(timeout);}
 }
 function render(){
  const rows=el('ipat-sites-rows');rows.replaceChildren();
  const q=filter.value.trim().toLowerCase();
  const visible=sites.filter(s=>[s.code,s.display_name].some(t=>t.toLowerCase().includes(q)));
  el('ipat-sites-count').textContent=sites.length+' registered Site(s) · persistent owner-only master';
  if(!visible.length){const tr=document.createElement('tr');const td=textNode('td',sites.length?'No matching sites.':'No Sites/POPs registered. Use Create Site / POP.');td.colSpan=5;tr.append(td);rows.append(tr);return;}
  for(const site of visible){
   const tr=document.createElement('tr');
   for(const name of [site.code,site.display_name,site.assigned_device_count])tr.append(textNode('td',name));
   const stat=textNode('td',site.can_remove?'Available':'Assigned · protected');
   stat.className=site.can_remove?'ready':'blocked';tr.append(stat);
   const cell=document.createElement('td');
   const edit=textNode('button','Edit');edit.type='button';edit.className='secondary';edit.addEventListener('click',()=>showForm(site));cell.append(edit);
   const remove=textNode('button','Remove');remove.type='button';remove.className='danger';
   remove.disabled=!site.can_remove;
   remove.title=site.can_remove?'Remove unused site':'Cannot remove: move/unassign devices first';
   remove.addEventListener('click',()=>removeSite(site));cell.append(remove);
   tr.append(cell);rows.append(tr);
  }
 }
 async function load(){
  try{
   const data=await request(api);
   if(data.owner_private_lab_only!==true ||data.persistent!==true||!Array.isArray(data.pops)
     ||data.pops.some(p=>typeof p.assigned_device_count!=='number'||typeof p.can_remove!=='boolean'))
     throw Error('Invalid site-master response');
   sites=data.pops;render();notice.textContent=sites.length
    ?'Site master loaded. Manage sites here independently of Device Manager.'
    :'No sites yet. Create the first Site/POP; device registration will then use this list.';
  }catch(e){notice.textContent='Unable to load site master: '+e.message;sites=[];render();}
 }
 function showForm(site=null){
  editing=site;form.reset();
  el('ipat-sites-form-panel').hidden=false;
  el('ipat-sites-form-heading').textContent=site?'Edit Site / POP':'Create Site / POP';
  el('ipat-sites-code').value=site?.code||'';el('ipat-sites-code').disabled=Boolean(site);
  el('ipat-sites-name').value=site?.display_name||'';
  el('ipat-sites-form-notice').textContent='';
  el('ipat-sites-form-panel').scrollIntoView({block:'nearest'});
  el('ipat-sites-name').focus();
 }
 async function save(e){
  e.preventDefault();if(busy||!form.reportValidity())return;
  const code=editing?.code||el('ipat-sites-code').value.trim();
  busy=true;el('ipat-sites-save').disabled=true;
  try{
   const saved=await request(editing?api+'/'+encodeURIComponent(code):api,{
    method:editing?'PUT':'POST',body:JSON.stringify({
      ...(editing?{expected_revision:editing.revision}:{code}),
      display_name:el('ipat-sites-name').value.trim()
    })
   });
   if(!saved.pop||saved.pop.code!==code||saved.durable!==true)throw Error('Invalid site save response');
   editing=null;el('ipat-sites-form-panel').hidden=true;
   busy=false;await load();
   notice.textContent='Site '+code+' saved. '+(backToDevice?'Returning to Device Manager...':'Devices may now be assigned in Device Manager.');
   if(backToDevice){window.location.assign(optionReturn);}
  }catch(e){el('ipat-sites-form-notice').textContent='Site not saved: '+e.message;}
  finally{busy=false;el('ipat-sites-save').disabled=false;}
 }
 async function removeSite(site){
  if(busy ||!site.can_remove ||!window.confirm('Remove unused Site/POP '+site.code+'? This removes the site master record but never commands a device.'))return;
  busy=true;
  try{
   const result=await request(api+'/'+encodeURIComponent(site.code),{
    method:'DELETE',body:JSON.stringify({expected_revision:site.revision,confirm:'REMOVE POP '+site.code})
   });
   if(result.removed!==true)throw Error('Deletion not acknowledged');
   busy=false;await load();notice.textContent='Unused site '+site.code+' removed. No device configuration changed.';
  }catch(e){notice.textContent='Removal rejected: '+e.message;}
  finally{busy=false;}
 }
 el('ipat-sites-new').addEventListener('click',()=>showForm());
 el('ipat-sites-refresh').addEventListener('click',load);
 el('ipat-sites-cancel').addEventListener('click',()=>{el('ipat-sites-form-panel').hidden=true;editing=null;});
 form.addEventListener('submit',save);filter.addEventListener('input',render);
 load().then(()=>{if(backToDevice && !sites.length)showForm();});
})();
