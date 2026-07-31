#!/usr/bin/env bash
# ship-dispatch:pre-commit v1
# Refuses to commit .artifacts/ paths and any .gitignore change that weakens
# the .artifacts ignore. .artifacts is ephemeral scratch: gitignored, used to
# get work done, deleted with the branch. The durable output is code. Agents do
# not commit it and do not weaken the ignore that protects it. The only
# exception is a human running `git commit --no-verify`.
set -euo pipefail

staged="$(git diff --cached --name-only)"

# 1. No .artifacts/ path may be committed (kills `git add -f`).
if printf '%s\n' "$staged" | grep -qE '^\.artifacts/'; then
    {
        echo "✗ pre-commit: refusing to commit .artifacts/ — ephemeral scratch, dies with the branch."
        printf '%s\n' "$staged" | grep -E '^\.artifacts/' | sed 's/^/    /'
        echo "  Durable output is code. Human override only: git commit --no-verify"
    } >&2
    exit 1
fi

# 2. The .gitignore protection for .artifacts may not be weakened
#    (kills "agent carves an exception for its own docs").
if printf '%s\n' "$staged" | grep -qx '.gitignore'; then
    diff="$(git diff --cached -- .gitignore)"
    if printf '%s\n' "$diff" | grep -qE '^\+.*!.*\.artifacts'; then
        echo "✗ pre-commit: refusing a .gitignore change that un-ignores .artifacts. No exceptions. (override: git commit --no-verify)" >&2
        exit 1
    fi
    if printf '%s\n' "$diff" | grep -qE '^-\s*\.artifacts/'; then
        echo "✗ pre-commit: refusing to drop the .artifacts ignore from .gitignore. (override: git commit --no-verify)" >&2
        exit 1
    fi
fi

exit 0
