'use strict';
// R9.46: one truthful color status per physical device. A TCP route, an
// enrolled password, an old CLI transcript and a live collector process are
// individually INSUFFICIENT to show CONNECTED.
(() => {
  const targets = () => document.querySelectorAll('[data-ipat-device-status="DEV-01"]');
  const statusLabels = {
    CONNECTED: '● Connected',
    DISCONNECTED: '● Disconnected',
    PENDING: '● Pending',
    UNKNOWN: '● Unknown'
  };
  let previous = 'UNKNOWN';
  let lastSuccess = '';
  function show(kind, detail) {
    previous = kind;
    targets().forEach(node => {
      node.textContent = statusLabels[kind];
      node.className = 'ipat-device-signal ipat-device-signal--' + kind.toLowerCase();
      node.setAttribute('aria-label', 'Status perangkat DEV-01: ' + statusLabels[kind].slice(2));
      node.title = detail;
    });
    const summary = document.getElementById('ipat-c320-signal-detail');
    if (summary) summary.textContent = detail;
  }
  function consume(body) {
    if (!body || body.target !== 'DEV-01' || body.vendor !== 'ZTE'
        || body.model !== 'C320' || body.production_adopted !== false
        || body.physical_writes_enabled !== false
        || !['CONNECTED','DISCONNECTED','PENDING','UNKNOWN'].includes(body.device_status)
        || typeof body.connector_online !== 'boolean'
        || typeof body.credentials_enrolled !== 'boolean') {
      throw new Error('invalid physical device status contract');
    }
    const stamp = body.last_verified_at_utc;
    const verified = typeof stamp === 'string' && stamp.length > 0
      && Number.isFinite(Date.parse(stamp))
      && Date.now() - Date.parse(stamp) >= -30000
      && Date.now() - Date.parse(stamp) <= 360000;
    // A stale response must NEVER turn green, even if an older backend
    // mistakenly persists READ_ONLY_CONNECTED_LAB.
    const kind = body.device_status === 'CONNECTED' && !verified
      ? 'UNKNOWN' : body.device_status;
    if (kind === 'CONNECTED') {
      lastSuccess = stamp;
      show(kind, 'C320 · SSH terautentikasi, data fisik segar: ' + stamp);
    } else if (kind === 'DISCONNECTED') {
      show(kind, 'C320 · konektor aktif tetapi pembacaan fisik terbaru gagal atau kedaluwarsa.');
    } else if (kind === 'PENDING') {
      show(kind, body.credentials_enrolled
        ? 'C320 · kredensial tersimpan, menunggu pembacaan fisik pertama.'
        : 'C320 · konektor siap, perangkat belum didaftarkan dari dashboard.');
    } else {
      show('UNKNOWN', body.connector_online
        ? 'Status perangkat belum terverifikasi.'
        : 'Tidak ada jawaban terverifikasi dari konektor. Jangan menganggap OLT offline.');
    }
  }
  let pollRunning = false;
  async function poll() {
    if (pollRunning) return;
    pollRunning = true;
    const controller = new AbortController();
    const deadline = setTimeout(() => controller.abort(), 4500);
    try {
      const response = await fetch('/lab/c320-owner-connection', {
        cache: 'no-store', credentials: 'omit', signal: controller.signal
      });
      if (!response.ok) throw new Error('HTTP ' + response.status);
      consume(await response.json());
    } catch (_) {
      show('UNKNOWN','Status perangkat tidak tersedia; bukti terakhir'
        + (lastSuccess ? ' ' + lastSuccess : ' belum tersedia')
        + '. Tidak mengubah status menjadi hijau dari riwayat.');
    } finally {
      clearTimeout(deadline);
      pollRunning = false;
    }
  }
  window.addEventListener('ipat-device-connection-changed', poll);
  window.addEventListener('ipat-device-list-rendered', () => {
    // Keep the status of a newly re-rendered physical row consistent.
    show(previous,document.getElementById('ipat-c320-signal-detail')?.textContent || 'Menunggu pemeriksaan');
  });
  poll();
  setInterval(poll, 10000);
})();
