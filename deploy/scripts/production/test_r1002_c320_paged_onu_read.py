import importlib.util,os
from pathlib import Path
import unittest
from unittest.mock import patch
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("r1002",HERE/"r1002_c320_paged_onu_read.py")
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class PX:
    EOF=object();TIMEOUT=object()

class Child:
    def __init__(self,events):
        self.events=list(events);self.before=b"";self.sent=[];self.lines=[]
    def sendline(self,v):self.lines.append(v)
    def send(self,v):self.sent.append(v)
    def expect(self,patterns,timeout):
        idx,chunk=self.events.pop(0);self.before=chunk;return idx

class Parser:
    COMMANDS=m.COMMANDS
    def bounded(self,raw,command):
        if b"BAD" in raw:raise RuntimeError("bad")
        return raw
    def classify(self,command,raw):
        if command==m.COMMANDS[0]:
            return {"shape":"NO_UNCONFIGURED_REPORTED","rows":0}
        if command==m.COMMANDS[1]:
            return {"shape":"ONU_STATE_TABLE","rows":72,"online":17,"offline":55}
        return {"shape":"PON_REGISTERED_REFERENCE","rows":72}

class Tests(unittest.TestCase):
    def test_pager_continuation_is_only_space_and_bounded(self):
        c=Child([(1,b"page1\n"),(1,b"page2\n"),(0,b"tail\n")])
        raw,pages=m.paged_read(c,b"PROMPT",m.COMMANDS[1],PX)
        self.assertEqual(raw,b"page1\npage2\ntail\n")
        self.assertEqual(pages,2)
        self.assertEqual(c.lines,[m.COMMANDS[1].encode()])
        self.assertEqual(c.sent,[b" ",b" "])
    def test_unknown_timeout_controls_and_page_bound_fail_closed(self):
        with self.assertRaises(m.Denied):m.paged_read(Child([]),b"P","show users",PX)
        with self.assertRaises(m.Denied):m.paged_read(Child([(3,b"x")]),b"P",m.COMMANDS[0],PX)
        with self.assertRaises(m.Denied):m.paged_read(Child([(0,b"\x1b[1m")]),b"P",m.COMMANDS[0],PX)
        c=Child([(1,b"x") for _ in range(m.MAX_PAGES+1)])
        with self.assertRaises(m.Denied):m.paged_read(c,b"P",m.COMMANDS[1],PX)
    def test_sanitized_counts_never_expose_ids_or_raw(self):
        raw={x:b"sanitized-test" for x in m.COMMANDS}
        pages={m.COMMANDS[0]:0,m.COMMANDS[1]:3,m.COMMANDS[2]:2}
        out=m.summarize(Parser(),raw,pages)
        self.assertEqual(out["configured"],72)
        self.assertEqual(out["online"],17)
        self.assertEqual(out["offline"],55)
        self.assertEqual(out["pagination"]["state_pages"],3)
        self.assertFalse(out["serials_returned"])
        self.assertFalse(out["onu_ids_returned"])
        self.assertEqual(out["physical_writes"],0)
        self.assertNotIn("raw",str(out).lower())
    def test_pager_normalization_only_allows_leading_erase_bytes(self):
        raw=(b"head\n"+b"\x08"*18+b" 1/1/1:2 enable disable OffLine 1(GPON)\n")
        clean=m.normalize_pager_artifacts(raw,1)
        self.assertNotIn(b"\x08",clean)
        self.assertIn(b"1/1/1:2",clean)
        with self.assertRaises(m.Denied):m.normalize_pager_artifacts(raw,0)
        with self.assertRaises(m.Denied):
            m.normalize_pager_artifacts(b"1/1/1:\x082 bad\n",1)
        with self.assertRaises(m.Denied):
            m.normalize_pager_artifacts((b"\x08 x\n")*(m.MAX_PAGES+1),m.MAX_PAGES)
    def test_state_diagnostic_is_aggregate_only(self):
        raw=(b"OnuIndex Admin State OMCC State Phase State Channel\n"
             b"1/1/1:2 enable enable working 1(GPON)\n"
             b"1/1/1:3 enable disable OffLine 1(GPON)\n"
             b"1/1/1:3 enable disable OffLine 1(GPON)\n"
             b"\x08\x08 1/1/1:4 enable disable OffLine 1(GPON)\n"
             b"ONU Number: 1 / 2\n")
        out=m.state_diagnostics(raw)
        self.assertEqual(out["state_row_count"],3)
        self.assertEqual(out["recoverable_rows_if_backspace_removed"],1)
        self.assertEqual(out["lines_with_backspace"],1)
        self.assertEqual(out["unique_state_row_count"],2)
        self.assertEqual(out["duplicate_state_rows"],1)
        self.assertEqual(out["phase_state_histogram"],{"OffLine":2,"working":1})
        self.assertEqual(out["summary_online"],1)
        self.assertEqual(out["summary_total"],2)
        self.assertFalse(out["onu_ids_returned"])
        self.assertFalse(out["raw_transcript_returned"])
        self.assertNotIn("1/1/1:2",str(out))
    def test_count_mismatch_denied(self):
        class Bad(Parser):
            def classify(self,command,raw):
                v=super().classify(command,raw)
                if command==m.COMMANDS[2]:v["rows"]=71
                return v
        with self.assertRaises(m.Denied):
            m.summarize(Bad(),{x:b"x" for x in m.COMMANDS},{x:0 for x in m.COMMANDS})
    def test_no_optin_denies_before_status_or_secret(self):
        with patch.dict(os.environ,{},clear=False),patch.object(m,"status_idle") as status,patch.object(m,"load_r945") as load:
            os.environ.pop(m.OPTIN,None)
            with self.assertRaises(m.Denied):m.collect()
            status.assert_not_called();load.assert_not_called()

if __name__=="__main__":unittest.main()
