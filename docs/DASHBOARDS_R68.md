# IPAT R6.8 — tiga dashboard laboratorium dan kebijakan read-only terpisah

**Developer:** Mr. iPat
**Status:** PRATINJAU PRIVAT + TEST KEBIJAKAN SINTETIS.
**Belum siap:** autentikasi dan otorisasi pengguna, dashboard per perusahaan
yang menggunakan data nyata, serta tindakan jaringan operasional.

## Nilai pengiriman milestone

IPAT memiliki tiga **tampilan rancangan yang dapat dicoba**
melalui tautan privat pada halaman laboratorium:

`http://127.0.0.1:48765/lab/dashboard-preview`

Alamat ini HANYA berlaku pada Mac operator saat tunnel SSH
terverifikasi sudah dibuka; bukan domain publik atau layanan tenant.
Kode dan aset HTML/CSS/JS berasal dari proses nonroot
`apps/control-api` Ubuntu, dan hanya aktif jika
`IPAT_LAB_WEB=1` tanpa `IPAT_RUN_K3S_LAB`.
Semua halaman serta aset diberi cache `no-store`,
CSP `default-src 'none'`, hanya script/style/connect
same-origin dan larangan frame/form.

**Penting:** ketiga tombol/tampilan berganti melalui
selector lokal browser. Selector BUKAN login, impersonasi,
RBAC runtime, OIDC, pemilihan tenant yang tervalidasi,
ataupun hak akses pada API. Seluruh data yang dipresentasikan
bersifat sintetis atau status lab nonpelanggan.
Tidak ada data pelanggan, credential, kemampuan baca
tenant riil, transaksi billing, konfigurasi perangkat,
atau endpoint tindakan berisiko.

### Cakupan rancangan tiga dashboard

| Workspace | Fokus awal | Yang TIDAK ditampilkan/diaktifkan |
|---|---|---|
| Platform Admin | katalog metadata tenant, rencana domain, paket, keamanan dan kesiapan global | tidak ada akses otomatis kepada subscriber, device-secret ataupun konfigurasi pelanggan |
| Tenant Admin | tim/izin sendiri, branding, inventory dan kesiapan perangkat di satu ISP contoh | tidak ada kontrol global platform atau data tenant lain |
| Operasional NOC | pemetaan insiden, inventory POP dan diagnostik, gerbang persetujuan PPPoE massal | tidak ada penagihan, delegasi admin ataupun eksekusi perubahan tanpa izin |

Kartu metrik membedakan secara eksplisit
**angka contoh 8 slot target global** (bukan delapan
perangkat terhubung), **nol perangkat fisik
terdaftar**, serta status autentikasi dan telemetri
yang belum aktif. Peran yang sedang dipilih
tidak pernah dikirim ke backend untuk
menentukan privilege.

### Batas server dan contoh kebijakan masa depan

Penambahan endpoint asli
`/v1/platform/{*path}`,
`/v1/tenant/{*path}` dan
`/v1/operations/{*path}`
sengaja **selalu mengembalikan HTTP 401
dan Cache-Control: no-store** untuk
semua metode selama verifikasi OIDC,
keanggotaan tenant serta sumber POP
belum terintegrasi. Tes meminta
GET/POST/DELETE dengan
`Authorization`, `Host`,
`X-Verified-Role` dan
`X-Tenant-Id` palsu;
semuanya harus tetap ditolak.
API private `/v1/devices` juga
tetap menolak unauthenticated request.

`crates/authz-core/src/dashboard.rs` menyediakan
**kebijakan Rust pure reference** deny-default
yang membedakan ruang metadata platform
dari sumber daya milik tenant dan POP,
serta membedakan TenantAdmin, NocEngineer,
Helpdesk dan Auditor. Kebijakan menolak
semua `BulkPppoeWrite` tanpa pengecualian,
menolak NOC lintas POP, menolak akses
tenant lain, dan melarang PlatformOwner
melihat data perangkat/subscriber
tenant hanya karena jabatannya.
Fungsi ini **belum dihubungkan ke endpoint asli**
karena konteks `DashboardSubject`
belum bisa dibuat dari autentikasi OIDC
yang benar. Usulan matriks peran masih
tunduk ADR-009; perubahan menjadi
policy runtime memerlukan persetujuan
dan uji tenant/POP e2e lebih lanjut.

