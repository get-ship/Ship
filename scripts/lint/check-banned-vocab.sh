#!/usr/bin/env sh
# check-banned-vocab.sh
#
# The banned-vocabulary gate: a regression fence against deleted concepts
# crawling back into the tree. Reads `scripts/lint/banned-vocab.txt` and, for
# each non-comment entry, ripgreps the term across hand-authored source,
# proto, and tests. Any hit fails the build — the whole point is that a
# stale test reintroducing a dead word breaks the gate.
#
# Ban-list format (one entry per line): <regex-or-literal><TAB><reason>
# Comments (`#`) and blank lines are ignored. Empty list ⟹ no-op (exit 0).
#
# THE RULE: a term is appended to the ban-list only after its last
# legitimate use is deleted. Never ban a term that still compiles.

set -u

# Root the script at the repo top regardless of cwd.
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT" || exit 2

BANLIST="scripts/lint/banned-vocab.txt"
SELF="scripts/lint/check-banned-vocab.sh"

if [ ! -f "$BANLIST" ]; then
    echo "check-banned-vocab: ban-list $BANLIST not found" >&2
    exit 2
fi

# Directories we scan: proto, all Rust crates, all apps, hand-authored TS
# under packages/**/src, and tests (the whole point — tests are in scope).
SCAN_PATHS="proto crates apps packages"

# Glob rules — exclusions ONLY. A single non-negated `--glob` would flip
# ripgrep into allowlist mode and silently drop everything that didn't match
# it, so we scope by passing explicit roots ($SCAN_PATHS) and subtract noise
# here: build output, deps, scratch, generated code, non-`src` package output,
# and the two gate files (so the gate never flags its own reason strings).
set -- \
    --glob '!target/**' \
    --glob '!**/target/**' \
    --glob '!node_modules/**' \
    --glob '!**/node_modules/**' \
    --glob '!.artifacts/**' \
    --glob '!**/gen/**' \
    --glob '!**/*_pb.ts' \
    --glob '!**/*.gen.*' \
    --glob '!**/OUT_DIR/**' \
    --glob '!packages/*/dist/**' \
    --glob '!packages/*/build/**' \
    --glob "!$BANLIST" \
    --glob "!$SELF"

# Count real (non-comment, non-blank) ban-list entries up front so we can
# report a clean no-op on an empty list.
terms=0
violations=0

# Read each entry: pattern is field 1 (up to the first TAB), reason is the
# rest. A line with no TAB is treated as pattern-only (empty reason).
while IFS= read -r entry || [ -n "$entry" ]; do
    # Strip a leading comment marker / skip blanks.
    case "$entry" in
        ''|'#'*) continue ;;
    esac
    # Split on the first TAB.
    pattern=$(printf '%s' "$entry" | cut -f1)
    reason=$(printf '%s' "$entry" | cut -f2-)
    # cut returns the whole field for reason when no TAB exists; normalise.
    [ "$reason" = "$pattern" ] && reason="(no reason given)"
    [ -z "$pattern" ] && continue
    terms=$((terms + 1))

    # `--` terminates option parsing so a pattern starting with `-` is safe.
    matches=$(rg -n --no-heading "$@" -- "$pattern" $SCAN_PATHS 2>/dev/null || true)
    if [ -n "$matches" ]; then
        echo "BANNED VOCAB: $pattern — $reason"
        printf '%s\n' "$matches" | sed 's/^/  /'
        # Count each offending file:line as a violation.
        hits=$(printf '%s\n' "$matches" | grep -c '')
        violations=$((violations + hits))
    fi
done < "$BANLIST"

if [ "$violations" -gt 0 ]; then
    echo "check-banned-vocab: $violations violation(s) across $terms banned term(s)"
    exit 1
fi

echo "banned-vocab: $terms terms, clean"
exit 0
