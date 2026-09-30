# SOP IPAT — Adopsi OLT ZTE C320 melalui jaringan manajemen yang sudah terhubung

Status R9.30 (29 September 2026): pemilik menyatakan DEV-01 adalah
LAB tanpa pelanggan, namun seluruh perubahan adopsi diperlakukan
setara perangkat LIVE. Login nyata sekali melalui Telnet privat dan
SSH terenkripsi SUDAH BERHASIL, dan 3 kartu/5 baris versi diamati
secara manual; parser privat telah menerima snapshot fisik tersebut.
**ADOPSI SAAS OTOMATIS MASIH TERTAHAN** karena akun khusus/host trust
independen, recovery NATIVE vendor belum diuji pada chassis dan audit
serta production worker tenant belum selesai. Backup referensi konfigurasi
CLI eksternal terenkripsi Restic SUDAH dipulihkan byte-identik oleh
pemilik dan diverifikasi dari Mac. Ini BUKAN bukti file bisa diimpor
ke firmware OLT. Lihat milestone R9.33 di `PROJECT_STATUS.md`. Jangan mengklaim ada customer atau perangkat belum pernah
berhasil login. Lihat `docs/R930_C320_AUTHENTICATED_FIRST_REAL_LAB_READ.md`.

**Jalur yang diprioritaskan:** VPS IPAT → IP privat OLT yang sudah
terjangkau → SSH dengan host-key RSA yang dipin secara independen.
Tidak diperlukan WireGuard, TLS generik maupun SSH proxy tambahan
apabila jalur manajemen yang sudah tersedia memenuhi isolasi/ACL.
API-SSL adalah pilihan MikroTik pada firmware yang mendukungnya, bukan
pengganti universal CLI atau SNMPv3 ZTE.

## 1. Persyaratan pemilik lokasi dan keamanan sebelum LOGIN

- Tentukan tenant/POP dan pemilik change. DEV-01 sekarang DIKONFIRMASI
  pemilik tidak tersambung pelanggan namun prosedur perubahan harus
  memperlakukannya sama dengan OLT produksi. Untuk OLT lain, audit
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

## 2. Pembacaan terotomasi produksi setelah semua bukti nyata

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

## Historical R9.21/R9.22 SSH client negotiation: resolved in LAB

Historical DEFAULT SSH KEX group16-SHA512 timed out. R9.22 proved
process-scoped group14-SHA256 with exact RSA/AES128-CBC. R9.30 then
ACTUALLY authenticated via protected private SSH and executed safe
`show card`/`show version-running` matching earlier actual Telnet
readouts, with no OLT SSH server configuration changes. Do not keep
repeating old noauth KEX scans or call first hardware read NOT RUN.

R9.30 actual `show ssh` shows enabled SSH2 local CHAP and reported
`SSH init server key : not initialized`, YET the physical endpoint
presented a working RSA host key and accepted encrypted SSH login.
This firmware status is AMBIGUOUS, NOT proof keys need regeneration.
Never run device SSH server key generation or change firmware
based solely on this field. Independent physical-console host key
provenance remains a separate COMMERCIAL production gate, even if
owner-accepted first-read LAB endpoint returned matching data.

## R9.22 confirmed client-side compatibility resolution (2026-09-29)

The authorized owner VPS actually confirmed three necessary SSH
parameters to reach the remote authentication-method stage without
credentials: host key `ssh-rsa`, cipher `aes128-cbc`, and KEX
`diffie-hellman-group14-sha256`. The earlier default group16-SHA512
choice timed out before host key. This change MUST be PROCESS-SCOPED
and exact-device only, never a global system SSH setting or
nonconsensual OLT server modification.

IPAT collector's new explicit profile is
`ssh-strict-pinned-publickey-legacy-rsa-cbc-group14-sha256`, retaining
its required independent owner console known_hosts, separate
restricted key-based login and first-run fixed `show card` limit.
A successful no-credential network handshake DOES NOT supply any of
these trust/authorization requirements; even the original shared
privileged lab password should not be sent before independent OOB
RSA verification on a customer-serving distribution device.

## R9.30 operational LAB exception is not unattended adoption

User-authorized temporary factory-like password was used in one
bounded INTERACTIVE Telnet test and one separate pinned-network-RSA
SSH session, both now closed. It is not persisted in Git, evidence
or automatic worker. For future unattended adoption, create and
verify a firmware-supported least-privileged user, rotate the weak
test password, store a new secret in external vault, prove an
owner-approved secure backup/restore, tenant MFA/reviewer and true
scoped signed production worker. Until then, eight dashboard device
action POST routes are STILL 403 and no production writes allowed.
Current source `r79` is key-only and is NOT a valid auto-login client
for the observed temporary password-only SSH account.

## Status R9.33 backup nyata dan akun LAB, 29 September 2026

Backup yang sebelumnya BLOCKED kini **SELESAI** untuk cakupan
referensi CLI: owner berhasil menjalankan Mac Restic off-VPS,
independent snapshot lookup dan isolasi restore SHA-256 identik
119980 byte lulus. Tetap **TERHALANG** vendor-native firmware
import/recovery drill. Login owner-admin sementara menunjukkan
level15; `show username` terbaru melaporkan dua akun level15,
BUKAN akun service baca-saja. Firmware `show alarm crtv-active`
menerima sintaks, tetapi format/semantik alarm aktual belum
diuji. Katalog private localhost :3002 memperlihatkan backup
yang nyata dan tiga kartu historis, delapan aksi otomatis masih
HTTP403. Jangan menjalankan `username`, `write`, menghapus
akses recovery atau mematikan Telnet hanya karena raw referensi
CLI kini tersedia.

## R9.34 working one-time LAB read adapter

A real owner-interactive scripted SSH `show card` completed on C320;
protected Rust normalization confirmed all three previously seen
physical cards INSERVICE. This proves the IPAT LAB adapter can read
real hardware, not that an unattended limited-privilege SaaS worker,
physical identity, independent reviewer or native recovery is ready.
See R934_C320_EPHEMERAL_ONE_SHOT_LAB_SSH_READ.md for verified scope.
