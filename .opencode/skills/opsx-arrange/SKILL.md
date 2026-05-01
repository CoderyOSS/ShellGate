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

## STATE 1: EXTRACT & BUILD

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

If any task could not be classified, mark it as "unclassified".

### 1.4 Build the dependency tree

Construct an ASCII tree that serves as both the classification visualization and the
review artifact. Each leaf node shows `[task-id task-description]` so the tree is
self-documenting — no separate table needed.

**Tree construction rules:**

1. Root nodes are dependency layers (0 through 7). Skip layers with no tasks.
2. Within a layer, components are ordered by dependency (most-depended-on first).
3. Components that depend on each other are nested (parent → child).
4. Independent components at the same layer are siblings (separate `├──` branches).
5. Every task appears exactly once in the tree.
6. If a branch label differs from "main", annotate with `(branch: <name>)`.

**Format:**

```
<layer-name>
├── <component>
│   ├── <sub-component>
│   │   └── [X.Y <task description>]
│   └── <sub-component>
│       └── [X.Y <task description>]
└── <component>  (branch: <name>)
    └── [X.Y <task description>]

<next-layer-name>
├── <component>
│   └── [X.Y <task description>]
└── <component>
    └── [X.Y <task description>]
```

**Example:**

```
infrastructure
├── database
│   └── schema/migrations
│       └── [1.1 Set up PostgreSQL with user and session tables]
└── web server
    ├── framework setup
    │   └── [1.2 Initialize Express server with TypeScript]
    └── config
        └── [1.3 Create environment configuration module]

core data
└── models
    ├── user model
    │   └── [3.2 Create User model with email, password hash, timestamps]
    └── session model
        └── [3.5 Create Session model with token, expiry, user relation]

security/auth
├── authentication
│   └── [2.1 Implement JWT authentication middleware]
└── authorization
    └── [2.2 Add role-based access control middleware]

client framework
└── router setup
    └── [4.1 Configure React Router with route definitions]

ui features  (branch: auth-features)
├── login page
│   └── [1.4 Build login form with email and password fields]
└── registration page
    └── [3.3 Create registration form with validation]

polish
└── error handling
    └── [5.1 Add error boundary and fallback UI for all pages]
```

### 1.5 Present tree and confirm

Show the full dependency tree to the user.

Then use the `question` tool to ask:

> "Review the dependency tree above. Tasks grouped by layer — foundation at top.
> Confirm to proceed, or specify corrections (e.g., '3.2 should be in core data',
> '1.4 and 3.3 belong in the same branch')."

Options: **Confirmed** / **I have corrections**

STOP and wait for the user's response.

If the user provides corrections:
- Apply the corrections
- Rebuild and re-show the tree
- Ask again (loop until confirmed)

### 1.6 Save dependency tree

After the user confirms, write the ASCII tree to `openspec/changes/<name>/dependency-tree.md`:

```markdown
# Dependency Tree — <change-name>

Generated by opsx-arrange.

<tree>

## Layer Order

| Layer | Name | Tasks |
|-------|------|-------|
| 0 | Infrastructure | 1.1, 1.2, 1.3 |
| 1 | Core Data | 3.2, 3.5 |
| ... | ... | ... |

## Branches

| Branch | Tasks | Description |
|--------|-------|-------------|
| main | 1.1–1.3, 2.1, 2.2, 3.2, 3.5, 4.1, 5.1 | Core build path |
| auth-features | 1.4, 3.3 | Independent auth UI |
```

### STATE 1 OUTPUT

- All tasks parsed with full content preserved
- Every task classified into a layer
- Dependencies identified, branches assigned
- Dependency tree built with task descriptions at every node
- Tree confirmed by user
- Tree saved to `dependency-tree.md`

**GATE**: Tree confirmed by user. File saved.

---

## STATE 2: RESOLVE

### 2.1 Check for ordering ambiguity

Examine the classified tasks. Ambiguity exists when:

1. **Inter-layer ambiguity**: Two layers could swap order without breaking
   dependencies (e.g., "external integrations" before or after "security/auth"
   when neither depends on the other).

2. **Intra-layer ambiguity**: Tasks within the same layer have no dependency
   between them and could go in either order (e.g., "login page" before
   "registration page" or vice versa).

3. **Branch ambiguity**: Two branches at the same layer could be interleaved
   in multiple ways.

If **no ambiguity**: Skip to STATE 3. Announce "No ordering ambiguity detected."

### 2.2 Generate ordering options

For each ambiguity found, generate 2–3 valid orderings. For each option, provide:
- A label (e.g., "Auth-first", "Feature-first")
- The specific task order that differs
- A 1–2 sentence rationale for why this ordering makes sense

### 2.3 Present options

Use the `question` tool to present the options:

> "Found N ordering ambiguity(ies). Choose a preferred ordering:"

Show each option with its label and rationale. Include a "custom" option so the
user can specify their own ordering.

STOP and wait for the user's choice.

### 2.4 Apply user choice

