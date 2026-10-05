"""Read-only release-evidence fail-closed tests, no network and no device."""
import unittest
import sys
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parent))
import r979_owner_release_evidence as m


class OwnerReleaseEvidence(unittest.TestCase):
    def test_no_owner_checkbox_promotes_release_without_proof(self):
        with patch.object(m.r963, 'remote', return_value={
            'available': True, 'PRIVATE_C320': 'CONNECTED',
            'SERVICES': 'active,active,', 'OS': 'ubuntu:26.04',
            'PORT443': '0', 'PRIVILEGED_ADMIN': 'NOT_AVAILABLE',
            'PRIVATE_DOMAIN_HTTPS': 'False',
            'PRIVATE_DOMAIN_SAFE_TO_POINT': 'False',
        }), patch.object(m.r963, 'snapshot', return_value={
            'private_c320_observation': 'VPS_LOOPBACK_CONNECTED',
            'private_services': 'active,active,',
            'gates': {
                'public_tcp_443_from_owner_mac': False,
                'public_domain_runtime_attests_https': False,
                'public_domain_runtime_attests_safe_to_point': False,
                'real_privileged_deployment_identity': False,
            },
        }), patch.object(m, 'public_dns_status', return_value='NXDOMAIN'):
            x = m.report('ipat.id')
        self.assertFalse(x['public_go'])
        self.assertEqual(x['release_decision'], 'BLOCKED')
        self.assertEqual(x['observed']['public_dns_lookup_status'], 'NXDOMAIN')
        self.assertEqual(x['observed']['private_c320_status'], 'VPS_LOOPBACK_CONNECTED')
        self.assertEqual(len(x['evidence_items']), 5)
        self.assertTrue(all(x['evidence_items'][k]['owner_reported_ready']
                            and not x['evidence_items'][k]['verified']
                            for k in x['evidence_items']))

    def test_dns_never_queries_untrusted_name(self):
        with patch.object(m.subprocess, 'run') as run:
            for invalid in ['--help', 'ipat.id;whoami', 'localhost',
                            'sub..ipat.id', 'https://ipat.id']:
                with self.assertRaises(ValueError):
                    m.public_dns_status(invalid)
            run.assert_not_called()

    def test_public_dns_fail_closed_when_unavailable(self):
        with patch.object(m.subprocess, 'run', side_effect=OSError):
            self.assertEqual(m.public_dns_status('example.org'),
                             'UNVERIFIED_QUERY_UNAVAILABLE')

    def test_public_dns_nxdomain_is_not_ownership_proof(self):
        class P:
            returncode = 0
            stdout = ';; ->>HEADER<<- opcode: QUERY, status: NXDOMAIN, id: 88'
        with patch.object(m.subprocess, 'run', return_value=P()):
            self.assertEqual(m.public_dns_status('ipat.id'), 'NXDOMAIN')

    def test_private_ssh_failure_keeps_no_go(self):
        with patch.object(m.r963, 'remote', return_value={'available': False}), \
             patch.object(m.r963, 'snapshot', return_value={
                 'private_c320_observation': 'UNVERIFIED_VPS_LOOPBACK',
                 'private_services': None,
                 'gates': {
                     'public_tcp_443_from_owner_mac': False,
                     'public_domain_runtime_attests_https': False,
                     'public_domain_runtime_attests_safe_to_point': False,
                     'real_privileged_deployment_identity': False,
                 },
             }):
            x = m.report()
        self.assertFalse(x['public_go'])
        self.assertEqual(x['next_gate'],
                         'RESTORE_READ_ONLY_PRIVATE_ACCESS_FOR_INDEPENDENT_INSPECTION')


if __name__ == '__main__':
    unittest.main()
