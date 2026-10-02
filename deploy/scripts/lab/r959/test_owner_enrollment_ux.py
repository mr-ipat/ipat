"""R9.59 static owner UX and physical-safety regression tests; no real secrets."""
from pathlib import Path
import re
import unittest
ROOT=Path(__file__).resolve().parents[4]
HTML=(ROOT/'web/lab/device-workbench.html').read_text()
JS=(ROOT/'web/lab/c320-connection-setup.js').read_text()
RUST=(ROOT/'apps/control-api/src/c320_live_lab.rs').read_text()
class OwnerUx(unittest.TestCase):
    def test_clipboard_flow_produces_success_ack_but_never_prints_secret(self):
        code=re.search(r'<code id="ipat-owner-code-command">(.*?)</code>',HTML)
        self.assertIsNotNone(code)
        command=code.group(1).replace('&amp;','&')
        self.assertIn('set -o pipefail;',command)
        self.assertIn('| pbcopy && printf',command)
        self.assertIn('IPAT: KODE TERSALIN',command)
        self.assertNotIn('echo $(',command)
        self.assertNotIn('| tee ',command)
        self.assertIn('Command + V',HTML)
        self.assertIn('bukan perintah Terminal',HTML)
    def test_persisted_pending_status_and_not_measured_connected(self):
        for phrase in ('OLT SUDAH TERSIMPAN','Lanjutkan Koneksi SSH',
                       'looksLikeCommand','owner.setCustomValidity',
                       'v.device_status===\'CONNECTED\''):
            self.assertIn(phrase,JS)
        self.assertIn('id="ipat-live-deployment-alert"',HTML)
        self.assertIn('id="ipat-owner-step-status"',HTML)
        self.assertIn('PERINGATAN PRD',HTML)
        self.assertIn('physical_writes_enabled',RUST)
        self.assertNotIn('localStorage',JS)
    def test_exact_private_rollout_preserves_old_binary_and_rejects_public(self):
        script=(ROOT/'deploy/scripts/lab/r959/rollout-private-owner-ui.sh').read_text()
        for required in ('R959_ROLLBACK_OLD_BINARY','r953-release/control-api',
                         '127.0.0.1:3002','R959_BASELINE_DRAFT_SAVED_NO_LIVE_AUTH',
                         'R959_ACTUAL_PRIVATE_HTTP_UI_AND_REAL_PENDING_CONNECTOR',
                         'physical_writes_enabled','R959_PRIVATE_OWNER_GUI_DEPLOYED'):
            self.assertIn(required,script)
        self.assertNotIn('bootstrap-token',script)
        self.assertNotIn('cat /home/openai/.local/share/ipat/r945-connection',script)

    def test_telnet_never_impersonates_existing_pinned_ssh(self):
        self.assertIn('Port Telnet laboratorium :323',HTML)
        self.assertIn('SSH 10.10.13.233:321',HTML)
        self.assertIn('input.ssh_port != 321',RUST)
        self.assertIn('TCP_REACHABLE_AUTH_NOT_TESTED',RUST)
        self.assertIn('prod',RUST)
if __name__=='__main__':unittest.main()
