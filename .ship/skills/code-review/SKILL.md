---
name: code-review
audience: internal
stable-id: code-review
description: Use when reviewing a PR or branch diff. Security, correctness, test coverage, architecture drift. Produces structured verdicts with line references.
tags: [review, code-quality, security]
authors: [ship]
---

# Review Workflow

Structured code review for PRs and branch diffs.

## Protocol

### 1. Read the diff

```bash
# PR number
gh pr diff <number>

# Or branch diff
git diff main..HEAD
```

### 2. Classify changes

Build a change map:

```markdown
<!-- ${REVIEW_DIR}/review-map.md -->
# Review Map

| File | Type | Risk | Reviewer focus |
|------|------|------|---------------|
| src/auth.ts | feature | high | security, auth flow |
| src/utils.ts | refactor | low | behavior preservation |
| migrations/001.sql | schema | high | data safety, rollback |
```

`REVIEW_DIR` resolves from `${SHIP_MISSION_SCRATCH:-$(git rev-parse --show-toplevel)/.artifacts}`
(mission scratch first, repo-local fallback). The same evidence home as
`web-qa` / `diff-review` / `audit`.

### 3. Review each file

For each changed file, check in order:

**Security (blocks merge):**
- SQL injection via string interpolation
- XSS via unescaped user input
- Secrets in code or config
- Auth bypass (missing middleware, unchecked permissions)
- LLM trust boundary violations (user input → system prompt)

**Correctness (blocks merge):**
- Logic errors, off-by-one, null handling
- Error paths that swallow failures silently
- Race conditions in async code
- Missing cleanup (open handles, temp files)

**Tests:**
- New behavior has test coverage
- Bug fixes have regression tests
- Edge cases covered (empty, null, boundary)

**Architecture:**
- Changes follow existing patterns
- No unintended coupling introduced
- File scope appropriate (agent didn't stray)

**Style (non-blocking):**
- Naming conventions
- Dead code
- Unnecessary complexity

### 4. Write the review
<!-- ${REVIEW_DIR}/review.md -->
# Review: <PR title or branch>

## Verdict: APPROVE | REQUEST_CHANGES | COMMENT

## Blockers
- **[security]** `src/auth.ts:42` — user input passed directly to SQL query
  ```diff
  - db.query(`SELECT * FROM users WHERE id = ${id}`)
  + db.query(`SELECT * FROM users WHERE id = ?`, [id])
  ```

## Suggestions
- **[test]** `src/utils.ts` — `parseDate` has no test for invalid input
- **[style]** `src/api.ts:15` — unused import `Response`

## Approved
- `migrations/001.sql` — clean schema change, rollback safe
- `src/components/` — UI changes match spec
```

### 5. Post or surface

{% if post_to_github %}
Post to GitHub:
```bash
gh pr review <number> --request-changes --body-file "${REVIEW_DIR}/review.md"
# or
gh pr review <number> --approve --body-file "${REVIEW_DIR}/review.md"
```
