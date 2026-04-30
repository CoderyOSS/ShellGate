---
name: opsx-arrange
description: >
  Reorder OpenSpec change tasks by dependency layer — foundation first, abstract later.
  Reads tasks.md, classifies each task into dependency layers (infrastructure through
  polish), builds a dependency tree, and rewrites tasks.md in build order with
  renumbered IDs. Use when tasks need logical reordering for implementation sequence.
---

# OpenSpec Task Arrangement

Reorder change tasks by dependency layer — foundation first, abstract later.

## Overview

This skill reads all artifacts for an OpenSpec change, classifies every task into
dependency layers (infrastructure → polish), builds an ASCII dependency tree showing
the build order, and rewrites `tasks.md` with tasks in implementation order and
renumbered IDs.

**Input**: Optionally specify a change name. If omitted, auto-detected from `openspec list --json`.

**Announce at start:** "Arranging tasks for change: **<name>**"

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

---

## STATE 0: SETUP

### 0.1 Detect the change name

If a name was provided as input, use it. Otherwise:
- Run: `openspec list --json`
- Auto-select if only one active (non-archived) change
- If ambiguous, ask the user which change to arrange

Always announce: "Arranging tasks for change: **<name>**"

### 0.2 Verify artifacts exist

```bash
ls openspec/changes/<name>/tasks.md openspec/changes/<name>/design.md 2>/dev/null
```

