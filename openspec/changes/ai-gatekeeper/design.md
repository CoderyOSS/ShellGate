## Context

ShellGate runs on the host, intercepting shell commands from a sandboxed container via patched zsh + Unix socket. The current classifier is purely syntactic — `gh pr:create` maps to a GitHub action type, checked against static grants and defaults. No context about *why* commands are run.

The system already has: SQLite for all state, MCP server for agent communication, Telegram bot for notifications, and an approval workflow.

## Goals / Non-Goals

**Goals:**
- ShellGate understands project context (agenda) and uses it to make intelligent allow/block decisions
- LLM inference is kept off the hot path for the majority of commands (pattern matching handles 80-90%)
- LLM deliberates inline only for genuinely unknown commands (the long tail)
- Multiple agenda sources work simultaneously (MCP, OpenSpec watch, interactive)
- Pipeline is configurable — stages can be added, removed, reordered without code changes
- First iteration ships conservative; tuned based on observed behavior

**Non-Goals:**
- Training or fine-tuning the Bonsai model (use prompting only for v1)
- Audit log mining for pattern learning (deferred to v2)
- Per-agent trust levels or identity
- Cloud LLM APIs (all inference is local)

## Decisions

### 1. Candle for in-process LLM inference

**Decision**: Use Candle (Rust ML framework) to load and run Ternary Bonsai model directly inside gate-server. No sidecar process, no IPC.

**Alternatives considered**:
- Python sidecar (llama.cpp/llamafile): Additional process to manage, IPC latency, Python dependency on host.
- ONNX Runtime: Heavier dependency, less Rust-native.
- HTTP API to external LLM: Latency, privacy concerns (commands sent externally), dependency on network.

**Rationale**: Candle compiles to native Rust, no external process. Model loaded at startup, inference on demand. Bonsai models are small enough (0.37-1.75 GB) to fit in VPS memory alongside gate-server. Single binary, single process, no ops overhead.

### 2. Two-phase LLM usage: batch rules + inline deliberation

**Decision**: The LLM runs in two modes:
1. **Batch**: When an agenda is created/updated, LLM generates derived grants (pattern rules) cached in SQLite. Pattern matching on every command — no LLM inference.
2. **Inline**: When no pattern matches and no static grant covers the command, LLM deliberates on the hot path (~200-500ms). Command is already blocked, so latency is acceptable.

**Rationale**: Most commands hit pattern rules (fast path). Only the long tail of unanticipated commands wake the LLM. The inline path is acceptable because the command is held anyway — the user would wait for approval regardless. LLM is faster than human.

### 3. Trait-based deliberation pipeline

**Decision**: Command evaluation flows through a configurable pipeline of stages. Each stage implements a `DeliberationStage` trait, returns a verdict (Allow/Block/Pass/AllowAndPass). Pipeline configuration is per-flow (command_check, mcp_request, interactive_bootstrap) and defined in config.toml.

```
Pipeline stages (first iteration):
  AllowListStage → CatchListStage → LlmStage → HumanApprovalStage
```

**Rationale**: Hardcoded if/else chains don't survive iteration. The pipeline needs to be flexible — stages will be tuned, added, removed as the system proves itself. Trait-based stages with per-flow config let us iterate without refactoring the decision logic.

### 4. Permits only — no block grants

**Decision**: Both static grants and derived grants are permit-only. If no grant matches, the command is not permitted and falls through to the next pipeline stage. No explicit "block" grant type.

**Rationale**: Simpler mental model. Allow list is additive — everything not explicitly permitted is implicitly restricted. Derived grants from the LLM only specify what TO allow. Everything else escalates naturally.

### 5. Three notification strategies

**Decision**: Three distinct messaging patterns via Telegram:

| Strategy | Trigger | Behavior |
|----------|---------|----------|
| INTERCEPT 🔴 | catch_list match, LLM block, LLM unavailable | Command held, user must act |
| ADVISORY 🟡 | LLM allows unknown command, derived grant with `notification=advisory` | Command proceeds, user can intervene retroactively |
| REQUEST 🔵 | MCP `request_pre_approval` call | Explicit ask, blocks until human responds |

**Rationale**: Not every decision needs human attention. INTERCEPT for must-act, ADVISORY for eyes-open, REQUEST for proactive asks. Telegram inline buttons for all three with different action sets.

### 6. Three agenda signal sources

**Decision**: Agendas arrive from three independent sources, aggregated into a unified `agendas` table:

1. **MCP `set_agenda`**: OpenCode calls explicitly at task start. Most reliable, requires agent cooperation.
2. **OpenSpec file watch**: ShellGate monitors `openspec/changes/*/proposal.md` on bind-mounted projects directory. Zero-config when OpenSpec is in use.
3. **Interactive bootstrap**: When INTERCEPT approves a command and no agenda exists, LLM generates follow-up questions. User answers create an agenda. Works without any agent cooperation.

**Rationale**: Multiple sources provide resilience. MCP is the primary channel, OpenSpec is automatic bonus, interactive bootstrap is the fallback that works even without cooperation.

### 7. Agendas expire, derived grants inherit TTL

**Decision**: Agendas have configurable TTL (default 24h). Derived grants inherit expiry from their parent agenda. When an agenda expires, all its derived grants are invalidated.

**Rationale**: Stale rules from a previous task would incorrectly allow/block commands for the new task. TTL ensures the system self-cleans. Default 24h covers a typical work session.

## Risks / Trade-offs

**[LLM can approve things it shouldn't]** → Acknowledged. The LLM will make mistakes. Mitigation: conservative prompting, ADVISORY notification for LLM-approved unknowns, user can retroactively revoke. The LLM is better than blocking everything and worse than a human — acceptable tradeoff for reduced friction.

**[Model size on VPS]** → Bonsai 4B at 0.86 GB leaves plenty of headroom on a 4-8 GB VPS. Can start with 1.7B (0.37 GB) and scale up if rule quality is insufficient.

**[Batch rule quality]** → If the LLM generates poor patterns, too many commands hit the inline path (slow) or get incorrectly auto-allowed. Mitigation: ADVISORY notifications on edge-case derived grants, user feedback loop, tune prompts over time.

**[Pipeline complexity]** → Flexible pipeline is harder to reason about than hardcoded flow. Mitigation: default config is simple and well-documented, logging shows which stage made each decision.

**[Candle maturity]** → Candle is relatively new. If it proves unstable, fallback to llama.cpp sidecar. Architecture is designed so LLM is a stage — swapping implementation doesn't affect the pipeline.

## Open Questions

- Which Bonsai model size to ship first? Recommend 4B, test all three and compare rule quality.
- Sampling rate for allow_list hits sent to LLM: start at 5%, tune based on observed behavior.
- Should derived grants have a `notification` field (`silent` vs `advisory`)? Yes for first iteration — let the LLM decide which of its generated rules are edge-case enough to flag.
- Confidence thresholds for LLM stage: start with allow >= 0.7, uncertain = pass to human.
