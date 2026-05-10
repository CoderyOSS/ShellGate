# ShellGate E2E Proof Records (Rust)

**Date:** 2026-05-10T08:44:21.216113414+00:00
**Tests:** 5 run

---

## e2e_gate_server_starts_and_accepts_connections

**Status:** ✓ pass | **Duration:** 502ms

### Send

```json
{
  "action": "connect"
}
```

### Response

```json
{
  "result": "ok"
}
```

---

## e2e_catch_list_blocks_auth_command

**Status:** ✓ pass | **Duration:** 507ms

### Send

```json
{
  "args": [],
  "command": "auth:login"
}
```

### Response

```json
{
  "action": "reject",
  "reason": "matched catch pattern: auth:*"
}
```

---

## e2e_catch_list_blocks_rm_rf

**Status:** ✓ pass | **Duration:** 507ms

### Send

```json
{
  "args": [
    "-rf",
    "/"
  ],
  "command": "rm"
}
```

### Response

```json
{
  "action": "reject",
  "reason": "matched catch pattern: rm -rf *"
}
```

---

## e2e_safe_command_allowed_with_grant

**Status:** ✓ pass | **Duration:** 525ms

### Send

```json
{
  "args": [
    "hello"
  ],
  "command": "echo"
}
```

### Response

```json
{
  "action": "allow"
}
```

---

## e2e_spawn_shell_connects

**Status:** ✓ pass | **Duration:** 503ms

### Send

```json
{
  "type": "spawn_shell"
}
```

### Response

```json
{
  "result": "connected"
}
```

---

