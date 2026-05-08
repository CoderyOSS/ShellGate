## Context

ShellGate gate-server has a multi-stage deliberation pipeline (allow_list → catch_list → LLM → human) that evaluates shell commands. The core pipeline logic is implemented, but test coverage is minimal (6 unit tests in `lib.rs`). Several infrastructure modules (`server`, `protocol`, `classifier`, `proxy`) are stubbed with `todo!()`.

## Goals / Non-Goals

**Goals:**
- Verify allow_list stage correctly matches derived grants and returns Allow/AllowAndNotify
- Verify catch_list stage correctly blocks commands matching glob patterns
- Verify pipeline composition: allow stops early, pass-through reaches later stages, all-pass falls through
- Verify LLM stage degrades gracefully when model unavailable (returns Pass)
- Verify prompt parsing edge cases
- Verify handler integration: full path from GateRequest → GateResponse with real DB
- All tests run via `cargo test` in the apps container (not sandbox)

**Non-Goals:**
- Testing stubbed modules (classifier, grants, approvals, protocol, proxy, tokens)
- Testing Telegram notification sending
- Testing unix socket wire protocol
- Testing MCP stdio interface
- Testing bonsai model inference with a real model file (graceful skip)

## Decisions

### 1. In-memory SQLite per test

Each test creates its own `:memory:` SQLite database, runs `schema::run_migrations()`, and seeds data as needed. No shared state between tests. No file I/O.

### 2. Apps container execution

Tests compile and run in the apps container (`ssh gem@apps`) where the Rust toolchain is installed at `~/.cargo/bin/`. The sandbox container lacks write permissions to the cargo registry.

### 3. Conditional LLM tests

LLM stage tests check `BonsaiModel::is_available()`. When no model file exists (default), tests verify graceful degradation (Pass verdict). Tests gated with `#[cfg(feature = "bonsai")]` for model-dependent paths.

### 4. Test file structure

Single integration test file `tests/pipeline_integration.rs` with a shared `setup_db()` helper. Unit tests for prompt parsing remain in `src/lib.rs`.

## Test Categories

| # | Category | What it verifies |
|---|----------|-----------------|
| 1 | Catch list | Glob pattern matching, BlockAndNotify verdict |
| 2 | Allow list / derived grants | Grant matching, Allow vs AllowAndNotify, expired grants ignored |
| 3 | Pipeline composition | Stage ordering, early termination, fall-through |
| 4 | LLM stage | Graceful degradation without model |
| 5 | Prompt parsing | Edge cases for parse_deliberation, parse_rules, parse_questions |
| 6 | Handler integration | Full GateRequest → GateResponse with real DB |

## File Changes

| File | Change |
|------|--------|
| `tests/pipeline_integration.rs` | New — all integration tests |
| `AGENTS.md` | New — project conventions including apps container testing |
| `src/lib.rs` | Existing tests stay, no changes needed |

## Run Command

```bash
ssh gem@apps 'export PATH="$HOME/.cargo/bin:$PATH" && cd /home/gem/projects/ShellGate && cargo test 2>&1'
```
