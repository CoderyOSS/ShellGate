# Tasks — ai-gatekeeper

> Ordered by dependency layer. Foundation tasks first.

## ID Mapping

| Old ID | New ID | Description |
|--------|--------|-------------|
| 1.1 | 1.1 | Add Candle dependencies |
| 1.2 | 1.2 | Agendas + derived_grants schema |
| 1.3 | 1.3 | Pipeline trait + enums |
| 1.4 | 1.4 | Config pipeline sections |
| 2.1 | 2.1 | Agenda CRUD |
| 2.2 | 2.2 | Derived grant CRUD + matching |
| 2.3 | 2.3 | Allow list stage |
| 2.4 | 2.4 | Catch list stage |
| 3.1 | 3.1 | Bonsai model loader |
| 3.2 | 3.2 | Batch rule generation prompt |
| 3.3 | 3.3 | Inline deliberation prompt |
| 3.4 | 3.4 | Question generation prompt |
| 4.1 | 4.1 | LLM deliberation stage |
| 4.2 | 4.2 | Human approval stage (refactor existing) |
| 4.3 | 4.3 | Pipeline runner |
| 4.4 | 4.4 | Wire pipeline into handler |
| 5.1 | 5.1 | MCP set_agenda tool |
| 5.2 | 5.2 | MCP request_pre_approval update |
| 5.3 | 5.3 | OpenSpec file watcher |
| 5.4 | 5.4 | Interactive bootstrap flow |
| 6.1 | 6.1 | INTERCEPT Telegram template |
| 6.2 | 6.2 | ADVISORY Telegram template |
| 6.3 | 6.3 | REQUEST Telegram template |
| 6.4 | 6.4 | Bootstrap questions Telegram template |
| 7.1 | 7.1 | Pipeline unit tests |
| 7.2 | 7.2 | LLM prompt evaluation tests |
| 7.3 | 7.3 | End-to-end flow test |
| 7.4 | 7.4 | Config validation |

---

## Layer 0: Infrastructure

- [x] 1.1 Add Candle crate dependencies to Cargo.toml: `candle-core`, `candle-nn`, `candle-transformers`, `gguf` format reader. Feature-gate behind `bonsai` feature flag.

- [x] 1.2 Create SQLite migrations for `agendas` and `derived_grants` tables in schema.rs. Agendas: id, source, description, scope, status, created_at, expires_at. Derived grants: id, agenda_id (FK), command_pattern, args_pattern, path_pattern, notification (silent/advisory), reason, confidence, created_at, expires_at.

- [x] 1.3 Define `DeliberationStage` trait and `StageVerdict` enum in new `src/pipeline.rs`. StageVerdict: Allow, AllowAndNotify, Block, BlockAndNotify, Pass, AllowAndPass. DeliberationContext struct with command, args, cwd, agendas, recent_history, config.

- [x] 1.4 Add `[pipeline]`, `[stages]`, `[bonsai]` sections to Config struct and config.rs. Pipeline: per-flow stage lists. Stages: per-stage params (sampling_rate, confidence thresholds). Bonsai: model_path, model_size.

---

## Layer 1: Data Layer

- [x] 2.1 Implement agenda CRUD in new `src/agenda.rs`: create (from MCP/OpenSpec/interactive), read active, expire (TTL background task), list. Aggregation of multiple sources into unified table.

- [x] 2.2 Implement derived grant CRUD + pattern matching in new `src/derived_grants.rs`: create (from bonsai output), match (command + args + cwd vs glob patterns), expire (cascade from agenda TTL), list by agenda. Match uses existing `globset` crate.

- [x] 2.3 Implement `AllowListStage`: load derived grants + static grants, match command against patterns, return Allow or AllowAndPass (based on sampling rate). Sampling: random X% flagged for async LLM review.

- [x] 2.4 Implement `CatchListStage`: load configured catch patterns, match command, return Block if matched. Patterns from config `[stages.catch_list]`.

---

## Layer 2: LLM Integration

