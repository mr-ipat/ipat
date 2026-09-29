'use strict';
// Private historic hardware evidence, never a live command or live poll.
(async function () {
  const output=document.getElementById('c320-real-inventory');
  const status=document.getElementById('c320-real-inventory-status');
  if(!output||!status)return;
  try {
    const reply=await fetch('/lab/c320-first-real-inventory',{
      cache:'no-store',credentials:'omit'
    });
    if(!reply.ok)throw new Error('no private historical evidence');
    const r=await reply.json();
    if(r.target!=='DEV-01'||r.mode!=='ACTUAL_HISTORICAL_OWNER_LAB_PHYSICAL_READ_NO_WORKER'
      ||r.recorded_date!=='2026-09-29'||r.observation_is_live!==false
      ||r.owner_restic_isolated_byte_identical_restore_verified!==true
      ||r.device_native_restore_rehearsed!==false
      ||r.actual_one_shot_lab_scripted_ssh_read_verified!==true
      ||r.actual_one_shot_scripted_cards_matched!==3
      ||r.scripted_read_was_unattended_production_worker!==false
      ||r.real_saas_device_adopted!==false||r.production_worker_enabled!==false
      ||r.dedicated_verified_limited_role_account_exists!==false
      ||!Array.isArray(r.cards)||r.cards.length!==3
      ||!r.cards.every(c=>c.card_reported_status==='INSERVICE'))
      throw new Error('unverifiable server physical evidence');
    const expected=[['1/1/1','GTGHK','GTXK'],['1/1/3','PRAM',null],['1/1/4','SMXA','SMXA']];
    if(r.cards.some((c,i)=>c.slot!==expected[i][0]
      ||c.physical!==expected[i][1]
      ||c.reported_mvr_filetype!==expected[i][2]))throw new Error('unreviewed vendor layout');
    output.replaceChildren();
    for(const c of r.cards){
      const row=document.createElement('div');row.className='physical-gate';
      const version=c.reported_mvr_version===null?'MVR belum terlapor':
        (c.mvr_card_alias_verified?'MVR '+c.reported_mvr_version:'MVR tipe berbeda: BELUM TERKONFIRMASI');
      for(const value of [c.slot,c.physical+' · '+c.card_reported_status,version]){
        const span=document.createElement('span');span.textContent=value;row.append(span);
      }
      output.append(row);
    }
    status.textContent='LAB 29 Sep: SSH satu kali berhasil, 3 kartu cocok, referensi Restic pulih identik. BUKAN polling otomatis atau adopsi.';
    try {
      const gateReply=await fetch('/lab/c320-action-readiness',{cache:'no-store',credentials:'omit'});
      if(!gateReply.ok)throw new Error('gate report unavailable');
      const gateData=await gateReply.json();
      const g=gateData.adoption_gate_report;
      if(!g||g.mode!=='HISTORICAL_LAB_DISPLAY_NOT_AUTHORIZATION'
        ||g.required_gate_count!==10||g.verified_gate_count!==2
        ||g.all_gates_verified!==false||gateData.device_adopted!==false
        ||gateData.worker_enabled!==false||!Array.isArray(g.gates)||g.gates.length!==10)
        throw new Error('gate report invalid');
      const missing=g.gates.filter(item=>item.verified!==true).map(item=>item.gate);
      if(missing.length!==8)throw new Error('unexpected physical gate state');
      const gateLine=document.createElement('div');
      gateLine.textContent='ADOPSI DITAHAN ('+g.verified_gate_count+'/'+g.required_gate_count+'). Persyaratan belum terpenuhi: '+missing.join(', ')+'. Registrasi ONT nyata belum diizinkan.';
      status.append(gateLine);
      const capabilities=await fetch('/lab/c320-ont-feature-readiness',{
        cache:'no-store',credentials:'omit'
      });
      if(!capabilities.ok)throw new Error('ONT readiness unavailable');
      const ont=await capabilities.json();
      if(ont.mode!=='PRIVATE_HISTORICAL_ONT_REVIEW_ONLY'
        ||ont.real_hardware_adopted!==false||ont.physical_ont_register_enabled!==false
        ||ont.physical_ont_config_enabled!==false||ont.network_actions!==0
        ||!Array.isArray(ont.available_offline_modules)||ont.available_offline_modules.length!==3)
        throw new Error('invalid ONT capability state');
      const ontLine=document.createElement('div');
      ontLine.textContent='ONT: modul offline penemuan ONU, validasi registrasi satu ONT, dan pemeriksaan profil bridge/VLAN telah disiapkan. Perintah nyata dan interoperabilitas firmware BELUM diuji.';
      status.append(ontLine);
      const snapshotResponse=await fetch('/lab/c320-owner-manual-onu-snapshot',{
        cache:'no-store',credentials:'omit'
      });
      if(!snapshotResponse.ok)throw new Error('manual ONU snapshot unavailable');
      const inventory=await snapshotResponse.json();
      if(inventory.source!=='OWNER_ATTESTED_ACTUAL_MANUAL_TELNET323_CLI'
        ||inventory.snapshot_is_live!==false||inventory.registered_onu_status_rows!==72
        ||inventory.onu_online!==0||inventory.onu_offline!==72
        ||inventory.registered_onu_config_declarations!==72
        ||inventory.unconfigured_onus_reported!==0||inventory.real_olt_adopted!==false)
        throw new Error('owner manual snapshot mismatch');
      const manual=document.createElement('div');
      manual.textContent='Bukti nyata manual C320 ('+inventory.observation_date
        +', BUKAN LIVE): PON '+inventory.pon+', konfigurasi ONU '
        +inventory.registered_onu_config_declarations+', online '
        +inventory.onu_online+', offline '+inventory.onu_offline
        +', ONU baru terdeteksi '+inventory.unconfigured_onus_reported
        +'. Pendaftaran ONT nyata TETAP DIBLOKIR.';
      status.append(manual);
    } catch {
      const gateLine=document.createElement('div');
      gateLine.textContent='Kesiapan adopsi tidak dapat diverifikasi. Semua aksi OLT/ONT tetap diblokir.';
      status.append(gateLine);
    }
  }catch{
    output.replaceChildren();status.textContent='Bukti historis gagal divalidasi; tidak ada klaim inventaris atau adopsi otomatis.';
  }
})();

