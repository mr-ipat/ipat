import importlib.util
import os
from pathlib import Path
import unittest
from unittest.mock import patch
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("r1001",HERE/"r1001_c320_exact_tuple_read.py")
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
CARDS=b"""Rack Shelf Slot CfgType RealType Port HardVer SoftVer Status
1 1 1 GTGO GTGOG 8 120301 V2.0.0 INSERVICE
1 1 3 SMXA SMXA 0 110702 V2.0.0 INSERVICE
1 1 4 SMXA SMXA 0 110702 V2.0.0 STANDBY
"""
VERS=b"""PhyLoc FileType VerType VerTag BuildTime VerLength
1/1/1 GTGOG MVR V4.0.1 2026-01-01 01:02:03 100
1/1/1 GTGOG FW FW-A 2026-01-01 01:02:03 200
1/1/3 SMXA MVR V4.0.2 2026-02-02 03:04:05 101
"""
class TestTuple(unittest.TestCase):
    def test_exact_card_mvr_correlation(self):
        c=m.parse_cards(CARDS);v=m.parse_versions(VERS);x=m.correlate(c,v)
        self.assertEqual(len(c),2);self.assertEqual(len(v),3)
        self.assertEqual(x[0]["real_type"],"GTGOG")
        self.assertEqual(x[0]["running_version_tag"],"V4.0.1")
        self.assertEqual(x[1]["running_file_type"],"SMXA")
    def test_configured_alias_same_slot_allowed(self):
        v=m.parse_versions(VERS.replace(b"GTGOG MVR",b"GTGO MVR"))
        self.assertEqual(m.correlate(m.parse_cards(CARDS),v)[0]["running_file_type"],"GTGO")
    def test_mismatch_missing_duplicate_denied(self):
        c=m.parse_cards(CARDS)
        with self.assertRaises(m.Denied):m.correlate(c,m.parse_versions(VERS.replace(b"GTGOG MVR",b"BOGUS MVR")))
        with self.assertRaises(m.Denied):m.correlate(c,m.parse_versions(VERS.replace(b"1/1/3 SMXA MVR",b"1/1/2 SMXA MVR")))
        with self.assertRaises(m.Denied):m.parse_cards(CARDS+CARDS.splitlines()[1]+b"\n")
    def test_control_and_shape_rejected(self):
        with self.assertRaises(m.Denied):m.parse_cards(CARDS+b"\x1b[0m")
        with self.assertRaises(m.Denied):m.parse_versions(VERS.replace(b"MVR",b"BAD",1))
    def test_diagnostic_exposes_only_sanitized_mapping_candidates(self):
        c=m.parse_cards(CARDS)
        v=m.parse_versions(VERS.replace(b"GTGOG MVR",b"GTGOX MVR"))
        x=m.mapping_diagnostics(c,v)
        self.assertEqual(x[0],{
            "location":"1/1/1","configured_type":"GTGO","real_type":"GTGOG",
            "hardware_version":"120301","software_version":"V2.0.0",
            "mvr_file_types":["GTGOX"],"mvr_version_tags":["V4.0.1"],
            "mvr_builds":["2026-01-01T01:02:03"]})
        self.assertNotIn("serial",str(x).lower())

    def test_no_optin_denies_before_secret_or_network(self):
        with patch.dict(os.environ,{},clear=False),patch.object(m,"status_idle") as status,patch.object(m,"load_r945") as load:
            os.environ.pop(m.OPTIN,None)
            with self.assertRaises(m.Denied):m.collect()
            status.assert_not_called();load.assert_not_called()
if __name__=="__main__":unittest.main()
