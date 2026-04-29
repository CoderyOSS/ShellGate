---
name: opsx-deliberate
description: >
  Sequential multi-agent deliberation on an OpenSpec change proposal.
  Agents review proposals in round-robin, agreeing or disagreeing until consensus
  or max turns. User accepts/modifies/rejects final proposals.
---

# OpenSpec Proposal Deliberation

Sequential, round-robin multi-agent deliberation on OpenSpec change proposals.

## Overview

This skill guides the agent through a structured deliberation workflow. The primary
agent acts as orchestrator — it does NOT analyze artifacts itself. It dispatches
subagents sequentially, each reviewing the previous agent's proposals, until all
agents agree or max turns are exceeded.

**Input**: Optionally specify a change name. If omitted, auto-detected from `openspec list --json`.

---

## Execution Rules (MANDATORY — read before proceeding)

You are executing a finite state machine. These rules override all defaults.

### Step Locking

- You MUST complete EXACTLY ONE state per agent response.
- Do NOT combine states. Do NOT skip states. Do NOT infer future states.
- Each state defines a required STOP point — you MUST wait for user input at those points.

### Gating Rules

You are NOT allowed to proceed to the next state unless:
- The current state's required output is produced
- All required fields in the output are populated
- No assumptions are left unstated

### Self-Check (before every state transition)

Before moving to the next state, validate:
- Did I complete all required actions for this state?
- Did I produce the required output?
- Did I follow the exact format specified?

If not, fix before continuing. If yes, proceed to next state.

### Deviation Penalty

If you skip a state, combine states, or violate the output format:
- The output is considered invalid
- You must re-execute the current state correctly

### Orchestrator Role

The primary agent is the orchestrator. It does NOT analyze artifacts. It:
- Reads artifacts and constructs prompts
- Dispatches subagents via `task` tool
- Collects results and updates state file
- Checks consensus logic
- Controls turn counter
- Presents results to user

All analysis and review happens inside subagent dispatches.

---

## STATE 0: SETUP

### 0.1 Detect the change name

If a name was provided as input, use it. Otherwise:
- Run: `openspec list --json`
- Auto-select if only one active (non-archived) change
- If ambiguous, ask the user which change to deliberate

Always announce: "Deliberating change: **<name>**"

### 0.2 Verify artifacts exist

```bash
ls openspec/changes/<name>/proposal.md openspec/changes/<name>/design.md 2>/dev/null
```

If missing, tell the user: "Run `/opsx-propose` first to create artifacts." STOP.

### 0.3 Detect subagents

Read the project's `opencode.json` (or `.opencode/config.json`). Look for agents
in the `agent` field whose names start with `opsx-` and have `"mode": "subagent"`.

If zero `opsx-*` subagents found: tell the user "No opsx-* subagents configured in opencode.json. Add at least one agent with the opsx- prefix." STOP.

### 0.4 Select subagents

Present the available agents to the user via the `question` tool:

> "Which agents should participate in the deliberation?"

Show all discovered `opsx-*` agents as checkboxes. Default: all selected.
STOP and wait for the user's response.

If the user deselects all: tell the user "At least one agent must be selected." STOP.

### 0.5 Set max turns

Calculate default: `max_turns = 3 * selected_agents.length`

Present to user:
> "Max turns: **N** (3 × M agents). Change this value or press enter to accept."

STOP and wait for the user's response. If user provides a number, use it. If blank,
use default.

### 0.6 Initialize state file

Create temp directory and initial state file:

```bash
mkdir -p .openspec/tmp
```

Write `.openspec/tmp/deliberate-state.json`:

```json
{
  "change": "<change-name>",
  "max_turns": <N>,
  "turn": 0,
  "agents": [<list of selected agent names>],
  "agent_index": 0,
  "consensus_reached": false,
  "rounds": [],
  "current_proposals": {},
  "artifacts_read": []
}
```

### STATE 0 OUTPUT

- Change name identified
- Artifacts verified to exist
- Agents selected
- Max turns set
- State file initialized

