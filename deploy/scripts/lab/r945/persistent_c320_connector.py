#!/usr/bin/env python3
"""IPAT R9.45 fixed-target owner-VPS persistent C320 read connector.
One-time browser enrollment (private SSH-tunneled laboratory dashboard);
encrypted credential, strictly pinned SSH and read-only CLI allowlist.
NOT a multitenant credential service or authorization for physical writes.
"""
import hashlib
import hmac
import importlib.util
import json
import os
import secrets
import signal
import socket
import socketserver
import stat
import sys
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

from cryptography.fernet import Fernet, InvalidToken

ROOT = Path('/home/openai/.local/share/ipat/r945-connection')
SOCKET = Path('/home/openai/.local/share/ipat/r940-live-agent/live.sock')
SOURCE = Path(__file__).resolve().parents[1] / 'r940' / 'owner_supervised_c320_read_agent.py'
# Canonical repo intentionally never contains private OLT host/user/pin data.
# ALL NEW connector startups use ONLY the previously owner-recovered
# 0600 exact checksum-pinned bundle, ignoring even extant legacy cache
# scripts. The currently running old connector is never restarted here.
# private adapter and its sibling 0600 R938 parser. No env/header path input.
RECOVERY_ROOT = Path('/home/openai/.local/share/ipat/r966-private-reader-recovery')
RECOVERY_R940_SHA256 = '5f262daba18f57cc188f96a5af12e9d461bbcad5a000e915910c678deeb7be10'
RECOVERY_R938_SHA256 = '9f4a6874b2236365928a55c7a74a71411f7c2e8d0cc753d01689d0e2f0e2205c'
TOKEN = ROOT / 'bootstrap-token'
TOKEN_HASH = ROOT / 'bootstrap-sha256'
KEY = ROOT / 'envelope-key'
CREDENTIAL = ROOT / 'device.fernet'
DRAFT = ROOT / 'device-draft.json'
MAX_REQUEST = 1024
MAX_ATTEMPTS = 5
WAIT_SECONDS = 900
MODE = 'OWNER_SUPERVISED_REAL_C320_READ_ONLY'
SAFE_ACTIONS = {b'REFRESH\n', b'CARDS\n', b'FIRMWARE\n'}
PAGED_COMMANDS = (
    'show gpon onu uncfg',
    'show gpon onu state gpon-olt_1/1/1',
    'show run interface gpon-olt_1/1/1',
)
PAGED_MAX_BYTES = 32768
PAGED_MAX_PAGES = 12
LOCK = threading.Lock()
CLI_LOCK = threading.Lock()
STATE = {'failures': 0, 'blocked_until': 0.0, 'last_verified': '', 'last_kind': '', 'poll_count': 0, 'last_monotonic': 0.0, 'ever_verified': False}
PARENT = SOCKET.parent


def private_file(path, expected=0o600):
    value = path.lstat()
    if not stat.S_ISREG(value.st_mode) or value.st_uid != os.getuid() or value.st_nlink != 1 \
            or stat.S_IMODE(value.st_mode) != expected:
        raise ValueError('invalid private owner file')


def private_dir(path):
    value = path.lstat()
    if not stat.S_ISDIR(value.st_mode) or value.st_uid != os.getuid() \
            or stat.S_IMODE(value.st_mode) != 0o700:
        raise ValueError('invalid private owner directory')


def secret_write(path, value):
    tmp = path.with_name(path.name + '.next')
    if tmp.exists() or tmp.is_symlink():
        raise ValueError('stale staging secret file')
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    try:
        with os.fdopen(fd, 'wb') as writer:
            writer.write(value)
            writer.flush()
            os.fsync(writer.fileno())
        os.replace(tmp, path)
    finally:
        if tmp.exists():
            tmp.unlink()


