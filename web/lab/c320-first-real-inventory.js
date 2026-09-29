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
    } catch {
      const gateLine=document.createElement('div');
      gateLine.textContent='Kesiapan adopsi tidak dapat diverifikasi. Semua aksi OLT/ONT tetap diblokir.';
      status.append(gateLine);
    }
  }catch{
    output.replaceChildren();status.textContent='Bukti historis gagal divalidasi; tidak ada klaim inventaris atau adopsi otomatis.';
  }
})();
