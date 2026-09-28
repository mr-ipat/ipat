# SOP IPAT — Adopsi OLT ZTE C320 melalui jaringan manajemen yang sudah terhubung

Status: prosedur siap ditinjau; pelaksanaan pertama pada DEV-01 masih
**TERHALANG IDENTITAS PERANGKAT DAN AKUN TERBATAS**. Jangan menganggap
contoh skrip, credential bawaan atau hasil banner sebagai adopsi.

**Jalur yang diprioritaskan:** VPS IPAT → IP privat OLT yang sudah
terjangkau → SSH dengan host-key RSA yang dipin secara independen.
Tidak diperlukan WireGuard, TLS generik maupun SSH proxy tambahan
apabila jalur manajemen yang sudah tersedia memenuhi isolasi/ACL.
API-SSL adalah pilihan MikroTik pada firmware yang mendukungnya, bukan
pengganti universal CLI atau SNMPv3 ZTE.

## 1. Persyaratan pemilik lokasi dan keamanan sebelum LOGIN

- Tentukan tenant/POP dan pemilik change; perangkat distribusi melayani
  pelanggan nyata. Tentukan jendela baca-saja, pemantauan aktif,
  penanggung jawab teknis lokasi, dan mekanisme konsol pemulihan.
- Ekspor **PUBLIC HOST RSA KEY** yang benar-benar berasal dari konsol
  fisik/inventaris tepercaya perangkat; cocokkan dengan pengamatan
  jaringan menggunakan alat offline
  `deploy/scripts/lab/r914/verify_owner_console_host_key.py`.
  Kunci yang diambil dengan `ssh-keyscan` dari jaringan yang sama
  BUKAN bukti independen. Jika fingerprint berbeda, HENTIKAN.
- Buktikan route nyata dan segmen manajemen/ACL terakhir terpisah dari
  pelanggan, meskipun VPS sudah dapat menerima banner port SSH.
  Batasi izin source dan ukur beban. Jangan memaksa pemasangan VPN.
- Sediakan akun OLT KHUSUS BACA-SAJA sesuai RBAC firmware yang
  sesungguhnya; verifikasi penolakan perintah konfigurasi dan dukungan
  public-key SSH bila perangkat menyediakan. Akun bawaan administrator
  uji coba tidak otomatis memenuhi least privilege pada jaringan aktif.
- Siapkan metrik sebelum/sesudah (CPU OLT, event alarm aktif, jumlah
  sesi PPPoE, jumlah ONT online/LOS, paket drop dan latensi kontrol)
  beserta ambang penghentian yang disetujui pemilik; JANGAN membuat
  angka asumsi atau menjalankan operasi jika telemetri belum ada.
- Wajib ada identitas Tenant Admin/MFA yang sungguhan, pembatasan
  tenant+POP, persetujuan pemilik lokasi dan reviewer independen,
  lease/lock DEV-01, audit append-only, serta rencana rollback.

## 2. Satu pembacaan fisik pertama (hanya setelah semua bukti nyata)

Gunakan packet owner-only `r79/c320-ssh-readonly.py` dalam mode
`--check-plan` sebelum `--first-read`. Seluruh bahan (plan, known
hosts yang sudah dipin dari konsol, private key akun terbatas)
harus berada di luar repository, pada direktori 0700 dengan file
0600; jangan kirim ke chat, GitHub, workflow/artifact, atau dashboard
LAB tanpa identitas. Collector saat ini dikunci pada isolated_lab:
sebelum digunakan pada live distribution, diperlukan implementasi
worker produksi dengan signed MFA, 6 bukti fisik di DB, per-device
lock, auditing dan abort. Jangan memalsukan status `isolated_lab`
untuk melewati kebijakan.

- Eksekusi hanya SATU perintah baca-saja `show card` (jika dukungan
  firmware sudah dibuktikan) dari source manajemen terverifikasi,
  dengan host-key pin independen. Hindari scan, polling banyak kartu,
  loop percobaan password, atau opsi `StrictHostKeyChecking=no`.
- Simpan hasil terbatas owner-only, catat waktu dan SHA-256 evidence,
  pantau ambang abort live; HENTIKAN bila terjadi gangguan atau
  firmware output tidak sesuai format.
- Validasi struktur kartu via `olt-evidence --cards ... --out ...`
  lokal; apabila lolos review awal dan baseline tetap sehat, lakukan
  satu `show version-running` sesuai firmware terverifikasi (tidak
  semua firmware mendukung SSH exec mode yang sama). Normalisasi
  ulang hasil dua berkas, lalu review model/firmware dan per-kartu.
- Simpan bukti sensitif secara terenkripsi di luar Git; jangan
  tampilkan nomor seri atau transkrip mentah pada pratinjau publik.