def init():
    os.umask(0o077)
    if os.geteuid() == 0:
        raise ValueError('root may not initialize device secrets')
    for path in [ROOT, PARENT]:
        if not path.exists():
            path.mkdir(mode=0o700)
        private_dir(path)
    if KEY.exists() or TOKEN.exists() or CREDENTIAL.exists():
        raise ValueError('bootstrap already initialized: no silent key regeneration')
    secret_write(KEY, Fernet.generate_key())
    token = secrets.token_urlsafe(32)
    secret_write(TOKEN, (token + '\n').encode('ascii'))
    secret_write(TOKEN_HASH, hashlib.sha256(token.encode('ascii')).digest())
    print('R945_PRIVATE_BROWSER_BOOTSTRAP_READY_TOKEN_IN_PRIVATE_FILE_ONLY')


def fixed_reader_source():
    # Never prefer a mutable/unknown adjacent legacy adapter over a sealed
    # owner-private source, even when an old stage cache happens to exist.
    # Existing *in-progress* r945 remains unchanged until separate cutover.
    # A new clean checkout therefore cannot depend on disposable caches.
    # Test stable private recovery provenance BEFORE importing any Python.
    parent = RECOVERY_ROOT.parent
    private_dir(parent)
    private_dir(RECOVERY_ROOT)
    base = RECOVERY_ROOT / 'deploy'
    lab = base / 'scripts' / 'lab'
    for part in (base, base / 'scripts', lab, lab / 'r938', lab / 'r940'):
        private_dir(part)
    reader = lab / 'r940' / 'owner_supervised_c320_read_agent.py'
    parser = lab / 'r938' / 'owner_c320_onu_first_inventory.py'
    for path, expected in ((reader, RECOVERY_R940_SHA256),
                           (parser, RECOVERY_R938_SHA256)):
        private_file(path)
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError('fixed private reader provenance mismatch')
    return reader


def module():
    spec = importlib.util.spec_from_file_location('r945_fixed_c320', fixed_reader_source())
    if not spec or not spec.loader:
        raise ValueError('fixed real SSH adapter missing')
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    reader_module = reader.module()
    reader_module.private_regular(reader_module.PIN)
    if not reader_module.PIN.read_text('ascii').startswith(
            f'[{reader_module.HOST}]:{reader_module.PORT} ssh-rsa '):
        raise ValueError('previously observed strict host-key pin unavailable')
    return reader, reader_module


def load_draft():
    if not DRAFT.exists():
        return None
    private_file(DRAFT)
    result = json.loads(DRAFT.read_text('utf-8'))
    if not isinstance(result, dict) or set(result) != {'device_name', 'device_profile'} or result['device_profile'] != 'zte_c320_lab' or not isinstance(result['device_name'], str) or not 1 <= len(result['device_name']) <= 64:
        raise ValueError('invalid draft metadata')
    return result


def save_draft(name):
    if not isinstance(name, str) or not 1 <= len(name.strip()) <= 64 or any(ord(c) < 32 or ord(c) == 127 for c in name):
        raise ValueError('invalid draft label')
    if enrolled():
        raise ValueError('already enrolled: immutable initial lab target')
    secret_write(DRAFT, safe_response({'device_name': name.strip(), 'device_profile': 'zte_c320_lab'}))
    return {'draft_saved': True, 'target': 'DEV-01', 'device_name': name.strip(),
            'adoption_state': 'DRAFT_SAVED_AWAITING_AUTH', 'physical_writes_enabled': False}


def enrolled():
    try:
        private_file(CREDENTIAL)
        return True
    except FileNotFoundError:
        return False


def get_password():
    private_file(KEY)
    private_file(CREDENTIAL)
    plaintext = Fernet(KEY.read_bytes()).decrypt(CREDENTIAL.read_bytes())
    if not 1 <= len(plaintext) <= 128:
        raise ValueError('invalid decrypted credential')
    return plaintext.decode('utf-8')


