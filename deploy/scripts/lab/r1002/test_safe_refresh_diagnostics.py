import ast
import json
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[4]
SRC=ROOT/'deploy/scripts/lab/r945/persistent_c320_connector.py'
SOURCE=SRC.read_text('utf-8')
TREE=ast.parse(SOURCE)
wanted={'safe_failure_stage','safe_response'}
nodes=[n for n in TREE.body if isinstance(n,ast.FunctionDef) and n.name in wanted]
if {n.name for n in nodes} != wanted:
    raise RuntimeError('expected diagnostic functions missing')
module=ast.Module(body=nodes,type_ignores=[])
ast.fix_missing_locations(module)
ns={'json':json}
exec(compile(module,str(SRC),'exec'),ns)
safe_failure_stage=ns['safe_failure_stage']
safe_response=ns['safe_response']

class SafeRefreshDiagnostics(unittest.TestCase):
    def test_specific_parser_errors_have_fixed_safe_codes(self):
        expected={
          'bounded command response rejected':'C320_REFRESH_BOUNDARY_REJECTED',
          'unconfigured ONU table unrecognized':'C320_UNCONFIGURED_TABLE_UNSUPPORTED',
          'unconfigured ONU rows not bounded':'C320_UNCONFIGURED_ROWS_OUT_OF_BOUNDS',
          'state output unsupported on actual firmware':'C320_STATE_HEADER_UNSUPPORTED',
          'actual ONU state rows unrecognized':'C320_STATE_ROWS_UNSUPPORTED',
          'actual state totals inconsistent':'C320_STATE_TOTALS_INCONSISTENT',
          'bounded PON config response incomplete':'C320_PON_CONFIG_SHAPE_INCOMPLETE',
          'state and config count discrepancy':'C320_STATE_CONFIG_COUNT_MISMATCH',
        }
        self.assertEqual({k:safe_failure_stage(k) for k in expected},expected)

    def test_unknown_exception_never_echoed(self):
        secret='root@example.invalid password=SHOULD_NOT_LEAK serial=SECRET'
        self.assertEqual(safe_failure_stage(secret),'DEVICE_CONNECTION_FAILED_UNCLASSIFIED')
        body=safe_response({'error':safe_failure_stage(secret),
            'physical_writes_enabled':False})
        self.assertNotIn(b'SHOULD_NOT_LEAK',body)
        self.assertNotIn(b'SECRET',body)
        self.assertLess(len(body),256)

    def test_codes_are_ascii_bounded_and_not_credentials(self):
        messages=[
          'bounded command response rejected','unconfigured ONU table unrecognized',
          'unconfigured ONU rows not bounded','state output unsupported on actual firmware',
          'actual ONU state rows unrecognized','actual state totals inconsistent',
          'bounded PON config response incomplete','state and config count discrepancy']
        for detail in messages:
            code=safe_failure_stage(detail)
            self.assertRegex(code,r'^[A-Z0-9_]{8,64}$')
            self.assertNotIn('PASSWORD',code)
            self.assertNotIn('SERIAL',code)
            self.assertNotIn('HOST',code)

if __name__=='__main__':
    unittest.main()
