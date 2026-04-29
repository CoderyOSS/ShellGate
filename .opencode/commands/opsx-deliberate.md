---
description: Deliberate on an OpenSpec change proposal - sequential multi-agent consensus review
---

Deliberate on an OpenSpec change proposal — sequential round-robin review where agents
build on each other's proposals until consensus or max turns. User accepts/modifies/rejects.

**Prerequisite**: Change must have artifacts created (run `/opsx-propose` first).

---

**Input**: Optionally specify a change name (e.g., `/opsx-deliberate add-auth`). If omitted, auto-detected.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Auto-detect by running: `openspec list --json`
   - Select if only one active change exists
   - If ambiguous, ask the user to pick

   Always announce: "Deliberating change: <name>"

2. **Verify artifacts exist**

   Check that the change has artifacts to review:
   ```bash
   ls openspec/changes/<name>/proposal.md openspec/changes/<name>/design.md 2>/dev/null
   ```

   If missing, suggest: "Run `/opsx-propose` first to create artifacts."

3. **Follow the opsx-deliberate skill instructions exactly**

   The skill guides you through a finite state machine (STATE 0 through STATE 5).
   Steps 1-2 above handle STATE 0 (setup). The skill will continue from STATE 1:

   - STATE 0: Setup — detect change, verify artifacts, find `opsx-*` subagents,
     user selects agents, set max turns (default: 3 × agent count), initialize state file
   - STATE 1: Initial review — first agent reads artifacts, produces structured proposals,
     writes to state file
   - STATE 2: Consensus cycle — next agent reviews artifacts + current proposals, agrees
     or disagrees (with counter-proposals), may add new proposals. Turn counter increments.
     Cycle continues until all agents agree in a full cycle OR max turns hit
   - STATE 3: Present results to user — formatted summary of agreed/disputed proposals,
     user accepts, modifies, or rejects
   - STATE 4: Apply — apply approved proposals to artifact files, commit, cleanup
   - STATE 5: Summary — final report

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
   - The primary agent is an orchestrator — it does NOT analyze artifacts
   - All analysis happens inside subagent dispatches
   - Agents run sequentially (not in parallel) — each sees the previous agent's results
   - Consensus = one full cycle where every agent had zero disagreements and zero new proposals
   - Max turns (default 3 × agents) halts the process if consensus isn't reached
   - **STOP** at STATE 3 after presenting results and wait for user's response
   - Do NOT proceed to apply changes until the user explicitly accepts or modifies
   - Single commit after user acceptance (not per-turn)

**Guardrails**

- Only artifact files in the change directory are modified (never source code)
- Nothing is written until the user accepts at STATE 3
- User can reject at STATE 3 — no changes applied
- State file at `.openspec/tmp/deliberate-state.json` is internal — never shown to user
