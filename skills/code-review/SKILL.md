---
name: code-review
description: Use when reviewing a branch diff. File findings as claims (kind: observation / bug / annotation); the operator triages via the claim inbox. No artifact files — the new world finds, claims, clears.
stable-id: code-review
---

# Review Workflow

Structured code review for branch diffs. The output is **claims**, not
artifact files — the operator triages the inbox, the gate reads the
verdict, and nothing is committed to a `.artifacts/` review-map.md (the
artifacts concept is retired; everything goes through Helm).

## Protocol

### 1. Read the diff

```bash
# Branch diff against the integration branch (NOT main — agents land
# onto feat/next; you review the same diff the cmdr will Land).
git -C <main-repo> diff feat/next..job/<slug> --stat
git -C <main-repo> log feat/next..job/<slug> --oneline
git -C <main-repo> diff feat/next..job/<slug>
```

### 2. Classify changes

Build a change map (in your head or in scratch — NOT in a committed file):

| File | Type | Risk | Reviewer focus |
|------|------|------|---------------|
| `src/auth.ts` | feature | high | security, auth flow |
| `src/utils.ts` | refactor | low | behavior preservation |
| `migrations/001.sql` | schema | high | data safety, rollback |

### 3. Review each file

For each changed file, check in order:

**Security (file as `bug` claim):**
* SQL injection via string interpolation
* XSS via unescaped user input
* Secrets in code or config
* Auth bypass (missing middleware, unchecked permissions)
* LLM trust boundary violations (user input → system prompt)

**Correctness (file as `bug` claim):**
* Logic errors, off-by-one, null handling
* Error paths that swallow failures silently
* Race conditions in async code
* Missing cleanup (open handles, temp files)

**Tests (file as `observation` claim if missing):**
* New behavior has test coverage
* Bug fixes have regression tests
* Edge cases covered (empty, null, boundary)

**Architecture (file as `observation` claim if drift):**
* Changes follow existing patterns
* No unintended coupling introduced
* File scope appropriate (agent didn't stray)

**Style (non-blocking; usually an `annotation` claim):**
* Naming conventions
* Dead code
* Unnecessary complexity

### 4. File the findings as claims

For each finding, use the `file_claim` MCP tool (or
`ship ops call claims.file_claim`). The diff is the citation; the claim
is the message.

```bash
# A blocking security finding
ship ops call claims.file_claim --json "$(cat <<EOF
{
  "kind": "bug",
  "target": {"subsystem": "<name>", "feature": "<name>"},
  "title": "src/auth.ts:42 — user input passed directly to SQL query",
  "body": "String interpolation in db.query() is an injection vector. Replace with parameterized query. The implementer cannot ship this as-is.",
  "citation": {
    "file": "src/auth.ts",
    "line": 42,
    "commit": "<job-branch SHA>"
  }
}
EOF
)"

# A non-blocking test gap
ship ops call claims.file_claim --json "$(cat <<EOF
{
  "kind": "observation",
  "target": {"subsystem": "<name>", "feature": "<name>"},
  "title": "src/utils.ts — parseDate has no test for invalid input",
  "body": "Add a unit test for the null/empty/whitespace path; current coverage is happy-path only."
}
EOF
)"

# A style nit (won't gate)
ship ops call claims.file_claim --json "$(cat <<EOF
{
  "kind": "annotation",
  "target": {"subsystem": "<name>", "feature": "<name>"},
  "title": "src/api.ts:15 — unused import Response",
  "body": "Trivial; left for the implementer to clean up post-merge."
}
EOF
)"
```

### 5. Hand off the verdict

On the gate handoff doc (the job's tracking doc, owned by the cmdr),
record the verdict and link the claim IDs:

```markdown
## Review verdict

* `bug` claims filed: N (block Land until reconciled)
* `observation` claims filed: N (inform the implementer; do not block)
* `annotation` claims filed: N (post-merge cleanup)

Claim IDs: c-…, c-…

Gate recommendation: PASS | FAIL
* If any `bug` claim is unreconciled → FAIL
* If only `observation` / `annotation` claims → PASS with notes
```

## Notes

* The cmdr is responsible for triaging the claim inbox and deciding
  whether to Land; you file the findings, they own the decision.
* The diff is the citation. Never cite a line you haven't read in the
  current `feat/next..job/<slug>` diff.
* `gh pr review` is no longer the workflow — the agent reviews its own
  branch (not a remote PR), files claims, and the cmdr/operator triages
  via the claim inbox. If you find yourself reaching for `gh`, stop —
  the workflow changed.
