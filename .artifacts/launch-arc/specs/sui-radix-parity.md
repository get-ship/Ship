# Job Spec: SUI primitives parity with Radix UI

**Branch:** job/sui-radix-parity
**Agent:** ship-designer
**Mode:** autonomous

## Goal

The previous parity bar (Lit primitives in `packages/ship-set/`) was the wrong reference — those primitives are getting deleted, and they're a much lower a11y/behavior bar than the industry standard. **Radix UI** is the new reference. Audit SUI primitives against Radix's behavior + API surface, port mechanical gaps, document divergences-by-design.

**Critical scope distinction:** Radix is *headless* (zero styling, behavior-only). SUI primitives are *styled* (visual identity matters). Port the **behavior, API shape, and a11y contract** — NOT the lack-of-styling. Keep SUI's visual design. The goal is "SUI primitives behave like Radix primitives," not "SUI looks like Radix."

## What to do

1. **Inventory both sets.**
   - SUI primitives under `packages/ui-components/src/` (the current `<sui-*>` set).
   - Radix Primitives' full surface: Accordion, AlertDialog, AspectRatio, Avatar, Checkbox, Collapsible, ContextMenu, Dialog, DropdownMenu, Form, HoverCard, Label, Menubar, NavigationMenu, Popover, Progress, RadioGroup, ScrollArea, Select, Separator, Slider, Switch, Tabs, Toast, Toggle, ToggleGroup, Tooltip. (Reference: https://www.radix-ui.com/primitives — read-only research via WebFetch is fine here.)

2. **Build the parity matrix.** For each Radix primitive, classify the SUI counterpart:
   - **MISSING** — Radix has it, SUI doesn't.
   - **BEHAVIOR-INCOMPLETE** — exists but missing critical a11y/keyboard/focus/ARIA behavior Radix specifies.
   - **API-DIVERGENT** — exists but the controlled/uncontrolled shape, props, or events don't match Radix's contract in ways consumers would feel.
   - **AT PARITY** — exists, behavior matches, API maps cleanly.
   - **DIVERGENT BY DESIGN** — exists and works but deliberately differs (document why).

3. **Fix the MECHANICAL gaps.** For BEHAVIOR-INCOMPLETE: missing keyboard handlers, missing ARIA attributes, broken focus-trap, incorrect controlled-state semantics — fix in place. For MISSING: port only if the primitive is mechanically simple and clearly needed (CLAUDE.md: no scaffolds — a half-port is worse than a logged gap).

4. **Specific behavior checks to verify per primitive** (the things that separate Radix-grade from "renders but doesn't quite work"):
   - **Focus management.** Modal/popover surfaces trap focus correctly; close returns focus to trigger. Tabs/Menu navigate via arrow keys. Auto-focus first focusable.
   - **Keyboard support.** Escape to close. Arrow keys for navigation. Home/End. Enter/Space activation. Tab order sane.
   - **ARIA.** Correct role/aria-* attributes per WAI-ARIA Authoring Practices. Labels associated. Live regions for transient content.
   - **Controlled + uncontrolled.** Components accept both controlled (`value` + `onChange`) and uncontrolled (`defaultValue`) shapes.
   - **Portaling.** Dialog/Popover/Tooltip/DropdownMenu render to body/portal, not in-tree. Stacking handled.
   - **Collision detection.** Popover/Tooltip/HoverCard flip/shift to stay in viewport.
   - **Composition.** Compound components work (`Dialog.Root` + `Dialog.Trigger` + `Dialog.Content` pattern, or SUI's equivalent).

5. **Produce the matrix report** at `.artifacts/launch-arc/reports/sui-radix-parity.md`:
   - Full matrix (every Radix primitive → SUI state)
   - "Fixed" list with root cause per fix
   - "Logged gap" list (missing primitives needing design, divergent-by-design entries)
   - Per-primitive a11y notes where they meaningfully differ

## File scope

`packages/ui-components/src/` (SUI primitive fixes/ports), `.artifacts/launch-arc/reports/sui-radix-parity.md` (new). Read-only on Radix docs via WebFetch.

Do NOT touch: shell2, SSR, runtime, anything in `crates/`. Do NOT add the `.artifacts/` line to `.gitignore` (this is a recurring auto-add bug).

## Architectural context
- SUI primitives are web components (`<sui-*>` tags) — Radix patterns from React-land have to translate. Compound-component patterns translate via slots + light-DOM children. Controlled state translates via reflected attributes + `change` events. Focus management translates 1:1.
- Don't reach for `@radix-ui/*` packages — they're React-specific. Use them as behavior reference only.
- CLAUDE.md: no placeholders, no scaffolds. A logged gap is better than a half-built primitive. File cap 300.

## Acceptance criteria

1. `.artifacts/launch-arc/reports/sui-radix-parity.md` exists. Every Radix primitive mapped to a SUI state. Every mechanically-fixed primitive listed with root cause. Every gap or divergence logged.
2. Every BEHAVIOR-INCOMPLETE / API-DIVERGENT primitive flagged in the matrix is either fixed or has a justified gap-log entry.
3. `bun run --filter @ship/ui-components test` passes.
4. `bun run --filter @ship/ui-components build` succeeds.
5. Spot-check no regression: `bun run test:e2e -- tests/e2e/shell2-views-smoke.spec.ts`.
6. `git diff --stat` confined to file scope.

---
> Execute this job yourself. Do not delegate the implementation to a subagent — subagents are only for read-only parallel research (e.g. WebFetch of Radix docs), never for writing the code this job asks for.
>
> If you notice a bug or problem outside your file scope, log it via `mcp__ship__log_progress` and continue your work.
>
> **When all acceptance criteria are met:**
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary of what changed.
> 2. Call `mcp__ship__mesh_broadcast` with `{ "topic": "job.completed", "workspace_id": "<id>", "summary": "<same summary>" }`. If shipd is not running, skip step 2.
