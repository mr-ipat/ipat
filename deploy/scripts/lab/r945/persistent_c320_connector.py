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
TOKEN = ROOT / 'bootstrap-token'
TOKEN_HASH = ROOT / 'bootstrap-sha256'
KEY = ROOT / 'envelope-key'
CREDENTIAL = ROOT / 'device.fernet'
DEVICE_META = ROOT / 'device.json'
MAX_REQUEST = 1024
MAX_ATTEMPTS = 5
WAIT_SECONDS = 900
MODE = 'OWNER_SUPERVISED_REAL_C320_READ_ONLY'
SAFE_ACTIONS = {b'REFRESH\n', b'CARDS\n', b'FIRMWARE\n'}
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


def module():
    spec = importlib.util.spec_from_file_location('r945_fixed_c320', SOURCE)
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


def safe_response(payload):
    return (json.dumps(payload, separators=(',', ':'), sort_keys=True) + '\n').encode('ascii')


def verify_and_store(token, password, reader, reader_module, device_name='ZTE C320 Lab'):
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
    if not isinstance(device_name, str) or not device_name.strip() or len(device_name)>64 or any(ord(x)<32 for x in device_name):
        raise ValueError('invalid device name')
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
    secret_write(DEVICE_META, safe_response({'name': device_name.strip(), 'device_type': 'olt', 'device_profile': 'zte_c320_lab'}))
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
            'device_name': device_name.strip(),
            'commercial_production_adopted': False, 'physical_writes_enabled': False}


def status():
    configured = enrolled()
    with LOCK:
        last = STATE['last_verified']
        last_kind = STATE['last_kind']
        recent = bool(last) and time.monotonic() - STATE['last_monotonic'] <= 360
        ever_verified = STATE['ever_verified']
    label = 'ZTE C320 Lab'
    if configured and DEVICE_META.exists():
        private_file(DEVICE_META)
        saved = json.loads(DEVICE_META.read_text('utf-8'))
        if saved.get('device_profile') == 'zte_c320_lab' and isinstance(saved.get('name'), str) and 1 <= len(saved['name']) <= 64:
            label = saved['name']
    return {'mode': MODE, 'agent_ready': configured,
            'read_in_progress': CLI_LOCK.locked(),
            'requests_left': 5 if configured else 0,
            'seconds_left': 900 if configured else 0,
            'persistent_connector': True,
            'credentials_enrolled': configured,
            'device_name': label,
            'last_verified_at_utc': last,
            'last_verified_kind': last_kind,
            'host_identity_level': 'NETWORK_OBSERVED_SSH_PIN',
            'actual_olt_connectivity_verified': recent,
            'ever_verified_since_start': ever_verified,
            'device_adopted': False, 'device_writes': 0}


def read(request, reader, reader_module):
    if not enrolled():
        raise ValueError('not enrolled')
    if not CLI_LOCK.acquire(blocking=False):
        raise ValueError('device read busy')
    try:
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
                elif raw.startswith(b'ENROLL '):
                    if not CLI_LOCK.acquire(blocking=False):
                        raise ValueError('connection busy')
                    try:
                        data = json.loads(raw[7:])
                        if type(data) is not dict or set(data) != {'bootstrap_code', 'password',
                                                                  'device_profile', 'device_name'} or \
                                data['device_profile'] != 'zte_c320_lab':
                            raise ValueError('invalid allowlisted target')
                        answer = verify_and_store(data['bootstrap_code'], data['password'],
                                                  reader, reader_module, data['device_name'])
                    finally:
                        CLI_LOCK.release()
                else:
                    raise ValueError('unknown request')
                response = safe_response(answer)
            except Exception:
                # Fail closed on vendor adapter exceptions as well; never print CLI or secret.
                response = safe_response({'error':'CONNECTOR_REQUEST_FAILED_CLOSED',
                                          'physical_writes_enabled':False})
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
