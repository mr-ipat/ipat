'use strict';
// The server enforces role, exact Host, CSRF and current session on every call.
(()=>{
 const $=id=>document.getElementById(id);
 let pending=null;
 const notice=msg=>{$('notice').textContent=msg};
 const csrf=()=>document.cookie.split(';').map(x=>x.trim())
  .find(x=>x.startsWith('__Host-ipat_platform_csrf='))?.split('=')[1];
 async function api(path,method='GET',body){
  const headers={'Accept':'application/json'};
  if(method!=='GET'){
   if(!csrf())throw Error('Platform CSRF cookie missing. Sign in again.');
   headers['X-IPAT-Platform-CSRF']=csrf();
   if(body!==undefined)headers['Content-Type']='application/json';
  }
  const response=await fetch(path,{method,credentials:'same-origin',cache:'no-store',
   headers,body:body===undefined?undefined:JSON.stringify(body)});
  const result=await response.json().catch(()=>({ok:false}));
  if(!response.ok||!result.ok)throw Error(response.status===401
   ?'Session missing, expired or revoked. Sign in using the approved Platform Owner identity.'
   :'Request rejected by current authorization (HTTP '+response.status+').');
  return result;
 }
 async function refresh(){
  const result=await api('/api/v1/platform/reservations');
  const tbody=$('rows');tbody.replaceChildren();
  for(const r of result.reservations){
   const tr=document.createElement('tr');
   for(const value of [r.tenant_slug,r.tenant_state,r.reserved_at,r.tenant_id]){
    const td=document.createElement('td');td.textContent=String(value);tr.append(td);
   }
   tbody.append(tr);
  }
  $('panel').hidden=false;
 }
 $('reserve-form').addEventListener('submit',async event=>{
  event.preventDefault();
  const slug=$('slug').value;
  if(!/^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/.test(slug)){
   notice('Use a valid lower-case slug, 1–63 characters.');return;
  }
  if(!pending||pending.slug!==slug)
   pending={slug,request_id:crypto.randomUUID(),tenant_id:crypto.randomUUID()};
  const button=event.currentTarget.querySelector('button');
  button.disabled=true;
  try{
   await api('/api/v1/platform/reservations','POST',pending);
   pending=null;notice('Reserved as Suspended. No tenant login, DNS or billing has been activated.');
   await refresh();
  }catch(error){notice(error.message+' Retrying the same slug preserves the same request IDs.');}
  finally{button.disabled=false;}
 });
 $('logout').addEventListener('click',async()=>{
  try{await api('/api/v1/platform/logout','POST');$('panel').hidden=true;
   notice('Platform session ended.');}catch(error){notice(error.message);}
 });
 refresh().then(()=>notice('Current Platform Owner authorization verified.'))
  .catch(error=>{ $('panel').hidden=true;notice(error.message);});
})();
