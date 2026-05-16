# Job Spec: Harvest OG j2 shell — Playwright behavioral inventory

**Branch:** job/j2-shell-harvest
**Agent:** ship-designer
**Mode:** autonomous

## Goal

Before j2/lit deletion, capture what works well in the OG shell (`apps/shell/`) that shell2/SUI doesn't replicate. This becomes input to the per-view intent docs that drive the launch-arc.

**Critical constraint: this MUST be a behavioral inventory using Playwright, not a screenshot tour.** 2D screenshots have been failing as evidence for weeks. They show layout but miss the things that matter — hover states, focus rings, scroll inertia, animation timing, keyboard navigation, state transitions, command-bar velocity, click→response latency. Describe BEHAVIOR verbally. Screenshots may accompany verbal descriptions but cannot replace them.

## What to do

1. **Boot OG shell.** Daemon should already be running at port 9385 with shell at `/shell/`. If not, `just dev-restart` and verify `curl http://127.0.0.1:9385/shell/` returns the shell HTML.
2. **For every j2 view and chrome surface**, drive the OG shell with Playwright and record behavior:
   - **Chrome features** to cover: top-nav (theme picker, command-bar trigger, view-picker, project picker), command palette (`Cmd+K` behavior — modes, results, navigation, keyboard sequences), rail (collapse/expand, item activation, focus), status bar (workspace indicator, theme indicator, daemon health), dock (open/close, panel composition, transitions), launcher (open behavior, content, navigation), notifications/toasts (positioning, duration, dismissal).
   - **Views** to cover (j2 views under `.ship/views/`): agents, agents/[id] detail, agents/create, skills, skills/[id], collections, artifacts, dashboard, events, mesh, pipelines, pipelines-board, git, settings, theme-studio, specs, sessions, ship-ui-studio, chat, canvas, overview, home, setup.
3. **For each surface, record what to look for:**
   - **Interaction latency** — measurable with Playwright timers. Click → DOM change: how fast?
   - **Focus management** — what gets focus on open/close/tab/route-change? Are focus rings visible? Trap behavior on modals?
   - **Keyboard sequences** — what keys do what? Arrow nav? Esc to close? Tab order?
   - **Animations + transitions** — what animates? Duration? Easing? Does it block interaction?
   - **State transitions** — loading → resolved → error. What's the visual + a11y behavior at each step?
   - **Scroll behavior** — does the rail scroll independently? Inertia? Sticky headers?
   - **Hover states** — what reveals on hover? Tooltip timing? Cursor changes?
   - **Layout density** — text wrapping, whitespace, information density. Compare to shell2 where applicable.
4. **Identify two categories per surface:**
   - **PRESERVE** — patterns that work well, should land in the SUI replacement. Verbal description of the behavior + why it matters.
   - **AVOID** — anti-patterns, things that don't work, things to leave behind. Verbal description + why it's bad.
5. **Output: `.artifacts/launch-arc/reports/j2-shell-harvest.md`** structured per-surface so each section can become input to a future intent doc. Each surface gets:
   - Overview (what the surface does for the user)
   - Behavior log (the Playwright-observed interactions, verbal)
   - PRESERVE list (specific behaviors)
   - AVOID list (specific anti-patterns)
   - Open questions (things that need design call, not just port)

## File scope

Read-only across `apps/shell/`, `packages/ship-set/`, `.ship/views/*/index.html.j2`, related daemon endpoints. Probe scripts may be written to `/tmp/`. Writes only to `.artifacts/launch-arc/reports/j2-shell-harvest.md`.

Do NOT modify j2 or Lit source. Do NOT touch shell2. Do NOT add the `.artifacts/` line to `.gitignore`.

## Architectural context
- The OG shell is served at `/shell/<view>`. Navigo handles routing (works after the recent navigo-base fix? May still 404 on some routes — note in the report if so).
- Three pre-existing daemon endpoint mismatches: `/api/v1/view-source/...`, `/api/v1/lint/view`, `/api/settings`. These will 404; note in report, don't fix them in this job.
- Playwright is already a dev dep. Run via `bunx playwright` or `bun run` against existing test infra.
- This report drives the intent docs. Quality matters more than completeness — better to cover 10 surfaces with depth than 21 with bullet-points.

## Acceptance criteria

1. `.artifacts/launch-arc/reports/j2-shell-harvest.md` exists with per-surface entries covering at minimum: chrome (top-nav, cmdk, rail, status, dock, launcher) + the 9 default-enabled v1 views (Home, Canvas, Collections, Artifacts, Agents, Skills, Chat, Settings, Setup) + the toggleable dev views (Git) + as many other j2 views as time permits.
2. Each entry has **verbal behavior descriptions**, not just screenshots. Verbal evidence required; visual evidence supplementary.
3. PRESERVE and AVOID lists per surface, populated.
4. No code changes outside the report file.
5. `git diff --stat` confined to `.artifacts/launch-arc/reports/j2-shell-harvest.md` (+ no `.gitignore` `.artifacts/` add).

---
> Execute this job yourself. You MAY use the Playwright MCP or write Playwright scripts. Do not delegate the analysis/writing to a subagent — the writing IS the work.
>
> If you notice a bug or problem outside your file scope, log it via `mcp__ship__log_progress` and continue your work.
>
> **When all acceptance criteria are met:**
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary of what changed.
> 2. Call `mcp__ship__mesh_broadcast` with `{ "topic": "job.completed", "workspace_id": "<id>", "summary": "<same summary>" }`. If shipd is not running, skip step 2.