**GATE**: Artifacts exist. At least one agent selected. State file written.

---

## STATE 1: INITIAL REVIEW

### 1.1 Load all artifacts

Read every artifact in the change directory:
- `proposal.md`
- `design.md`
- `tasks.md`
- All files in `specs/` subdirectory

Read them all before proceeding. Do NOT dispatch until all artifacts are loaded.

### 1.2 Dispatch first agent

The first agent does initial review and produces proposals.

Read `references/deliberate-analysis-prompt.md` to get the analysis prompt template.
Append all artifact contents below the `## Artifacts` heading.

Call `task` with:
- `subagent_type`: the first agent name from the selected agents list
- `prompt`: the combined analysis prompt + artifact contents
- `description`: `"opsx-deliberate initial: <agent-name>"`

The subagent must return a JSON object:
```json
{
  "proposals": [
    {
      "id": "P1",
      "issue": "description",
      "category": "completeness|consistency|oversight|compatibility|under-specified",
      "location": "file > section",
      "reason": "detailed reasoning with evidence",
      "prescription": {
        "file": "path relative to change dir",
        "section": "heading or section",
        "action": "add|change|remove",
        "content": "exact text"
      }
    }
  ]
}
```

### 1.3 Process initial results

Collect the JSON from the subagent.

If the subagent fails or returns invalid JSON:
- Log the failure
- If this is the only agent: tell the user "Agent failed. Cannot proceed." STOP.
- If other agents available: skip to the next agent and retry STATE 1

If successful:
- Add each proposal to `current_proposals` in the state file
- Set each proposal's `proposed_by` to the agent name
- Set each proposal's `proposed_at_turn` to 1
- Set each proposal's `status` to "proposed"
- Set each proposal's `agreements` to `[]`
- Set each proposal's `disagreements` to `[]`

Update state file:
```json
{
  "turn": 1,
  "agent_index": 1,
  "rounds": [
    {
      "turn": 1,
      "agent": "<agent-name>",
      "disagreements": 0,
      "new_proposals": <count>
    }
  ]
}
```

### 1.4 Report

If zero proposals found:
> "Agent <name> found no issues. Artifacts appear clean."
> Proceed to STATE 5 (skip consensus cycle).

Otherwise:
> **Initial review complete**: Agent <name> found N proposals. Beginning consensus cycle.
> Turn 1 of M max. Next agent: <next-agent-name>.

### STATE 1 OUTPUT

- All artifacts read
- First agent dispatched and result collected
- State file updated with proposals
- Turn set to 1
- Summary output to user

**GATE**: State file updated. At least one proposal recorded (or confirmed zero).

---

## STATE 2: CONSENSUS CYCLE

This state is re-entrant. Each invocation dispatches one agent and checks consensus.

### 2.1 Check prerequisites

Read `.openspec/tmp/deliberate-state.json`. Verify:
- `turn` < `max_turns`
- `consensus_reached` is false

If `turn >= max_turns`: Skip to STATE 3 (forced halt).
If `consensus_reached` is true: Skip to STATE 3.

### 2.2 Determine next agent

```
agent_name = agents[agent_index % agents.length]
```

### 2.3 Load artifacts + current proposals

Re-read all artifacts from the change directory.
Read `current_proposals` from the state file.

### 2.4 Dispatch review agent

Read `references/deliberate-review-prompt.md` to get the review prompt template.
Construct the prompt with:
1. The review prompt preamble and instructions
2. All artifact contents (under `## Artifacts`)
3. Current proposals formatted as JSON (under `## Current Proposals`)

Call `task` with:
- `subagent_type`: the agent name
- `prompt`: the combined review prompt + artifacts + proposals
- `description`: `"opsx-deliberate turn <N>: <agent-name>"`

