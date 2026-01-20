#!/bin/bash
# Pre-commit documentation validation hook for Claude Code
# Runs before git commit to check for undocumented code and CHANGELOG updates

set -e

# Read JSON input from Claude Code
INPUT=$(cat)

# Extract the git command being run
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // ""')

# Only process git commit commands
if [[ ! "$COMMAND" =~ ^git\ commit ]]; then
    exit 0
fi

# Get project directory
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(pwd)}"
cd "$PROJECT_DIR"

WARNINGS=""
ERRORS=""

# Check 1: Look for undocumented pub fn in Rust files (staged changes only)
STAGED_RS=$(git diff --cached --name-only --diff-filter=AM | grep '\.rs$' || true)
if [[ -n "$STAGED_RS" ]]; then
    for file in $STAGED_RS; do
        if [[ -f "$file" ]]; then
            # Find pub fn without preceding /// doc comment
            UNDOC=$(awk '/^[[:space:]]*pub fn/ { if (prev !~ /\/\/\//) print NR": "$0 } { prev=$0 }' "$file" || true)
            if [[ -n "$UNDOC" ]]; then
                WARNINGS="${WARNINGS}Undocumented pub fn in $file:\n$UNDOC\n\n"
            fi
        fi
    done
fi

# Check 2: Look for undocumented exports in TypeScript files (staged changes only)
STAGED_TS=$(git diff --cached --name-only --diff-filter=AM | grep -E '\.(ts|tsx)$' || true)
if [[ -n "$STAGED_TS" ]]; then
    for file in $STAGED_TS; do
        if [[ -f "$file" ]]; then
            # Find export function without preceding /** doc comment
            UNDOC=$(awk '/^export (async )?function/ { if (prev !~ /\*\//) print NR": "$0 } { prev=$0 }' "$file" || true)
            if [[ -n "$UNDOC" ]]; then
                WARNINGS="${WARNINGS}Undocumented export in $file:\n$UNDOC\n\n"
            fi
        fi
    done
fi

# Check 3: Check if CHANGELOG.md has been updated for non-trivial commits
STAGED_CODE=$(git diff --cached --name-only --diff-filter=AM | grep -E '\.(rs|ts|tsx|py)$' | wc -l | tr -d '[:space:]')
CHANGELOG_UPDATED=$(git diff --cached --name-only | grep -c 'CHANGELOG.md' 2>/dev/null || echo "0")
CHANGELOG_UPDATED=$(echo "$CHANGELOG_UPDATED" | tr -d '[:space:]')

if [[ "$STAGED_CODE" -gt 2 && "$CHANGELOG_UPDATED" -eq 0 ]]; then
    WARNINGS="${WARNINGS}CHANGELOG.md not updated but $STAGED_CODE code files staged.\nConsider running /documentation-automation first.\n\n"
fi

# Check 4: Detect feat/fix commits without CHANGELOG
COMMIT_MSG_FILE=".git/COMMIT_EDITMSG"
if [[ "$COMMAND" =~ -m.*\(feat\|fix\) ]] && [[ "$CHANGELOG_UPDATED" -eq 0 ]]; then
    WARNINGS="${WARNINGS}Commit message suggests feat/fix but CHANGELOG.md not updated.\n\n"
fi

# Output results
if [[ -n "$ERRORS" ]]; then
    echo -e "Documentation errors found:\n$ERRORS" >&2
    exit 2  # Block the commit
fi

if [[ -n "$WARNINGS" ]]; then
    # Return context for Claude instead of blocking
    # Escape the warnings for JSON and remove newlines
    ESCAPED_WARNINGS=$(printf '%s' "$WARNINGS" | sed 's/"/\\"/g' | tr '\n' ' ')
    printf '{\n  "hookSpecificOutput": {\n    "hookEventName": "PreToolUse",\n    "additionalContext": "Documentation check warnings: %s Consider running /documentation-automation to update docs, or proceed if these are intentional."\n  }\n}\n' "$ESCAPED_WARNINGS"
    exit 0
fi

# All good
exit 0
