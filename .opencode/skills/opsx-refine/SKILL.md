---
name: opsx-refine
description: >
  Interactive tier-by-tier refinement of an OpenSpec change proposal.
  Reviews artifacts for consistency, completeness, and oversights. Classifies issues
  as editorial/architectural/speculative. For each tier: presents findings, waits for user
  response, applies approved changes, commits by default.
---

# OpenSpec Proposal Refinement

Interactive, tier-by-tier review and refinement of an OpenSpec change proposal.

## Overview

This skill guides the agent through a structured review of OpenSpec change artifacts.
The workflow is a multi-turn conversation — the agent presents findings at each tier,
the user responds with approvals/corrections/abort, and the agent acts accordingly.

**Input**: Optionally specify a change name. If omitted, auto-detected from `openspec list --json`.

---

## Execution Rules (MANDATORY — read before proceeding)

You are executing a finite state machine. These rules override all defaults.

### Step Locking

- You MUST complete EXACTLY ONE state per agent response during tier interaction.
- Do NOT combine states. Do NOT skip states. Do NOT infer future states.
- Do NOT show issues from a tier until the previous tier is fully resolved (including commit).
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

---

## STATE 0: SETUP

### 0.1 Detect the change name

If a name was provided as input, use it. Otherwise:
- Run: `openspec list --json`
- Auto-select if only one active (non-archived) change
- If ambiguous, ask the user which change to refine

Always announce: "Refining change: **<name>**"

### 0.2 Verify artifacts exist

```bash
ls openspec/changes/<name>/proposal.md openspec/changes/<name>/design.md 2>/dev/null
```

If missing, tell the user: "Run `/opsx-propose` first to create artifacts." STOP.

### STATE 0 OUTPUT

- Change name identified
- Artifacts verified to exist

**GATE**: Artifacts must exist. If not, STOP.

---

## STATE 1: ANALYZE

STATE 1 has two modes: **multi-agent** (when `opsx-*` subagents are configured in opencode.json)
and **single-agent** (fallback when no subagents are configured). The output is identical either way.

### 1.1 Load all artifacts

Read every artifact in the change directory:
- `proposal.md`
- `design.md`
- `tasks.md`
- All files in `specs/` subdirectory

Read them all before proceeding. Do not begin analysis until all artifacts are loaded.

Create the temp directory:
```bash
mkdir -p .openspec/tmp
```

### 1.2 Detect subagents

Read the project's `opencode.json` (or `.opencode/config.json`). Look for agents
in the `agent` field whose names start with `opsx-` and have `"mode": "subagent"`.

Example opencode.json configuration:
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

If zero `opsx-*` subagents found: skip to step 1.7 (single-agent fallback).

### 1.3 Select subagents

Present the available agents to the user via the `question` tool:

> "Which agents should run the analysis?"

Show all discovered `opsx-*` agents as checkboxes. Default: all selected.
STOP and wait for the user's response.

If the user deselects all: fall back to single-agent mode (step 1.7).

### 1.4 Dispatch subagents

For each selected agent, dispatch a `task` in parallel:

- Read `references/analysis-prompt.md` to get the analysis prompt template
- Append all artifact contents below the `## Artifacts` heading
- Call `task` with:
  - `subagent_type`: the agent name (e.g., `"opsx-deepseek"`)
  - The `task` tool accepts any agent name defined in the `agent` config with
    `"mode": "subagent"` as a valid `subagent_type`. This is not limited to
    built-in types like `explore` or `general`.
  - `prompt`: the combined analysis prompt + artifact contents
  - `description`: `"opsx-refine analysis: <agent-name>"`

Dispatch ALL selected agents in parallel. Do NOT wait for one to finish before
dispatching the next.

Each subagent must return ONLY a JSON object matching this schema:

