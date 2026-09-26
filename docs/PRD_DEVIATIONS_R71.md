# PERINGATAN MERAH — Gap PRD dan Pengujian ZTE C320 (R7.1)

**26 September 2026 · Developer: Mr. iPat · KEPUTUSAN: NO_GO OLT FISIK / FIRMWARE**

> **BELUM TERPENUHI:** Dashboard tiga ruang kerja masih pratinjau privat; OLT
> ZTE C320 fisik belum pernah terhubung, diperiksa atau diperbarui firmware-nya.
> Tidak boleh ada klaim kompatibilitas, dukungan firmware ataupun SLA.

## Audit terhadap PRD asli, bukan rekaan persyaratan baru

| PRD / Target | Status terverifikasi | Kekurangan |
|---|---|---|
| FR-016, DEV-01 ZTE C320 dan TC-OLT-01 | Target terdaftar; parser Rust dua perintah READ-ONLY sintetis R7.1 | Tipe kartu, firmware, akses tepercaya dan discovery nyata belum ada |
| FR-017 adapter per versi/protokol/fitur | Tidak ada tuple perangkat nyata validated | Terlebih dahulu perlu satu perintah aman terbukti pada actual C320 |
| FR-001..006 dan FR-029 tiga dashboard | Tiga pratinjau privat; API bisnis tetap HTTP 401 | Real OIDC MFA, membership tenant, POP, RBAC+ABAC/RLS dan telemetry |
| FR-009/010 ONT lewat ACS asli | Rust parser, mTLS dan RPC hanya terbukti di laboratorium | Belum ada ONT sungguhan dan sesi HTTPS CWMP+tenant yang valid |
| Permintaan update firmware hari ini | Tambahan BERISIKO TINGGI, bukan jaminan S1 dalam Project Brief | Image resmi cocok tiap kartu, hash, restore, approval, jadwal, rollback belum tersedia |

**Pembedaan penting:** FR-016 secara eksplisit mengizinkan simulator bila
perangkat fisik belum tersedia. Parser baru itu progres yang mematuhi
cakupan MVP, tetapi TIDAK memenuhi harapan menghubungkan OLT nyata.
Firmware update tidak dijanjikan pada tujuh hari pertama; keamanan
perubahan tetap tunduk FR-017, RBAC/ABAC dan approval PRD.

## Implementasi R7.1 yang sebenarnya

- Rust crate olt-core hanya memproses teks transkrip sintetis dua
  calon perintah baca: show card dan show version-running. Tidak memiliki
  driver jaringan, kredensial, upload, firmware execution ataupun
  endpoint tulis; WRITE_ENABLED tetap false.
- Parser memakai batas output, menolak control characters, format
  yang tidak dikenal, slot ganda, versi ganda dan input berbahaya.
  Output terstruktur hanya posisi, jenis, status kartu serta versi.
  Parser TIDAK menyimpan transkrip, serial ataupun secret asli.
- Firmware review adalah pemeriksaan kelengkapan bukti pada sisi
  software saja: hasil maksimal HumanReviewOnly, BUKAN boleh upgrade.
  Boolean yang dimasukkan ke software bukan pengganti bukti independen
  atau tandatangan persetujuan manusia.
- Ketiga dashboard privat menampilkan banner MERAH BESAR tentang
  ketidaksiapan nyata, TC-OLT-01 dan upgrade firmware tertahan.

## Gerbang agar satu unit C320 bisa benar-benar diuji

1. Dapatkan akses pemilik yang disetujui, verifikasi unit fisik DEV-01
   dari konsol atau jalur manajemen yang independen dan tepercaya.
   Catat jenis chassis, HW revision, semua kartu kontrol/PON/uplink
   dan running firmware/build masing-masing, tanpa menyimpan raw
   serial, alamat management atau secret di Git/chat.
2. Validasi jalur SSH ber-host-key tepercaya dan akun khusus
   read-only jika tersedia; jika tidak, evaluasi SNMPv3 atau vendor
   channel yang BENAR-BENAR tersedia pada firmware aktual.
   Jangan membuka Telnet/SNMPv2/FTP ke internet demi uji.
3. Hanya setelah dokumentasi cocok firmware aktual, jalankan
   perintah baca yang telah diverifikasi. Redaksi identitas pribadi
   dan rahasiakan raw output. Output yang formatnya berbeda
   harus fail closed, bukan menebak kompatibilitas.
4. Beri label evidence physical hanya setelah tes nyata
   tenant/device binding, review dan negasi akses lulus.

## Gerbang terpisah sebelum update firmware OLT

ZTE memiliki dokumentasi produk C320 resmi, dan referensi
maintenance/upgrade lama menggarisbawahi backup, status kartu,
alarm, urutan sesuai rilis dan uji pasca-upgrade. Referensi lama
B U K A N otorisasi memilih firmware untuk perangkat 2026.
Perlu release notes resmi yang sesuai exact firmware dan kartu
yang terpasang, paket vendor resmi beserta checksum yang
diverifikasi independen; uji restore backup, status alarm
sehat, penilaian dampak pelanggan, maintenance window
disetujui, maker-checker independen, operator konsol fisik
dan rollback teruji. Setiap kekurangan berarti BLOCKED.
Walaupun semua bukti terpenuhi, versi R7.1 tetap HUMAN_REVIEW
ONLY dan tidak dapat melakukan upgrade karena belum ada
aktuator firmware atau pengujian keselamatan dengan OLT.

Referensi publik yang harus dicocokkan dengan rilis aktual:
https://support.zte.com.cn/support/docmap/00000455/en/operation.html
Dokumen ZXA10 C300/C320 Command Reference dan C320 Maintenance
Manual hanyalah pembanding awal, bukan sertifikasi satu unit.

## Keputusan proyek

ADR-001/002/012 tetap berlaku; FR-016 maju hanya untuk parser
offline, TC-OLT-01 dan FR-017 write tetap NOT RUN/BLOCKED.
Tidak ada perubahan firewall penyedia, jaringan customer,
K3s, PostgreSQL dan firmware unit fisik pada milestone ini.