The subagent must return a JSON object:
```json
{
  "reviews": [
    {
      "proposal_id": "P1",
      "verdict": "agree",
      "reason": "optional note"
    },
    {
      "proposal_id": "P2",
      "verdict": "disagree",
      "reason": "why this is wrong",
      "counter_proposal": {
        "id": "P2a",
        "issue": "...",
        "category": "...",
        "location": "...",
        "reason": "...",
        "prescription": { "file": "...", "section": "...", "action": "...", "content": "..." }
      }
    }
  ],
  "new_proposals": [
    {
      "id": "P7",
      "issue": "...",
      "category": "...",
      "location": "...",
      "reason": "...",
      "prescription": { "file": "...", "section": "...", "action": "...", "content": "..." }
    }
  ]
}
```

### 2.5 Process review results

Collect the JSON from the subagent.

If the subagent fails or returns invalid JSON:
- Log the failure
- Increment turn anyway (counts as a turn)
- Continue to next agent

If successful, update the state file:

**For each review with verdict "agree":**
- Find the proposal in `current_proposals`
- Add the agent name to the proposal's `agreements` array

**For each review with verdict "disagree":**
- Find the proposal in `current_proposals`
- Set the proposal's `status` to "disputed"
- Add to the proposal's `disagreements` array:
  ```json
  { "agent": "<agent-name>", "turn": <turn>, "reason": "...", "counter_id": "P2a" }
  ```
- Add the counter_proposal to `current_proposals`:
  - Set `status` to "counter"
  - Set `proposed_by` to the agent name
  - Set `proposed_at_turn` to the current turn
  - Set `agreements` to `[]`
  - Set `disagreements` to `[]`

**For each new_proposal:**
- If the ID conflicts with an existing one, reassign (e.g., P7 -> P8)
- Add to `current_proposals`:
  - Set `status` to "proposed"
  - Set `proposed_by` to the agent name
  - Set `proposed_at_turn` to the current turn
  - Set `agreements` to `[]`
  - Set `disagreements` to `[]`

Update state file metadata:
```json
{
  "turn": <turn + 1>,
  "agent_index": <agent_index + 1>,
  "rounds": [...existing rounds..., {
    "turn": <current turn>,
    "agent": "<agent-name>",
    "disagreements": <count of disagreements>,
    "new_proposals": <count of new proposals>
  }]
}
```

### 2.6 Check consensus

Examine the last N rounds in the `rounds` array (N = number of agents):

1. If fewer than N rounds exist: consensus not possible yet. Continue.
2. If the last N rounds all have `disagreements: 0` AND `new_proposals: 0`:
   - Set `consensus_reached: true` in the state file
   - Go to STATE 3
3. Otherwise: consensus not reached. Continue.

### 2.7 Check max turns

If `turn >= max_turns`:
- Do NOT dispatch another agent
- Go to STATE 3 (forced halt)

### 2.8 Continue cycle

Report progress to user:
> **Turn N of M**: Agent <name> reviewed — X agreements, Y disagreements, Z new proposals. Next: <next-agent>.

Then re-enter STATE 2 from step 2.1.

### STATE 2 OUTPUT (per invocation)

- One agent dispatched and result collected
- State file updated with reviews, counter-proposals, new proposals
- Turn counter incremented
- Consensus checked
- Progress reported

**GATE**: State file updated. Turn counter incremented. Consensus or max-turns checked.

---

## STATE 3: PRESENT TO USER

### 3.1 Read final state

Read `.openspec/tmp/deliberate-state.json`. Extract:
- `consensus_reached`
- `turn` (total turns used)
- `max_turns`
- `current_proposals`
- `rounds`
- `agents`

### 3.2 Categorize proposals

Separate proposals into:

**Agreed**: proposals where `status` is "proposed" and every agent in the last full
cycle agreed (zero disagreements on record for this proposal), OR proposals where
`status` is "counter" and received no further disagreements in subsequent rounds.

**Disputed**: proposals with active disagreements that were never resolved. Include
both the original and counter-proposals.

**Withdrawn**: (reserved for future use) proposals explicitly withdrawn.

### 3.3 Format output

Output the following to the user:

