"use strict";
// Mr. iPat / IPAT: browser-only health indicators; no auth or device calls.
const byId = (id) => document.getElementById(id);
const refresh = byId("refresh");
const status = byId("api-status");
const badge = byId("api-badge");
const checked = byId("checked-at");

async function checkPrivateLab() {
  refresh.disabled = true;
  status.textContent = "Memeriksa";
  badge.textContent = "Memeriksa";
  badge.className = "status-pill checking";
  const controller = new AbortController();
  const deadline = window.setTimeout(() => controller.abort(), 6000);
  try {
    const [health, details] = await Promise.all([
      fetch("/healthz", {cache:"no-store",signal:controller.signal}),
      fetch("/lab/status", {cache:"no-store",signal:controller.signal})
    ]);
    if (!health.ok || !details.ok || (await health.text()).trim() !== "ok") {
      throw new Error("Health not available");
    }
    const state = await details.json();
    if (state.mode !== "ssh-loopback-only" || state.production_access !== false
        || state.authentication_enabled !== false) {
      throw new Error("Unexpected lab mode");
    }
    status.textContent = "Terhubung";
    badge.textContent = "Aktif";
    badge.className = "status-pill ok";
  } catch {
    status.textContent = "Tidak terhubung";
    badge.textContent = "Periksa tunnel";
    badge.className = "status-pill error";
  } finally {
    window.clearTimeout(deadline);
    checked.textContent = "Terakhir diperiksa: "
      + new Intl.DateTimeFormat("id-ID",{hour:"2-digit",minute:"2-digit",
        timeZone:"Asia/Jakarta"}).format(new Date()) + " WIB";
    refresh.disabled = false;
  }
}
refresh.addEventListener("click", checkPrivateLab);
void checkPrivateLab();
