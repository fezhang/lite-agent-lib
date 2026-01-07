# Agent Rules: Specification Implementation

**CRITICAL**: These are mandatory rules for all coding agents working with specifications in this project. You MUST read and follow these rules before implementing any spec.

## Automatic Behavior Required

When you encounter or are asked to implement a spec file, you MUST automatically:

1. Read this file (`.agent/spec-rules.md`) first
2. Read the spec file completely
3. Create an implementation log file
4. Follow all rules below without deviation

---

## Rule 1: Spec File Naming Convention

All spec files MUST follow this exact pattern:

```
spec_yyyymmdd_NNNNNN_<type>_--<description>.md
```

**Components:**
- `yyyymmdd`: Creation date (e.g., 20260106)
- `NNNNNN`: 6-digit unique ID (000001-999999)
- `<type>`: Either `feature` or `bug`
- `<description>`: Kebab-case brief description

**Example:**
```
spec_20260106_000001_feature_--user-authentication-system.md
```

---

## Rule 2: Implementation Log (MANDATORY)

### You MUST Create Implementation Log

At the start of ANY spec implementation, you MUST create:

```
spec_yyyymmdd_NNNNNN_<type>_--<description>.implementation.md
```

This file name mirrors the spec file with `.implementation` before `.md`.

### Append-Only Rules (CRITICAL)

**YOU MUST:**
- ✅ ONLY append to the implementation log file
- ✅ Add timestamped entries: `## [YYYY-MM-DD HH:MM:SS] Event Title`
- ✅ Log every significant event, decision, error, and milestone
- ✅ Maintain chronological order

**YOU MUST NOT:**
- ❌ NEVER delete any content from the log
- ❌ NEVER modify previous log entries
- ❌ NEVER rewrite or reorganize the log

### What You MUST Log

Log these events as they occur:

1. **Implementation Start**
   ```markdown
   ## [YYYY-MM-DD HH:MM:SS] Implementation Started
   - Spec: spec_20260106_000001_feature_--description.md
   - Agent: [your identifier]
   - Initial assessment: [your analysis]
   ```

2. **Files Created/Modified**
   ```markdown
   ## [YYYY-MM-DD HH:MM:SS] Files Modified
   - Created: src/auth/login.ts
   - Modified: src/index.ts
   - Purpose: [why these changes]
   ```

3. **Errors Encountered**
   ```markdown
   ## [YYYY-MM-DD HH:MM:SS] Error Encountered
   - Error type: TypeError in authentication module
   - Error message: ```
     [full error stack trace]
     ```
   - Resolution attempted: [what you tried]
   - Outcome: [success/failure/workaround]
   ```

4. **Test Execution**
   ```markdown
   ## [YYYY-MM-DD HH:MM:SS] Tests Executed
   - Unit tests written: 5 tests in auth.test.ts
   - Results: 5 passed, 0 failed
   - Integration tests: [status]
   ```

5. **Key Decisions**
   ```markdown
   ## [YYYY-MM-DD HH:MM:SS] Decision Made
   - Context: [what decision was needed]
   - Decision: [what you chose]
   - Rationale: [why you chose it]
   - Alternatives considered: [other options]
   ```

6. **Blockers**
   ```markdown
   ## [YYYY-MM-DD HH:MM:SS] Blocker Encountered
   - Issue: [description]
   - Impact: [what is blocked]
   - Resolution: [how resolved or awaiting input]
   ```

7. **Implementation Complete**
   ```markdown
   ## [YYYY-MM-DD HH:MM:SS] Implementation Completed
   - Status: [Success/Partial/Failed]
   - Total duration: [time taken]
   - Unit tests: [count] passing
   - Integration tests: [count] passing
   - Summary: [what was accomplished]
   ```

---

## Rule 3: Completion Criteria

A spec is ONLY complete when ALL of these are satisfied:

1. ✅ All code changes implemented
2. ✅ **All unit tests written AND passing**
3. ✅ **All integration tests written AND passing**
4. ✅ Implementation log complete with final status
5. ✅ All acceptance criteria from spec met

### Special Case: No Integration Tests

**IF** a spec does not require integration tests:

1. **STOP immediately**
2. **DO NOT proceed** with marking complete
3. **PROMPT the user** with:
   ```
   This spec appears to have no integration tests defined.

   Reason: [explain why you think no integration tests are needed]

   Options:
   1. Confirm no integration tests needed (provide justification)
   2. Define integration test requirements

   Please confirm before I mark this spec as complete.
   ```
