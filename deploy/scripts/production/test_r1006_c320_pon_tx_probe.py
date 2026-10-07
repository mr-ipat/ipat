import importlib.util
from pathlib import Path
import unittest
P=Path(__file__).with_name("r1006_c320_pon_tx_probe.py")
S=importlib.util.spec_from_file_location("r1006",P);m=importlib.util.module_from_spec(S);S.loader.exec_module(m)

class Probe(unittest.TestCase):
    def test_numeric_exact_value_only(self):
        out=m.parse(b"show pon power olt-tx gpon-olt_1/1/1\r\nTx power: 2.106(dbm)\r\n")
        self.assertEqual(out["tx_power_dbm"],2.106)
        self.assertTrue(out["measurement_available"])
        self.assertTrue(out["exact_command_echo_removed"])
        self.assertEqual(out["physical_writes"],0)
        self.assertFalse(out["diagnostic_health_inferred"])
    def test_exact_c320_channel_table_shape(self):
        raw=(b"show pon power olt-tx gpon-olt_1/1/1\r\n"
             b"Channel             Tx power\r\n"
             b"------------------------------------\r\n"
             b"1(GPON)             3.488(dbm)\r\n")
        out=m.parse(raw)
        self.assertEqual(out["tx_power_dbm"],3.488)
        na=m.parse((b"Channel             Tx power\r\n"
                    b"------------------------------------\r\n"
                    b"1(GPON)             N/A\r\n"))
        self.assertFalse(na["measurement_available"])
        self.assertIsNone(na["tx_power_dbm"])
        for bad in [
            raw.replace(b"1(GPON)",b"2(GPON)"),
            raw.replace(b"Channel             Tx power\r\n",b""),
        ]:
            with self.assertRaises(m.Denied):m.parse(bad)

    def test_na_is_measurement_unavailable_not_health(self):
        out=m.parse(b"show pon power olt-tx gpon-olt_1/1/1\r\nTx power: N/A(dbm)\r\n")
        self.assertIsNone(out["tx_power_dbm"]);self.assertFalse(out["measurement_available"])
    def test_rejects_raw_onu_or_secret_shape(self):
        for payload in [
            b"gpon-onu_1/1/1:1 Tx power: 1.0(dbm)\n",
            b"Serial: ABC\nTx power: 1.0(dbm)\n",
            b"subscriber x\nTx power: 1.0(dbm)\n",
        ]:
            with self.assertRaises(m.Denied):m.parse(payload)
    def test_rejects_ambiguous_unexpected_or_implausible(self):
        for payload in [
            b"Tx power: 1.0(dbm)\nTx power: 2.0(dbm)\n",
            b"Rx power: -20.0(dbm)\n",
            b"Tx power: 99.0(dbm)\n",
            b"something else\n",
        ]:
            with self.assertRaises(m.Denied):m.parse(payload)
    def test_shape_classifier_never_returns_arbitrary_token_values(self):
        shaped=m.sanitized_shape([
            b"Channel Tx power",
            b"1(GPON) 3.488(dbm)",
            b"secret-customer-name",
        ])
        self.assertEqual(shaped,
            "CHANNEL,TX,POWER|GPON_CHANNEL_1,DBM_VALUE|ATOM_OTHER")
        self.assertNotIn("secret",shaped)
        self.assertNotIn("3.488",shaped)

    def test_controls_and_bounds_fail_closed(self):
        with self.assertRaises(m.Denied):m.parse(b"\x1b[31mTx power: 1(dbm)\n")
        with self.assertRaises(m.Denied):m.parse(b"x"*(m.MAX_BYTES+1))
    def test_status_contract_requires_canonical_no_write_fields(self):
        src=Path(m.__file__).read_text()
        self.assertIn('value.get("device_adopted") is not False',src)
        self.assertIn('value.get("device_writes") != 0',src)
        self.assertNotIn('value.get("physical_writes_enabled")',src)
        self.assertNotIn('value.get("production_adopted")',src)

    def test_fixed_command_has_no_runtime_target_input(self):
        self.assertEqual(m.COMMAND,"show pon power olt-tx gpon-olt_1/1/1")
        self.assertEqual(m.PORT,"1/1/1")
        self.assertNotIn("config",m.COMMAND.lower())
        self.assertNotIn("onu_",m.COMMAND.lower())
if __name__=="__main__":unittest.main()