def safe_failure_stage(detail):
    """Map strict parser failures to fixed nonsecret diagnostic categories.

    Never return raw CLI, addresses, usernames, serials, passwords, or arbitrary
    exception text. Unknown failures remain generic.
    """
    mapping = {
        'bounded command response rejected': 'C320_REFRESH_BOUNDARY_REJECTED',
        'unconfigured ONU table unrecognized': 'C320_UNCONFIGURED_TABLE_UNSUPPORTED',
        'unconfigured ONU rows not bounded': 'C320_UNCONFIGURED_ROWS_OUT_OF_BOUNDS',
        'state output unsupported on actual firmware': 'C320_STATE_HEADER_UNSUPPORTED',
        'actual ONU state rows unrecognized': 'C320_STATE_ROWS_UNSUPPORTED',
        'actual state totals inconsistent': 'C320_STATE_TOTALS_INCONSISTENT',
        'bounded PON config response incomplete': 'C320_PON_CONFIG_SHAPE_INCOMPLETE',
        'state and config count discrepancy': 'C320_STATE_CONFIG_COUNT_MISMATCH',
    }
    return mapping.get(detail, 'DEVICE_CONNECTION_FAILED_UNCLASSIFIED')


def safe_response(payload):
    return (json.dumps(payload, separators=(',', ':'), sort_keys=True) + '\n').encode('ascii')


def verify_and_store(token, password, reader, reader_module):
    now = time.monotonic()
    with LOCK:
        if STATE['blocked_until'] > now:
            raise ValueError('enrollment temporarily limited')
        if STATE['failures'] >= MAX_ATTEMPTS:
            STATE['blocked_until'] = now + WAIT_SECONDS
            STATE['failures'] = 0
            raise ValueError('enrollment temporarily limited')
        STATE['failures'] += 1
    if enrolled():
        raise ValueError('already enrolled; rotation requires separate owner-approved operation')
    private_file(TOKEN_HASH)
    if not isinstance(token, str) or len(token) > 128 or \
       not hmac.compare_digest(hashlib.sha256(token.encode('utf-8')).digest(),
                               TOKEN_HASH.read_bytes()):
        raise ValueError('incorrect bootstrap code')
    if not isinstance(password, str) or not 1 <= len(password.encode('utf-8')) <= 128:
        raise ValueError('invalid credential length')
    # Credential is stored ONLY after the EXACT pinned real device answers.
    result = reader.run_three_reads(reader_module, password, b'CARDS\n')
    if result.get('cards_in_service', 0) < 1:
        raise ValueError('physical device card response unverified')
    private_file(KEY)
    secret_write(CREDENTIAL, Fernet(KEY.read_bytes()).encrypt(password.encode('utf-8')))
    TOKEN.unlink(missing_ok=True)
    TOKEN_HASH.unlink(missing_ok=True)
    with LOCK:
        STATE['last_verified'] = datetime.now(timezone.utc).isoformat()
        STATE['last_kind'] = 'CARDS'
        STATE['last_monotonic'] = time.monotonic()
        STATE['ever_verified'] = True
        STATE['failures'] = 0
    return {'enrolled_for_read': True, 'physical_card_count': result['cards_in_service'],
            'verified_at_utc': STATE['last_verified'],
            'host_identity_level': 'NETWORK_OBSERVED_SSH_PIN',
            'commercial_production_adopted': False, 'physical_writes_enabled': False}


def status():
    configured = enrolled()
    draft = load_draft()
    with LOCK:
        last = STATE['last_verified']
        last_kind = STATE['last_kind']
        recent = bool(last) and time.monotonic() - STATE['last_monotonic'] <= 360
        ever_verified = STATE['ever_verified']
    return {'mode': MODE, 'agent_ready': configured,
            'read_in_progress': CLI_LOCK.locked(),
            'requests_left': 5 if configured else 0,
            'seconds_left': 900 if configured else 0,
            'persistent_connector': True,
            'credentials_enrolled': configured,
            'draft_saved': draft is not None,
            'device_name': draft['device_name'] if draft else 'ZTE C320 Lab',
            'last_verified_at_utc': last,
            'last_verified_kind': last_kind,
            'host_identity_level': 'NETWORK_OBSERVED_SSH_PIN',
            'actual_olt_connectivity_verified': recent,
            'ever_verified_since_start': ever_verified,
            'device_adopted': False, 'device_writes': 0}


