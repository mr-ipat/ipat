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

## Aktualisasi dashboard privat VPS R9.17 — benar-benar dijalankan

The full R9.17 application-source SHA
`7031924e8634d66fcc0b8256b7f6581808b54367` was transferred from
the authorized owner Mac to a NEW 0700 nonroot VPS checkout under
`/home/openai/.cache/ipat/r917-release/src`, using an independently
hash-verified Git delta bundle SHA256
`6cdbf0b9db958cec0d8a471274965a97a73577fa3b7d21f6916a9d498460bc46`.
The previous R9.16 preview source and compiled runtime were not
modified. Existing build cache was COPIED into an isolated r917
build directory; pinned `cargo fmt --all -- --check`, full locked
offline `cargo test --locked --offline -p control-api` and locked
control-api build PASSED. Rust control API 41/41 actual test cases
passed, including new deny-by-default per-device protocol handler.
Compiled R9.17 binary SHA256:
`dac06ed8e93ebaa35e8e0ba138cfdc44ce4ddaad116258cd5dadeed25fc182db`.

Review-only private LAB service unit:
`deploy/scripts/lab/r917/ipat-r911-preview.service` SHA256
`c7c8def73e2b087103f550fd2c67c135032b9e6ada5e7c4c89de9900d706804b`.
Actual no-secret HTTP smoke:
`deploy/scripts/lab/r917/actual_private_direct_protocol_http_smoke.py`
SHA256 `d75a8ffcf5ea5b310c34443e9a9b84c921a2155fabf930012e119963490e5543`.
Strict owner-only systemd user upgrade and rollback script:
`deploy/scripts/lab/r917/deploy_private_direct_protocol_preview.sh`
SHA256 `ca1e82cde07bf4787a0851b4df8ecca52eb1233e9beaaa8b9a4100543729bf63`.

First R9.17 on-VPS startup failed and safely rolled back because the
initial user unit accidentally used `r917-preview` while the verified
new build resided under `r917-release`. A temporary diagnostic run
then observed the handler was not yet accepting loopback connections
immediately after systemd reported ACTIVE. The final versioned
release unit fixed the actual path and the release script now waits
with a fixed bounded loop for the **PRIVATE loopback** HTML route
before running HTTP smoke, and reuses a pinned independently verified
prior user-unit backup. No OLT/Router B credential or device traffic
was generated by any of these private server tests.

Actual final owner-VPS nonroot rollout PASSED. The Rust private LAB
`127.0.0.1:3002` served R9.17 Indonesian Server Pusat IPAT/Gateway
Lokasi HTML/JavaScript, approved direct ZTE SSH candidate from
historical no-auth evidence, denied MikroTik-only API SSL for ZTE,
validated MikroTik secure API as a *candidate* only, denied unknown
password fields and missing Origin, preserved old R9.16 optional
manual-disabled RouterOS B key review, denied real business API routes,
and checked strict historical TCP443 negative evidence with adoption
FALSE. Original `127.0.0.1:3000` API remained HTTP200 before/after.
No WG listener, device login, commands or changes were made.

A separate on-owner-Mac temporary SSH local forwarding session also
ACTUALLY loaded current R9.17 HTML, confirmed Server Pusat IPAT and
Gateway Lokasi labels, fetched historical TLS443 negative evidence
and posted a secure private synthetic lab direct-C320-SSH selection
against the real deployed Rust backend: all fail-closed checks PASS;
the temporary Mac forwarding process was then closed.

Rollback remains owned by the nonroot `openai` VPS account:

```sh
ssh ipat-lab
cp -p /home/openai/.cache/ipat/r917-release/rollback-unit.service \
  ~/.config/systemd/user/ipat-r911-preview.service
systemctl --user daemon-reload
systemctl --user reset-failed ipat-r911-preview.service
systemctl --user restart ipat-r911-preview.service
```

Operator Mac private LAB access (not tenant production):

```sh
ssh -N -L 3302:127.0.0.1:3002 ipat-lab
# http://127.0.0.1:3302/lab/device-workbench
```

The real signed tenant dashboard/tenant-scoped production-adoption
worker is NOT mounted, so no authenticated ZTE C320 read has occurred.
The existing GitHub R9.16 PR #117 remains DRAFT due billing-spend
blockers. R9.17 PR #118 stacks on #117 and cannot be merged into
main or treated as commercial production acceptance while the full
latest CI/real hardware gates remain incomplete.
