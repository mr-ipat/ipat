"use strict";
const byId=(id)=>document.getElementById(id);

function row(record){
  const item=document.createElement("div");
  item.className="record";
  const type=document.createElement("span");
  type.className="record-type";
  type.textContent=record.record_type;
  const name=document.createElement("code");
  name.textContent=record.name;
  const value=document.createElement("code");
  value.textContent=record.value;
  const stage=document.createElement("small");
  stage.textContent=record.stage==="after_ownership_verification"
    ?"Terapkan setelah ownership verification"
    :record.stage;
  item.append(type,name,value,stage);
  return item;
}

async function showInstructions(){
  const button=byId("show-instructions");
  const status=byId("domain-status");
  const records=byId("dns-records");
  const hostname=byId("domain-hostname").value.trim();
  button.disabled=true;
  status.textContent="Meminta profil DNS dari backend IPAT...";
  records.replaceChildren();
  try{
    const response=await fetch("/v1/domains/instructions",{
      method:"POST",
      cache:"no-store",
      credentials:"omit",
      headers:{"content-type":"application/json"},
      body:JSON.stringify({hostname})
    });
    const data=await response.json();
    if(!response.ok) throw new Error(data.error||"DNS_PROFILE_UNAVAILABLE");
    if(data.authorization_granted!==false || data.activation_requires_verification!==true
      || data.verification_record_type!=="TXT"
      || data.routing_target_known!==true
      || typeof data.routing_ready!=="boolean"
      || typeof data.safe_to_point_now!=="boolean"
      || !Array.isArray(data.routing_records) || !data.routing_records.length){
      throw new Error("INVALID_BACKEND_RESPONSE");
    }
    byId("routing-mode").textContent=data.routing_mode.replaceAll("_"," ").toUpperCase();
    byId("routing-mode").className="pill";
    byId("verify-name").textContent=data.verification_record_name;
    byId("verify-value").textContent=data.verification_value_issued_after_save
      ?"Dibuat setelah Save oleh backend tenant-admin"
      :"Tidak tersedia";
    const fragment=document.createDocumentFragment();
    for(const record of data.routing_records) fragment.append(row(record));
    records.replaceChildren(fragment);
    status.textContent=data.safe_to_point_now
      ?"Target DNS tersedia dan runtime routing siap. Tetap selesaikan ownership verification sebelum aktivasi."
      :"Target DNS sudah diketahui, tetapi JANGAN POINTING DULU: ingress/TLS/authoritative DNS belum dinyatakan siap oleh deployment IPAT.";
  }catch(error){
    byId("routing-mode").textContent="BELUM TERSEDIA";
    byId("routing-mode").className="pill muted";
    byId("verify-name").textContent="_ipat-verify.<domain>";
    byId("verify-value").textContent="Dibuat setelah Save";
    const p=document.createElement("p");
    p.className="empty";
    p.textContent="Profil DNS runtime belum aktif atau domain tidak valid. Tidak ada perubahan DNS dilakukan.";
    records.replaceChildren(p);
    status.textContent="Instruksi DNS belum dapat diterbitkan: "+String(error.message||error);
  }finally{
    button.disabled=false;
  }
}
byId("show-instructions").addEventListener("click",()=>void showInstructions());


let lastInstruction=null;
let domainWriteCapability=false;

function stateLabel(state){
  return ({
    pending_dns:"PENDING DNS",
    ownership_verified:"OWNERSHIP VERIFIED",
    routing_ready:"ROUTING READY",
    tls_ready:"TLS READY",
    active:"ACTIVE",
    disabled:"DISABLED"
  })[state]||"UNKNOWN";
}

function setSaveCapability(enabled){
  domainWriteCapability=enabled===true;
  const button=byId("save-domain");
  button.disabled=!domainWriteCapability || !lastInstruction;
  button.title=domainWriteCapability
    ?"Simpan domain sebagai Tenant Admin"
    :"Authenticated Tenant Admin BFF belum tersedia";
}

