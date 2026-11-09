import importlib.util
from pathlib import Path
from unittest import TestCase
from unittest.mock import patch
import os
import stat
import tempfile

path=Path(__file__).with_name("company_login_activation_audit.py")
spec=importlib.util.spec_from_file_location("r1055_audit",path)
m=importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class AuditContract(TestCase):
    def test_unprivileged_never_claims_missing_root_files_or_mfa(self):
        def fake(*args):
            if args[0]=="pg_lsclusters":
                return "18 ipat 5432 online postgres /dir /log"
            if args[0]=="ss":
                return "LISTEN 0 128 127.0.0.1:3003 0.0.0.0:*"
            if args[0]=="systemctl":
                return "active"
        with patch.object(m.os,"geteuid",return_value=1000), patch.object(m,"command",side_effect=fake):
            x=m.inspect()
        self.assertTrue(x["postgres18_ipat_online"])
        self.assertTrue(x["commercial_services_active"]["ipat-tenant-api"])
        self.assertEqual(x["protected_environment_files"]["tenant-api"],"unknown_requires_root")
        self.assertIsNone(x["protected_tenant_oidc_instances"])
        self.assertFalse(x["oidc_human_mfa_accepted"])
        self.assertFalse(x["public_production_go"])

    def test_no_tools_remains_fail_closed(self):
        with patch.object(m.os,"geteuid",return_value=1000),patch.object(m,"command",return_value=None):
            x=m.inspect()
        self.assertFalse(x["postgres18_ipat_online"])
        self.assertFalse(any(x["commercial_services_active"].values()))
        self.assertIsNone(x["listeners"]["443"])
        self.assertFalse(x["two_company_real_browser_login_accepted"])

    def test_metadata_rejects_symlink_directory_and_group_readable_secret(self):
        with tempfile.TemporaryDirectory() as td:
            base=Path(td)
            file=base/"a.env"
            file.write_text("DO_NOT_OUTPUT=not_a_real_secret")
            file.chmod(0o640)
            self.assertEqual(m.protected(file),"unsafe")
            self.assertEqual(m.protected(base),"unsafe")
            (base/"link.env").symlink_to(file)
            self.assertEqual(m.protected(base/"link.env"),"unsafe")
            self.assertEqual(m.protected(base/"missing.env"),"missing")
