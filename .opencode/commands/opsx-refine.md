---
description: Refine an OpenSpec change proposal - interactive tier-by-tier review, classification, and fix application
---

Refine an OpenSpec change proposal - automated review for consistency, completeness, and oversights.
Classifies issues as editorial/architectural/speculative, presents each tier for interactive user approval,
and applies approved fixes. Commits by default after each tier.

**Prerequisite**: Change must have artifacts created (run `/opsx-propose` first).

---

**Input**: Optionally specify a change name (e.g., `/opsx-refine add-auth`). If omitted, auto-detected.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Auto-detect by running: `openspec list --json`
   - Select if only one active change exists
   - If ambiguous, ask the user to pick

   Always announce: "Refining change: <name>"

2. **Verify artifacts exist**

   Check that the change has artifacts to review:
   ```bash
   ls openspec/changes/<name>/proposal.md openspec/changes/<name>/design.md 2>/dev/null
   ```

   If missing, suggest: "Run `/opsx-propose` first to create artifacts."

3. **Follow the opsx-refine skill instructions exactly**

   The skill guides you through a finite state machine (STATE 0 through STATE 5).
   Steps 1-2 above handle STATE 0 (setup). The skill will continue from STATE 1:
   - STATE 1: Load artifacts → detect `opsx-*` subagents in opencode.json → user selects agents →
     dispatch analysis to selected agents (parallel) → merge/deduplicate results →
     write analysis to `.openspec/tmp/refine-analysis.json`
   - STATE 2: Present editorial fixes only → user approves/modifies/aborts → apply → commit (default yes)
   - STATE 3: Present architectural decisions only → user picks approaches → apply → commit (default yes)
   - STATE 4: Present speculative questions only → user describes direction → apply → commit (default yes)
   - STATE 5: Summary

   If no `opsx-*` subagents are configured in opencode.json, STATE 1 falls back to
   single-agent analysis (same output format, same analysis prompt).

   **Multi-agent configuration** — add to your project's `opencode.json`:
   ```json
   {
     "agent": {
       "opsx-deepseek": {
         "mode": "subagent",
         "model": "deepseek/deepseek-v4-flash",
         "description": "OpenSpec reviewer — DeepSeek",
         "hidden": true
       }
     }
   }
   ```
   Any agent name starting with `opsx-` is auto-detected. Users select which agents
   to use at runtime via checkbox.

   **Critical rules**:
   - At each tier, **STOP** after presenting findings and wait for the user's response.
   - Do NOT proceed to apply changes until the user explicitly approves.
   - Do NOT show issues from a future tier until the current tier is fully resolved (including commit).
   - Each tier reads only its category from the temp file — issues are revealed one tier at a time.
   - Commits happen by default after each tier — the user must say "no" to skip.

5. **After workflow completes**, report the final summary per the skill instructions.

**Guardrails**

- Only artifact files in the change directory are modified (never source code)
- Nothing is written until the user approves at each tier
- User can abort at any tier (previously committed changes preserved)
- Commits are the default — user must explicitly say no to skip
- Each fix follows the prescribed change exactly — no unsolicited improvements
- The temp file at `.openspec/tmp/refine-analysis.json` is internal state — never shown to user