async function refreshDomains(){
  const status=byId("domain-list-status");
  const body=byId("domain-list");
  try{
    const response=await fetch("/v1/tenant/domains",{
      method:"GET",cache:"no-store",credentials:"same-origin",
      headers:{"accept":"application/json"}
    });
    if(response.status===401 || response.status===403 || response.status===404 || response.status===503){
      setSaveCapability(false);
      status.textContent="Domain BFF write belum aktif pada runtime ini. Instruksi DNS tetap dapat dipreview tanpa memberi otorisasi.";
      body.innerHTML='<tr><td colspan="5" class="empty">Login Tenant Admin / domain BFF belum tersedia.</td></tr>';
      return;
    }
    const data=await response.json();
    if(!response.ok || data.tenant_admin!==true || !Array.isArray(data.domains))
      throw new Error("INVALID_DOMAIN_LIST");
    setSaveCapability(data.write_available===true);
    body.replaceChildren();
    if(!data.domains.length){
      body.innerHTML='<tr><td colspan="5" class="empty">Belum ada custom domain tersimpan.</td></tr>';
    }else{
      for(const domain of data.domains){
        const tr=document.createElement("tr");
        for(const value of [
          domain.hostname,
          domain.routing_mode||"—",
          stateLabel(domain.activation_state),
          domain.last_checked_at||"Belum diperiksa",
          domain.last_error_code||"—"
        ]){
          const td=document.createElement("td");
          td.textContent=value;
          tr.append(td);
        }
        body.append(tr);
      }
    }
    status.textContent=data.write_available
      ?"BFF Tenant Admin aktif. Save/List domain menggunakan session dan current membership."
      :"Daftar tersedia read-only; write belum diaktifkan.";
  }catch{
    setSaveCapability(false);
    status.textContent="Status domain tidak dapat diverifikasi; tidak ada perubahan yang dilakukan.";
    body.innerHTML='<tr><td colspan="5" class="empty">Domain inventory unavailable.</td></tr>';
  }
}

async function saveDomain(){
  const status=byId("domain-status");
  if(!domainWriteCapability || !lastInstruction){
    status.textContent="Save ditahan: authenticated Tenant Admin BFF belum aktif.";
    return;
  }
  const csrf=document.querySelector('meta[name="ipat-csrf"]')?.content||"";
  if(!csrf){
    status.textContent="Save ditahan: CSRF session token tidak tersedia.";
    return;
  }
  const button=byId("save-domain");
  button.disabled=true;
  try{
    const response=await fetch("/v1/tenant/domains",{
      method:"POST",cache:"no-store",credentials:"same-origin",
      headers:{"content-type":"application/json","x-csrf-token":csrf},
      body:JSON.stringify({
        hostname:lastInstruction.hostname,
        routing_mode:lastInstruction.routing_mode
      })
    });
    const data=await response.json();
    if(!response.ok || data.saved!==true || typeof data.verification_value!=="string")
      throw new Error(data.error||"DOMAIN_SAVE_REJECTED");
    byId("verify-name").textContent=data.verification_name;
    byId("verify-value").textContent=data.verification_value;
    status.textContent="Domain tersimpan sebagai PENDING DNS. Tambahkan TXT verifikasi; IPAT belum menganggap domain aktif.";
    await refreshDomains();
  }catch(error){
    status.textContent="Domain tidak tersimpan: "+String(error.message||error);
  }finally{
    setSaveCapability(domainWriteCapability);
  }
}

const originalShow=showInstructions;
showInstructions=async function(){
  lastInstruction=null;
  setSaveCapability(domainWriteCapability);
  await originalShow();
  const hostname=byId("domain-hostname").value.trim();
  try{
    const response=await fetch("/v1/domains/instructions",{
      method:"POST",cache:"no-store",credentials:"omit",
      headers:{"content-type":"application/json"},
      body:JSON.stringify({hostname})
    });
    const data=await response.json();
    if(response.ok && data.routing_target_known===true){
      lastInstruction={hostname:data.hostname,routing_mode:data.routing_mode};
    }
  }catch{}
  setSaveCapability(domainWriteCapability);
};

byId("save-domain").addEventListener("click",()=>void saveDomain());
byId("refresh-domains").addEventListener("click",()=>void refreshDomains());
void refreshDomains();