def _paged_safe_chunk(chunk):
    if not isinstance(chunk,bytes) or b'\0' in chunk or b'\x1b' in chunk:
        raise ValueError('unsafe paged CLI chunk')
    for byte in chunk:
        if byte < 32 and byte not in (8,9,10,13):
            raise ValueError('unsupported paged CLI control byte')

def _normalize_pager_artifacts(raw,pages):
    _paged_safe_chunk(raw)
    if not isinstance(pages,int) or not 0 <= pages <= PAGED_MAX_PAGES:
        raise ValueError('invalid pager count')
    if b'\x08' not in raw:
        return raw
    if pages == 0:
        raise ValueError('backspace without observed pager')
    lines=raw.splitlines(keepends=True)
    out=[];touched=0
    for line in lines:
        if b'\x08' not in line:
            out.append(line);continue
        touched += 1
        if touched > pages or line.count(b'\x08') > 32:
            raise ValueError('pager erase artifact exceeds bound')
        stripped=line.lstrip(b' \t\x08')
        prefix=line[:len(line)-len(stripped)]
        if b'\x08' in stripped or any(x not in b' \t\x08' for x in prefix):
            raise ValueError('backspace inside CLI data')
        out.append(line.replace(b'\x08',b''))
    clean=b''.join(out)
    if len(clean) > PAGED_MAX_BYTES or b'\x08' in clean:
        raise ValueError('pager normalization failed')
    return clean

def _paged_read(child,prompt,command,pexpect_module):
    if command not in PAGED_COMMANDS:
        raise ValueError('command outside paged read allowlist')
    child.sendline(command.encode('ascii'))
    chunks=[];total=0;pages=0
    while True:
        idx=child.expect([prompt,rb'--More--',pexpect_module.EOF,pexpect_module.TIMEOUT],timeout=15)
        chunk=child.before or b''
        _paged_safe_chunk(chunk)
        total += len(chunk)
        if total > PAGED_MAX_BYTES:
            raise ValueError('paged read byte bound exceeded')
        chunks.append(chunk)
        if idx == 0:
            return b''.join(chunks), pages
        if idx == 1:
            pages += 1
            if pages > PAGED_MAX_PAGES:
                raise ValueError('paged read page bound exceeded')
            child.send(b' ')
            continue
        if idx == 2:
            raise ValueError('paged read session closed')
        raise ValueError('paged read timed out')

def paged_refresh(reader_module,password):
    import pexpect
    m=reader_module
    if tuple(getattr(m,'COMMANDS',())) != PAGED_COMMANDS:
        raise ValueError('sealed parser command set changed')
    ssh=['ssh','-F','/dev/null','-tt','-p',str(m.PORT),
      '-o','HostKeyAlgorithms=ssh-rsa','-o','Ciphers=aes128-cbc',
      '-o','KexAlgorithms=diffie-hellman-group14-sha256',
      '-o','StrictHostKeyChecking=yes','-o',f'UserKnownHostsFile={m.PIN}',
      '-o','GlobalKnownHostsFile=/dev/null','-o','NumberOfPasswordPrompts=1',
      '-o','PreferredAuthentications=password','-o','PubkeyAuthentication=no',
      '-o','ProxyCommand=none','-o','ClearAllForwardings=yes',
      '-o','ConnectionAttempts=1','-o','ConnectTimeout=8',f'{m.USER}@{m.HOST}']
    c=None
    try:
        c=pexpect.spawn(ssh[0],ssh[1:],encoding=None,timeout=12,echo=False,maxread=65536)
        if c.expect([rb'(?i)password:\s*$',pexpect.EOF,pexpect.TIMEOUT]) != 0:
            raise ValueError('private SSH challenge missing')
        c.sendline(password.encode())
        if c.expect([m.PROMPT,pexpect.EOF,pexpect.TIMEOUT],timeout=10) != 0:
            raise ValueError('authenticated C320 prompt missing')
        raw={};pages={}
        for command in PAGED_COMMANDS:
            raw[command],pages[command]=_paged_read(c,m.PROMPT,command,pexpect)
        shapes=[]
        for command in PAGED_COMMANDS:
            clean=_normalize_pager_artifacts(raw[command],pages[command])
            shapes.append(m.classify(command,m.bounded(clean,command)))
        uncfg,state,config=shapes
        if state.get('rows') != config.get('rows'):
            raise ValueError('state and config count discrepancy')
        if state.get('online',0) + state.get('offline',0) != state.get('rows'):
            raise ValueError('state totals inconsistent')
        return {
          'mode':MODE,'read_kind':'ONU_INVENTORY','snapshot_is_live':True,
          'port':'1/1/1','unconfigured':uncfg.get('rows'),
          'configured':state.get('rows'),'online':state.get('online'),
          'offline':state.get('offline'),'configuration_rows':config.get('rows'),
          'serials_returned':False,'device_adopted':False,'provisioning_enabled':False,
          'device_writes':0,'pagination_pages':{
             'uncfg':pages[PAGED_COMMANDS[0]],'state':pages[PAGED_COMMANDS[1]],
             'config':pages[PAGED_COMMANDS[2]]}}
    finally:
        if c is not None:
            try:
                if c.isalive():
                    c.sendline(b'exit');c.expect(pexpect.EOF,timeout=2)
            except Exception:
                c.close(force=True)

