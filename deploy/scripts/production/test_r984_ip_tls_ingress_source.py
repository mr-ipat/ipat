from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
APPLY=(ROOT/"scripts/production/r984_apply_ip_tls_ingress.sh").read_text()
HTTP=(ROOT/"nginx/r984-platform-ip-bootstrap.conf.template").read_text()
TLS=(ROOT/"nginx/r984-platform-ip-https.conf.template").read_text()
SERVICE=(ROOT/"systemd/ipat-ip-cert-renew.service.template").read_text()
TIMER=(ROOT/"systemd/ipat-ip-cert-renew.timer").read_text()
PREP=(ROOT/"scripts/production/r984_prepare_ip_tls_prereqs.sh").read_text()

class R984Source(unittest.TestCase):
    def test_no_firewall_dns_or_private_device_mutation(self):
        combined="\n".join([APPLY,HTTP,TLS,SERVICE,TIMER]).lower()
        for forbidden in ["iptables ","nft ","ufw ","firewall-cmd","10.10.13.233","127.0.0.1:3002"]:
            self.assertNotIn(forbidden,combined)

    def test_acme_ip_is_shortlived_webroot_and_staging_first_is_available(self):
        self.assertIn("--preferred-profile shortlived",APPLY)
        self.assertIn("--webroot-path",APPLY)
        self.assertIn("--ip-address",APPLY)
        self.assertIn("--staging",APPLY)
        self.assertIn("ipat-platform-ip-staging",APPLY)
        self.assertIn('delete --non-interactive --cert-name "$cert_name"',APPLY)
        self.assertIn("Certbot >= 5.4",APPLY)
        self.assertIn("-checkip",APPLY)

    def test_https_routes_only_platform_services(self):
        self.assertIn("proxy_pass http://127.0.0.1:3006",TLS)
        self.assertIn("proxy_pass http://127.0.0.1:3005",TLS)
        for forbidden in ["127.0.0.1:3002","127.0.0.1:3003","127.0.0.1:3004"]:
            self.assertNotIn(forbidden,TLS)
        self.assertIn('if ($host != "__IP__") { return 421; }',TLS)
        self.assertNotIn("$http_x_forwarded_host",TLS)

    def test_production_requires_both_backends_on_exact_loopback_before_ingress(self):
        backend=APPLY.index("Platform Owner API must listen exactly on 127.0.0.1:3005")
        armed=APPLY.index("systemd-run --quiet")
        rendered=APPLY.index('render "$bootstrap" "$site"')
        self.assertLess(backend,armed)
        self.assertLess(backend,rendered)
        self.assertIn("127.0.0.1:3006",APPLY)
        self.assertIn("/__r984_ingress_probe",APPLY)
        self.assertIn("must never bind wildcard public interfaces",APPLY)
        self.assertIn("https://$ip/platform/auth/oidc/start",APPLY)
        self.assertIn('[[ $oidc_status == 303 ]]',APPLY)

    def test_rollback_is_armed_before_site_change_and_no_force_renewal(self):
        armed=APPLY.index("systemd-run --quiet")
        rendered=APPLY.index('render "$bootstrap" "$site"')
        self.assertLess(armed,rendered)
        self.assertIn("rollback_now",APPLY)
        self.assertIn("default.enabled.link",APPLY)
        self.assertIn("existing default enabled site is not a symlink",APPLY)
        self.assertIn("managed enabled-site path must be a symlink or absent",APPLY)
        self.assertIn('rm -f -- "$default_enabled"',APPLY)
        self.assertNotIn("--force-renewal",APPLY)

    def test_failed_production_cutover_removes_new_renewal_artifacts(self):
        self.assertIn("existing managed renewal artifact requires explicit upgrade review",APPLY)
        self.assertIn("/bin/systemctl disable --now ipat-ip-cert-renew.timer",APPLY)
        self.assertIn('rm -f -- "\\$hook" "\\$service" "\\$timer"',APPLY)
        rollback=APPLY[APPLY.index('cat > "$rollback_script"'):APPLY.index('chmod 0700 "$rollback_script"')]
        self.assertIn("daemon-reload",rollback)

    def test_renewal_is_bounded_and_reload_hook_rechecks_exact_ip(self):
        self.assertIn("OnUnitActiveSec=6h",TIMER)
        self.assertIn("RandomizedDelaySec=30min",TIMER)
        self.assertIn("flock -n /run/ipat-certbot-ip.lock",SERVICE)
        self.assertIn("renew --cert-name ipat-platform-ip --quiet",SERVICE)
        self.assertIn("--dry-run --run-deploy-hooks",APPLY)
        self.assertIn("openssl x509 -in",APPLY)
        self.assertIn("-checkip",APPLY)

    def test_prereq_uses_current_snap_certbot_without_auto_start(self):
        self.assertIn("snap install --classic certbot",PREP)
        self.assertIn("snap refresh certbot",PREP)
        self.assertIn("/snap/bin/certbot",PREP)
        self.assertIn("must be >=5.4",PREP)
        self.assertIn("policy-rc.d",PREP)
        self.assertIn("systemctl stop nginx.service",PREP)
        self.assertNotIn("apt-get install -y certbot",PREP)
        for forbidden in ["iptables ","nft ","ufw ","firewall-cmd"]:
            self.assertNotIn(forbidden,PREP.lower())

    def test_http_unknown_host_is_not_open_redirect(self):
        self.assertIn('if ($host != "__IP__") { return 421; }',HTTP)
        self.assertIn("return 308 https://__IP__$request_uri",HTTP)
        self.assertNotIn("https://$host",HTTP)

if __name__=="__main__":
    unittest.main()