Incorporate the user's selection into the task ordering. If the user chose "custom",
parse their instructions and apply them.

### STATE 2 OUTPUT

- All ambiguities identified and resolved
- Final ordering determined
- Ready to rewrite tasks.md

**GATE**: Ordering is finalized. No remaining ambiguities.

---

## STATE 3: OUTPUT

### 3.1 Renumber tasks

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

### 3.2 Update cross-references

Scan every task description and detail for references to old task IDs. Replace
each old ID with the corresponding new ID using the mapping table.

Look for patterns like:
- "depends on X.Y"
- "see task X.Y"
- "requires X.Y"
- "(X.Y)" parenthetical references
- "after X.Y" / "before X.Y"

### 3.3 Write to temp file

Write the reordered tasks to `openspec/changes/<name>/tasks.md.tmp` (NOT `tasks.md`).
The original `tasks.md` must remain untouched until STATE 4 verification passes.

Write with:

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

### STATE 3 OUTPUT

- `tasks.md.tmp` written with reordered tasks and renumbered IDs
- Original `tasks.md` untouched
- Cross-references updated throughout
- ID mapping table included in temp file
- `dependency-tree.md` saved

**GATE**: Temp file written. Original untouched. Tree saved.

---

## STATE 4: VERIFY

### 4.1 Read both files

Read the original `tasks.md` and the new `tasks.md.tmp` into memory.

### 4.2 Extract task blocks

From both files, extract every task as a content block: everything belonging to one
task from its `- [ ] X.Y` line through to the next task header (or end of file).

Normalize each block for comparison:
- Strip the task ID from the header line (`- [ ] X.Y Description` → `Description`)
- Keep all other content unchanged (sub-items, code blocks, notes)

### 4.3 Verify completeness

Check all three conditions:

1. **Same count**: original has N task blocks, new has N task blocks. If counts differ,
   report the difference and STOP.

2. **Every task present**: for each original task block (normalized), find exactly one
   matching block in the new file. Use content comparison — the blocks should be
   identical after stripping IDs. If any original task has no match, report it as
   "MISSING" with the original task description. If any new task has no match in the
   original, report it as "EXTRA" with the new task description.

3. **Content unchanged**: for each matched pair, compare the full normalized content.
   The only allowed differences are:
   - Task ID numbers in the `- [ ]` header line (expected — renumbering)
   - Cross-reference IDs substituted in text (expected — old→new mapping)
   Any other difference (added/removed text, altered code blocks, missing sub-items)
   is a failure. Report the specific diff.

### 4.4 Verify cross-references

For each cross-reference that was substituted (old ID → new ID):

1. Find the reference text in the new file
2. Confirm the new ID resolves to the correct task — the task that originally had
   the old ID, now with its new ID

If any cross-reference points to the wrong task, report it as a broken reference.

### 4.5 Run diff

Use `diff` to compare original and new files directly:

```bash
diff openspec/changes/<name>/tasks.md openspec/changes/<name>/tasks.md.tmp
```

Review the diff output. Allowed changes:
- Reordering of task blocks (lines moved)
- Task ID changes in `- [ ] X.Y` lines
- Cross-reference ID substitutions in text
- Addition of the ID mapping header table
- Layer heading additions (`## Layer N: Name`)
- Separator lines (`---`)

Anything else is unexpected and must be reported.

### 4.6 Present results

Use the `question` tool to present the verification results:

```
## Verification — <change-name>

| Check | Result |
|-------|--------|
| Task count | N original = N new ✓ |
| All tasks present | N/N matched ✓ |
| Content unchanged | No unexpected diffs ✓ |
| Cross-references | Y/Y correct ✓ |
```

Options: **Confirm — replace tasks.md** / **Show diff** / **Abort — keep original**

STOP and wait for user response.

### 4.7 Finalize

**User confirms**: Replace original:
```bash
mv openspec/changes/<name>/tasks.md.tmp openspec/changes/<name>/tasks.md
```

Report:
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

**User requests diff**: Show the full diff output, then re-ask the question.

**User aborts**: Remove temp file:
```bash
rm openspec/changes/<name>/tasks.md.tmp
```
Report: "Aborted. Original tasks.md unchanged."

### STATE 4 OUTPUT

- Original and new files compared
- Every task accounted for with unaltered content
- Cross-references verified
- User confirmed or aborted
- If confirmed: tasks.md replaced, temp file removed
- If aborted: temp file removed, original untouched

**GATE**: User has made a final decision. Original replaced or preserved.

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

- Only modify `tasks.md` (via temp file verified in STATE 4) and create `dependency-tree.md` — never source code
- Preserve ALL original task content — only change IDs and cross-references
- Every task ID from the original must appear in the rewritten file exactly once
- Original `tasks.md` is NEVER overwritten until STATE 4 verification passes and user confirms
- Layer classification follows the rules in `references/layer-classification.md` —
  when uncertain, ask the user rather than guessing
- Ambiguity resolution always defers to the user — never pick silently
- Single agent workflow — no subagent dispatch needed