def read(request, reader, reader_module):
    if not enrolled():
        raise ValueError('not enrolled')
    if not CLI_LOCK.acquire(blocking=False):
        raise ValueError('device read busy')
    try:
        if request == b'REFRESH\n':
            result = paged_refresh(reader_module, get_password())
        else:
            result = reader.run_three_reads(reader_module, get_password(), request)
        with LOCK:
            STATE['last_verified'] = datetime.now(timezone.utc).isoformat()
            STATE['last_kind'] = request.strip().decode('ascii')
            STATE['poll_count'] += 1
            STATE['last_monotonic'] = time.monotonic()
            STATE['ever_verified'] = True
        result['read_at_utc'] = STATE['last_verified']
        # Existing Rust and GUI strict response contracts remain unchanged.
        return result
    finally:
        CLI_LOCK.release()


def background_read(stop, reader, reader_module):
    """Periodic bounded read is informational only; never writes device."""
    while not stop.is_set():
        if enrolled():
            try:
                read(b'CARDS\n', reader, reader_module)
            except Exception:
                # Vendor CLI/prompt/SSH failures from imported strict adapter also fail closed.
                with LOCK:
                    STATE['last_verified'] = ''
                    STATE['last_kind'] = ''
                    STATE['last_monotonic'] = 0.0
        if stop.wait(300):
            break


def ensure_socket_available():
    """Clean only a verifiably stale owner-private Unix socket after systemd stop.

    Refuse symlinks, untrusted paths or an existing active listener.
    """
    if SOCKET.is_symlink():
        raise ValueError('symlink socket refused')
    if not SOCKET.exists():
        return
    info = SOCKET.lstat()
    if not stat.S_ISSOCK(info.st_mode) or info.st_uid != os.getuid() or \
            stat.S_IMODE(info.st_mode) != 0o600:
        raise ValueError('foreign or unsafe socket')
    probe = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    probe.settimeout(1)
    try:
        probe.connect(str(SOCKET))
    except ConnectionRefusedError:
        SOCKET.unlink()
    else:
        raise ValueError('another owner agent is actively serving')
    finally:
        probe.close()


