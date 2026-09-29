'use strict';
/* IPAT R9.44: operator-first, truthful ZTE C320 GUI over EXISTING protected
   private laboratory APIs. No arbitrary CLI, secrets, physical writes or fake ONUs.
   This module complements the legacy preview while the new production BFF,
   tenant roles and permanent restricted collector remain separate milestones. */
(() => {
  const root = document.getElementById('ipat-c320-console');
  if (!root) return;
  const el = (id) => document.getElementById(id);
  const buttons = {
    inventory: el('ipat-c320-read-onus'),
    cards: el('ipat-c320-read-cards'),
    firmware: el('ipat-c320-read-firmware')
  };
  const fields = {
    configured: el('ipat-c320-configured'), online: el('ipat-c320-online'),
    offline: el('ipat-c320-offline'), unconfigured: el('ipat-c320-unconfigured'),
    cards: el('ipat-c320-cards'), firmware: el('ipat-c320-firmware')
  };
  const state = { ready: false, busy: false, busyDevice: false, latest: null };
  const stamp = el('ipat-c320-observation-stamp');
  const status = el('ipat-c320-console-status');
  const connectivity = el('ipat-c320-connectivity');
  const rows = el('ipat-c320-events');
  const writeNotice = el('ipat-c320-write-guard');
  const safeInteger = (n) => Number.isSafeInteger(n) && n >= 0 && n <= 128;
  const display = (node, value) => { if (node) node.textContent = String(value); };
  const record = (event) => {
    const item = document.createElement('li');
    item.textContent = new Date().toLocaleTimeString('id-ID') + ' · ' + event;
    rows.prepend(item);
    while (rows.children.length > 8) rows.lastChild.remove();
  };
  function disableIfNeeded() {
    for (const button of Object.values(buttons)) {
      button.disabled = !state.ready || state.busy || state.busyDevice;
    }
  }
  function errorMessage(code) {
    if (code === 403) return 'Ditolak: buka panel privat di Mac dan gunakan sesi yang berwenang.';
    if (code === 404) return 'Modul dashboard/backend belum dipasang pada server ini.';
    if (code === 503) return 'Agen tidak tersedia atau pembacaan fisik ditolak. Periksa terminal pemilik.';
    return 'Gagal (HTTP ' + code + '). Jangan anggap OLT online.';
  }
  async function jsonResponse(path, init = {}) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), init.method === 'POST' ? 60000 : 5000);
    try {
      const response = await fetch(path, {
        credentials: 'omit', cache: 'no-store', ...init, signal: controller.signal
      });
      if (!response.ok) throw new Error(errorMessage(response.status));
      return await response.json();
    } finally {
      clearTimeout(timer);
    }
  }
  function showHistorical(v) {
    if (!v || v.source !== 'OWNER_ATTESTED_ACTUAL_MANUAL_TELNET323_CLI'
      || v.snapshot_is_live !== false || v.real_olt_adopted !== false
      || v.pon !== '1/1/1' || !safeInteger(v.registered_onu_status_rows)
      || v.registered_onu_status_rows !== v.registered_onu_config_declarations
      || !safeInteger(v.onu_online) || !safeInteger(v.onu_offline)
      || !safeInteger(v.unconfigured_onus_reported)
      || v.onu_online + v.onu_offline !== v.registered_onu_status_rows) {
      throw new Error('Bukti inventaris manual tidak memenuhi kontrak.');
    }
    display(fields.configured, v.registered_onu_status_rows);
    display(fields.online, v.onu_online);
    display(fields.offline, v.onu_offline);
    display(fields.unconfigured, v.unconfigured_onus_reported);
    stamp.textContent = 'MANUAL · ' + v.observation_date + ' · bukan data terkini';
    record('Inventaris PON 1/1/1 historis dimuat dan tervalidasi.');
  }
  async function loadHistorical() {
    try {
      const v = await jsonResponse('/lab/c320-owner-manual-onu-snapshot');
      showHistorical(v);
    } catch (error) {
      stamp.textContent = 'Data historis tidak tersedia / tidak dapat diverifikasi';
      record(error.message);
    }
  }
  async function refreshAgent() {
    try {
      const v = await jsonResponse('/lab/c320-owner-agent-state');
      if (v.device_adopted !== false || v.physical_writes_enabled !== false
        || typeof v.agent_ready !== 'boolean') throw new Error('Kontrak status agen ditolak.');
      state.ready = v.agent_ready === true && safeInteger(v.seconds_left)
        && v.seconds_left > 0 && safeInteger(v.requests_left)
        && v.requests_left > 0;
      state.busyDevice = v.read_in_progress === true;
      connectivity.textContent = !state.ready ? 'AGEN OFFLINE · aktifkan sesi privat'
        : state.busyDevice ? 'AGEN SIBUK · pembacaan sedang berlangsung'
        : 'AGEN SIAP · ' + v.requests_left + ' baca · ' + v.seconds_left + ' detik';
      if (!state.ready && !state.busy) status.textContent =
        'Jalankan agen pemilik pada terminal privat. Konektivitas OLT belum terbukti.';
    } catch (error) {
      state.ready = false;
      state.busyDevice = false;
      connectivity.textContent = 'STATUS TIDAK TERVERIFIKASI';
      if (!state.busy) status.textContent = error.message;
    }
    disableIfNeeded();
  }
  function verifyCommon(v) {
    return v && v.device_adopted === false && v.physical_writes_enabled === false
      && typeof v.read_at_utc === 'string' && !Number.isNaN(Date.parse(v.read_at_utc));
  }
  async function execute(kind) {
    if (state.busy || !state.ready || state.busyDevice) return;
    const operations = {
      inventory: '/lab/c320-owner-live-refresh',
      cards: '/lab/c320-owner-live-cards',
      firmware: '/lab/c320-owner-live-firmware'
    };
    state.busy = true;
    disableIfNeeded();
    status.textContent = 'Menghubungi OLT untuk ' + kind + '. Tunggu hasil perangkat…';
    try {
      const v = await jsonResponse(operations[kind], {
        method: 'POST',
        headers: {'Content-Type': 'application/json', 'X-IPAT-Demo-Only': '1'},
        body: '{}'
      });
      if (!verifyCommon(v)) throw new Error('Hasil perangkat tidak memenuhi kontrak.');
      if (kind === 'inventory') {
        if (v.source !== 'VERIFIED_LOCAL_OWNER_AGENT_LAB_ONLY' || v.pon !== '1/1/1'
          || v.serials_returned !== false || !safeInteger(v.configured)
          || !safeInteger(v.online) || !safeInteger(v.offline)
          || !safeInteger(v.unconfigured) || v.online + v.offline !== v.configured)
          throw new Error('Inventaris perangkat tidak dapat divalidasi.');
        display(fields.configured, v.configured);
        display(fields.online, v.online);
        display(fields.offline, v.offline);
        display(fields.unconfigured, v.unconfigured);
        stamp.textContent = 'PEMBACAAN FISIK · ' + v.read_at_utc + ' · PON 1/1/1 saja';
      } else if (kind === 'cards') {
        if (v.read_kind !== 'CARDS' || !safeInteger(v.cards_in_service)
          || v.cards_in_service === 0) throw new Error('Data kartu ditolak.');
        display(fields.cards, v.cards_in_service);
      } else {
        if (v.read_kind !== 'FIRMWARE' || !safeInteger(v.firmware_rows)
          || v.firmware_rows === 0 || v.firmware_reconciled !== false)
          throw new Error('Data firmware ditolak.');
        display(fields.firmware, v.firmware_rows + ' baris (versi belum direkonsiliasi)');
      }
      state.latest = v.read_at_utc;
      status.textContent = 'SUKSES · hasil ' + kind + ' aktual diterima pada ' + v.read_at_utc
        + '. Ini bukan persetujuan perubahan perangkat.';
      record('Pembacaan fisik ' + kind + ' berhasil, tanpa akses konfigurasi.');
    } catch (error) {
      status.textContent = 'TIDAK BERHASIL · ' + error.message;
      record('Pembacaan ' + kind + ' gagal; data sebelumnya tidak dianggap segar.');
    } finally {
      state.busy = false;
      await refreshAgent();
    }
  }
  for (const [kind, button] of Object.entries(buttons)) {
    button.addEventListener('click', () => execute(kind));
  }
  writeNotice.textContent = 'TINDAKAN TERBATAS: tambah/hapus ONU, profil VLAN/PPPoE, '
    + 'reboot, restore dan upgrade firmware belum disediakan di panel ini. '
    + 'API perangkat tetap menolak aksi berisiko tanpa izin dan pemulihan teruji.';
  refreshAgent();
  loadHistorical();
  setInterval(refreshAgent, 10000);
})();