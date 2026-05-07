## Why

ShellGate's current permission model is static — command classifier maps commands to action types, grants and defaults decide allow/block. Every unexpected command escalates to human approval. This works but is slow: every `npm install`, every unfamiliar tool, every edge case requires a human tap on Telegram. The system has no understanding of *why* commands are being run, so it can't make intelligent decisions about whether an unexpected command is actually suspicious in context.

## What Changes

- **Agenda system**: ShellGate receives and stores project context (what the user/agent is working on) from multiple signal sources
- **Bonsai LLM integration**: Local Ternary Bonsai model (via Candle, compiled into gate-server) generates pattern-based allow rules from agendas, and deliberates on truly unknown commands
- **Deliberation pipeline**: Configurable chain of stages (allow_list → catch_list → LLM → human) processes commands. Stages are add/remove/reorder without code changes
- **Derived grants**: LLM-generated pattern rules cached in SQLite, matched via glob on command+args+path. Fast path — no LLM inference per command
- **Interactive bootstrap**: When no agenda exists and an unknown command is intercepted, the LLM generates follow-up questions after user approval to build an agenda reactively
- **Sampling**: A configurable percentage of allow_list hits are reviewed by the LLM asynchronously to maintain situational awareness
- **Three notification strategies**: INTERCEPT (held, must approve), ADVISORY (allowed, raised for visibility), REQUEST (MCP pre-approval ask)
- New MCP tools: `set_agenda`, updated `request_pre_approval`
- New SQLite tables: `agendas`, `derived_grants`
- New config sections: `[pipeline]`, `[stages.*]`, `[bonsai]`

## Capabilities

### New Capabilities
- `gate-agenda`: Receives, stores, and expires project agendas from multiple sources (MCP, OpenSpec file watch, interactive bootstrap)
- `gate-bonsai`: Local LLM integration via Candle — batch rule generation from agendas, inline deliberation for unknowns, question generation for interactive bootstrap
- `gate-pipeline`: Configurable deliberation pipeline with trait-based stages, per-flow configuration, sampling support

### Modified Capabilities
- `gate-permissions`: Derived grants checked alongside static grants in the pipeline
- `gate-mcp`: New `set_agenda` tool, updated `request_pre_approval` to check allow_list before escalating
- `gate-telegram`: New message templates for INTERCEPT, ADVISORY, REQUEST strategies with inline buttons

## Impact

- **gate-server**: New Candle dependency, new modules (agenda, bonsai, pipeline, derived_grants), modified classifier and handler to use pipeline
- **SQLite**: New tables (agendas, derived_grants), new migrations
- **Config**: New `[pipeline]`, `[stages]`, `[bonsai]` sections in config.toml
- **Model file**: Bonsai model weights shipped alongside binary (~0.37-1.75 GB)
- **Dependencies**: candle-core, candle-nn, candle-transformers (Rust crates)
