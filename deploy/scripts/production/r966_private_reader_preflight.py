#!/usr/bin/env python3
"""Owner-only read-only import/recovery check, without network or credentials.

This checks the exact original C320 adapter's source provenance when the
canonical checkout has no private r938/r940. It is NOT an actual service cold
restart, physical device test, vendor-native backup, or public production GO.
"""
import hashlib
import importlib.util
import os
import socket
import sys
from pathlib import Path
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[3]
CONNECTOR=ROOT/'deploy/scripts/lab/r945/persistent_c320_connector.py'


def run():
    if os.geteuid()==0:
        return 'R966_READ_ONLY_PREFLIGHT_REFUSES_ROOT'
    if not CONNECTOR.is_file():
        return 'R966_CANONICAL_CONNECTOR_SOURCE_MISSING'
    spec=importlib.util.spec_from_file_location('r966_isolated_exact_connector',CONNECTOR)
    if not spec or not spec.loader:
        return 'R966_CONNECTOR_IMPORT_UNAVAILABLE'
    agent=importlib.util.module_from_spec(spec)
    # Never call serve/read; loading Python may not initiate a device SSH.
    with patch.object(socket,'socket',side_effect=RuntimeError('NO_NETWORK_IN_PREFLIGHT')):
        spec.loader.exec_module(agent)
        # A canonical production checkout must have no private hostname
        # or reader source in the repository. Otherwise do not claim the
        # off-cache fallback was actually verified.
        if agent.SOURCE.exists() or agent.SOURCE.is_symlink():
            return 'R966_CANONICAL_READER_NOT_ABSENT'
        resolved=agent.fixed_reader_source()
        stable=Path('/home/openai/.local/share/ipat/r966-private-reader-recovery')
        if not resolved.is_relative_to(stable):
            return 'R966_FIXED_PRIVATE_READER_PATH_UNEXPECTED'
        reader, parser=agent.module()
    if reader.__file__ != str(resolved) or reader.R938 != stable/'deploy/scripts/lab/r938/owner_c320_onu_first_inventory.py':
        return 'R966_INCORRECT_PRIVATE_RELATIVE_READER_DEPENDENCY'
    if not callable(reader.run_three_reads) or not callable(parser.classify):
        return 'R966_FIXED_READER_FUNCTIONS_MISSING'
    # Never access, print or probe the live credential file or device.
    return 'R966_REAL_OWNER_VPS_CLEAN_CHECKOUT_FIXED_PRIVATE_READER_IMPORT_NO_NETWORK=PASS'

if __name__=='__main__':
    try:
        result=run()
    except Exception:
        # Fail with a fixed string; private paths and host key contents
        # must never be printed in CI, ChatGPT or owner operator logs.
        result='R966_PRIVATE_READER_IMPORT_FAILED_CLOSED'
    print(result)
    sys.exit(0 if result.endswith('=PASS') else 3)
