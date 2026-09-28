# R9.17 — Koneksi langsung lebih dahulu dan istilah operasional Indonesia

Status 2026-09-28: genuine owner-VPS no-auth TCP/TLS transport check;
new private laboratory UI/API planning only. NO physical OLT login,
no worker dispatch, no live production tenant API, no WireGuard
activation, no customer/router/OLT/ONT changes.

## Penamaan UI resmi IPAT, bukan klaim penetapan SNI

| Istilah UI yang digunakan | Makna |
|---|---|
| Server Pusat IPAT | Server/backend/worker dan pengelola tunnel pusat jika dipilih |
| Gateway Lokasi | Router di POP/lokasi yang operator lokasi konfigurasi sendiri |
| Jaringan Manajemen | Jalur IP menuju antarmuka manajemen perangkat |
| Metode Koneksi | Pilihan langsung SSH, SNMPv3, HTTPS, CWMP/USP; VPN opsional |
| Kandidat Perangkat | Perangkat teramati yang belum diadopsi |

`site_a` dan `site_b` tetap sebagai internal LAB-only field pada API
legacy untuk kompatibilitas tes historis sampai ada migrasi API
berversi; JANGAN tampilkan keduanya sebagai nama UI atau istilah SOP.
Ini penamaan berbahasa Indonesia untuk kebutuhan operasional IPAT,
BUKAN pernyataan bahwa SNI menetapkan nama komponen tersebut.

## Keputusan direct-first

Jika jalur jaringan yang sudah ada dapat mencapai perangkat,
protokol didukung oleh model/firmware sesungguhnya, enkripsi dan
identitas endpoint dapat diverifikasi, ACL perangkat dan segmen
manajemen memadai, gunakan koneksi LANGSUNG; WireGuard/IPsec TIDAK
menjadi prasyarat. RFC1918 dan `ip route get` saja tidak membuktikan
segmen manajemen aman, namun rute via gateway default juga TIDAK
berarti alamat privat mustahil terjangkau. Internet-facing admin API
jangan dibuka begitu saja: batasi alamat asal, sertifikat, hak akses,
pencatatan dan pengendalian penyalahgunaan. VPN tambahan dapat
berguna jika perlu isolasi lintas jaringan, NAT yang tidak bisa
diatur, akses aman site luar atau kebijakan tenant, bukan pemaksaan
setiap adopsi.

Perangkat menggunakan adapter masing-masing, bukan API-SSL umum:
- RouterOS 7: RouterOS API-SSL (biasanya TCP 8729) atau HTTPS `/rest`
  (`www-ssl`) hanya setelah sertifikat perangkat diverifikasi dan
  peran baca-saja dibuktikan pada RouterOS firmware aktual.
- ZTE C320: SSH CLI dengan **host key pin independen** menjadi
  kandidat pertama karena transport SSH privat dari VPS SUDAH
  pernah teramati. SNMPv3 authPriv alternatif hanya bila konfigurasi
  firmware nyata diketahui. Jangan menyamakan ONT TR-069 dengan
  remote OLT API, dan jangan menyimpulkan HTTPS API generik ada.
- C-DATA: periksa model, revisi firmware dan protokol vendor secara
  independen. SSH/SNMPv3/HTTPS adalah KANDIDAT, bukan kompatibilitas.
- ONT kompatibel: ACS CWMP/TR-069 melalui HTTPS atau native USP/TR-369
  melalui pengontrol terautentikasi, sesuai protokol yang benar-benar
  didukung unit dan firmware. Jangan memaksakan API-SSL router.

Public references checked 2026-09-28:
https://manual.mikrotik.com/docs/developer-guides/api/
https://manual.mikrotik.com/docs/developer-guides/rest-api/
https://support.zte.com.cn/support/docmap/00000455/en/index.html
Historical C320 product documentation includes SSH and SNMPv3; actual
DEV-01 firmware capability and configured account remain UNVERIFIED.

## Actual owner-VPS one-connection no-auth TLS-443 negative evidence

`deploy/scripts/lab/r917/private_tls_noauth_probe.py` is nonroot,
default OFF, exact one private IPv4 and one TCP/443 candidate, max 3
seconds, uses system CA+identity verification, no HTTP request,
credentials, authenticated OLT command, brute-force retries or TLS
validation bypass. Actually ran ONCE from authorized IPAT VPS to
the previously owner-supplied PRIVATE DEV-01 address at port 443.
Result: TCP NOT REACHABLE from this source at test time; no TLS
server identity verified; no HTTPS API proven. Actual existing
private :3002 dashboard HTTP200 both before and after. This does
NOT disprove other non-443 vendor management ports; never scan them
blindly. Earlier documented actual direct private SSH connection
received untrusted `ZTE_SSH.1.0` from DEV-01; NO credentials or
commands. Neither observed TCP transport nor two identical network
fingerprints proves actual device chassis identity, protected last
hop, firmware support or production no-impact acceptance.

Do not use the chat-disclosed factory account over public Telnet or
unverified SSH. Before ONE bounded actual authenticated read, require
independently confirmed trusted device RSA host key, approved restricted
account and exact read-only command, genuine signed tenant MFA and
separate review, verified last-hop ACL/isolation and live baseline+
abort. Collect real SSH model/firmware output first then reconcile
manufacturer matrix and mark the specific protocol validated; only
then may an audited genuine tenant inventory adoption flow be enabled.

## What is actually implemented R9.17

- Pure offline `protocol_selection.py` rejects API-SSL as assumed
  capability of ZTE C320, prioritizes direct existing network and
  marks every claim of device authentication/adoption FALSE.
- Restricted noauth TCP/443 strict TLS preflight (no device HTTP).
- Private LAB server `/lab/demo/direct-protocol-review` accepts
  ONLY device-class, candidate-protocol and historical path ENUMs,
  not credentials, private target addresses or jobs. Backend
  returns the device-specific next proof gate, and force-disables
  login, inventory adoption and worker dispatch.
- Private Device Manager uses Indonesian Server Pusat IPAT and
  Gateway Lokasi labels and an actual Rust-backed direct protocol
  selection form. Existing historical `site_a` internal field IDs
  persist for test compatibility only. This is still PRIVATE LAB
  not the commercial tenant/identity-enabled production dashboard.

R9.17 tests are chained into the locked existing R9.0 safety suite;
latest PR is layered on top of outstanding R9.16 DRAFT due GitHub
Actions account billing failure. Do not merge/deploy into production
without latest CI and tenant-safe real protocol worker E2E.
