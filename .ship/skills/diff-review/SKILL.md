---
name: diff-review
audience: internal
stable-id: diff-review
description: Use when reviewing a git diff or PR. Renders an interactive diff view in Ship Studio's Session page with annotatable file changes.
tags: [review, git, diff, workflow]
authors: [ship]
---

# Diff Review

> **RETIRE-CANDIDATE (operator to confirm, defensive).** Per
> `census-sources.md` Part 1, diff review is a Studio UI surface — there's
> no code path that depends on the skill text, only the Studio view. The
> skill text's only job was to point at `.artifacts/canvas.html` /
> `.artifacts/annotations.json` / `.artifacts/review.md`, all retired.
> The Studio diff view embeds its own instructions; this skill is
> dead-letter. **Retire; if the Studio surface needs skill-text updates,
> those are the Studio view's own docs, not a `.ship/skills/` skill.**
> Banner added defensively in this job — land-hardening re-pointed the
> evidence home to the mission scratch, but the underlying skill remains
> a retire candidate.

Invoked as `/diff` or triggered when reviewing changes.

## Workflow

### 1. Get the diff

Determine what to diff:
- No args: diff unstaged changes
- Branch name: diff against that branch
- PR number: fetch PR diff via gh CLI

### 2. Render to Session canvas

Write the diff review artifacts to the mission scratch root when one is
in scope (the typical dispatched-agent case), else to the repo-local
`.artifacts/`. Hard-fail when a dispatch expects an evidence home and
`SHIP_MISSION_SCRATCH` is unset — the wrong dir makes the review
unreadable end to end.

```bash
if [[ -n "${SHIP_MISSION_SCRATCH:-}" ]]; then
    ART_DIR="$SHIP_MISSION_SCRATCH"
else
    ART_DIR="$(git rev-parse --show-toplevel)/.artifacts"
fi
mkdir -p "$ART_DIR"
```

Write `$ART_DIR/canvas.html` with an interactive diff view:

- File list at the top, clickable, scrolls to that file's diff
- Per-file sections with line numbers, green additions, red deletions
- Hunk headers as section dividers
- Stats: files changed, total insertions, total deletions
- Ship design tokens: JetBrains Mono for code, warm amber accents

Include review action buttons:
```html
<button data-ship-action="approve">Approve</button>
<button data-ship-action="request-changes">Request Changes</button>
```

### 3. Read annotations

After the reviewer annotates in Studio, read `$ART_DIR/annotations.json`:
- Click annotations on diff lines = inline review comments
- Action annotations = the review decision
- Box annotations = section-level feedback

### 4. Write review summary

Output to `$ART_DIR/review.md` with the decision, per-file comments, and referenced line numbers.
## Styling

Dark theme with light mode support. Use CSS variables:
- `--primary: #c67b2e` (amber accent)
- `--add: #2d5a27` / `--add-text: #7ec876` (green for additions)
- `--del: #5a2727` / `--del-text: #e87676` (red for deletions)
- Font: JetBrains Mono for code, DM Sans for UI