```json
{
  "summary": { "editorial": 0, "architectural": 0, "speculative": 0, "total": 0 },
  "issues": [
    {
      "id": "E1",
      "issue": "description",
      "category": "editorial|architectural|speculative",
      "location": "file > section",
      "reason": "why this is an issue",
      "prescription": {
        "file": "path relative to change dir",
        "section": "heading or section",
        "action": "add|change|remove",
        "content": "exact text to add/replace/remove"
      }
    }
  ]
}
```

ID conventions:
- `E1`, `E2`, ... for editorial
- `A1`, `A2`, ... for architectural
- `Q1`, `Q2`, ... for speculative

For architectural issues, each issue must include:
```json
"recommendation": {
  "label": "a",
  "description": "recommended approach",
  "prescription": { "file": "...", "section": "...", "action": "...", "content": "..." }
},
"alternatives": [
  { "label": "b", "description": "...", "prescription": { ... } }
]
```

For speculative issues, each issue must include:
```json
"clarified_question": "the actual decision the user needs to make",
"options": ["option 1", "option 2"],
"recommendation": "which direction + what info is needed"
```

### 1.5 Collect results

Wait for all dispatched subagents to return. Collect each JSON output.

If a subagent fails or returns invalid JSON:
- Log which agent failed
- Continue with results from the other agents
- If ALL agents fail, fall back to single-agent mode (step 1.7)

### 1.6 Merge and deduplicate

Combine all agent results into a single deduplicated list:

1. Collect all issues from all agents into one list
2. Tag each issue with its source agent for attribution
3. Group issues by `location` (file > section)
4. Within each location group, compare issue descriptions semantically:
   - Two issues are "duplicates" if they describe the same problem at the same location
   - Among duplicates, keep the one with the most detailed `reason` field
   - If agents disagree on category, keep the one with better reasoning (more specific,
     more evidence, deeper analysis wins)
5. Collect all surviving unique issues
6. Reassign sequential IDs: `E1`, `E2`, ... then `A1`, `A2`, ... then `Q1`, `Q2`, ...
7. Count totals per category

Write the merged JSON to:
```
.openspec/tmp/refine-analysis.json
```

Include metadata:
```json
{
  "change": "<change-name>",
  "analyzed_at": "<ISO timestamp>",
  "agents_used": ["opsx-deepseek", "opsx-kimi"],
  "agents_failed": [],
  "artifacts_read": ["proposal.md", "design.md", "..."],
  "summary": { "editorial": N, "architectural": N, "speculative": N, "total": N },
  "issues": [...]
}
```

Skip to step 1.8.

### 1.7 Single-agent fallback

If no `opsx-*` subagents are configured, or all were deselected, or all failed:

Using the review criteria in `references/review-criteria.md` and classification rules in
`references/classification-rules.md`, analyze ALL artifacts and produce the JSON directly.

Read `references/analysis-prompt.md` for the analysis preamble and instructions — apply
the same thinking process even though you are a single agent.

Produce the same JSON format and write to `.openspec/tmp/refine-analysis.json`:
```json
{
  "change": "<change-name>",
  "analyzed_at": "<ISO timestamp>",
  "agents_used": [],
  "agents_failed": [],
  "artifacts_read": ["proposal.md", "design.md", "..."],
  "summary": { "editorial": N, "architectural": N, "speculative": N, "total": N },
  "issues": [...]
}
```

### 1.8 Report findings summary

Output ONLY the summary line. Do NOT show individual issues yet.

Format:
> **Analysis complete for change "<name>"**: Found N issues — E editorial, A architectural, Q speculative.
>
> Proceeding to first tier with issues.

If zero issues found: "No issues found. All artifacts are consistent and complete."
STOP entirely. Clean up temp file: `rm -f .openspec/tmp/refine-analysis.json`

### STATE 1 OUTPUT

- All artifacts read and analyzed
- Complete JSON written to `.openspec/tmp/refine-analysis.json`
- Summary line with counts output to user
- Do NOT output individual issues — those are revealed per-tier

**GATE**: JSON file must exist and be valid. Summary must be output.