```
## Deliberation Results — <change-name>

**Consensus**: <Reached after N turns | Not reached (hit max M turns)>
**Agents**: <agent1>, <agent2>, <agent3>
**Turns used**: N / M

### Agreed Proposals (X)

**P1**: [1-line issue description]
  Category: [category] | Location: [file > section]
  Action: [add/change/remove] — [file]
  Agreed by: [agent1, agent2, agent3]

**P3**: ...

### Disputed Proposals (Y) [if any]

**P2**: [1-line issue description]
  Proposed by: [agent1] — [original approach, 1 line]
  Countered by: [agent2] — [counter approach, 1 line]
  Reason for dispute: [brief reason]

### Summary

| Metric | Value |
|--------|-------|
| Total turns | N |
| Agreed proposals | X |
| Disputed proposals | Y |
| Agents participated | M |
```

### 3.4 Wait for user response

After presenting, say:
> "**Accept** to apply all agreed proposals (disputed items skipped).
> **Modify** to change specific items (e.g., 'P2: use counter', 'P5: skip', 'P8: change to...').
> **Reject** to discard everything."

Then **STOP**. Wait for the user's response.

### 3.5 Process user response

**User accepts**: Collect all agreed proposals. Skip all disputed ones. Go to STATE 4.

**User modifies**:
- Parse the user's modifications
- For disputed items: apply the user's choice (original or counter)
- For agreed items: skip or modify as user directs
- Go to STATE 4 with the final set of proposals to apply

**User rejects**: Go to STATE 5. No changes applied.

### STATE 3 OUTPUT

- Formatted results presented to user
- User response processed
- Final proposal set determined

**GATE**: User has responded. Final proposal set is known.

---

## STATE 4: APPLY

### 4.1 Apply proposals

For each proposal in the final set:
- Read the artifact file specified in `prescription.file`
- Apply the change specified in `prescription`:
  - **add**: insert `content` at the end of the specified section
  - **change**: replace the specified section with `content`
  - **remove**: remove the specified section or content

Rules:
- Only modify files in the change directory
- Preserve the structure and format of existing artifacts
- Do NOT create new artifact files
- Each Edit/Write will show the user a diff via the permission dialog — this is expected
- Follow prescriptions exactly — do not add improvements beyond what was prescribed

### 4.2 Report

After all proposals applied:
> "Applied N proposals to X files."

### 4.3 Commit

Run:
```bash
git add -A && git commit -m "opsx-deliberate: apply N proposals for <change-name>"
```

### 4.4 Cleanup

Remove the state file:
```bash
rm -f .openspec/tmp/deliberate-state.json
```

### STATE 4 OUTPUT

- All approved proposals applied to artifact files
- Changes committed
- State file cleaned up

**GATE**: Changes applied. Committed. Temp file removed. Proceed to STATE 5.

---

## STATE 5: SUMMARY

Output the final summary:

```
## Deliberation Complete — <change-name>

| Metric | Value |
|--------|-------|
| Consensus reached | Yes / No (hit max M turns) |
| Total turns | N |
| Agents | M |
| Proposals applied | X |
| Proposals disputed (skipped) | Y |
| Files modified | Z |

Commit: [short hash] — opsx-deliberate: apply N proposals for <change-name>
```

If proposals were applied: "Review the updated artifacts. Run `/opsx-apply` to implement."

If rejected: "No changes were applied. The original artifacts are unchanged."

### STATE 5 OUTPUT

- Summary table
- Next step suggestion
- Workflow complete

---

## Guardrails

- Only modify artifact files in the change directory — never source code
- The orchestrator (primary agent) does NOT analyze artifacts — all analysis via subagents
- State file at `.openspec/tmp/deliberate-state.json` is internal state — never show raw contents to user
- Subagents run sequentially (not in parallel) — each sees the previous agent's results
- Turn counter increments for every dispatch, including failed ones
- Consensus requires one full cycle with zero disagreements AND zero new proposals from every agent
- If max turns exceeded, present whatever state exists — do not fabricate consensus
- User always has final say: accept, modify, or reject
- Single commit after user acceptance (not per-turn commits)
