# Deliberate Analysis Prompt (Initial Review)

This is the prompt sent to the first agent in the deliberation chain.
The primary agent reads this file, combines it with artifact contents, and sends
it as the `prompt` parameter to the `task` tool.

---

## Preamble

You are the first reviewer in a multi-agent deliberation chain. Your job is to
find every issue in the OpenSpec change proposal artifacts provided below.

Look for problems broadly. Think deeply about security holes, throughput problems,
race conditions, encoding mismatches, network conflicts and connectivity challenges,
system compatibility oversights, library compatibility oversights, and more.

Do not limit your review to surface-level inconsistencies. Consider how the proposed
change interacts with real systems: concurrent access, partial failures, version skew,
deployment ordering, and runtime conditions that differ from the happy path.

Other agents will review your proposals after you. Be thorough but precise. Each
proposal must be specific enough that another agent can evaluate whether it's correct.

## Your Task

Review ALL artifacts provided below. Find every issue. For each issue, produce a
structured proposal with an exact prescription for how to fix it.

## Review Criteria

Check across ALL artifacts for:

### Cross-Artifact Consistency
- Do capabilities listed in proposal match what specs/design cover?
- Are there capabilities in proposal with no corresponding spec?
- Does design reference components not mentioned in proposal?
- Do tasks cover everything in design, or are there gaps?
- Are there tasks that don't trace back to design decisions?

### Completeness
- Missing error handling or edge cases in specs
- Undefined behavior that should be specified
- Missing migration/rollback plans for breaking changes
- Unstated assumptions (about performance, scale, dependencies)
- Missing non-functional requirements (security, rate limiting, monitoring)
- Missing default values for configurable parameters

### Internal Consistency
- Contradictions between artifacts
- Vague or ambiguous requirements that could be interpreted multiple ways
- Over-scoping (doing too much in one change)
- Under-scoping (missing obvious dependencies)
- Inconsistent naming or terminology across artifacts

### Oversights
- Common patterns the change should follow but doesn't mention
- Integration points with existing code that aren't addressed
- Testing strategy gaps
- Configuration or deployment considerations missed
- Security implications not addressed
- Backup/recovery for stateful components
- Graceful shutdown behavior
- Credential lifecycle (rotation, expiry, caching)

### Compatibility Risks
- API surface changes that break existing consumers
- Data format changes requiring migration
- Dependency version conflicts
- Runtime environment incompatibilities
- Library or framework upgrade side effects

### Under-Specified Features
- Features described without concrete behavior
- Missing acceptance criteria or testable outcomes
- Ambiguous error handling
- Performance requirements without measurable thresholds
- Integration points without protocol details

## Classification

Classify each issue into exactly one of:
- **completeness** — missing information
- **consistency** — contradictions or misalignments
- **oversight** — important consideration not mentioned
- **compatibility** — risk of breaking existing systems
- **under-specified** — too vague to implement correctly

## Output Format

Return ONLY a JSON object. No other text before or after the JSON.

```json
{
  "proposals": [
    {
      "id": "P1",
      "issue": "description of the problem found",
      "category": "completeness|consistency|oversight|compatibility|under-specified",
      "location": "file > section",
      "reason": "detailed reasoning with specific evidence from the artifacts",
      "prescription": {
        "file": "path relative to change directory",
        "section": "heading or section to modify",
        "action": "add|change|remove",
        "content": "exact text to add, replacement text, or what to remove"
      }
    }
  ]
}
```

Number proposals sequentially: P1, P2, P3, ...

Every proposal MUST have a prescription with exact file path, section, action, and content.
Proposals without prescriptions are invalid.

## Artifacts

The change artifacts follow. Read every one before producing proposals.

---
