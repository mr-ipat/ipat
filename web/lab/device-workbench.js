"use strict";
// A real interactive LAB dashboard backed by the ACTUAL private Rust server's
// VOLATILE fake-device memory. No bearer/password, real IP, serial or network I/O.
const node = (id) => document.getElementById(id);
const source = "/lab/demo/device-candidates";
let candidates = [];
function el(tag, cls, content) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (content !== undefined) e.textContent = String(content);
  return e;
}
function td(content) { return el("td", "", content); }
function badge(content, cls) { return el("span", "flag "+cls, content); }
function status(message, failure=false) {
  node("list-status").textContent = message;
  node("list-status").style.color = failure ? "#ffb9bf" : "#9ed5d0";
}
function draw() {
  const pop = node("pop-filter").value;
  const kind = node("kind-filter").value;
  const view = candidates.filter((d) =>
    (pop === "all" || d.pop_id === pop)
    && (kind === "all" || d.device_kind === kind));
  const fragment = document.createDocumentFragment();
  for (const d of view) {
    const row = el("tr");
    const name = el("td");
    name.append(el("strong","",d.display_name),el("small","",d.id));
    row.appendChild(name);
    const type = el("td");
    type.append(el("strong","",d.device_kind.toUpperCase()),el("small","",d.vendor+" · "+d.exact_model));
    row.appendChild(type);
    row.appendChild(td(d.pop_id.toUpperCase()));
    const adoption = el("td");
    adoption.appendChild(badge("MENUNGGU REVIEW",""));
    row.appendChild(adoption);
    const connectivity = el("td");
    connectivity.appendChild(badge("UNKNOWN","unknown"));
    row.appendChild(connectivity);
    const health = el("td");
    health.appendChild(badge("BELUM DIUKUR","unknown"));
    row.appendChild(health);
    const remove = el("td");
    const button = el("button","danger","Hapus demo");
    button.type = "button";
    button.setAttribute("aria-label","Hapus kandidat demo "+d.display_name);
    button.addEventListener("click", () => { void removeDemo(d.id); });
    remove.appendChild(button);
    row.appendChild(remove);
    fragment.appendChild(row);
  }
  if (!view.length) {
    const row = el("tr");
    const cell = el("td","empty","Tidak ada kandidat untuk filter ini. Gunakan formulir untuk menambah perangkat demo.");
    cell.colSpan = 7;
    row.appendChild(cell);
    fragment.appendChild(row);
  }
  node("rows").replaceChildren(fragment);
  node("candidate-count").textContent = String(candidates.length);
}
async function refresh() {
  node("refresh").disabled = true;
  try {
    const resp = await fetch(source,{cache:"no-store",credentials:"omit"});
    if (!resp.ok) throw new Error("Tidak dapat mengambil data");
    const body = await resp.json();
    if (body.lab_only !== true || body.source !== "volatile-in-memory-demo"
        || body.real_device_count !== 0 || body.physical_connection_checked !== false
        || !Array.isArray(body.devices) || body.devices.length > 24
        || body.devices.some(d => d.lab_only !== true
          || d.adoption_state !== "PENDING_REVIEW" || d.connectivity !== "UNKNOWN"
          || d.health !== "NOT_MEASURED" || d.last_verified_at !== null)) {
      throw new Error("Status backend tidak cocok; inventaris ditolak");
    }
    candidates=body.devices;
    draw();
    status(String(candidates.length)+" kandidat virtual · 0 koneksi nyata · kondisi UNKNOWN");
  } catch {
    candidates=[];
    draw();
    status("Tidak berhasil memverifikasi backend privat. Jangan menganggap perangkat aktif.",true);
  } finally {
    node("refresh").disabled=false;
  }
}
async function mutate(method,path,body) {
  return fetch(path,{
    method,credentials:"omit",cache:"no-store",
    headers:{"X-IPAT-Demo-Only":"1","Content-Type":"application/json"},
    ...(body ? {body:JSON.stringify(body)} : {})
  });
}
async function removeDemo(id) {
  try {
    const response=await mutate("DELETE",source+"/"+encodeURIComponent(id));
    if (!response.ok) throw new Error("Penolakan backend");
    await refresh();
  } catch {
    status("Penghapusan demo ditolak oleh server.",true);
  }
}
async function submit(event) {
  event.preventDefault();
  const button=node("add");
  button.disabled=true;
  node("form-status").textContent="Memvalidasi kandidat virtual…";
  const body={
    display_name:node("name").value.trim(),
    device_kind:node("kind").value,
    vendor:node("vendor").value,
    exact_model:node("model").value.trim(),
    pop_id:node("pop").value
  };
  try {
    const response=await mutate("POST",source,body);
    if (!response.ok) {
      node("form-status").textContent=response.status===409
        ? "Nama+POP sudah terdaftar atau batas 24 kandidat tercapai."
        : "Backend menolak input. Gunakan hanya identitas LAB- dan model VIRTUAL-.";
      return;
    }
    const result=await response.json();
    if (result.lab_only !== true || result.accepted !== true
        || result.device?.adoption_state !== "PENDING_REVIEW") {
      throw new Error("Respon tidak aman");
    }
    node("form-status").textContent="Kandidat "+body.display_name+" ditambahkan sebagai PENDING_REVIEW. BUKAN perangkat online.";
    await refresh();
  } catch {
    node("form-status").textContent="Permintaan gagal. Periksa koneksi privat tanpa mengaktifkan perangkat.";
  } finally {
    button.disabled=false;
  }
}
node("device-form").addEventListener("submit",event=>{void submit(event);});
node("refresh").addEventListener("click",()=>{void refresh();});
node("pop-filter").addEventListener("change",draw);
node("kind-filter").addEventListener("change",draw);
void refresh();