def serve():
    os.umask(0o077)
    if os.geteuid() == 0:
        raise ValueError('root must not run connector')
    private_dir(ROOT)
    private_dir(PARENT)
    private_file(KEY)
    reader, reader_module = module()
    ensure_socket_available()
    # SIGTERM from systemd must release the private Unix socket on normal stop.
    signal.signal(signal.SIGTERM, lambda _sig, _frame: sys.exit(0))
    class Handler(socketserver.BaseRequestHandler):
        def handle(self):
            try:
                self.request.settimeout(65)
                if hasattr(socket, 'SO_PEERCRED'):
                    import struct
                    _, uid, _ = struct.unpack('3i', self.request.getsockopt(
                        socket.SOL_SOCKET, socket.SO_PEERCRED, 12))
                    if uid != os.getuid():
                        raise ValueError('foreign UID')
                wire = bytearray()
                while len(wire) <= MAX_REQUEST and not wire.endswith(b'\n'):
                    part = self.request.recv(128)
                    if not part:
                        break
                    wire.extend(part)
                if len(wire) > MAX_REQUEST:
                    raise ValueError('request too large')
                raw = bytes(wire)
                if raw == b'STATUS\n':
                    answer = status()
                elif raw in SAFE_ACTIONS:
                    answer = read(raw, reader, reader_module)
                elif raw.startswith(b'DRAFT '):
                    data = json.loads(raw[6:])
                    if type(data) is not dict or set(data) != {'device_name', 'device_profile'} or data['device_profile'] != 'zte_c320_lab':
                        raise ValueError('unsupported device draft')
                    answer = save_draft(data['device_name'])
                elif raw.startswith(b'ENROLL '):
                    if not CLI_LOCK.acquire(blocking=False):
                        raise ValueError('connection busy')
                    try:
                        data = json.loads(raw[7:])
                        if type(data) is not dict or set(data) != {'bootstrap_code', 'password',
                                                                  'device_profile'} or \
                                data['device_profile'] != 'zte_c320_lab':
                            raise ValueError('invalid allowlisted target')
                        answer = verify_and_store(data['bootstrap_code'], data['password'],
                                                  reader, reader_module)
                    finally:
                        CLI_LOCK.release()
                else:
                    raise ValueError('unknown request')
                response = safe_response(answer)
            except Exception as exc:
                # Fixed diagnostic codes only; never return a raw SSH transcript,
                # username, secret, address from an exception or subprocess.
                detail = str(exc)
                if detail == 'incorrect bootstrap code' or detail.startswith('enrollment temporarily limited'):
                    stage = 'OWNER_VERIFICATION_FAILED'
                elif detail == 'private SSH challenge missing':
                    stage = 'SSH_HANDSHAKE_OR_AUTH_METHOD'
                elif detail == 'authenticated C320 prompt missing':
                    stage = 'SSH_AUTH_FAILED_OR_UNKNOWN_PROMPT'
                elif detail in ('card table unrecognized', 'card row shape rejected'):
                    stage = 'DEVICE_CLI_RESPONSE_UNSUPPORTED'
                elif detail.startswith('already enrolled'):
                    stage = 'ALREADY_ENROLLED'
                else:
                    stage = safe_failure_stage(detail)
                response = safe_response({'error':stage,'physical_writes_enabled':False})
            try:
                self.request.sendall(response)
            except (BrokenPipeError, ConnectionResetError):
                pass
    class ThreadedServer(socketserver.ThreadingMixIn, socketserver.UnixStreamServer):
        daemon_threads = True
        block_on_close = False
        request_queue_size = 8
    stop = threading.Event()
    monitor = threading.Thread(target=background_read,
        args=(stop, reader, reader_module), daemon=True)
    try:
        with ThreadedServer(str(SOCKET), Handler) as server:
            SOCKET.chmod(0o600)
            print('R945_PERSISTENT_READ_CONNECTOR_STARTED_NO_RAW_CREDENTIALS', flush=True)
            monitor.start()
            server.serve_forever(poll_interval=0.5)
    finally:
        stop.set()
        if monitor.is_alive():
            monitor.join(timeout=2)
        if SOCKET.exists() and SOCKET.is_socket():
            SOCKET.unlink()
        print('R945_PERSISTENT_READ_CONNECTOR_STOPPED', flush=True)


if __name__ == '__main__':
    try:
        if sys.argv[1:] == ['--init']:
            init()
        elif sys.argv[1:] == ['--serve']:
            serve()
        else:
            sys.exit(4)
    except (ValueError, OSError, FileNotFoundError):
        print('R945_CONNECTOR_FAIL_CLOSED_NO_SECRET_OUTPUT', file=sys.stderr)
        sys.exit(4)
