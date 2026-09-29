'use strict';
// R9.45: one-time encrypted credential enrollment for the PRIVATE laboratory
// C320. No browser storage, arbitrary CLI, device IP selection or public route.
(() => {
  const root = document.getElementById('ipat-c320-enroll-form');
  if (!root) return;
  const state = document.getElementById('ipat-c320-enroll-state');
  const result = document.getElementById('ipat-c320-enroll-result');
  const button = document.getElementById('ipat-c320-enroll-button');
  const bootstrap = document.getElementById('ipat-c320-bootstrap');
  const password = document.getElementById('ipat-c320-device-password');
  let busy = false;
  function label(text) { state.textContent = text; }
  function update(v) {
    if (v.target !== 'DEV-01' || v.model !== 'C320' || v.production_adopted !== false
      || v.physical_writes_enabled !== false) throw new Error('Kontrak status perangkat tidak sesuai');
    if (v.credentials_enrolled === true) {
      const last = typeof v.last_verified_at_utc === 'string' && v.last_verified_at_utc
        ? ' · pembacaan terakhir ' + v.last_verified_at_utc
        : ' · belum ada pembacaan setelah layanan dimulai';
      label('KREDENSIAL TERSIMPAN' + last);
      root.hidden = true;
      result.textContent = 'Akses langsung dari VPS telah disiapkan. Gunakan tombol baca OLT untuk hasil terkini; perubahan konfigurasi tetap terkunci.';
    } else {
      label(v.connector_online === true ? 'SIAP DIDAFTARKAN' : 'KONEKTOR SERVER BELUM AKTIF');
      root.hidden = false;
      button.disabled = busy || v.connector_online !== true;
    }
  }
  async function refresh() {
    try {
      const response = await fetch('/lab/c320-owner-connection',{
        cache:'no-store',credentials:'omit'
      });
      if (!response.ok) throw new Error('Endpoint perangkat tidak tersedia');
      update(await response.json());
    } catch {
      label('KONEKTOR TIDAK TERSEDIA');
      if (!busy) button.disabled = true;
      if (!busy) result.textContent =
        'Konektor belum dipasang atau jalur privat Mac ke VPS tidak tersedia. Tidak ada status adopted yang diandaikan.';
    }
  }
  root.addEventListener('submit',async(event) => {
    event.preventDefault();
    if (busy || button.disabled) return;
    busy = true;
    button.disabled = true;
    result.textContent = 'Memverifikasi autentikasi dan membaca kartu fisik C320 dari VPS...';
    const oneTime = bootstrap.value;
    const credential = password.value;
    // Never retain credentials in HTML fields or client storage after submission.
    bootstrap.value = '';
    password.value = '';
    try {
      if (!oneTime || !credential) throw new Error('Kode pemilik dan password wajib diisi.');
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(),60000);
      let response;
      try {
        response = await fetch('/lab/c320-owner-enroll',{
          method:'POST',credentials:'omit',cache:'no-store',
          headers:{'Content-Type':'application/json','X-IPAT-Demo-Only':'1'},
          body:JSON.stringify({device_profile:'zte_c320_lab',
            bootstrap_code:oneTime,password:credential}),
          signal:controller.signal
        });
      } finally { clearTimeout(timer); }
      if (!response.ok) throw new Error(response.status === 403
        ? 'Kode pemilik tidak sesuai, bukti perangkat gagal, atau akun sudah didaftarkan.'
        : 'Koneksi belum berhasil (HTTP '+response.status+').');
      const value = await response.json();
      if (value.enrolled_for_read !== true || value.production_adopted !== false
        || value.physical_writes_enabled !== false
        || value.adoption_state !== 'READ_ONLY_CONNECTED_LAB'
        || !Number.isSafeInteger(value.card_count) || value.card_count < 1) {
        throw new Error('Verifikasi fisik atau kontrak keamanan gagal.');
      }
      result.textContent = 'PERANGKAT TERHUBUNG UNTUK BACA: '
        + value.card_count + ' kartu aktif diverifikasi, '+value.verified_at_utc
        + '. Anda sekarang dapat menggunakan tombol baca tanpa autentikasi Terminal.';
      await refresh();
    } catch (error) {
      result.textContent = 'BELUM TERHUBUNG: '+error.message
        +' Jangan menganggap perangkat adopted bila verifikasi gagal.';
    } finally {
      busy = false;
      await refresh();
    }
  });
  refresh();
  setInterval(refresh,15000);
})();
