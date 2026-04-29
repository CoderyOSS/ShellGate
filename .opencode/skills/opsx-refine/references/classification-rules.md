# Issue Classification Rules

Every issue found during review must be classified into exactly one of three tiers.

**Note**: The formats below describe the conceptual fields for each issue type.
All output must be structured as JSON per the schema defined in `SKILL.md` (STATE 1)
and `references/analysis-prompt.md`. Do NOT output free-form markdown.

## Editorial

Rudimentary issues — inconsistencies, obvious omissions, slop. Fixing these makes the plans tight and correct. Solution is clear, one-liner prescription possible.

Examples:
- Missing field or section
- Typo or naming inconsistency
- Obvious gap (missing default value, missing error case)
- Inconsistency between two artifacts where one is clearly correct
- Missing explicit mention of something implied elsewhere
- Paths, names, or terminology that don't match across artifacts

Prescription format:
```
- **File**: [exact file path relative to change directory]
- **Section**: [which heading or section to modify]
- **Action**: add|change|remove
- **Content**: [the exact text to add, or the replacement text, or what to remove]
- **Issue**: [which issue this fixes]
```

## Architectural

Issues requiring design decisions. Usually have a best option or a couple best options, but require some research or tradeoff analysis to sort out. Architectural in nature — these affect how the system works, not just what it says.

Examples:
- Choice between caching strategies, auth models, data models
- Security boundary decisions (what to gate vs. allow)
- API design choices with compatibility implications
- Performance tradeoffs requiring measurement
- Decisions about what's in-scope vs. out-of-scope for v1
- Missing threat model or security properties
- Credential lifecycle decisions (rotation, caching, injection method)
- Deployment or migration strategy choices

Output format per issue:
```
- **Issue**: [the problem]
- **Analysis**: [tradeoff analysis, 3-5 sentences]
- **Recommendation**: [chosen path with rationale]
- **Prescription**: [exact change to make, which file and section]
- **Alternatives**: [1-3 other viable approaches with brief tradeoffs]
```

## Speculative

Issues that are underspecified or unrealistic. The plans give no clear indication of the user's intent, or the plans call for things that can't be built as described. Ideally this tier is empty — most issues should fit into editorial or architectural. If something lands here, it means the proposal needs more input from the user before it can be resolved.

Examples:
- "Should we use X library?" when X is unknown and no tradeoffs are discussed
- Unclear integration requirements depending on unspecified external behavior
- Open design questions where the answer depends on user preference not stated in the plan
- Plans that call for unrealistic capabilities (e.g., "the program reads the user's mind")
- Ambiguities that can't be resolved without asking the user about their intent
- Dependencies on technology or services not yet chosen

Output format per issue:
```
- **Issue**: [the unclear problem]
- **Analysis**: [reasoned assessment from domain expertise, 3-5 sentences]
- **Clarified question**: [the actual decision the user needs to make]
- **Options**: [2-4 viable approaches with brief tradeoffs]
- **Recommendation**: [which direction to pursue, and what info is still needed]
```

## Zero Issues

If no issues found in a tier, report: "No [editorial/architectural/speculative] issues found."
