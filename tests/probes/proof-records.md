# ShellGate E2E Proof Records

**Date:** 2026-05-11T01:04:44.459Z
**Tests:** 2 run, 2 pass, 0 fail
**Duration:** 29ms

---

## llm deliberation > captures prompt and allows

**Status:** ✓ pass | **Duration:** 9ms

### Sequence

| # | Time | Direction | Step | Detail |
|---|------|-----------|------|--------|
| 1 | 01:04:44.445 | Setup | http.put | `{"choices":[{"message":{"content":"DECISION: ALLOW\nCONFIDENCE: 0.90\nREASON: command fits the active agenda"}}]}` |
| 2 | 01:04:44.445 | Send | unix.send | `{"type":"check_command","request":{"command":"unknown-tool","args":["--help"],"cwd":"/tmp","pid":9999}}` |
| 3 | 01:04:44.451 | Recv | http:request | `[{"method":"POST","path":"/v1/chat/completions","headers":{"accept":"*/*","authorization":"Bearer test","content-length":"756","content-type":"application/json","host":"127.0.0.1:19878"},"body":"{\"model\":\"deepseek-chat\",\"messages\":[{\"role\":\"system\",\"content\":\"You are a security gatek...` |
| 4 | 01:04:44.453 | Response | unix response | `{"action":"allow"}` |

---

## llm deliberation > blocks on LLM BLOCK verdict

**Status:** ✓ pass | **Duration:** 5ms

### Sequence

| # | Time | Direction | Step | Detail |
|---|------|-----------|------|--------|
| 1 | 01:04:44.453 | Setup | http.put | `{"choices":[{"message":{"content":"DECISION: BLOCK\nCONFIDENCE: 0.95\nREASON: dangerous operation detected"}}]}` |
| 2 | 01:04:44.453 | Send | unix.send | `{"type":"check_command","request":{"command":"unknown-dangerous","args":[],"cwd":"/tmp","pid":9999}}` |
| 3 | 01:04:44.457 | Recv | http:request | `[{"method":"POST","path":"/v1/chat/completions","headers":{"accept":"*/*","authorization":"Bearer test","content-length":"755","content-type":"application/json","host":"127.0.0.1:19878"},"body":"{\"model\":\"deepseek-chat\",\"messages\":[{\"role\":\"system\",\"content\":\"You are a security gatek...` |
| 4 | 01:04:44.458 | Response | unix response | `;{"action":"reject","reason":"dangerous operation detected"}` |

---