## File implementasi

- `web/lab/dashboard-preview.html`: markup
  responsif, selector desain peran, keterangan
  tidak ada akses perangkat maupun data asli.
- `web/lab/dashboard-preview.css`: visual
  dashboard responsif tanpa CDN atau fonts
  eksternal.
- `web/lab/dashboard-preview.js`: tiga
  variasi menu/metrik/workflows **sintetis**
  dan tiga GET same-origin saja ke
  `/healthz`, `/lab/status`,
  `/lab/device-targets`.
  Seluruh data dimasukkan melalui
  `textContent`, bukan `innerHTML`.
- `apps/control-api/src/main.rs`: host
  ketiga aset privat sekaligus menolak
  API nyata untuk setiap role/header palsu.
- `crates/authz-core/src/dashboard.rs`:
  kebijakan presentation reference
  read-only terpisah untuk tenant/POP,
  dengan tes negatif lintas ruang.
- `deploy/scripts/lab/r68/test_dashboard_preview.py`:
  enam uji statis UI dan keamanan.
- `deploy/scripts/lab/r68/test_dashboard_preview.mjs`:
  uji fungsional DOM dan selector
  dengan Node VM, tanpa dependensi luar
  serta tanpa akses perangkat.
- `deploy/scripts/lab/r68/private-dashboard-http-smoke.sh`:
  uji HTTP sungguhan di runner CI sekali pakai
  dengan Rust Axum loopback, memeriksa
  tiga aset serta penolakan semua API
  yang mengaku membawa token/role/tenant palsu;
  layanan yang dimulai skrip akan dihentikan
  sendiri. Script wajib `GITHUB_ACTIONS=true`
  dan flag opt-in; jangan jalankan
  pada VPS yang sedang melayani preview.

## Cara verifikasi repeatable

Di checkout sumber yang sudah
memiliki toolchain pinned,
tanpa secret pelanggan:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked --offline
python3 -m unittest discover \
  deploy/scripts/lab/r68 -p test_dashboard_preview.py -v
node --check web/lab/dashboard-preview.js
node deploy/scripts/lab/r68/test_dashboard_preview.mjs
```

Setelah perubahan sudah digabungkan
dan Mac+VPS+GitHub `main` bersih
dan sesuai, operator dapat
merestart preview nonroot
yang sudah ada dengan skrip
R5.9 dan mengakses halaman baru
di Mac sendiri. Ini **tidak**
mengaktifkan K3s, database
live, ACS publik atau firewall.

## Tahapan menuju dashboard sungguhan

**MUST sebelum digunakan tenant:**

- OIDC+MFA yang sudah teruji,
  sesi server autentikasi dan
  pembatasan token/CSRF/cookie
  untuk tiap domain.
- Verifikasi DNS/custom domain
  dan membership tenant
  di database tepercaya;
  tidak boleh memilih tenant
  dari Host/header yang dibuat klien.
- UI menu dihitung dari
  entitlement nyata tetapi
  backend tetap mengulangi semua
  pemeriksaan RBAC+ABAC.
  Tes negatif silang tenant,
  POP, role, privilege escalation,
  MFA/expiry dan approval wajib lulus.
- Runtime PostgreSQL role
  non-owner/no-BYPASSRLS dengan
  FORCE RLS di semua tabel tenant,
  audit dan retensi yang jelas.
- Dashboard operasi harus
  terhubung ke device inventory
  yang identitas dan kompatibilitas
  fisiknya diuji; status offline/stale
  tidak boleh dianggap sehat.
- ACS harus punya autentikasi
  per-device SPKI/enrollment, sesi
  CWMP sebenarnya dan bukti satu
  ONT nyata; native USP Controller
  tetap harus dibangun terpisah.
- Deployment produksi harus
  melewati gate pemulihan VPS,
  network isolation,
  DB HA/PITR dan rollback.

**SHOULD:** antarmuka Next.js/TypeScript
terstruktur setelah ADR-006 disetujui,
komponen design system, accessibility
review dan uji e2e Playwright.

**LATER:** tenant branding canggih,
paket komersial, billing pembayaran,
topology live dan orkestrasi jaringan
massal memerlukan integrasi dan
kontrol risiko masing-masing.

Dokumen ini mencatat deliverable
desain dan test, **bukan klaim
tiga dashboard operasional sudah selesai**.
