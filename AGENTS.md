# ShellGate AGENTS.md

## Build & Test Environment

**All compilation and testing happens in the apps container, not the sandbox.**

```bash
# SSH into apps container and run cargo commands
ssh gem@apps 'export PATH="$HOME/.cargo/bin:$PATH" && cd /home/gem/projects/ShellGate && cargo test 2>&1'
```

Rust toolchain is at `/home/gem/.cargo/bin/` in the apps container. The sandbox container lacks cargo registry write permissions.

## Commands

| Action | Command |
|--------|---------|
| Run tests | `ssh gem@apps 'export PATH="$HOME/.cargo/bin:$PATH" && cd /home/gem/projects/ShellGate && cargo test 2>&1'` |
| Build | `ssh gem@apps 'export PATH="$HOME/.cargo/bin:$PATH" && cd /home/gem/projects/ShellGate && cargo build 2>&1'` |
| Build with bonsai | append `--features bonsai` |
| Check | `ssh gem@apps 'export PATH="$HOME/.cargo/bin:$PATH" && cd /home/gem/projects/ShellGate && cargo check 2>&1'` |
| Clippy | `ssh gem@apps 'export PATH="$HOME/.cargo/bin:$PATH" && cd /home/gem/projects/ShellGate && cargo clippy 2>&1'` |

## Project Structure

- `src/` — gate-server binary + library
- `src/pipeline.rs` — stage runner, verdict types, PipelineConfig
- `src/stages/` — deliberation stages (allow_list, catch_list, llm, human)
- `src/derived_grants.rs` — derived grant CRUD + glob matching
- `src/agenda.rs` — agenda CRUD with expiry
- `src/schema.rs` — SQLite migrations
- `src/handler.rs` — orchestrates pipeline from GateRequest → GateResponse
- `src/prompts.rs` — prompt templates + LLM output parsers
- `src/bonsai.rs` — local GGUF model (candle, optional `bonsai` feature)
- `src/types.rs` — all shared data types
- `tests/` — integration tests

## Implemented vs Stubbed

Implemented: pipeline, stages/*, derived_grants, agenda, schema, handler, prompts, bonsai, config, mcp, telegram, bootstrap, watcher, types

Stubbed (`todo!()`): classifier, grants, approvals, audit, permissions, tokens, proxy, protocol, server, api

## Testing

Integration tests use in-memory SQLite (`:memory:`) with `schema::run_migrations()`. No external services needed. LLM tests skip gracefully when no model file present.
