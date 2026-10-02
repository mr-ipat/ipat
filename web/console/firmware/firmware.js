/* R9.58 future authenticated production console. NOT mounted until the
 * protected BFF + encrypted artifact storage + current MFA are implemented.
 * Capability flags are server-signed session results, NEVER client authority.
 */
"use strict";
const byId = id => document.getElementById(id);
const state = { csrf: null, permissions: new Set(), ready: false, deviceIds: new Set() };
const security = byId("security-status");
function fail(message) {
  state.csrf = null;
  state.ready = false;
  state.permissions.clear();
  state.deviceIds.clear();
  for (const id of ["operator-form", "review-form", "jobs"]) byId(id).hidden = true;
  byId("create").disabled = true;
  security.textContent = message;
}
async function jsonGet(path) {
  const res = await fetch(path, {
    credentials: "same-origin", headers: { "Accept": "application/json" }, cache: "no-store"
  });
  if (!res.ok) throw new Error("UNAVAILABLE");
  return res.json();
}
async function jsonPost(path, body) {
  const res = await fetch(path, {
    method: "POST", credentials: "same-origin", cache: "no-store",
    headers: { "Content-Type": "application/json", "Accept": "application/json", "X-IPAT-CSRF": state.csrf },
    body: JSON.stringify(body)
  });
  if (!res.ok) throw new Error("NOT_ACCEPTED");
  return res.json();
}
function setTextRow(container, text) {
  const p = document.createElement("p");
  p.textContent = text;
  container.appendChild(p);
}
function addJob(container, job) {
  const block = document.createElement("section");
  setTextRow(block, `${job.device_name || "Perangkat"} • ${job.target_version || "?"} • ${job.state || "unknown"}`);
  if (state.permissions.has("firmware:execute") && job.execution_ready === true &&
      job.adapter_qualified === true && job.state === "APPROVED") {
    const run = document.createElement("button");
    run.type = "button";
    run.textContent = "Jalankan Pembaruan (perintah operator)";
    run.addEventListener("click", async () => {
      if (!state.ready) return;
      if (!confirm("Pastikan jendela pemeliharaan, identitas perangkat, dan persetujuan masih berlaku. Ajukan eksekusi?")) return;
      run.disabled = true;
      try {
        const answer = await jsonPost(`/v1/firmware/changes/${encodeURIComponent(job.id)}/execute`, { confirmed: true });
        if (answer.state !== "EXECUTION_REQUESTED") throw new Error("NOT_ACCEPTED");
        security.textContent = "Permintaan eksekusi tersimpan. Tunggu worker berkualifikasi dan bukti perubahan aktual.";
        await refreshJobs();
      } catch {
        security.textContent = "Permintaan tidak diterima. Periksa izin, bukti, worker, dan jadwal; perangkat tidak boleh diasumsikan berubah.";
      }
    });
    block.appendChild(run);
  }
  if (state.permissions.has("firmware:review") && job.review_ready === true &&
      job.state === "AWAITING_EVIDENCE") {
    for (const [decision, label] of [["approved", "Setujui sebagai pemeriksa independen"], ["rejected", "Tolak"]]) {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = label;
      button.addEventListener("click", async () => {
        if (!state.ready || !confirm(`Konfirmasi keputusan ${decision}?`)) return;
        button.disabled = true;
        try {
          await jsonPost(`/v1/firmware/changes/${encodeURIComponent(job.id)}/review`, { decision });
          await refreshJobs();
        } catch { security.textContent = "Persetujuan ditolak sistem. MFA/peran/bukti/perbedaan pembuat harus diverifikasi."; }
      });
      block.appendChild(button);
    }
  }
  container.appendChild(block);
}
async function refreshJobs() {
  if (!state.ready) return;
  const list = await jsonGet("/v1/firmware/changes");
  const container = byId("jobs-list");
  container.replaceChildren();
  if (!Array.isArray(list.items)) throw new Error("INVALID_JOBS");
  for (const job of list.items.slice(0, 100)) addJob(container, job);
}
async function boot() {
  fail("Fitur belum aktif: memerlukan HTTPS, MFA, dan izin perusahaan yang terverifikasi.");
  if (location.protocol !== "https:") return;
  let cfg;
  try { cfg = await jsonGet("/v1/firmware/capabilities"); } catch { return; }
  if (!cfg || cfg.authenticated !== true || cfg.tenant_verified !== true ||
      cfg.https_verified !== true || !Array.isArray(cfg.permissions) ||
      typeof cfg.csrf !== "string" || cfg.csrf.length < 16) return;
  state.csrf = cfg.csrf;
  state.permissions = new Set(cfg.permissions);
  const canCreate = state.permissions.has("firmware:request");
  const canReview = state.permissions.has("firmware:review");
  const canList = canCreate || canReview || state.permissions.has("firmware:read");
  state.ready = true;
  byId("operator-form").hidden = !canCreate;
  byId("review-form").hidden = !canReview;
  byId("jobs").hidden = !canList;
  security.textContent = "Sesi terverifikasi. Tindakan tetap memerlukan pemeriksaan API dan database.";
  if (canCreate) {
    try {
      const devices = await jsonGet("/v1/firmware/devices");
      if (!Array.isArray(devices.items)) throw new Error("INVALID_DEVICES");
      const select = byId("device");
      select.replaceChildren();
      for (const device of devices.items) {
        if (!device || typeof device.id !== "string" || typeof device.display_name !== "string") continue;
        const option = document.createElement("option");
        option.value = device.id;
        option.textContent = device.display_name;
        select.appendChild(option);
        state.deviceIds.add(device.id);
      }
      byId("create").disabled = !(cfg.workflow_ready === true &&
        cfg.artifact_upload_ready === true && state.deviceIds.size > 0);
      byId("file-notice").textContent = byId("create").disabled
        ? "Upload, pemeriksaan vendor, atau API produksi belum siap; jangan unggah firmware."
        : "Firmware diproses oleh layanan unggah privat sebelum dapat diajukan.";
    } catch { byId("create").disabled = true; }
  }
  if (canList) {
    try { await refreshJobs(); } catch { security.textContent = "Daftar pekerjaan tidak tersedia. Tidak ada izin yang diasumsikan."; }
  }
}
byId("firmware-form").addEventListener("submit", async event => {
  event.preventDefault();
  if (!state.ready || !state.permissions.has("firmware:request") || byId("create").disabled) return;
  const file = byId("firmware-file").files[0];
  const deviceId = byId("device").value;
  const start = new Date(byId("start").value);
  const end = new Date(byId("end").value);
  if (!file || !state.deviceIds.has(deviceId) || !(file.size >= 1024 && file.size <= 1073741824) ||
      !Number.isFinite(start.getTime()) || !Number.isFinite(end.getTime()) || end <= start) {
    security.textContent = "Periksa perangkat, file, ukuran, dan jadwal yang dipilih.";
    return;
  }
  byId("create").disabled = true;
  security.textContent = "Mengirim firmware ke penyimpanan privat; perangkat belum berubah.";
  try {
    const form = new FormData();
    form.append("file", file);
    form.append("device_id", deviceId);
    form.append("vendor_release_ref", byId("vendor-release").value);
    form.append("target_version", byId("version").value);
    const uploaded = await fetch("/v1/firmware/uploads", {
      method: "POST", credentials: "same-origin", cache: "no-store",
      headers: { "X-IPAT-CSRF": state.csrf }, body: form
    });
    if (!uploaded.ok) throw new Error("UPLOAD_REJECTED");
    const image = await uploaded.json();
    if (!image || image.metadata_staged !== true || typeof image.artifact_id !== "string") throw new Error("INVALID_UPLOAD");
    const requested = await jsonPost("/v1/firmware/changes", {
      device_id: deviceId, artifact_id: image.artifact_id, request_id: crypto.randomUUID(),
      reason: byId("reason").value, window_start: start.toISOString(), window_end: end.toISOString()
    });
    if (requested.state !== "AWAITING_EVIDENCE") throw new Error("REQUEST_REJECTED");
    security.textContent = "Permintaan dicatat; menunggu bukti kompatibilitas dan persetujuan independen. Belum dieksekusi.";
    await refreshJobs();
  } catch {
    security.textContent = "Upload atau permintaan gagal. Jangan menganggapnya tersimpan atau perangkat sudah berubah.";
  } finally { byId("create").disabled = false; }
});
void boot();
