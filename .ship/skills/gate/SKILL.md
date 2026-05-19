---
name: gate
stable-id: gate
description: Review a completed job branch against acceptance criteria. Returns pass/fail with evidence. Run as subagent or inline.
tags: [gate, review, quality]
authors: [ship]
---

# Gate Review

Review a completed job branch against its acceptance criteria. Return a clear pass/fail verdict with evidence.

## Protocol

```
1. Read the job spec (acceptance criteria + file scope)
2. Read handoff.md from the worktree
3. For each acceptance criterion:
   - Run the check or inspect the output
   - Do not accept the agent's word alone
4. Verify commits are scoped to file_scope only
5. Run automated checks (below)
6. All pass → PASS with evidence
   Any fail → FAIL with specific evidence
```

## Verdict Format

### PASS

```
GATE PASS: [job title]
Branch: job/[slug]

Evidence:
  ✓ [criterion 1] — [test name / commit hash / observable behavior]
  ✓ [criterion 2] — [evidence]
  ...

Automated checks: all passed
Ready to merge.
```

### FAIL

```
GATE FAIL: [job title]
Branch: job/[slug]

  ✓ [criterion 1] — passed
  ✗ [criterion 2] — expected [X], got [Y]
    Evidence: [command output or diff]
  ✗ [criterion 3] — [specific reason]

What needs to change: [actionable, specific]
```

## Automated Checks

Run all four. Any failure blocks the gate.

### 1. Build verification

```bash
just build
```

Must exit 0.

### 2. Test verification

```bash
cargo test -p <relevant-crate>
```

Must exit 0.

### 3. Scope violation check

```bash
git diff main...HEAD --name-only
```

Every changed file must be within the declared `file_scope`. Files outside scope → FAIL.

### 4. Silent fallback scan (Rust only)

Scan non-test code for silent error suppression:

```bash
grep -rn --include="*.rs" \
  -e 'unwrap_or_else(|_|' \
  -e 'unwrap_or_default()' \
  -e 'unwrap_or(' \
  -e '\.unwrap()' \
  src/ \
  | grep -v '#\[cfg(test)\]' \
  | grep -v 'mod tests'
```

Each match needs a justifying comment (e.g., `// infallible: static data`). Unjustified matches → FAIL.

### 5. Abandoned problem detection

```bash
git diff main...HEAD -- '*.rs' '*.ts' '*.tsx' \
  | grep '^+' \
  | grep -iE 'TODO.*error|TODO.*broken|FIXME.*error|FIXME.*broken|not my fault|leaving.?alone'
```

A noticed problem not filed as a follow-on job is a FAIL. The fix: file the job, then re-gate.

### 6. MCP config verification

```bash
cat <worktree>/.mcp.json
```

Must exist and contain `ship mcp serve`. An agent without MCP cannot have logged progress properly.

### 7. Config directory cleanliness

```bash
find .ship/ -name "*.db" -o -name "*.db-wal" -o -name "*.db-shm" \
  -o -name "*.pid" -o -name "*.port" -o -name "*.sock" \
  | grep -v node_modules
```

`.ship/` is configuration only. Any runtime artifacts (databases, pid files, sockets) are a FAIL. These belong in `~/.ship/` (global) or `.artifacts/` (ephemeral).

### 8. UI evidence check (web-qa artifact gate)

When the gated branch touches any UI surface, require a fresh
`web-qa` artifact for it. Without one, the gate fails — taking a
screenshot in plan-mode is not QA.

