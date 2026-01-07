# Specification Guidelines for Authors

This guide helps you write clear, effective specifications for features and bug fixes. For coding agent implementation rules, see [`.agent/spec-rules.md`](../.agent/spec-rules.md).

## Why We Use Specs

Specifications serve as a contract between stakeholders and implementers. They:
- Force critical thinking before coding
- Provide clear acceptance criteria
- Enable better code reviews
- Create a historical record of decisions
- Reduce miscommunication and rework

**Philosophy**: Write the spec before the code. If you can't clearly describe what you're building, you're not ready to build it.

---

## Naming Convention

Every spec file follows this pattern:

```
spec_yyyymmdd_NNNNNN_<type>_--<description>.md
```

**Components:**
- `yyyymmdd`: Creation date (e.g., 20260106)
- `NNNNNN`: Unique 6-digit ID (000001-999999)
- `<type>`: Either `feature` or `bug`
- `<description>`: Brief kebab-case description

**Examples:**
```
spec_20260106_000001_feature_--user-authentication-system.md
spec_20260107_000002_bug_--login-timeout-error.md
spec_20260108_000003_feature_--export-metrics-to-csv.md
```

**Getting the next ID**: Check existing spec files and increment from the highest number.

---

## Spec Structure

Every spec must contain these five sections:

### 1. Critical Questions

Start by listing the key questions that must be answered:
- What problem are we solving?
- Who is affected by this?
- What constraints exist?
- What are the risks?

**Tip**: If you can't answer these questions, you need more research before writing the spec.

### 2. What