// R9.5 connection selector is PRESENTATION ONLY: no persist, network or secrets.
function describeConnection() {
  const method=node("connection-method").value;
  const gateway=node("connection-gateway").value;
  const messages={
    direct_secure:"Hanya bila perangkat mendukung SSH dengan identitas terverifikasi atau SNMPv3 authPriv; wajib membatasi sumber dan izin. Telnet tidak termasuk.",
    wireguard:"WireGuard membutuhkan gateway yang mendukungnya, rute /32 yang disetujui, autentikasi peer, isolasi hop terakhir dan persetujuan terpisah.",
    ipsec:"IPsec dapat digunakan dengan gateway yang kompatibel setelah parameter kriptografi, identitas peer, rute dan pemulihan diverifikasi.",
    agent:"IPAT site gateway adalah rencana pengembangan, belum dapat diinstal atau digunakan untuk adopsi fisik.",
    public_telnet:"DITOLAK: Telnet melalui IP publik tidak aman untuk autentikasi atau perintah. Pengujian tanpa kredensial hanya mencatat bukti jaringan, bukan adopsi."
  };
  let result=messages[method];
  if(method==="wireguard" && gateway==="routeros6")
    result="TIDAK KOMPATIBEL: WireGuard bawaan tidak tersedia pada RouterOS 6. Pilih IPsec atau gateway lain yang terverifikasi.";
  if(method==="wireguard" && gateway==="none")
    result="Gateway diperlukan untuk mengamankan akses perangkat Telnet-only; tidak ada tunnel site yang dapat dideploy dari pilihan ini.";
  if(method==="direct_secure" && gateway==="routeros6")
    result+=" Versi gateway tidak membuktikan bahwa OLT mendukung protokol aman.";
  node("connection-result").textContent=result+" Ini hanya simulasi pilihan UI; tidak ada konfigurasi yang dikirim.";
}
node("connection-method").addEventListener("change",describeConnection);
node("connection-gateway").addEventListener("change",describeConnection);
describeConnection();
