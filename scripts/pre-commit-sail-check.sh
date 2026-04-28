#!/usr/bin/env bash
# Pre-commit hook fragment: run `sail check` on every staged .set file.
#
# Install:
#   ln -sf ../../scripts/pre-commit-sail-check.sh .git/hooks/pre-commit
#
# Or wire into a husky/lefthook/lintstaged setup as the .set step.
#
# Skips if no .set files are staged. Builds sail if missing. Exits
# non-zero on any rule error.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

mapfile -t staged < <(git diff --cached --name-only --diff-filter=ACMR | grep '\.set$' || true)
if [ "${#staged[@]}" -eq 0 ]; then
  exit 0
fi

if [ ! -x target/debug/sail ]; then
  echo "Building sail binary…"
  cargo build --bin sail --quiet
fi

failed=0
for f in "${staged[@]}"; do
  if ! ./target/debug/sail check "$f" 2>&1; then
    failed=$((failed + 1))
  fi
done

if [ "$failed" -gt 0 ]; then
  echo
  echo "Pre-commit: $failed .set file(s) failed sail check."
  echo "Fix the diagnostics above, or bypass with --no-verify (not recommended)."
  exit 1
fi
