"""Concurrent agent/broken-pipe regression, mocked physical SSH, no credentials or device calls."""
import importlib.util,json,socket,sys,tempfile,threading,time,unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
SCRIPT=Path(__file__).with_name('owner_supervised_c320_read_agent.py')
spec=importlib.util.spec_from_file_location('agent_concurrency_under_test',SCRIPT)
agent=importlib.util.module_from_spec(spec);spec.loader.exec_module(agent)

class ConcurrentStatusTests(unittest.TestCase):
 @unittest.skipUnless(sys.platform.startswith("linux") and hasattr(socket,"SO_PEERCRED"),"Linux Unix peer credential is required")
 def test_busy_status_and_dropped_peers_do_not_break_agent(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp)/'private';sock=root/'live.sock';root.mkdir(mode=0o700)
      pin=Path(tmp)/'pin';pin.write_text('[example.invalid]:321 ssh-rsa SYNTHETIC')
      m=SimpleNamespace(PIN=pin,HOST='example.invalid',PORT=321,private_regular=lambda _:None)
      started=threading.Event();finish=threading.Event();exceptions=[]
      def fake_read(_m,_pw,_req):
        started.set();self.assertTrue(finish.wait(2));return {'fake':'count-only'}
      def agent_task():
        try:agent.main()
        except BaseException as exc:exceptions.append(type(exc).__name__)
      with patch.object(agent,'ROOT',root),patch.object(agent,'SOCKET',sock),\
           patch.object(agent,'MAX_REQUESTS',2),patch.object(agent,'SESSION_SECONDS',3),\
           patch.object(agent,'module',return_value=m),patch.object(agent,'run_three_reads',side_effect=fake_read),\
           patch.object(agent.sys,'argv',['agent','--owner-terminal']),\
           patch.object(agent.sys.stdin,'isatty',return_value=True),\
           patch.object(agent.os,'geteuid',return_value=1000),\
           patch.dict(agent.os.environ,{'IPAT_R940_OWNER_READ_AGENT':'YES'}),\
           patch('builtins.input',return_value=agent.PROMPT),\
           patch.object(agent.getpass,'getpass',return_value='SYNTHETIC_NOT_A_DEVICE_PASSWORD'):
        t=threading.Thread(target=agent_task);t.start()
        for _ in range(100):
          if sock.exists():break
          time.sleep(.01)
        self.assertTrue(sock.exists())
        def send(data,expect_reply=True):
          c=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
          c.connect(str(sock));c.sendall(data)
          if not expect_reply:c.close();return None
          c.settimeout(1);result=c.recv(512);c.close();return json.loads(result)
        physical=threading.Thread(target=send,args=(b'CARDS\n',False));physical.start()
        self.assertTrue(started.wait(1))
        # Underlying server MUST remain responsive to status while device read is busy.
        status=send(b'STATUS\n')
        self.assertTrue(status['read_in_progress'])
        self.assertEqual(status['requests_left'],1)
        # Deliberate peer disconnect before result: no unhandled BrokenPipe.
        send(b'STATUS\n',False)
        finish.set();physical.join(1);t.join(5)
        self.assertFalse(t.is_alive());self.assertEqual(exceptions,[])
        self.assertFalse(sock.exists())

if __name__=='__main__':unittest.main()