4. **WAIT** for user confirmation
5. **DOCUMENT** the decision in both spec and implementation log

**YOU MUST NOT:**
- ❌ Assume integration tests are not needed
- ❌ Skip this prompt
- ❌ Mark spec complete without integration tests unless explicitly approved

---

## Rule 4: Spec File Structure

Every spec MUST contain these sections:

1. **Critical Questions** - Key questions to answer before implementation
2. **What** - Clear description of what will be implemented
3. **Why** - Justification and context
4. **How** - Technical approach and architecture
5. **Verification** - Testing requirements:
   - Unit Tests (specific requirements)
   - Integration Tests (specific scenarios)
   - Acceptance Criteria (measurable success metrics)

**Before implementing**, verify ALL sections are present and complete.

---

## Rule 5: Error Handling Protocol

When you encounter ANY error:

1. **Log immediately** to implementation log with full details
2. **Include full error message** and stack trace
3. **Document resolution attempt** before trying again
4. **Log outcome** of each attempt
5. **DO NOT** hide or skip logging errors
6. **If blocked**, log blocker and prompt user

---

## Rule 6: Implementation Workflow

Follow this exact workflow:

```
1. Read .agent/spec-rules.md (this file) ← YOU ARE HERE
2. Read specs/README.md for context
3. Read the target spec file completely
4. Create implementation log file
5. Log implementation start
6. Implement according to spec
7. Log all significant events as they happen
8. Write unit tests
9. Log unit test results
10. Write integration tests (or prompt if none needed)
11. Log integration test results
12. Verify all acceptance criteria
13. Log implementation completion
14. Report to user
```

**DO NOT skip any step.**

---

## Rule 7: Transparency and Audit Trail

**Purpose of Implementation Logs:**
- Provides permanent audit trail on disk
- Enables debugging when errors occur
- Tracks decision-making process
- Documents actual implementation journey
- Serves as learning resource for future work

**Your Responsibility:**
- Be thorough in logging
- Be honest about errors and blockers
- Be clear about decisions and rationale
- Maintain professional, factual tone

---

## Rule 8: File Operations

**When working with specs:**

- **READ** spec file before any implementation
- **CREATE** implementation log at start
- **APPEND** to implementation log throughout
- **NEVER DELETE** implementation log entries
- **NEVER MODIFY** past implementation log entries
- **UPDATE** spec file only if deviations occur (document in update log)

---

## Quick Reference Checklist

Before starting ANY spec implementation, verify:

- [ ] I have read `.agent/spec-rules.md`
- [ ] I have read the complete spec file
- [ ] I have created the implementation log file
- [ ] I understand all requirements and acceptance criteria
- [ ] I know what tests are required

During implementation, I will:

- [ ] Log all significant events with timestamps
- [ ] Log all errors with full details
- [ ] Log all key decisions and rationale
- [ ] ONLY append to implementation log, never delete/modify
- [ ] Write required unit tests
- [ ] Write required integration tests OR prompt for confirmation
- [ ] Track all files created/modified

Before marking complete, I verify:

- [ ] All code implemented per spec
- [ ] All unit tests written and passing
- [ ] All integration tests written and passing (or explicitly approved as not needed)
- [ ] Implementation log contains complete timeline
- [ ] Final status logged

---

## Non-Negotiable Rules Summary

1. **ALWAYS** create implementation log file
2. **NEVER** delete or modify log entries (append-only)
3. **ALWAYS** log errors with full details
4. **ALWAYS** prompt if no integration tests defined
5. **NEVER** mark complete without all tests passing
6. **ALWAYS** include timestamps in log entries
7. **ALWAYS** follow the exact naming conventions

---

## Questions or Issues?

If you encounter ambiguity in a spec or these rules:

1. **LOG** the issue in implementation log
2. **PROMPT** the user for clarification
3. **WAIT** for response before proceeding
4. **DOCUMENT** the decision once clarified

**DO NOT:**
- Make assumptions about unclear requirements
- Skip steps because they seem unnecessary
- Deviate from these rules without explicit user approval

---

**End of Agent Rules**

Remember: These rules exist to ensure quality, traceability, and reliability. Following them strictly benefits both you and the project.
