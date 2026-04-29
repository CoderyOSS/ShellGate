# OpsxRefine Conversational Workflow Design

## Architecture

The refinement workflow is a multi-turn conversation between the AI agent and the user,
driven by a finite state machine defined in `SKILL.md`. No external DAG engine needed —
the agent's reasoning and the user's natural language responses replace formal approval gates.

The agent enforces step locking: exactly one state per response during tier interaction,
with gating rules that prevent skipping or combining states.

## Pipeline

```
User: /opsx-refine <change-name>
  │
  ├─ STATE 0: SETUP
  │   detect change name → verify artifacts → ask model preferences
  │   GATE: artifacts must exist
  │
  ├─ STATE 1: ANALYZE (two modes: multi-agent or single-agent)
  │   ├─ 1.1 Load all artifacts
  │   ├─ 1.2 Detect opsx-* subagents in opencode.json
  │   │   if none found → skip to 1.7 (single-agent fallback)
  │   ├─ 1.3 Select subagents (user checkmarks which to use)
  │   │   STOP. wait for user response
  │   ├─ 1.4 Dispatch subagents via task tool (parallel)
  │   │   each gets: analysis prompt (from references/analysis-prompt.md)
  │   │              + artifact contents
  │   │              + JSON output schema
  │   ├─ 1.5 Collect results from all subagents
  │   ├─ 1.6 Merge and deduplicate → write JSON to temp file
  │   │   (skip to 1.8)
  │   ├─ 1.7 Single-agent fallback (if no subagents configured/selected)
  │   │   analyze directly → write JSON to temp file
  │   └─ 1.8 Report findings summary (counts only)
  │   GATE: temp file written + valid
  │
  ├─ STATE 2: EDITORIAL TIER (multi-turn)
  │   read only editorial issues from temp file →
  │   present numbered prescriptions →
  │   STOP. wait for user response (approve/modify/abort) →
  │   apply via Edit/Write (user sees diffs in permission dialogs) →
  │   commit gate (default: commit unless user says no)
  │   GATE: fixes applied + commit resolved
  │
  ├─ STATE 3: ARCHITECTURAL TIER (multi-turn)
  │   read only architectural issues from temp file →
  │   present decision sheet with recommendations →
  │   STOP. wait for user response →
  │   apply chosen approaches →
  │   commit gate (default: commit unless user says no)
  │   GATE: fixes applied + commit resolved
  │
  ├─ STATE 4: SPECULATIVE TIER (multi-turn)
  │   read only speculative issues from temp file →
  │   present clarified questions →
  │   STOP. wait for user response →
  │   translate directions to changes →
  │   commit gate (default: commit unless user says no) →
  │   cleanup temp file
  │   GATE: fixes applied + commit resolved + temp file removed
  │
  └─ STATE 5: SUMMARY
      final table of what was applied/skipped/committed
```

## Tier Definitions

| Tier | Label | Nature | Ideal count |
|------|-------|--------|-------------|
| 2 | Editorial | Rudimentary: inconsistencies, omissions, slop. Tightening the plans. | Several |
| 3 | Architectural | Design decisions with researchable best options. Tradeoff analysis needed. | Few |
| 4 | Speculative | Underspecified or unrealistic. Can't resolve without user input. | Zero (ideally) |

## State Persistence

The analysis state is persisted to `.openspec/tmp/refine-analysis.json` after STATE 1.
This file serves as the single source of truth for all tier states:

- Each tier reads ONLY its category from this file
- The file is created in STATE 1 and deleted in STATE 4 (or on abort)
- If the session is interrupted, the temp file enables resuming:
  the user can say "continue refining" and the agent reads the file
  to determine which tier to resume from

## Commit Behavior

After each tier's fixes are applied, the agent commits **by default**.
The user must explicitly say no to skip. This ensures:

- Progress is saved at each stage
- Git history clearly shows which tier introduced which changes
- Easy rollback to any tier boundary

Commit messages: `opsx-refine: apply N <tier> fixes`

The commit gate is NOT a question — it's a notification with an opt-out:
> "Committing these N editorial fixes. Say **no** to skip."

## Sequential Tier Revelation

The agent reveals issues ONE TIER AT A TIME. After STATE 1:

