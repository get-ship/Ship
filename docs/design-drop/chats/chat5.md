# Ship Components Review

_Started 2026-05-19 01:46 UTC_

---

## User

please go through ship primitives and other components and suggest any design improvements we can make.

## Assistant

_[tool: list_files]_

_[tool: read_file]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: show_html]_

_[tool: show_html]_

_[tool: save_screenshot]_

_[tool: view_image]_

_[tool: get_webview_logs]_

_[tool: save_screenshot]_

_[tool: view_image]_

_[tool: eval_js]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

The screenshot tool doesn't capture shadow DOM. Let me audit by reading actual component source code instead.

_[tool: screenshot_user_view]_

Let me try a real pixel screenshot tool instead:

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: grep]_

I have plenty of material. Let me also peek at a couple more for the cross-system patterns, then build the audit deliverable.

_[tool: read_file]_

_[tool: read_file]_

_[tool: grep]_

_[tool: grep]_

_[tool: update_todos]_

_[tool: write_file]_

I have enough material. Building the audit doc now.

_[tool: done]_

_[tool: update_todos]_

_[tool: fork_verifier_agent]_

Audit is up at **`workbench/Primitives Audit.html`**.

**TL;DR — 8 priority findings + 14 nits:**

1. **Three different focus-ring implementations** (button uses stacked box-shadow, input uses outline, chip uses both) — converge on tokens.
2. **19 hardcoded `font-size: Npx`** values across primitives, down to 9px. None scale when themes override the type ramp.
3. **Default props drift** — chip defaults to `sm/muted`, alert has no `default` tone, status-dot defaults to `success`. A bare row of primitives doesn't compose visually.
4. **`solid + warning` button fails contrast** on light themes — uses page background as text color instead of `--warning-foreground`.
5. **`sui-theme-toggle` ships raw emoji** (🌙/☀) instead of `sui-icon` — renders differently per OS.
6. **`sui-alert` close uses `&#215;`** in 18×26px tap target — only primitive not using SVG / Lucide.
7. **`sui-card` decorations are subtle to discover** — accent-bar is hover-only; status-rail draws a foreground-colored stripe on default-tone cards.
8. **Selected chips fall back to `--foreground`** instead of `--primary` — reverses every framework convention.

Plus a Part II on system-level patterns (catalog hygiene, density vocabulary, loading-state inconsistency, iconography rules) and a Part IV with a suggested ~3-day order of work — token sweep first, since it cascades through everything else.

Read the doc for the full evidence (code excerpts + recommended diffs); it's structured so each finding is self-contained if you want to triage them individually.