```bash
# Match touched files to surface roots
git diff main...HEAD --name-only | awk '
  /^apps\/shell\//                              { print "chrome/"; next }
  /^packages\/ui-components\/src\/sui-[^/]+\//  { sub("^packages/components/ui-components/src/", ""); sub("/.*$", ""); print "components/" $0; next }
  /^views\/[^/]+\//                             { sub("^views/", ""); sub("/.*$", ""); print "views/" $0; next }
  /^crates\/sui\//                              { print "runtime/"; next }
' | sort -u > /tmp/gate-surfaces.txt

# For each surface, require fresh report.md
FAIL=0
while read surface; do
  [ -z "$surface" ] && continue
  latest="$(ls -1dt .artifacts/web-qa/${surface}*/2*/report.md 2>/dev/null | head -1)"
  if [ -z "$latest" ]; then
    echo "MISSING web-qa artifact for surface: $surface"
    FAIL=1
    continue
  fi
  # report.md frontmatter must be DONE or DONE_WITH_CONCERNS, console_errors=0, network_failures=0
  status="$(awk '/^status:/ {print $2; exit}' "$latest")"
  cerr="$(awk '/^console_errors:/ {print $2; exit}' "$latest")"
  nfail="$(awk '/^network_failures:/ {print $2; exit}' "$latest")"
  case "$status" in
    DONE|DONE_WITH_CONCERNS) : ;;
    *) echo "BAD status in $latest: $status"; FAIL=1 ;;
  esac
  [ "${cerr:-0}" -eq 0 ] || { echo "Unwaived console_errors=$cerr in $latest"; FAIL=1; }
  [ "${nfail:-0}" -eq 0 ] || { echo "Unwaived network_failures=$nfail in $latest"; FAIL=1; }
  # Report must be newer than the latest commit touching the surface
  surf_path="$(echo "$surface" | sed -E 's|^chrome/$|apps/shell/|; s|^components/(.+)$|packages/components/ui-components/src/\1/|; s|^views/(.+)$|views/\1/|; s|^runtime/$|crates/sui/|')"
  last_commit_ts="$(git log -1 --format=%ct -- "$surf_path" 2>/dev/null)"
  report_ts="$(stat -c %Y "$latest" 2>/dev/null || stat -f %m "$latest")"
  if [ -n "$last_commit_ts" ] && [ "$report_ts" -lt "$last_commit_ts" ]; then
    echo "STALE web-qa artifact for $surface (older than last commit touching $surf_path)"
    FAIL=1
  fi
done < /tmp/gate-surfaces.txt
[ "$FAIL" -eq 0 ] || exit 1
```

Surfaces map per the contract in `skills/web-qa/SKILL.md` (the "Gate
contract" section). If the touched files don't match any mapped root,
no QA artifact is required.

To fix: invoke the `web-qa` skill with the surface name, address every
finding it produces, then re-gate. There is no manual override.

## Land Step (on PASS)

Gate owns the merge. Commander never touches git.

```bash
# 1. Stage only files within file_scope
git -C <worktree-path> add <file1> <file2> ...

# 2. Commit with message derived from spec Goal
git -C <worktree-path> commit -m "<type>: <goal summary>"

# 3. Merge into target branch (default: v0.2.0)
git checkout <target-branch>
git merge --no-ff job/<slug> -m "merge: job/<slug>"

# 4. Clean up worktree and branch
git worktree remove <worktree-path>
git branch -d job/<slug>
```

Report back to commander: `LANDED: <commit-hash>`

If any step fails, report `LAND FAILED: <reason>` — do not leave partial state.
Commander does not retry the land; human decides next action.

## Signal commander on completion

After every verdict (PASS+land or FAIL), notify commander so it can act without human relay:

1. Call `mcp__ship__mesh_broadcast` with:
   ```json
   { "topic": "gate.result", "verdict": "LANDED|GATE FAIL", "slug": "<slug>", "detail": "<one line>" }
   ```
2. If the job spec includes a `commander_workspace_id` field, also call `mcp__ship__emit_studio_event` with:
   ```json
   {
     "event_type": "studio.gate.result",
     "target_workspace_id": "<commander_workspace_id>",
     "payload": { "verdict": "LANDED|GATE FAIL", "slug": "<slug>", "detail": "<one line>" }
   }
   ```
   This writes to the commander workspace's inbox and triggers the MCP resource notification that wakes the commander session.

If shipd is not running, skip step 1. Step 2 is best-effort — never block on it.
