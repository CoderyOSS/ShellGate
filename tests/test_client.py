import socket
import json
import struct
import sys
import time

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(180)
s.connect("/tmp/gate.sock")
command = sys.argv[1]
args = sys.argv[2:] if len(sys.argv) > 2 else []
req = {"command": command, "args": args, "cwd": "/home/gem/projects/test", "pid": 1234}
data = json.dumps(req).encode()
t0 = time.time()
s.sendall(struct.pack(">I", len(data)) + data)
resp_len_bytes = s.recv(4)
t1 = time.time()
resp_len = struct.unpack(">I", resp_len_bytes)[0]
resp = json.loads(s.recv(resp_len))
elapsed = t1 - t0
print(f"RESULT ({elapsed:.1f}s): {json.dumps(resp, indent=2)}")
s.close()
