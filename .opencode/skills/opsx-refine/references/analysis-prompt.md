# Analysis Prompt for Subagents

This is the prompt sent to each subagent during multi-agent analysis dispatch.
The primary agent reads this file, combines it with artifact contents, and sends
it as the `prompt` parameter to the `task` tool for each selected subagent.

---

## Preamble

Look for problems broadly. Think deeply about security holes, throughput problems,
race conditions, encoding mismatches, network conflicts and connectivity challenges,
system compatibility oversights, library compatibility oversights, and more.

Do not limit your review to surface-level inconsistencies. Consider how the proposed
change interacts with real systems: concurrent access, partial failures, version skew,
deployment ordering, and runtime conditions that differ from the happy path.

## Your Task

You are reviewing an OpenSpec change proposal. Analyze ALL artifacts provided below
for issues. Classify every issue you find into exactly one of three tiers.

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

## Classification Rules

### Editorial
Rudimentary issues — inconsistencies, obvious omissions, slop. Fixing these makes
the plans tight and correct. Solution is clear, one-liner prescription possible.

Examples: missing field, typo, naming inconsistency, obvious gap, inconsistency
where one artifact is clearly correct.

### Architectural
Issues requiring design decisions. Usually have a best option but require tradeoff
analysis. Architectural in nature — affect how the system works.

Examples: choice between caching strategies, security boundary decisions, API design
with compatibility implications, performance tradeoffs, scope decisions.

### Speculative
Underspecified or unrealistic. The plans give no clear indication of intent, or call
for things that can't be built as described. Ideally this tier is empty.

Examples: unclear integration requirements, open design questions depending on
unspecified behavior, dependencies on technology not yet chosen.

## Output Format

Return ONLY a JSON object. No other text before or after the JSON.

```json
{
  "summary": { "editorial": 0, "architectural": 0, "speculative": 0, "total": 0 },
  "issues": [
    {
      "id": "E1",
      "issue": "description of the issue",
      "category": "editorial",
      "location": "file > section",
      "reason": "why this is an issue, with specific evidence from the artifacts",
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

ID conventions:
- `E1`, `E2`, ... for editorial
- `A1`, `A2`, ... for architectural
- `Q1`, `Q2`, ... for speculative

For architectural issues, add these fields to the issue object:
```json
"recommendation": {
  "label": "a",
  "description": "recommended approach",
  "prescription": { "file": "...", "section": "...", "action": "...", "content": "..." }
},
"alternatives": [
  {
    "label": "b",
    "description": "alternative approach",
    "prescription": { "file": "...", "section": "...", "action": "...", "content": "..." }
  }
]
```

For speculative issues, add these fields:
```json
"clarified_question": "the actual decision the user needs to make",
"options": ["option 1 with brief tradeoff", "option 2 with brief tradeoff"],
"recommendation": "which direction to pursue and what info is still needed"
```

## Artifacts

The change artifacts follow. Read every one before classifying.

---
