---
name: web-qa
stable-id: web-qa
description: Systematic QA for a UI surface — every interactive element pressed, multi-theme, console + network captured, evidence written to .artifacts/web-qa/. Use before claiming any UI work "done". Gate-required when touching apps/shell, packages/ui-components, views, or crates/sui.
tags: [qa, testing, browser, playwright, gate-required]
authors: [ship]
license: MIT
artifacts: [image, markdown, jsonl]
allowed-tools:
  - "Bash(playwright-cli*)"
  - "Bash(npx playwright*)"
  - "Bash(bunx playwright*)"
  - "Bash(curl*)"
  - Read
  - Write
---

# Web QA — gated UI evidence

You are not done with UI work until this skill has produced a fresh artifact for
the surface you touched. Taking a screenshot and saying "looks fine" is not QA.
Real QA exercises every declared interactive element, in every declared theme,
captures the console and network for the whole run, and verifies that elements
both exist in the DOM **and** render with non-zero box, non-hidden visibility,
and >0 opacity.

If you skip this skill, the gate will reject your branch.

## Setup

```bash
REPO="$(git rev-parse --show-toplevel)"
QA_RUN="$(date -u +%Y-%m-%dT%H-%M)"
QA_SURFACE="${1:?surface name required, e.g. dev-panel/edit-mode/components-tab}"
QA_DIR="$REPO/.artifacts/web-qa/$QA_SURFACE/$QA_RUN"
mkdir -p "$QA_DIR/screenshots"

# Resolve playwright-cli — see references/runtime.md for fallback walk
if command -v playwright-cli &>/dev/null; then PW="playwright-cli"
elif command -v bunx &>/dev/null; then PW="bunx playwright-cli"
else PW="npx playwright-cli"; fi
$PW --version >/dev/null || { echo "playwright-cli unavailable" >&2; exit 1; }
```

Use `$PW` for every browser command and `$QA_DIR` for every path. Never use
relative paths — playwright-cli resolves from its own working directory.

## Required upfront declaration

Before touching the browser, write `$QA_DIR/plan.md` declaring:

1. **Surface name** — `dev-panel/edit-mode/components-tab`, `views/agents/list`, etc. Use slashes for hierarchy.
2. **Inventory** — every interactive element on the surface (button, input, tab, link, drag handle, keyboard shortcut). One per row. Include declared aria-labels.
3. **States** — every state the surface can be in (empty, loading, populated, error, hover, focused, disabled, overflow). One per row.
4. **Theme matrix** — at minimum `ship-light`, `ship-dark`, `ship-console`. Add any project theme that ships.
5. **Viewport matrix** — at minimum `1280x720` (desktop) and `375x812` (mobile-narrow). Add others when the surface is responsive-specific.

If you cannot declare these without invention, you don't understand the surface
well enough to QA it. Stop and re-read the source.

## Per-element probe protocol

For every element × every state × every theme:

```bash
# 1. Set theme (project-specific — themes are toggled via localStorage)
$PW localstorage-set ship-theme "$THEME"
$PW reload

# 2. Pre-press evidence
$PW snapshot --filename="$QA_DIR/snap-$ELEMENT-$STATE-$THEME-before.yml"
$PW screenshot --filename="$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-before.png"

# 3. The press itself (one of: click, fill, press, hover, drag)
$PW click "$REF"

# 4. Post-press evidence
$PW snapshot --filename="$QA_DIR/snap-$ELEMENT-$STATE-$THEME-after.yml"
$PW screenshot --filename="$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-after.png"

# 5. Visibility assertion — the failure mode I keep shipping past
$PW --raw eval "(() => {
  const el = $LOCATOR;
  if (!el) return JSON.stringify({ok:false, reason:'not in DOM'});
  const r = el.getBoundingClientRect();
  const cs = getComputedStyle(el);
  return JSON.stringify({
    ok: r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && cs.display !== 'none' && parseFloat(cs.opacity) > 0,
    rect: {w:r.width, h:r.height},
    visibility: cs.visibility, display: cs.display, opacity: cs.opacity,
  });
})()"

# 6. Console + network capture after each interaction
$PW console --errors >> "$QA_DIR/console.log"
$PW requests >> "$QA_DIR/network.log"

# 7. Append one line to evidence.jsonl per probe
jq -nc \
  --arg element "$ELEMENT" --arg state "$STATE" --arg theme "$THEME" \
  --arg pre "$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-before.png" \
  --arg post "$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-after.png" \
  --argjson visibility "$VISIBILITY_JSON" \
  '{element:$element, state:$state, theme:$theme, pre:$pre, post:$post, visibility:$visibility, ts: now|todate}' \
  >> "$QA_DIR/evidence.jsonl"
```

**Element-rendered check (step 5) is the load-bearing one.** A `chipCount: 13`
in shadow DOM means nothing if every chip has `display: none` from a CSS rule
that doesn't reach into the slot. The visibility assertion catches this.

## Console + network gates

```bash
# Any console error or warning fails the run unless explicitly waived in report.md
$PW console >> "$QA_DIR/console.log"
$PW console --errors > "$QA_DIR/console-errors.log"
$PW console --warnings > "$QA_DIR/console-warnings.log"

# Any non-2xx network response fails the run unless explicitly waived
$PW requests > "$QA_DIR/network.log"
```