---

## STATE 2: EDITORIAL TIER

**Precondition**: STATE 1 complete. Temp file exists.

If `editorial` count is 0: Skip silently. Proceed to STATE 3.

### 2.1 Load editorial issues

Read `.openspec/tmp/refine-analysis.json` and filter to only `category: "editorial"` issues.

### 2.2 Present prescriptions

For each editorial issue, output a compact prescription:

```
**E1**: [1-line description of what changes]
  File: [file] | Section: [section] | Action: [add/change/remove]

**E2**: ...
```

After the list, say:
> "Review the above. **Approve all** to apply everything, or **give corrections** for
> specific items (e.g. 'E3: skip, E5: change to...'). Say **abort** to stop the entire workflow."

Then **STOP**. Wait for the user's response. Do NOT proceed.

### 2.3 Process user response

**User approves all** (or says nothing / says yes): Apply all prescriptions. Go to 2.4.

**User gives corrections**:
- Skip items the user said to skip
- Modify items where the user gave different instructions
- Apply the rest as-is
- Go to 2.4.

**User says abort**: Output "Workflow aborted." STOP entirely.

### 2.4 Apply editorial fixes

For each approved prescription, use the Edit or Write tool to modify the artifact file.
Follow prescriptions exactly — do not add improvements beyond what was prescribed.

Rules:
- Only modify files in the change directory
- Preserve the structure and format of existing artifacts
- Do NOT create new artifact files
- Each Edit/Write will show the user a diff via the permission dialog — this is expected

After all fixes applied, output:
> "Applied N editorial fixes to X files."

### 2.5 Commit gate

**Default is to commit. The user must explicitly say no to skip.**

Say:
> "Committing these N editorial fixes. Say **no** to skip."

Then run:
```bash
git add -A && git commit -m "opsx-refine: apply N editorial fixes"
```

Only skip the commit if the user explicitly objects before you run the command.
If the user says nothing or says yes, commit.

### STATE 2 OUTPUT

- Editorial issues presented to user
- User response processed
- Approved fixes applied to artifact files
- Commit completed (or explicitly skipped)

**GATE**: All editorial fixes applied. Commit resolved. Then proceed to STATE 3.

---

## STATE 3: ARCHITECTURAL TIER

**Precondition**: STATE 2 complete (or skipped if empty).

If `architectural` count is 0: Skip silently. Proceed to STATE 4.

### 3.1 Load architectural issues

Read `.openspec/tmp/refine-analysis.json` and filter to only `category: "architectural"` issues.

### 3.2 Present decision sheet

For each architectural issue, output a compact decision block:

```
**A1**: [1-line issue summary]
  Recommended: **(a)** [approach, 1 line]
  Alternatives: **(b)** [alt 1] | **(c)** [alt 2]

**A2**: ...
```

After the list, say:
> "For each issue, pick **(a)** for recommended, **(b)**/**(c)** for an alternative, or **skip**.
> Example: 'A1: a, A2: b, A3: skip'. Leave blank to accept all recommendations.
> Say **abort** to stop."

Then **STOP**. Wait for the user's response. Do NOT proceed.

### 3.3 Process user response

**User approves defaults** (or says nothing / says yes): Apply all recommendations (option a for each). Go to 3.4.

**User picks specific options**: Apply those choices. Skip items user said to skip. Go to 3.4.

**User says abort**: Output "Workflow aborted." STOP entirely.

### 3.4 Apply architectural fixes

For each chosen option, use the full analysis context (from temp file) to find the
corresponding prescription and apply it via Edit/Write tools.

Rules:
- Only modify files in the change directory
- Apply the specific approach the user chose, not the default
- Preserve the structure and format of existing artifacts

After all fixes applied, output:
> "Applied N architectural fixes to X files."

### 3.5 Commit gate

**Default is to commit.** Same rules as step 2.5.

Say:
> "Committing these N architectural fixes. Say **no** to skip."

