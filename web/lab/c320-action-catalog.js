'use strict';
/* R10.03 English-first owner-private capabilities; never grants permissions.
 * Server emits immutable allowlisted status. Only the three physically accepted
 * bounded read operations have browser execution handlers in this release. */
(() => {
  const el = id => document.getElementById(id);
  const table = el('ipat-c320-capabilities-rows');
  if (!table) return;
  const state = el('ipat-c320-capabilities-state');
  const detail = el('ipat-c320-capability-detail');
  const events = el('ipat-c320-capability-events');
  const refresh = el('ipat-c320-refresh-capabilities');
  const endpoints = Object.freeze({
    cards: '/lab/c320-owner-live-cards',
    firmware: '/lab/c320-owner-live-firmware',
    onu_counts: '/lab/c320-owner-live-refresh'
  });
  let busy = false;
  const notices = {
    AVAILABLE_READ_ONLY: 'Verified read available',
    CONNECTION_REQUIRED: 'Reconnect to unlock verified read',
    DEGRADED: 'Read failed; requalification required',
    NOT_QUALIFIED: 'Physical driver/format not validated',
    NOT_IMPLEMENTED: 'Not implemented',
    APPROVAL_AND_DRIVER_REQUIRED: 'Vendor driver, approvals & recovery required',
    AGENT_INTEROP_REQUIRED: 'Actual CPE protocol interoperability required'
  };
  function cell(row, value) {
    const td = document.createElement('td');
    td.textContent = value;
    row.append(td);
  }
  function log(result) {
    const item = document.createElement('li');
    item.textContent = new Date().toLocaleString('en-US') + ' · ' + result;
    events.prepend(item);
    while (events.children.length > 12) events.lastChild.remove();
  }
  async function call(path, options, timeoutMs) {
    const abort = new AbortController();
    const timer = setTimeout(() => abort.abort(), timeoutMs);
    try {
      const response = await fetch(path, {
        cache: 'no-store', credentials: 'omit', ...options, signal: abort.signal
      });
      if (!response.ok) throw Error('HTTP ' + response.status);
      return await response.json();
    } finally {
      clearTimeout(timer);
    }
  }
  async function execute(id) {
    if (busy || !Object.hasOwn(endpoints, id)) return;
    busy = true;
    const endpoint = endpoints[id];
    detail.textContent = 'Reading the actual ZTE C320. Do not repeat while in progress...';
    try {
      const v = await call(endpoint, {
        method: 'POST', headers: { 'Content-Type': 'application/json', 'X-IPAT-Demo-Only': '1' },
        body: '{}'
      }, 60000);
      if (v.physical_writes_enabled !== false || v.device_adopted !== false
          || typeof v.read_at_utc !== 'string' || Number.isNaN(Date.parse(v.read_at_utc)))
        throw Error('Unverifiable physical read response');
      let summary;
      if (id === 'cards' && v.read_kind === 'CARDS'
          && Number.isSafeInteger(v.cards_in_service)
          && v.cards_in_service > 0 && v.cards_in_service <= 22) {
        summary = 'Active line cards: ' + v.cards_in_service;
      } else if (id === 'firmware' && v.read_kind === 'FIRMWARE'
          && Number.isSafeInteger(v.firmware_rows)
          && v.firmware_rows > 0 && v.firmware_rows <= 66
          && v.firmware_reconciled === false) {
        summary = 'Firmware inventory: ' + v.firmware_rows
          + ' records. Exact board/version mapping is NOT yet validated.';
      } else if (id === 'onu_counts' && v.pon === '1/1/1'
          && Number.isSafeInteger(v.configured) && Number.isSafeInteger(v.unconfigured)
          && Number.isSafeInteger(v.online) && Number.isSafeInteger(v.offline)
          && v.configured >= 0 && v.configured <= 128
          && v.unconfigured >= 0 && v.unconfigured <= 128
          && v.online >= 0 && v.offline >= 0
          && v.online + v.offline === v.configured
          && v.serials_returned === false) {
        summary = 'PON 1/1/1 aggregate: ' + v.configured + ' configured, '
          + v.online + ' online, ' + v.offline + ' offline, '
          + v.unconfigured + ' unconfigured. No ONU identity/serial displayed.';
      } else {
        throw Error('Unexpected device response; not displaying unverified output');
      }
      detail.textContent = summary + ' · Live UTC: ' + v.read_at_utc
        + ' · Physical configuration writes: DISABLED';
      log(id + ' succeeded: ' + summary);
    } catch (error) {
      detail.textContent = 'Operation failed (' + error.message
        + '). Connection status alone does not establish operation success.';
      log(id + ' failed; no configuration command sent');
    } finally {
      busy = false;
      await load();
    }
  }
  async function load() {
    if (busy) return;
    refresh.disabled = true;
    try {
      const catalog = await call('/lab/c320-owner-action-catalog', {}, 5000);
      if (catalog.mode !== 'OWNER_PRIVATE_LAB_CAPABILITIES'
          || catalog.target !== 'DEV-01' || catalog.production_adopted !== false
          || catalog.physical_writes_enabled !== false || !Array.isArray(catalog.entries)
          || catalog.entries.length < 10 || typeof catalog.connected !== 'boolean')
        throw Error('Capability response failed validation');
      table.replaceChildren();
      let available = 0;
      for (const capability of catalog.entries) {
        if (typeof capability.id !== 'string' || typeof capability.group !== 'string'
            || typeof capability.label !== 'string' || typeof capability.scope !== 'string'
            || !Object.hasOwn(notices, capability.state))
          throw Error('Unknown capability format');
        const executable = catalog.connected && capability.state === 'AVAILABLE_READ_ONLY'
          && capability.operation === 'READ'
          && Object.hasOwn(endpoints, capability.id)
          && capability.endpoint === endpoints[capability.id];
        const tr = document.createElement('tr');
        cell(tr, capability.group);
        cell(tr, capability.label);
        cell(tr, notices[capability.state]);
        const action = document.createElement('td');
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'secondary';
        button.disabled = !executable;
        button.textContent = executable ? 'Run read' : 'Not available';
        button.title = capability.scope;
        if (executable) {
          available++;
          button.addEventListener('click', () => execute(capability.id));
        }
        action.append(button);
        const info = document.createElement('button');
        info.type = 'button';
        info.className = 'secondary';
        info.textContent = 'Requirements';
        info.addEventListener('click', () => {
          detail.textContent = capability.label + ': ' + capability.scope;
        });
        action.append(info);
        tr.append(action);
        table.append(tr);
      }
      state.textContent = (catalog.connected ? 'CONNECTED · ' : 'NOT CONNECTED · ')
        + available + ' qualified read operations · ' + catalog.entries.length
        + ' catalogued operations · Last device read: '
        + (catalog.last_verified_at_utc || 'not verified');
    } catch (error) {
      table.replaceChildren();
      state.textContent = 'Capability service unavailable; execution disabled ('
        + error.message + ').';
    } finally {
      refresh.disabled = false;
    }
  }
  refresh.addEventListener('click', load);
  window.addEventListener('ipat-device-connection-changed', load);
  load();
  setInterval(load, 15000);
})();