**A waiver is a one-line entry in `report.md` under `## Waivers` with a reason.**
Examples of legitimate waivers: third-party tracker pings, dev-mode HMR
heartbeats. "Looks fine" is not a reason.

## Theme matrix

For each theme in the matrix:

```bash
for THEME in ship-light ship-dark ship-console; do
  $PW localstorage-set ship-theme "$THEME"
  $PW reload
  $PW screenshot --filename="$QA_DIR/screenshots/_baseline-$THEME.png"
  # ... full per-element protocol above ...
done
```

The baseline screenshot is your eyeball check for theme-aware regressions
(invisible-against-bg text, contrast loss, color-mix() unwinding wrong).

## Viewport matrix

```bash
for VP in "1280x720" "375x812"; do
  $PW resize ${VP%x*} ${VP#*x}
  $PW screenshot --filename="$QA_DIR/screenshots/_baseline-$VP.png"
  # ... full protocol ...
done
```

Capture overflow + truncation behavior at each width.

## report.md format

Every run ends with `$QA_DIR/report.md`:

```markdown
---
surface: dev-panel/edit-mode/components-tab
status: DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT
run: 2026-05-16T14-30
elements_probed: 12
themes_covered: ship-light, ship-dark, ship-console
viewports_covered: 1280x720, 375x812
console_errors: 0
console_warnings: 1
network_failures: 0
health_score: 87
---

## Summary

<1-2 sentences: what surface, what tier, top finding.>

## Findings

For each failing probe:

### F-001 — [severity] component does not render with non-zero box in ship-dark
- **Element:** `<sui-button slot="action">` ref `e15`
- **State:** populated
- **Theme:** ship-dark
- **Viewport:** 1280x720
- **Visibility check:** `{ok: false, rect: {w:0, h:0}, display: 'flex'}`
- **Evidence:** screenshots/e15-populated-ship-dark-after.png
- **Reproduction:**
  1. `playwright-cli goto http://localhost:3270/dev-panel`
  2. `playwright-cli localstorage-set ship-theme ship-dark`
  3. `playwright-cli reload`
  4. `playwright-cli click e3` (open Components tab)
  5. Observe: tab has slot content but content has zero size
- **Suspected cause:** host stylesheet defines `.components-tab > *` but the
  slotted children render inside `<sui-va-components>` shadow root, beyond
  reach. Move the rule into the inner component's :host stylesheet.

## Waivers

<Empty by default. Every waiver is one line + reason.>

## Health score

| Category      | Weight | Score |
|---------------|--------|-------|
| Console clean | 15%    | 100   |
| Network clean | 10%    | 100   |
| Rendered      | 20%    | 75    |
| Themed        | 20%    | 80    |
| Responsive    | 10%    | 90    |
| A11y          | 15%    | 70    |
| Interactive   | 10%    | 100   |
| **Total**     | 100%   | **87** |
```

A `health_score` ≥ 90 is shippable. 70–89 is `DONE_WITH_CONCERNS`. <70 is
`BLOCKED`.

## Completion status protocol

Every run ends with one of:

- **DONE** — every probe passed, score ≥ 90, console + network clean.
- **DONE_WITH_CONCERNS** — passed but with caveats. List each concern as a
  finding even if you couldn't fix it this turn.
- **BLOCKED** — something prevents completion (daemon down, surface 404s,
  required state unreachable). State the blocker + what you tried + what you
  recommend.
- **NEEDS_CONTEXT** — you don't know enough about the surface to declare its
  inventory. State exactly what's missing.

Lying — declaring DONE when there are unprobed elements or unaddressed
findings — is a worse failure than honest `DONE_WITH_CONCERNS`.

## Gate contract

When this skill writes a report with `status: DONE` or `DONE_WITH_CONCERNS`,
the gate will accept the touched surface for that commit window. The mapping:

- `apps/shell/**` → surface name starts with `chrome/` or `shell/`
- `packages/ui-components/src/sui-*/**` → surface name starts with `components/<tag>/`
- `views/<name>/**` → surface name starts with `views/<name>/`
- `crates/sui/**` → surface name starts with `runtime/` or `compiler/`

The gate scans `.artifacts/web-qa/<surface>/<latest>/report.md`, checks the
`run:` timestamp is newer than the latest commit touching the matched files,
and parses `status:` + `console_errors:` + `network_failures:` from the
frontmatter. Missing or stale artifact = gate FAIL.

## Anti-patterns (do not do)

- Taking one screenshot of one tab and committing
- Probing only `ship-light`
- Probing only one viewport
- Asserting on DOM presence without checking computed styles
- Calling `.click()` on an element you haven't snapshot-ref'd (drag/drop and
  composed-event paths don't fire from synthetic `.click()` in shadow DOM)
- Marking DONE when console has unwaived errors
- Marking DONE when the score is < 90 without justifying every dropped point

## References

- `references/playwright-cli.md` — every playwright-cli verb worth knowing
- `references/health-rubric.md` — full scoring math
- `references/element-locators.md` — when to use ref vs selector vs role vs
  testid, and how to find clickable divs the a11y tree misses