Clearly describe what will be implemented or fixed:
- Specific functionality or bug being addressed
- Scope and boundaries (what's included and excluded)
- Expected outcomes and deliverables

**Be specific**: "Add OAuth2 login" not "improve authentication"

### 3. Why

Explain the justification and context:
- Business or technical rationale
- User impact
- Priority and urgency
- How it connects to project goals

**Good test**: If someone asks "Why are we doing this?", this section should answer it completely.

### 4. How

Describe the technical approach:
- High-level architecture or implementation strategy
- Key components or modules affected
- Technology choices and dependencies
- Alternative approaches considered (and why not chosen)

**Note**: This is high-level guidance, not detailed code. Leave room for implementation flexibility.

### 5. Verification

Define how success will be measured:
- **Unit Tests**: Specific test requirements
- **Integration Tests**: Scenarios that must be tested end-to-end
- **Acceptance Criteria**: Clear, measurable success metrics
- **Manual Testing Steps**: If applicable

**Important**: If no integration tests are needed, explicitly state why. The coding agent will prompt for confirmation during implementation.

---

## Writing a New Spec

1. **Research first**: Understand the problem completely
2. **Determine next ID**: Check existing specs for the highest number
3. **Choose type**: `feature` or `bug`
4. **Answer critical questions**: If you can't, keep researching
5. **Use the template** below
6. **Get review**: Have stakeholders and technical leads review before approval

---

## Updating an Existing Spec

Specs may need updates during implementation if:
- Requirements change
- New information emerges
- Implementation reveals better approaches

**When updating:**

1. Add an **Update Log** entry at the top:
   ```markdown
   ## Update Log
   - **2026-01-10**: Added load testing requirement - @username
   - **2026-01-08**: Revised API endpoint design - @username
   ```

2. **Don't delete history**: Use ~~strikethrough~~ for obsolete content
3. **Explain why**: Document the reason for each change

**Example:**
```markdown
## How

~~Use REST API~~ Updated: Use GraphQL for more flexible querying

Reason: Discovered client needs dynamic field selection
```

---

## Best Practices

### Write Specs Before Code
Don't start implementation without an approved spec. The spec is your roadmap and contract.

### Keep It Atomic
One spec per feature or bug. If a feature is large, break it into multiple specs with clear dependencies.

### Be Specific, Not Prescriptive
- ✅ "Login timeout should be configurable between 5-60 minutes"
- ❌ "Make login work better"

But don't over-specify:
- ✅ "Store user sessions securely with appropriate encryption"
- ❌ "Use AES-256-GCM with PBKDF2 (10k iterations, SHA-256)..." (too prescriptive)

### Link Related Specs
Create a Dependencies section when needed:

```markdown
## Dependencies
- Depends on: spec_20260105_000012_feature_--database-connection-pool.md
- Related to: spec_20260103_000008_bug_--connection-leak.md
```

### Include Examples
When helpful, add:
- Code snippets showing API usage
- Diagrams of architecture
- Mock-ups of UI changes
- Sample inputs and expected outputs

### Define Success Measurably
- ✅ "API response time < 200ms for 95th percentile"
- ❌ "API should be fast"

### Consider Edge Cases
Document known edge cases, error conditions, and boundary behaviors. This helps prevent bugs and guides testing.

### Use Status Indicators
Add a status badge at the top of each spec:

```markdown
**Status**: Draft | Approved | In Progress | Testing | Completed | Archived
```

**Typical lifecycle:**
```
Draft → Review → Approved → In Progress → Testing → Completed → (Archived)
```

---

## Spec Template

Copy this template when creating a new spec:

```markdown
# Spec ID: NNNNNN - <Title>

**Status**: Draft
**Type**: Feature | Bug
**Created**: YYYY-MM-DD
**Author**: @username

## Update Log
(Updates will be added here as spec evolves)

---

## Critical Questions

1. What problem are we solving?
2. Who is affected by this?
3. What are the constraints?
4. What are the risks?

## What

[Clear description of what will be implemented or fixed]

- Specific functionality:
- Scope (what's included):
- Scope (what's excluded):
- Expected outcomes:

## Why

[Justification and context]

- Business/technical rationale:
- User impact:
- Priority:
- Connection to project goals:

## How

### Approach
[High-level implementation strategy]

### Components Affected
- Component 1
- Component 2

### Dependencies
- [External libraries, APIs, or other specs]

### Alternatives Considered
- **Alternative 1**: [Description] - Not chosen because [reason]
- **Alternative 2**: [Description] - Not chosen because [reason]

## Verification

### Unit Tests
- [ ] Test case 1: [Description]
- [ ] Test case 2: [Description]

### Integration Tests
- [ ] Integration scenario 1: [Description]
- [ ] Integration scenario 2: [Description]

**If no integration tests needed**: [Explain why]

### Acceptance Criteria
- [ ] Criterion 1: [Measurable outcome]
- [ ] Criterion 2: [Measurable outcome]

### Manual Testing (if applicable)
- [ ] Step 1
- [ ] Step 2

## Additional Notes

[Any other relevant information, constraints, or context]

---

## Implementation Notes

When a coding agent implements this spec, they will create a companion file:
`spec_YYYYMMDD_NNNNNN_type_--description.implementation.md`

This log file will track the implementation progress, decisions, and any issues encountered.
For details on implementation rules, see [`.agent/spec-rules.md`](../.agent/spec-rules.md).
```

---

## Implementation Details

When a spec is implemented, a coding agent will:

1. **Create an implementation log**: `spec_*.implementation.md`
2. **Track all progress**: Timestamped entries for milestones, errors, decisions
3. **Ensure all tests pass**: Unit tests AND integration tests
4. **Prompt if unclear**: If integration tests aren't defined, the agent will ask for confirmation

For complete implementation rules and agent protocols, see **[`.agent/spec-rules.md`](../.agent/spec-rules.md)**.

---

## Tips for Great Specs

**Start with "Why"**: If you can't articulate why something matters, it probably doesn't.

**Think in outcomes, not solutions**: Focus on what needs to be achieved, not how to code it.

**Review with fresh eyes**: Have someone unfamiliar with the problem read your spec. If they're confused, rewrite.

**Update as you learn**: Specs aren't set in stone. Update them when implementation reveals better approaches.

**Archive, don't delete**: Obsolete specs are valuable historical context.

---

## Questions?

For questions about writing specs or the spec process, consult the project lead or open a discussion in the team channel.

For implementation details and agent rules, see [`.agent/spec-rules.md`](../.agent/spec-rules.md).
