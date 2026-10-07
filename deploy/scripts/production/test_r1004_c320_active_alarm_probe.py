import importlib.util
import unittest
from pathlib import Path

P=Path(__file__).with_name("r1004_c320_active_alarm_probe.py")
s=importlib.util.spec_from_file_location("r1004",P);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)

class FakeExpect:
    EOF=object();TIMEOUT=object()

class Child:
    def __init__(self,events):
        self.events=list(events);self.before=b"";self.sent=[]
    def sendline(self,x):self.sent.append(("line",x))
    def send(self,x):self.sent.append(("raw",x))
    def expect(self,patterns,timeout):
        i,b=self.events.pop(0);self.before=b;return i

class AlarmProbe(unittest.TestCase):
    def test_exact_command_echo_is_not_miscounted_as_alarm_payload(self):
        x=m.output_shape((m.COMMAND+"\r\n").encode(),0)
        self.assertTrue(x["exact_command_echo_removed"])
        self.assertTrue(x["payload_empty_after_exact_echo"])
        self.assertEqual(x["line_count"],0)
        self.assertEqual(x["byte_count"],0)
        self.assertTrue(x["current_command_accepted"])

    def test_shape_never_returns_raw_values(self):
        raw=(m.COMMAND.encode()+b"\r\n"+b"Alarm Severity Date Time Source Status\n"
             b"1 Major 2026-10-07 10:00 x/y ACTIVE\n"
             b"2 Minor 2026-10-07 10:01 x/z ACTIVE\n")
        x=m.output_shape(raw,0)
        self.assertTrue(x["current_command_accepted"])
        self.assertEqual(x["physical_writes"],0)
        self.assertFalse(x["raw_alarm_records_returned"])
        self.assertNotIn("x/y",repr(x))
        self.assertNotIn("Major",repr(x))
        self.assertTrue(x["generic_header_keyword_flags"]["alarm"])
    def test_rejected_command_detected_without_payload(self):
        x=m.output_shape(b"% Invalid input detected\n",0)
        self.assertTrue(x["current_command_rejected"])
        self.assertFalse(x["current_command_accepted"])
        self.assertNotIn("Invalid input",repr(x))
    def test_pager_only_sends_space_and_is_bounded(self):
        c=Child([(1,b"head\n"),(0,b"tail\n")])
        raw,pages=m.paged_read(c,b"#",FakeExpect)
        self.assertEqual(raw,b"head\ntail\n")
        self.assertEqual(pages,1)
        self.assertEqual(c.sent,[("line",m.COMMAND.encode()),("raw",b" ")])
    def test_controls_and_unsafe_backspace_fail_closed(self):
        with self.assertRaises(m.Denied):m.output_shape(b"ok\x1b[0m\n",0)
        with self.assertRaises(m.Denied):m.output_shape(b"abc\x08def\n",1)
        with self.assertRaises(m.Denied):m.output_shape(b"abc\x08def\n",0)
    def test_command_is_exact_read_only_constant(self):
        self.assertEqual(m.COMMAND,"show alarm crtv-active")
        self.assertNotIn("config",m.COMMAND.lower())
        self.assertNotIn("write",m.COMMAND.lower())
        self.assertNotIn("delete",m.COMMAND.lower())

if __name__=="__main__":
    unittest.main()
