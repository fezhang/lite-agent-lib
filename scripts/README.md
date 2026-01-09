# Spec Automation Tools

This directory contains automation tools to ensure spec files comply with `.agent/spec-rules.md`.

## Tools

### 1. `new-spec.sh` - Spec Generator

Creates new spec files that automatically comply with naming conventions and required sections.

**Usage:**
```bash
./scripts/new-spec.sh [feature|bug] "description"
```

**Examples:**
```bash
# Create a feature spec
./scripts/new-spec.sh feature "user authentication system"

# Create a bug fix spec
./scripts/new-spec.sh bug "fix memory leak in parser"
```

**What it does:**
- Generates spec ID automatically (e.g., 000001, 000002)
- Creates spec file with proper naming: `spec_YYYYMMDD_NNNNNN_<type>_--<description>.md`
- Creates implementation log file: `spec_...implementation.md`
- Pre-fills required sections (Critical Questions, What, Why, How, Verification)
- Opens files for editing

### 2. `githooks/pre-commit` - Pre-commit Validation

Git hook that validates spec files before commit.

**Installation:**
```bash
# Copy hook to git hooks directory
cp scripts/githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

**What it validates:**
- ✅ Filename matches required pattern
- ✅ Implementation log file exists
- ✅ Required sections present (Critical Questions, What, Why, How, Verification)
- ✅ Verification has subsections (Unit Tests, Integration Tests, Acceptance Criteria)

**Bypass if needed:**
```bash
git commit --no-verify
```

### 3. `.github/workflows/spec-validation.yml` - CI Validation

GitHub Actions workflow that validates specs in CI/CD pipeline.

**What it does:**
- Runs on every PR and push that modifies files in `specs/`
- Same validation as pre-commit hook
- Prevents non-compliant specs from being merged

## Compliance Layers

The automation provides defense-in-depth:

1. **Template Generator** (`new-spec.sh`) - Makes it easy to create compliant specs
2. **Pre-commit Hook** - Catches issues before commit
3. **CI Validation** - Final gate before merge
4. **Agent Instructions** (`.agent/spec-rules.md`) - Guides AI agents

## File Naming Convention

All spec files MUST follow this pattern:

```
spec_yyyymmdd_NNNNNN_<type>_--<description>.md
```

**Components:**
- `yyyymmdd`: Creation date (e.g., 20260108)
- `NNNNNN`: 6-digit unique ID (000001-999999)
- `<type>`: Either `feature` or `bug`
- `<description>`: Kebab-case brief description

**Examples:**
```
spec_20260108_000001_feature_--user-authentication.md
spec_20260108_000002_bug_--fix-memory-leak.md
```

## Required Spec Sections

Every spec MUST contain:

1. **Critical Questions** - Key questions to answer before implementation
2. **What** - Clear description of what will be implemented
3. **Why** - Justification and context
4. **How** - Technical approach and architecture
5. **Verification** - Testing requirements:
   - Unit Tests (specific requirements)
   - Integration Tests (specific scenarios)
   - Acceptance Criteria (measurable success metrics)

## Implementation Logs

Every spec MUST have a corresponding implementation log:

```
spec_yyyymmdd_NNNNNN_<type>_--<description>.implementation.md
```

**Rules:**
- ✅ ONLY append to the log file
- ✅ Add timestamped entries: `## [YYYY-MM-DD HH:MM:SS] Event Title`
- ✅ Log every significant event, decision, error, and milestone
- ❌ NEVER delete or modify previous log entries

## Workflow

### Creating a New Spec

1. Use the generator:
   ```bash
   ./scripts/new-spec.sh feature "my feature description"
   ```

2. Fill in the spec file with details

3. Implement according to the spec

4. Log progress in implementation log (append-only)

### Committing Specs

1. Stage your files:
   ```bash
   git add specs/spec_*.md
   ```

2. Attempt to commit:
   ```bash
   git commit -m "Add spec for feature X"
   ```

3. If validation fails, fix issues and try again

### Making Changes

When specs need updates during implementation:

1. Document the deviation in the spec's update log
2. Update the spec file
3. Log the change in implementation log
4. Commit with clear message about the update

## Troubleshooting

### Pre-commit hook not running

Check hook is installed and executable:
```bash
ls -la .git/hooks/pre-commit
```

Reinstall if needed:
```bash
cp scripts/githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Validation errors

Read the error message carefully:
- Check filename pattern
- Ensure implementation log exists
- Verify all required sections present
- Check Verification subsections

Refer to `.agent/spec-rules.md` for detailed requirements.

### Need to bypass temporarily

Only do this if you understand the risk:
```bash
git commit --no-verify
```

## References

- [Spec Rules](../.agent/spec-rules.md) - Complete spec implementation rules
- [Specs Directory](../specs/) - All spec files and implementation logs
