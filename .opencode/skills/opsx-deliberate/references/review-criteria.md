# Review Criteria for OpenSpec Change Proposals (Deliberate)

Analyze across ALL artifacts (proposal, design, tasks, specs) for the following categories.

## Cross-Artifact Consistency

- Do capabilities listed in proposal match what specs/design cover?
- Are there capabilities in proposal with no corresponding spec?
- Does design reference components not mentioned in proposal?
- Do tasks cover everything in design, or are there gaps?
- Are there tasks that don't trace back to design decisions?

## Completeness

- Missing error handling or edge cases in specs
- Undefined behavior that should be specified
- Missing migration/rollback plans for breaking changes
- Unstated assumptions (about performance, scale, dependencies)
- Missing non-functional requirements (security, rate limiting, monitoring)
- Missing default values for configurable parameters

## Internal Consistency

- Contradictions between artifacts
- Vague or ambiguous requirements that could be interpreted multiple ways
- Over-scoping (doing too much in one change)
- Under-scoping (missing obvious dependencies)
- Inconsistent naming or terminology across artifacts

## Oversights

- Common patterns the change should follow but doesn't mention
- Integration points with existing code that aren't addressed
- Testing strategy gaps
- Configuration or deployment considerations missed
- Security implications not addressed
- Backup/recovery for stateful components
- Graceful shutdown behavior
- Credential lifecycle (rotation, expiry, caching)

## Compatibility Risks

- API surface changes that break existing consumers
- Data format changes requiring migration
- Dependency version conflicts
- Runtime environment incompatibilities
- Configuration format breaking changes
- Behavioral changes that existing code depends on
- Library or framework upgrade side effects

## Under-Specified Features

- Features described in vague terms without concrete behavior
- Missing acceptance criteria or testable outcomes
- Ambiguous error handling ("handle errors" without specifying how)
- Performance requirements stated without measurable thresholds
- Integration points described without protocol details
- State management left unspecified
- Concurrency behavior undefined