1. Only total counts are shown (e.g., "Found 12: 5 editorial, 4 architectural, 3 speculative")
2. Editorial issues are presented first — user approves, fixes applied, commit resolved
3. Only THEN are architectural issues presented
4. Only after architectural is resolved are speculative issues shown

This prevents information overload and ensures the user focuses on one category at a time.

## How It Replaces Archon DAG

| Archon Concept | Conversational Equivalent |
|---|---|
| DAG node (prompt) | Agent reasoning step within a state |
| DAG node (bash) | Agent uses Bash tool within a state |
| Approval gate | Agent presents findings, STOPs, waits for user chat response |
| `capture_response` | User's chat message |
| `on_reject` with revision | User gives corrections, agent revises |
| Hard abort | User says "abort", agent stops |
| Variable substitution (`$node.output`) | Temp file `.openspec/tmp/refine-analysis.json` |
| Sequential tiers (depends_on) | State machine enforces order via gating rules |
| Commit approval node | Agent commits by default, user says no to skip |

## Interaction Points

Each tier has exactly 2 user interaction points:
1. **Approve/modify/abort** the proposed changes (chat response)
2. **Commit gate** — commit happens unless user says no (notification, not question)

File changes are also reviewed by the user via OpenCode's Edit/Write permission dialogs,
which show diffs for each modification.

## Cost Profile

- **Minimum** (no issues found): 1 reasoning pass (analysis only)
- **Typical single-agent**: 2-3 reasoning passes (analysis + apply for 1-2 tiers)
- **Typical multi-agent**: N parallel subagent passes + 1 merge + 1-2 apply passes
- **Maximum**: N parallel subagent passes + 1 merge + 3 apply passes (all 3 tiers)
- Subagent passes run in parallel — wall-clock time is the slowest agent, not the sum
- The primary agent reads the temp file to recall merged results across turns

## Advantages Over DAG

- **Natural interaction**: User types responses in plain language, not special syntax
- **Per-file diff review**: OpenCode's Edit tool shows each change for approval
- **No external dependency**: No Archon CLI, no SQLite, no workflow engine
- **Fewer AI calls**: Temp file carries state across turns — no need for separate triage nodes
- **Crash recovery**: Temp file persists analysis — user can resume after interruption
- **Flexible**: User can give free-form corrections that a DAG can't anticipate
- **State machine enforcement**: Gating rules + step locking prevent agent deviation
- **Multi-model analysis**: Different models catch different issues; merge produces higher quality

## Multi-Agent Configuration

Users define analysis subagents in the project's `opencode.json` under the `agent` field.
Agents with the `opsx-` naming prefix are discovered by the refine skill automatically.

### Configuration pattern

```json
{
  "agent": {
    "opsx-deepseek": {
      "mode": "subagent",
      "model": "deepseek/deepseek-v4-flash",
      "description": "OpenSpec reviewer — DeepSeek",
      "hidden": true
    },
    "opsx-kimi": {
      "mode": "subagent",
      "model": "kimi-for-coding/k2p6",
      "description": "OpenSpec reviewer — Kimi",
      "hidden": true
    },
    "opsx-glm": {
      "mode": "subagent",
      "model": "zai-coding-plan/glm-5.1",
      "description": "OpenSpec reviewer — GLM",
      "hidden": true
    }
  }
}
```

### Dispatch flow

1. Primary agent reads `opencode.json`, finds `opsx-*` agents in the `agent` field with `"mode": "subagent"`
2. User selects which agents to use (checkbox prompt, default: all)
3. For each selected agent, primary agent calls `task` with:
   - `subagent_type`: the agent name (e.g., `"opsx-deepseek"`)
   - `prompt`: analysis prompt from `references/analysis-prompt.md` + artifact contents
4. All dispatches run in parallel
5. Primary agent collects JSON results, merges and deduplicates

### Merge/dedup logic

- Issues grouped by `location` (file > section)
- Within each group, descriptions compared semantically
- Duplicates resolved by keeping the issue with the most detailed `reason` field
- Category disagreements resolved by best reasoning quality (more specific analysis wins)
- Surviving issues get new sequential IDs (E1, A1, Q1, ...)

### Fallback

If no `opsx-*` agents configured, user deselects all, or all agents fail:
the primary agent performs analysis directly using the same analysis prompt
from `references/analysis-prompt.md`. Output format is identical.