## 3. Transisi inventaris dan hak aksi

1. `OBSERVED_NOT_ADOPTED`: hanya banner/routing tanpa kredensial;
   tidak ada operasi dari dashboard produksi.
2. `IDENTITY_VERIFIED_PENDING_READ`: bukti host-key OOB, akun terbatas,
   POP/isolation, MFA dan pemantauan independen sudah disetujui.
3. `READ_CAPTURED_PENDING_REVIEW`: hasil `show card`/versi nyata sudah
   terbaca, fingerprint tetap valid, metrik sebelum/sesudah sehat;
   hash+audit disimpan privat, tanpa klaim kompatibilitas vendor lain.
4. `ADOPTED_READ_ONLY`: reviewer independen mengesahkan bukti model,
   firmware dan isolasi tenant/POP; production BFF/worker nyata sudah
   tervalidasi. Hanya dua aksi awal `Refresh Kartu` dan `Baca Versi`
   yang lolos prosedur yang sama, dengan rate limit dan audit.
5. Aksi berikutnya `Alarm`, `Daftar ONT`, `Optical Power`, dan
   `Diagnostik Distribusi` tetap TERKUNCI hingga syntax read-only
   firmware sesungguhnya diverifikasi dalam pengujian terpisah.
   `Firmware Upgrade`, pengubahan profile PON/VLAN, reboot,
   provisioning dan operasi masal butuh approval perubahan tingkat
   tinggi, backup/restore dan pengujian dampak tersendiri.

**ABORT:** host-key/role berubah, timeout berulang, lonjakan alarm,
penurunan online ONT, flapping PPPoE, hilang akses console atau
ketidakcocokan format CLI. Jangan mencoba perintah write untuk
menguji akses. Label kesehatan tidak berubah menjadi SEHAT dari
banner atau satu pembacaan kartu; perlu telemetri nyata berkelanjutan.

## 4. Catatan hasil pengujian yang wajib dibukukan

Catat waktu+operator terautentikasi, referensi bukti OOB (HASH SAJA),
versi SSH dan fingerprint, identitas kartu+firmware hasil parser,
jumlah perintah, status ingress/ACL, baseline sebelum/sesudah,
reviewer independen, alasan abort jika ada, dan issue firmware
kompatibilitas yang belum terbukti. Perbarui `DEVICE_MATRIX.md`,
`PROJECT_STATUS.md` dan keputusan arsitektur hanya berdasarkan
hasil yang benar-benar terjadi. Tidak boleh membuat status ADOPTED
berdasarkan pengujian sintetis atau kredensial administrator bawaan.

## Catatan kompatibilitas firmware ZTE

Historical ZXA10 C300/C320 documentation describes `show card`
and `show version-running` in operator modes other than initial
unprivileged user mode. The actual DEV-01 firmware privilege layout
and noninteractive SSH `exec` support remain unverified; never add
`enable`, a privileged password or arbitrary interactive prompts
to bypass failure. Confirm the smallest safe read-only role and
exact command support through approved console/inventory first.

## Current first-connection physical diagnostic blocker

The newest bounded credential-free noauth SSH handshake comparison
on owner Mac and owner VPS both reached SSH2 algorithm selection but
TIMED OUT before receiving a host key or authentication methods. Do
not classify this as an invalid test account or automatically retry
with passwords. Via trusted maintenance console, first inspect
actual SSH service availability, CPU/alarm baseline, enabled legacy
KEX/host-key algorithm support and the isolated management ACL/MTU
path. Compare the real local console host RSA public key with the
historically observed network key OUT OF BAND. No blind `telnet`,
`StrictHostKeyChecking=no`, global weak SSH client settings or OLT
SSH daemon restart should be used on active distribution merely to
bypass this diagnostic.

## Preauthentication SSH KEX timeout: targeted `show ssh` console check

Historical ZXA10 C320 vendor CLI guidance includes read-only
`show ssh` for the actual server enable flag, SSH version and host
key initialization status. The manual's *example* includes a not
initialized server key; this is NOT proof of DEV-01's settings.
Because both owner Mac and VPS KEX sessions timed out before obtaining
a server host key, review actual `show ssh` through a TRUSTED site
console (only if the exact firmware and current CLI role support it)
BEFORE attempting another password login or changing weak algorithms.
Use `deploy/scripts/lab/r921/inspect_show_ssh.py` locally on only the
owner-private bounded capture. See `docs/R921_C320_SSH_PREAUTH_SITE_DIAGNOSTIC.md`.
Do NOT initialize keys, alter management ACL/SSH protocol, or reboot
the live subscriber-serving OLT without an independently reviewed
maintenance window and reliable local recovery.
