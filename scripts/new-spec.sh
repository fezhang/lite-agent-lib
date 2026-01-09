#!/usr/bin/env bash
# new-spec.sh - Create a new spec file that complies with .agent/spec-rules.md
#
# Usage: ./scripts/new-spec.sh [feature|bug] "description"
#
# Example: ./scripts/new-spec.sh feature "user authentication system"

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if type argument is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Missing spec type${NC}"
    echo "Usage: $0 [feature|bug] \"description\""
    echo ""
    echo "Example: $0 feature \"user authentication system\""
    exit 1
fi

SPEC_TYPE="$1"

# Validate spec type
if [[ "$SPEC_TYPE" != "feature" && "$SPEC_TYPE" != "bug" ]]; then
    echo -e "${RED}Error: Spec type must be 'feature' or 'bug'${NC}"
    exit 1
fi

# Check if description argument is provided
if [ -z "$2" ]; then
    echo -e "${RED}Error: Missing description${NC}"
    echo "Usage: $0 [feature|bug] \"description\""
    exit 1
fi

DESCRIPTION="$2"

# Convert description to kebab-case
KEBAB_CASE=$(echo "$DESCRIPTION" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd '[:alnum:]-')

# Get current date in YYYYMMDD format
DATE=$(date +%Y%m%d)

# Find the next available spec ID
SPEC_DIR="specs"
mkdir -p "$SPEC_DIR"

# Find the highest existing spec ID
LAST_ID=$(ls -1 "$SPEC_DIR"/spec_*.md 2>/dev/null | grep -E 'spec_[0-9]{8}_[0-9]{6}_' | sed 's/.*spec_[0-9]\{8\}_\([0-9]\{6\}\)_.*/\1/' | sort -n | tail -1)

# Increment ID or start at 000001
if [ -z "$LAST_ID" ]; then
    NEXT_ID="000001"
else
    NEXT_ID=$(printf "%06d" $((10#$LAST_ID + 1)))
fi

# Construct filename
SPEC_FILE="$SPEC_DIR/spec_${DATE}_${NEXT_ID}_${SPEC_TYPE}_--${KEBAB_CASE}.md"
IMPL_FILE="$SPEC_DIR/spec_${DATE}_${NEXT_ID}_${SPEC_TYPE}_--${KEBAB_CASE}.implementation.md"

# Check if file already exists
if [ -f "$SPEC_FILE" ]; then
    echo -e "${RED}Error: Spec file already exists: $SPEC_FILE${NC}"
    exit 1
fi

# Create spec file from template
cat > "$SPEC_FILE" << EOF
# Spec: ${DESCRIPTION}

## Critical Questions

**Q1: [Question 1 about this spec]**
A: [Answer to question 1]

**Q2: [Question 2 about this spec]**
A: [Answer to question 2]

**Q3: [Question 3 about this spec]**
A: [Answer to question 3]

## What

[Clear description of what will be implemented]

## Why

[Justification and context for this change]

## How

[Technical approach and architecture details]

## Verification

### Unit Tests

[Describe specific unit tests required]

### Integration Tests

[Describe specific integration test scenarios]

### Acceptance Criteria

1. ✅ [Measurable criterion 1]
2. ✅ [Measurable criterion 2]
3. ✅ [Measurable criterion 3]
EOF

# Create implementation log file
cat > "$IMPL_FILE" << EOF
# Implementation Log: ${DESCRIPTION}

## [$(date '+%Y-%m-%d %H:%M:%S')] Implementation Started

**Spec:** spec_${DATE}_${NEXT_ID}_${SPEC_TYPE}_--${KEBAB_CASE}.md

**Agent:** [Agent identifier]

**Initial Assessment:**
- Task: [Brief description of what needs to be done]
- Current state: [Current state of the codebase]
- Required actions: [List of required actions]

## [$(date '+%Y-%m-%d %H:%M:%S')] Next Steps

[Document your progress here with timestamped entries]

EOF

echo -e "${GREEN}✓ Spec file created: $SPEC_FILE${NC}"
echo -e "${GREEN}✓ Implementation log created: $IMPL_FILE${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Edit the spec file to add details"
echo "2. Follow the implementation workflow in .agent/spec-rules.md"
echo "3. Log all progress in the implementation log file"
echo "4. Only append to the implementation log - never modify past entries"
