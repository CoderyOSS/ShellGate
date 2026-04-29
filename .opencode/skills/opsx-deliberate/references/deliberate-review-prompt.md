# Deliberate Review Prompt (Consensus Cycle)

This is the prompt sent to each subsequent agent in the deliberation chain.
The primary agent reads this file, combines it with artifact contents and current
proposals, and sends it as the `prompt` parameter to the `task` tool.

---

## Preamble

You are a reviewer in a multi-agent deliberation chain. Another agent has reviewed
the OpenSpec change proposal and produced a set of proposals. Your job is to:

1. Review each existing proposal — agree or disagree
2. If you disagree, provide a counter-proposal with reasoning
3. If you find issues the previous reviewer missed, add new proposals

Be rigorous. If a proposal is correct, agree with it. If it's wrong or incomplete,
disagree with specific reasoning and a better alternative. If something was missed,
add it.

## Artifacts

The change artifacts are provided below. Read them all before reviewing proposals.

---

## Current Proposals

The proposals from previous deliberation rounds follow. Review each one carefully.

For each proposal, you must provide a verdict:

- **agree** — the proposal is correct and the prescription is appropriate
- **disagree** — the proposal is wrong, the prescription is inappropriate, or
  there's a better approach. You MUST provide a counter-proposal.

## Adding New Proposals

If you find issues that none of the existing proposals cover, add them as new
proposals. Use the same format, with IDs starting after the last existing ID.

## Review Criteria

When evaluating proposals, consider:

- Is the issue real? Does it actually exist in the artifacts?
- Is the category correct? Should it be reclassified?
- Is the prescription precise enough to apply without ambiguity?
- Does the prescription actually fix the issue?
- Could the prescription introduce new problems?
- Are there better approaches not considered?

When finding new issues, use the same criteria as the initial review:

### Cross-Artifact Consistency
- Capabilities in proposal with no corresponding spec
- Design references not in proposal
- Tasks that don't cover design

### Completeness
- Missing error handling, edge cases, migration plans
- Unstated assumptions, missing NFRs

### Internal Consistency
- Contradictions, vague requirements, scope issues

### Oversights
- Missing patterns, integration points, testing, security

### Compatibility
- Breaking changes, migration needs, dependency conflicts

### Under-Specified
- Vague features, missing criteria, undefined behavior

## Classification

Classify any new issues into: completeness, consistency, oversight, compatibility, under-specified.

## Output Format

Return ONLY a JSON object. No other text before or after the JSON.

```json
{
  "reviews": [
    {
      "proposal_id": "P1",
      "verdict": "agree",
      "reason": "brief note (optional for agreements)"
    },
    {
      "proposal_id": "P2",
      "verdict": "disagree",
      "reason": "why this proposal is wrong or incomplete",
      "counter_proposal": {
        "id": "P2b",
        "issue": "corrected or reframed description",
        "category": "completeness|consistency|oversight|compatibility|under-specified",
        "location": "file > section",
        "reason": "why this approach is better",
        "prescription": {
          "file": "path relative to change directory",
          "section": "heading or section to modify",
          "action": "add|change|remove",
          "content": "exact text"
        }
      }
    }
  ],
  "new_proposals": [
    {
      "id": "P7",
      "issue": "description of newly discovered issue",
      "category": "...",
      "location": "file > section",
      "reason": "why this is an issue with evidence",
      "prescription": {
        "file": "...",
        "section": "...",
        "action": "...",
        "content": "..."
      }
    }
  ]
}
```

Rules:
- You MUST review every existing proposal. Do not skip any.
- Every disagreement MUST include a counter_proposal.
- counter_proposal IDs should append a letter: P2 → P2a, P2b, etc.
- New proposal IDs continue the sequence from the highest existing ID.
- If you agree with all proposals and have no new ones, return empty arrays.

---
