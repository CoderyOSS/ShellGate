---
description: Reorder OpenSpec change tasks by dependency layer — foundation first, abstract later
---

Reorder change tasks by dependency layer — foundation first, abstract later. Classifies tasks
into 8 layers (infrastructure through polish), builds a dependency tree, rewrites tasks.md
in build order with renumbered IDs.

**Prerequisite**: Change must have `tasks.md` (run task generation first).

---

**Input**: Optionally specify a change name (e.g., `/opsx-arrange add-auth`). If omitted, auto-detected.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Auto-detect by running: `openspec list --json`
   - Select if only one active change exists
   - If ambiguous, ask the user to pick

   Always announce: "Arranging tasks for change: <name>"

2. **Verify artifacts exist**

   Check that the change has tasks to arrange:
   ```bash
   ls openspec/changes/<name>/tasks.md 2>/dev/null
   ```

   If missing, suggest: "Generate tasks first before arranging."

3. **Follow the opsx-arrange skill instructions exactly**

   The skill guides you through a finite state machine (STATE 0 through STATE 4).
   Steps 1-2 above handle STATE 0 (setup). The skill continues from STATE 1:

   - STATE 0: Setup — detect change, verify tasks.md exists
   - STATE 1: Extract & build — read all artifacts, parse tasks, classify each into
     a dependency layer (0–7), build an ASCII dependency tree with task IDs and
     descriptions at every node. Show tree to user, ask to confirm or correct.
     Loop until confirmed, then save `dependency-tree.md`
   - STATE 2: Resolve — if ordering ambiguity exists (tasks that could go in
     multiple valid orders), present 2–3 options with rationale. User picks.
     If no ambiguity, skip to STATE 3
   - STATE 3: Output — write reordered tasks to `tasks.md.tmp` with renumbered IDs
     and updated cross-references. Original `tasks.md` untouched.
   - STATE 4: Verify — compare original `tasks.md` against `tasks.md.tmp`. Verify
     same task count, every task present, content unchanged (only IDs/cross-refs
     differ), cross-references correct. User confirms to replace or aborts.

   **Dependency layers** (foundation → abstract):

   | Layer | Name | What goes here |
   |-------|------|---------------|
   | 0 | Infrastructure | Database, server, build config, containers |
   | 1 | Core Data | Models, schemas, repositories, migrations |
   | 2 | Security/Auth | Authentication, authorization, sessions |
   | 3 | Core Services | Business logic, internal APIs, middleware |
   | 4 | External Integration | Third-party APIs, webhooks |
   | 5 | Client Framework | Frontend setup, state mgmt, router |
   | 6 | UI Features | Pages, forms, feature components |
   | 7 | Polish | Error handling, loading states, tests, docs |

   **Critical rules**:
   - Single agent workflow — no subagent dispatch
   - Preserve ALL original task content — only change IDs and cross-references
   - Ambiguity resolution always defers to the user — never pick silently
   - Original `tasks.md` is never overwritten until STATE 4 verification passes and user confirms
   - New tasks written to `tasks.md.tmp` first, verified, then replaced on user confirmation

**Output files**
- `openspec/changes/<name>/tasks.md` — rewritten with new ordering
- `openspec/changes/<name>/dependency-tree.md` — ASCII dependency tree

**Guardrails**

- Only modifies `tasks.md` (via temp file + verification) and creates `dependency-tree.md` — never source code
- Every original task ID appears exactly once in the rewritten file
- Original `tasks.md` untouched until STATE 4 verification passes
- Classification uses rules in `references/layer-classification.md`
- When uncertain about classification, ask the user rather than guessing