Then run:
```bash
git add -A && git commit -m "opsx-refine: apply N architectural fixes"
```

Only skip if the user explicitly objects.

### STATE 3 OUTPUT

- Architectural issues presented to user
- User response processed
- Approved fixes applied
- Commit completed (or explicitly skipped)

**GATE**: All architectural fixes applied. Commit resolved. Then proceed to STATE 4.

---

## STATE 4: SPECULATIVE TIER

**Precondition**: STATE 3 complete (or skipped if empty).

If `speculative` count is 0: Skip silently. Proceed to STATE 5.

### 4.1 Load speculative issues

Read `.openspec/tmp/refine-analysis.json` and filter to only `category: "speculative"` issues.

### 4.2 Present questions

For each speculative issue, output:

```
**Q1**: [the clarified question, 1-2 sentences]
  Context: [1 line of why this matters]
  Options: [numbered, 1 line each]

**Q2**: ...
```

After the list, say:
> "For each question, describe what you want. Examples:
> 'Q1: use option 2' or 'Q1: do nothing' or 'Q2: I'll handle this separately'
> Say **abort** to stop. 'Do nothing' for all items skips this tier."

Then **STOP**. Wait for the user's response. Do NOT proceed.

### 4.3 Process user response

**User gives directions**: Translate each direction into minimal artifact changes. Go to 4.4.

**User says "do nothing" for all** (or explicitly skips all): Skip this tier. Proceed to STATE 5.

**User says abort**: Output "Workflow aborted." STOP entirely.

### 4.4 Apply speculative fixes

Translate user's directions into artifact changes:
- If direction requires adding a new decision to design.md, add it
- If direction resolves an open question, update or remove it
- If direction adds a new requirement, add it to the appropriate spec
- For "do nothing" items, make no changes

Rules:
- Only modify files in the change directory
- Make minimal changes that implement the user's direction
- Preserve the structure and format of existing artifacts

After all fixes applied, output:
> "Applied N speculative resolutions to X files."

### 4.5 Commit gate

**Default is to commit.** Same rules as step 2.5.

Say:
> "Committing these N speculative resolutions. Say **no** to skip."

Then run:
```bash
git add -A && git commit -m "opsx-refine: apply N speculative resolutions"
```

Only skip if the user explicitly objects.

### 4.6 Cleanup

Remove the temp file:
```bash
rm -f .openspec/tmp/refine-analysis.json
```

### STATE 4 OUTPUT

- Speculative issues presented to user
- User response processed
- Approved fixes applied
- Commit completed (or explicitly skipped)
- Temp file cleaned up

**GATE**: All speculative fixes applied. Commit resolved. Temp file cleaned up. Proceed to STATE 5.

---

## STATE 5: SUMMARY

After all tiers are complete (or skipped if empty), output a final summary:

```
## Refinement Complete — <change-name>

| Tier | Issues | Applied | Skipped |
|------|--------|---------|---------|
| Editorial | N | N | N |
| Architectural | N | N | N |
| Speculative | N | N | N |

Files modified: [list]
Commits created: [count and refs]
```

Suggest next step:
- If fixes applied: "Review the updated artifacts. Run `/opsx-apply` to implement."
- If issues remain (architectural skipped, speculative unresolved): "Address remaining items and re-run."

### STATE 5 OUTPUT

- Summary table
- Next step suggestion
- Workflow complete

---

## Guardrails

- Only modify artifact files in the change directory — never source code
- Each tier requires explicit user approval before changes are applied
- **Commits are the default after each tier.** The user must explicitly say no to skip committing.
- User can abort at any tier — previously committed changes are preserved
- Rejections with corrections are handled per-item (not all-or-nothing)
- Follow prescriptions exactly — do not add improvements beyond what was prescribed or approved
- If a fix is ambiguous, apply the minimal interpretation
- The temp file at `.openspec/tmp/refine-analysis.json` is internal state — never show its contents to the user