If `tasks.md` missing: "No tasks.md found. Run task generation first." STOP.
If `design.md` missing: warn user but continue (design aids classification but isn't required).

### STATE 0 OUTPUT

- Change name identified
- Artifacts verified
- Ready to extract tasks

**GATE**: tasks.md exists. Change name known.

---

## STATE 1: EXTRACT

### 1.1 Load all artifacts

Read every available artifact:
- `tasks.md` (required)
- `design.md` (if exists)
- All files in `specs/` subdirectory (if exists)

Read them all before proceeding.

### 1.2 Parse tasks

For each task in `tasks.md` matching `- [ ] X.Y Description`:
- `task_id`: X.Y
- `task_desc`: Description text
- `task_detail`: Any sub-items, code blocks, or notes under this task

Preserve the full content of each task — you will need to rewrite it later.

### 1.3 Classify each task

Read `references/layer-classification.md` for the full classification rules.

For each parsed task, determine:
- **Layer** (0–7): Which dependency layer this task belongs to
- **Component**: What system component this task builds (e.g., "database", "auth", "login page")
- **Dependencies**: Which other tasks (by current ID) this task depends on — inferred from:
  - Explicit references to other task IDs in the description
  - Implicit dependencies (a task building "API endpoints" depends on tasks building "models")
  - Layer ordering (layer N depends on layers 0 through N-1 existing)
- **Branch**: Which parallel branch this task belongs to (default: "main"; tasks that are
  independent of each other at the same layer get separate branches)

### 1.4 Output classification table

Present the classification to the user:

```
### Task Classification — <change-name>

| Task | Description | Layer | Component | Dependencies | Branch |
|------|------------|-------|-----------|-------------|--------|
| 1.1  | Set up database | 0 - Infrastructure | database | (none) | main |
| 1.2  | Build login page | 5 - Client Framework | login UI | 2.1, 3.1 | auth |
| 2.1  | Add auth middleware | 2 - Security/Auth | auth | 1.1 | main |
| ... | ... | ... | ... | ... | ... |
```

If any task could not be classified, mark it as "unclassified" and ask the user
which layer it belongs to. STOP and wait.

### STATE 1 OUTPUT

- All tasks parsed with full content preserved
- Every task classified into a layer
- Dependencies identified for each task
- Branch assignments made
- Classification table shown to user

**GATE**: Every task has a layer. Classification table shown.

---

## STATE 2: BUILD TREE

### 2.1 Construct the dependency tree

Build an ASCII tree representing the build order. Structure:

- Root nodes are dependency layers (0 through 7, only include layers that have tasks)
- Under each layer, show components as nodes
- Under each component, show task IDs in brackets
- Separate branches for tasks that can be done in parallel

Template:

```
<layer-name>
├── <component> [<task-ids>]
│   ├── <sub-component> [<task-ids>]
│   └── <sub-component> [<task-ids>]
├── <component> [<task-ids>]  (branch: <name>)
└── <component> [<task-ids>]

<next-layer-name>
├── <component> [<task-ids>]  (depends on: <component>)
...
```

### 2.2 Tree construction rules

1. Layers appear in order (0, 1, 2, ..., 7). Skip layers with no tasks.
2. Within a layer, components are ordered by dependency (most-depended-on first).
3. Components that depend on each other are nested (parent → child).
4. Independent components at the same layer are siblings (separate `├──` branches).
5. Every task ID must appear exactly once in the tree.
6. If a branch label differs from "main", annotate with `(branch: <name>)`.

### 2.3 Example output

```
infrastructure [1.1, 1.2]
├── database [1.1]
│   └── schema/migrations [1.1]
└── web server [1.2, 1.3]
    ├── framework setup [1.2]
    └── config [1.3]

core data [1.4, 1.5]
└── models [1.4, 1.5]
    ├── user model [1.4]
    └── session model [1.5]

security/auth [2.1, 2.2]
├── authentication [2.1]
└── authorization [2.2]

client framework [3.1]
└── router setup [3.1]

ui features [3.2, 3.3]  (branch: auth-features)
├── login page [3.2]
└── registration page [3.3]

polish [4.1]
└── error handling [4.1]
```

### 2.4 Save dependency tree

Write the ASCII tree to `openspec/changes/<name>/dependency-tree.md`:

```markdown
# Dependency Tree — <change-name>

Generated by opsx-arrange.

<tree>

## Layer Order

| Layer | Name | Tasks |
|-------|------|-------|
| 0 | Infrastructure | 1.1, 1.2, 1.3 |
| 1 | Core Data | 1.4, 1.5 |
| ... | ... | ... |

## Branches

| Branch | Tasks | Description |
|--------|-------|-------------|
| main | 1.1–1.5, 2.1, 2.2, 3.1, 4.1 | Core build path |
| auth-features | 3.2, 3.3 | Independent from branch X |
```

### 2.5 Present tree

Show the ASCII tree to the user. Announce:
> "Dependency tree built. Saved to `dependency-tree.md`."

### STATE 2 OUTPUT

- ASCII dependency tree constructed
- All task IDs present in tree
- Tree saved to `dependency-tree.md`
- Tree shown to user

**GATE**: Tree is complete. Every task appears exactly once. File saved.

---

## STATE 3: RESOLVE

### 3.1 Check for ordering ambiguity

Examine the classified tasks. Ambiguity exists when:

1. **Inter-layer ambiguity**: Two layers could swap order without breaking
   dependencies (e.g., "external integrations" before or after "security/auth"
   when neither depends on the other).

2. **Intra-layer ambiguity**: Tasks within the same layer have no dependency
   between them and could go in either order (e.g., "login page" before
   "registration page" or vice versa).

3. **Branch ambiguity**: Two branches at the same layer could be interleaved
   in multiple ways.

If **no ambiguity**: Skip to STATE 4. Announce "No ordering ambiguity detected."

### 3.2 Generate ordering options

For each ambiguity found, generate 2–3 valid orderings. For each option, provide:
- A label (e.g., "Auth-first", "Feature-first")
- The specific task order that differs
- A 1–2 sentence rationale for why this ordering makes sense

### 3.3 Present options

Use the `question` tool to present the options:

> "Found N ordering ambiguity(ies). Choose a preferred ordering:"

Show each option with its label and rationale. Include a "custom" option so the
user can specify their own ordering.

STOP and wait for the user's choice.

### 3.4 Apply user choice

Incorporate the user's selection into the task ordering. If the user chose "custom",
parse their instructions and apply them.

### STATE 3 OUTPUT

- All ambiguities identified and resolved
- Final ordering determined
- Ready to rewrite tasks.md

**GATE**: Ordering is finalized. No remaining ambiguities.

---

## STATE 4: OUTPUT

### 4.1 Renumber tasks

Assign new task IDs based on the finalized ordering:

- Task group numbering follows layer order: first group = 1, second = 2, etc.
- Within a group, tasks are numbered sequentially: 1.1, 1.2, 1.3, ...
- Build a mapping table: `{ old_id: new_id }`

Example mapping:

```
Old → New
1.1 → 1.1  (stayed in place)
3.1 → 1.2  (moved up — core data, was under old group 3)
2.1 → 2.1  (security/auth)
1.2 → 3.1  (was UI, moved down)
3.2 → 3.2  (stayed in place)
```

### 4.2 Update cross-references

Scan every task description and detail for references to old task IDs. Replace
each old ID with the corresponding new ID using the mapping table.

Look for patterns like:
- "depends on X.Y"
- "see task X.Y"
- "requires X.Y"
- "(X.Y)" parenthetical references
- "after X.Y" / "before X.Y"

### 4.3 Rewrite tasks.md

Write the new `tasks.md` with:

1. **Header** (preserve original header if present, update title to reflect new ordering):
   ```markdown
   # Tasks — <change-name>

   > Ordered by dependency layer. Foundation tasks first.
   > Generated by opsx-arrange. Original task IDs mapped below.

   ## ID Mapping

   | Old ID | New ID | Description |
   |--------|--------|-------------|
   | 1.1 | 1.1 | Set up database |
   | 3.1 | 1.2 | Create user model |
   | ... | ... | ... |
   ```

2. **Tasks grouped by layer**, with layer headings:

   ```markdown
   ---

   ## Layer 0: Infrastructure

   - [ ] 1.1 Set up database
   <original task content with updated cross-references>

   - [ ] 1.2 Create user model
   <original task content with updated cross-references>

   ---

   ## Layer 2: Security/Auth

   - [ ] 2.1 Add auth middleware
   <original task content with updated cross-references>
   ```

3. Preserve ALL original task content (sub-items, code blocks, notes).
   Only change: task IDs and cross-reference IDs.

### 4.4 Report

Output to user:

```
## Arrangement Complete — <change-name>

| Metric | Value |
|--------|-------|
| Total tasks | N |
| Layers used | L (of 8) |
| Branches | B |
| Ambiguities resolved | A |
| Cross-references updated | C |

Files:
- tasks.md — rewritten with new ordering
- dependency-tree.md — ASCII dependency tree
```

### STATE 4 OUTPUT

- tasks.md rewritten with new ordering and renumbered IDs
- Cross-references updated throughout
- ID mapping table included
- dependency-tree.md saved
- Summary shown to user

**GATE**: tasks.md rewritten. Cross-references correct. Tree saved.

---

## Dependency Layers

Quick reference. Full rules in `references/layer-classification.md`.

| # | Layer | What goes here |
|---|-------|---------------|
| 0 | Infrastructure | Database, server, build config, containers, deployment setup |
| 1 | Core Data | Models, schemas, repositories, data access layer, migrations |
| 2 | Security/Auth | Authentication, authorization, sessions, encryption, permissions |
| 3 | Core Services | Business logic, internal APIs, middleware, validation, event handlers |
| 4 | External Integration | Third-party APIs, webhooks, message queues, external service clients |
| 5 | Client Framework | Frontend setup, state management, router, component library, layout |
| 6 | UI Features | Pages, forms, feature-specific components, user-facing interactions |
| 7 | Polish | Error handling UI, loading states, accessibility, final tests, docs |

---

## Guardrails

- Only modify `tasks.md` and create `dependency-tree.md` — never source code
- Preserve ALL original task content — only change IDs and cross-references
- Every task ID from the original must appear in the rewritten file exactly once
- Layer classification follows the rules in `references/layer-classification.md` —
  when uncertain, ask the user rather than guessing
- Ambiguity resolution always defers to the user — never pick silently
- No backup of original tasks.md (git is the backup)
- Single agent workflow — no subagent dispatch needed
