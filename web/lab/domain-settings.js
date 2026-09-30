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