// Real owner-supervised panel action. Never simulate a successful live refresh.
(() => {
  const button=document.getElementById('c320-read-live');
  const status=document.getElementById('c320-read-live-status');
  const summary=document.getElementById('c320-read-live-summary');
  if(!button||!status||!summary)return;
  button.addEventListener('click',async()=>{
    button.disabled=true;
    status.textContent='Menjalankan pembacaan fisik terbatas via agen pemilik…';
    summary.replaceChildren();
    try {
      const res=await fetch('/lab/c320-owner-live-refresh',{
        method:'POST',cache:'no-store',credentials:'omit',
        headers:{'X-IPAT-Demo-Only':'1','Content-Type':'application/json'},
        body:'{}'
      });
      if(!res.ok)throw new Error(res.status===404?'Backend privat belum memasang modul R9.40':
        'Agen pemilik tidak aktif, batas waktu, atau pembacaan gagal ('+res.status+')');
      const r=await res.json();
      if(r.mode!=='OWNER_SUPERVISED_REAL_C320_READ_ONLY'
        ||r.source!=='VERIFIED_LOCAL_OWNER_AGENT_LAB_ONLY'
        ||r.physical_writes_enabled!==false||r.device_adopted!==false
        ||r.serials_returned!==false||!Number.isInteger(r.unconfigured)
        ||!Number.isInteger(r.configured)||!Number.isInteger(r.online)
        ||!Number.isInteger(r.offline)||r.online+r.offline!==r.configured)
        throw new Error('Kontrak pembacaan fisik tidak sah');
      const values=[['PON',r.pon],['ONU terdaftar',String(r.configured)],
        ['Online',String(r.online)],['Offline',String(r.offline)],
        ['ONU belum terdaftar',String(r.unconfigured)]];
      for(const [name,value] of values){
        const line=document.createElement('div');line.className='physical-gate';
        const label=document.createElement('span');label.textContent=name;
        const result=document.createElement('strong');result.textContent=value;
        line.append(label,result);summary.append(line);
      }
      status.textContent='LIVE TERBATAS · '+r.read_at_utc+' · pembacaan aktual selesai. Adopsi produksi dan perubahan konfigurasi tetap terkunci.';
    } catch(e) {
      status.textContent='BELUM ADA BACAAN LIVE: '+e.message+'. Data historis tidak diganti.';
    } finally { button.disabled=false; }
  });
})();

// Two additional FIXED owner-supervised reads; no browser-defined CLI.
(() => {
  for(const config of [
    {button:'c320-read-cards',path:'/lab/c320-owner-live-cards',
      kind:'CARDS',name:'Kartu INSERVICE',field:'cards_in_service'},
    {button:'c320-read-firmware',path:'/lab/c320-owner-live-firmware',
      kind:'FIRMWARE',name:'Baris firmware terbaca (alias belum terverifikasi)',field:'firmware_rows'}
  ]) {
    const button=document.getElementById(config.button);
    const status=document.getElementById('c320-read-live-status');
    const summary=document.getElementById('c320-read-live-summary');
    if(!button||!status||!summary)continue;
    button.addEventListener('click',async()=>{
      button.disabled=true;
      status.textContent='Meminta pembacaan nyata '+config.name+' melalui agen terbatas…';
      try {
        const reply=await fetch(config.path,{method:'POST',cache:'no-store',
          credentials:'omit',headers:{'X-IPAT-Demo-Only':'1',
          'Content-Type':'application/json'},body:'{}'});
        if(!reply.ok)throw new Error('Agen offline, kuota habis, atau hasil firmware ditolak ('+reply.status+')');
        const result=await reply.json();
        if(result.read_kind!==config.kind||result.snapshot_is_live!==true
          ||result.device_adopted!==false||result.physical_writes_enabled!==false
          ||!Number.isInteger(result[config.field])||result[config.field]<1)
          throw new Error('Respons fisik tidak dapat diverifikasi');
        summary.replaceChildren();
        const line=document.createElement('div');line.className='physical-gate';
        const k=document.createElement('span');k.textContent=config.name;
        const v=document.createElement('strong');v.textContent=String(result[config.field]);
        line.append(k,v);summary.append(line);
        status.textContent='Pembacaan REAL '+result.read_at_utc+
          ' · data redaksi, bukan izin upgrade atau registrasi.';
      }catch(e){status.textContent='BELUM BERHASIL: '+e.message;}
      finally{button.disabled=false;}
    });
  }
})();