- [x] 3.1 Implement bonsai model loader in new `src/bonsai.rs`: load GGUF model from disk at startup via Candle, hold in memory. Configurable model path and size. Graceful degradation if model file missing (skip LLM stage, all unknowns go to human).

- [x] 3.2 Implement batch rule generation prompt: given agenda description + scope, generate structured JSON array of derived grant patterns (command_pattern, args_pattern, path_pattern, notification, reason, confidence). Parse output, validate, insert into derived_grants table.

- [x] 3.3 Implement inline deliberation prompt: given command + args + cwd + active agendas + recent command history + warning signs, respond ALLOW or BLOCK with reasoning and confidence. Parse structured output.

- [x] 3.4 Implement question generation prompt: given approved command + recent history (no agenda exists), generate 2-3 brief follow-up questions for the user. Parse structured output.

---

## Layer 3: Pipeline Assembly

- [x] 4.1 Implement `LlmStage`: on inline deliberation, assemble context (command, agendas, history), run bonsai inference, parse verdict. Confidence >= threshold → AllowWithNotify (ADVISORY). Confidence below → Block. Uncertain → Pass to human.

- [x] 4.2 Refactor existing human approval flow into `HumanApprovalStage`: create approval request, block, notify via Telegram, wait for resolution. Reuse existing approvals.rs logic.

- [x] 4.3 Implement `Pipeline` runner: iterate stages, respect verdict semantics (Allow stops pipeline, Pass continues, AllowAndPass continues but marks allowed). Return final Decision.

- [x] 4.4 Wire pipeline into `handler.rs`: replace hardcoded classify→check→respond with pipeline.run(). Different pipeline configs for command_check vs mcp_request flows.

---

## Layer 4: Signal Sources

- [x] 5.1 Implement MCP `set_agenda` tool in mcp.rs: accept description, scope, ttl. Create agenda in SQLite, trigger bonsai batch rule generation. Return agenda ID.

- [x] 5.2 Update MCP `request_pre_approval` tool: before creating approval request, run through allow_list stage. If match found, auto-approve and return grant immediately without human involvement.

- [x] 5.3 Implement OpenSpec file watcher: tokio::fs::watch on projects directory (bind-mounted). Detect new/changed `openspec/changes/*/proposal.md` files. Parse description + scope from proposal. Create agenda with source='openspec'.

- [x] 5.4 Implement interactive bootstrap flow: after user approves an INTERCEPT and no active agenda exists, run bonsai question generation. Send questions via Telegram. On user response, create agenda, trigger batch rule generation.

---

## Layer 5: Notification Templates

- [x] 6.1 Implement INTERCEPT Telegram template: 🔴, command details, "no grant covers this", Approve/Reject buttons. Post-approval: optional bootstrap questions if no agenda.

- [x] 6.2 Implement ADVISORY Telegram template: 🟡, command details, matched grant info, LLM reasoning, Revoke grant / Acknowledge buttons. Command already executed.

- [x] 6.3 Implement REQUEST Telegram template: 🔵, requested action, scope, duration, reason, Grant/Deny buttons.

- [x] 6.4 Implement bootstrap questions Telegram template: brief questions with inline button choices and/or free-text input. User answers create agenda.

---

## Layer 6: Testing & Polish

- [x] 7.1 Unit tests for pipeline: test each stage in isolation (AllowListStage matching, CatchListStage blocking, LlmStage verdict parsing), test pipeline runner with mock stages, test AllowAndPass sampling.

- [x] 7.2 LLM prompt evaluation: test batch rule generation with sample agendas, verify output parses correctly, test inline deliberation with sample commands, test question generation. Run against all three Bonsai sizes (1.7B, 4B, 8B) and compare quality.

- [x] 7.3 End-to-end flow test: set agenda via MCP → verify derived grants generated → run command that matches → verify silent allow → run unknown command → verify LLM deliberation → verify ADVISORY notification → verify interactive bootstrap.

- [x] 7.4 Config validation: verify pipeline configs parse correctly, verify missing model file degrades gracefully, verify invalid stage names produce clear errors.
