# Issue Classification Rules (Deliberate)

Every issue found during review must be classified into exactly one of five categories.

All output must be structured as JSON per the schema defined in `SKILL.md` and the
analysis/review prompt references. Do NOT output free-form markdown.

## Categories

### completeness
Missing information that should be present for the plan to be implementable.

Examples:
- Missing error handling or edge cases
- Undefined behavior that should be specified
- Missing migration/rollback plans
- Unstated assumptions about performance, scale, dependencies
- Missing non-functional requirements
- Missing default values for configurable parameters
- Tasks that don't cover everything in design

### consistency
Contradictions or misalignments between artifacts or within a single artifact.

Examples:
- Capabilities in proposal with no corresponding spec
- Design references components not mentioned in proposal
- Tasks that don't trace back to design decisions
- Inconsistent naming or terminology across artifacts
- Contradictions between artifacts
- One artifact clearly correct where another disagrees

### oversight
Important considerations the plan should address but doesn't mention at all.

Examples:
- Common patterns the change should follow but doesn't mention
- Integration points with existing code that aren't addressed
- Testing strategy gaps
- Configuration or deployment considerations missed
- Security implications not addressed
- Backup/recovery for stateful components
- Graceful shutdown behavior
- Credential lifecycle (rotation, expiry, caching)

### compatibility
Risks that the proposed change may break or conflict with existing systems.

Examples:
- API surface changes that break existing consumers
- Data format changes requiring migration
- Dependency version conflicts
- Runtime environment incompatibilities
- Configuration format breaking changes
- Behavioral changes existing code depends on
- Library or framework upgrade side effects

### under-specified
Features or requirements described too vaguely to implement correctly.

Examples:
- Features described without concrete behavior
- Missing acceptance criteria or testable outcomes
- Ambiguous error handling ("handle errors" without specifying how)
- Performance requirements without measurable thresholds
- Integration points described without protocol details
- State management left unspecified
- Concurrency behavior undefined

## Proposal Object Format

Every proposal must include:

```json
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
```

## Zero Issues

If no issues found: return empty arrays. No placeholder text.
