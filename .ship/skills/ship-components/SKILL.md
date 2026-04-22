---
name: ship-components
stable-id: ship-components
description: Charter for Ship's web-component library (@ship/set). Covers composition model, naming, props vocabulary, shadow DOM conventions, directory layout (core / islands / domain / viz), the distribution test, and anti-slop rules. Load this whenever authoring, reviewing, or extending `<ship-*>` elements in packages/ship-set/src/web-components/.
tags: [ui, components, web-components, lit, design-system, frontend]
authors: [ship]
---

# Ship Component Charter

The component library is a strategic surface. It must be extremely easy to
understand, sensibly named, and compositionally consistent. This document
is the rulebook. Every component in `packages/ship-set/src/web-components/`
follows it, without exception.

## Purpose

Ship renders UI across three surfaces:

1. **Daemon-rendered HTML** (first-party views, third-party extensions) —
   Datastar + SSE, no JS framework.
2. **Studio SPA** (React + TanStack) — interactive shell.
3. **CDN-served third-party views** — client-rendered against the daemon.

Web components are the **shared substrate** across all three. Same
`<ship-*>` element, same tokens, same visual language — independent of
who renders it or where.

## Visual parity mandate

**No UI degradation.** The component library exists to consolidate, not
to redesign. Every rewrite must reproduce the current UI's visual result:

- Paddings, radii, font sizes, line heights — match existing app and view
  usage to the pixel where possible.
- Colors — use the canonical OKLCH tokens (`packages/ship-set/src/tokens.css`);
  do not introduce new brand colors.
- Interactions — hover, active, focus, disabled states match current
  behavior.
- Animations — preserve any existing keyframes (e.g., `pulse`, `spin`,
  fade-in of tab panels).

If the current UI has no defined treatment for a state (e.g. a new size
variant), propose a consistent derivation from existing tokens; do not
invent.

## Directory layout

All four tiers are `<ship-*>` custom elements — consumers can't tell
them apart at usage time. The library organization is strict.

```
packages/ship-set/src/web-components/
  core/      — lightweight primitives (button, card, list-tile, input,
               badge, stat, icon, chip, status-dot, timestamp, etc.)
  islands/   — heavy single-purpose elements with their own brain
               (ship-code-editor, ship-markdown-editor, ship-terminal)
  domain/    — Ship platform concepts (ship-pipeline-*, ship-git-diff,
               ship-auth-dialog)
  viz/       — charts (ship-sparkline, ship-line-chart, ship-donut, …)
```

### core

Domain-free primitives. Names are generic nouns from HTML/UI tradition.
No Ship concepts (no `pipeline`, `spec`, `job`, `agent` in primitive
names, attrs, or events). Zero knowledge of the daemon API. Pure DOM,
built on Lit.

**Examples:** `ship-button`, `ship-card`, `ship-list-tile`, `ship-stat`,
`ship-input`, `ship-tabs`, `ship-empty-state`, `ship-badge`, `ship-icon`.

### islands

Heavy single-purpose elements that wrap a non-trivial third-party engine
(Monaco, xterm, tiptap). Bundled separately — `ship-code-editor.js`,
`ship-markdown-editor.js`, `ship-terminal.js` — so views that don't need
them don't pay the cost. The shell currently loads islands eagerly via
`requestIdleCallback` after first paint.

### domain

Ship-specific. Named for the Ship concept they represent. Compose core
primitives — never reimplement primitive behavior. MAY know about
Ship's data shapes (attrs like `state="running"` for a pipeline). Still
free of direct daemon API calls — receive data via attrs / props.

**Examples:** `ship-pipeline-step`, `ship-pipeline-run`,
`ship-pipeline-definition`, `ship-pipeline-dag`, `ship-git-diff`,
`ship-auth-dialog`.

### viz

Charts. Same rules as core (domain-free, pure DOM) but grouped because
they share canvas/SVG plumbing and because a view often pulls several
at once.

### Decision rule

When considering a new component, ask in order:

1. **Is there a Ship concept in the name, attrs, or events?** If yes →
   `domain/`. If no → `core/` (or `viz/` if it's a chart).
2. **Does it wrap a heavy third-party engine?** If yes → `islands/`.
3. **Does it compose existing primitives?** Domain components MUST.
   Core components MAY but usually don't.
4. **Could an unrelated project use it?** If yes → `core/` or `viz/`.
   If it only makes sense inside Ship → `domain/`.

Before the tier question even applies, the element must pass the
**distribution test** below. If it fails, it doesn't belong in the
library at all — it's a view-local composition.

## The distribution test

Before you write a new `<ship-*>` element, ask whether it belongs in
the distributed `@ship/set` library at all. An element earns its spot
**only if at least one** of these is true:

1. **Semantic primitive** — it has a known name in UI vocabulary. A web
   developer recognizes the name without seeing the element (button,
   card, combobox, list-tile, popover, tabs, badge). Mirrors what
   Radix, Material, or iOS would call it.
2. **Platform concept** — it represents a Ship domain object whose
   rendering is structurally tied to the concept. `ship-pipeline-step`
   qualifies because "pipeline step" is a Ship-owned concept with a
   specific shape (status, name, duration, provider). Not every domain
   object qualifies — only ones where the rendering *encodes* the
   concept.
3. **Has its own brain** — non-trivial internal logic that justifies
   isolation: Monaco, xterm, tiptap, a custom layout engine. These are
   *islands*, and they live in `islands/`.

If an element fails all three, it is a **composition**, not a library
element. Compositions live in
`.ship/views/<view>/_components/*.j2` as Jinja partials, not as web
components. When the same composition repeats across 3+ views, promote
it — preferably by adding slots to an existing library element, rarely
by introducing a new one. **Never ship a thin wrapper whose only value
is filling slots with view-specific content.**

| Passes the test | Fails the test (composition) |
|---|---|
| `ship-list-tile` — semantic primitive | `ship-agent-row` — `ship-list-tile` + slots |
| `ship-pipeline-step` — platform concept | `ship-skill-row` — `ship-list-tile` + slots |
| `ship-code-editor` — has its own brain | `ship-job-card` — `ship-list-tile variant=card` + slots |
| `ship-badge` — semantic primitive | `ship-settings-row` — `ship-list-tile` + slots |

The failing column is not hypothetical. Every row and tile pattern in
Ship today is already a `ship-list-tile` composition; every time an
agent proposes a new `<ship-X-row>` we fold it back into the primitive.
If your proposed element is "tile with this leading icon, this primary
line, this trailing badge," you are composing, not distributing.

## Composition model

### Naming

- Tags: `ship-<noun>` or `ship-<noun>-<noun>` (kebab-case). No prefixes
  beyond `ship-`.
- Attributes: kebab-case, matching HTML conventions
  (`selected`, `disabled`, `tone`, `variant`, `size`).
- Properties: camelCase counterparts of attributes (`selected`, `tone`).
- Events: `ship-<verb>` or `ship-<noun>-<verb>`
  (`ship-select`, `ship-dismiss`, `ship-tab-change`).
- CSS custom properties for theming overrides:
  `--ship-<component>-<prop>` (e.g., `--ship-button-radius`). Use
  sparingly; default is to inherit from canonical tokens.

### Fixed vocabulary

These enums are reused across every component. Do not invent new values.

- **tone** — semantic color: `default | info | success | warning | destructive | accent | muted`
- **variant** — visual treatment: `solid | soft | outline | ghost`
- **size** — dimension: `sm | md | lg` (default `md`)
- **state attrs** (boolean, reflect): `selected`, `disabled`, `loading`,
  `active`, `pulse`

A component picks the subset relevant to it. `ship-button` uses
`variant + size + tone`; `ship-badge` uses `tone + size`;
`ship-status-dot` uses `tone + pulse`.

### Slots over attrs for content

Content goes through slots, not attributes. Attributes are reserved for
**state, vocabulary, identity**.

```html
<!-- WRONG -->
<ship-empty-state title="No results" description="Try another query"></ship-empty-state>

<!-- RIGHT -->
<ship-empty-state>
  <ship-icon slot="icon" name="search"></ship-icon>
  <span slot="title">No results</span>
  Try another query
</ship-empty-state>
```

Slots:
- Unnamed default slot for the primary content.
- Named slots for structural regions: `header`, `footer`, `actions`,
  `icon`, `title`, `trailing`, `leading`.
- Document every slot in the component's header comment.

### Native underneath

Wrap native elements wherever possible. Keyboard, accessibility, and form
participation come for free.

- `ship-button` wraps `<button>`.
- `ship-input` wraps `<input>`.
- `ship-textarea` wraps `<textarea>`.
- `ship-select` wraps `<select>`.
- `ship-dialog` wraps `<dialog>`.
- `ship-checkbox` wraps `<input type="checkbox">`.

Expose the wrapped element via `part="<role>"` so consumers can style if
needed (e.g., `ship-button::part(button) { ... }`).

### Events

- All events are `CustomEvent` with `{ detail: {...} }`.
- `bubbles: true`, `composed: true` (cross shadow boundaries) — unless
  there's a documented reason not to.
- Named consistently: `ship-<verb>` for actions the user initiated
  (`ship-select`, `ship-dismiss`, `ship-toggle`), `ship-<noun>-<verb>`
  for state transitions on a sub-entity (`ship-tab-change`).
- Document every event in the component's header comment with its
  `detail` shape.

### Accessibility defaults

- Open shadow DOM (`{ mode: 'open' }`).
- `delegatesFocus: true` on interactive shadow roots (button, input,
  select, etc.).
- Role attrs set on host where appropriate when not wrapping a native
  element with implicit role.
- Keyboard handlers: Enter/Space activate, Escape dismiss, Arrow keys
  navigate where relevant.
- `aria-*` attrs on the wrapped native element, reflected from host
  attrs where it makes sense (`aria-label`, `aria-describedby`).

### Design tokens — read the highest layer that fits

Ship has a four-layer token system. Components should **never** read hardcoded
`oklch(0 0 0 / 40%)` / `rgba(0,0,0,0.08)` / bespoke `color-mix` formulas.
Read from the highest layer that captures intent. Themes override these
layers; your component rides along without change.

**Layer 1 — semantic surfaces and intents.** Reach for these first.

| What you want | Token |
|---|---|
| Page background | `var(--surface-0)` |
| Card surface | `var(--surface-1)` |
| Raised surface (nested panel, sticky header) | `var(--surface-2)` |
| Popover / overlay surface | `var(--surface-3)` |
| Dialog / drawer backdrop | `var(--backdrop)` |
| Light dim behind a drawer | `var(--scrim)` |
| Hover tint on an interactive row | `var(--hover-bg)` |
| Active / pressed tint | `var(--active-bg)` |
| Selected-row tint | `var(--selected-bg)` |
| Selected-row border | `var(--selected-border)` |
| Thin divider | `var(--divider)` or `var(--border-subtle)` |
| Hard boundary in high-contrast themes | `var(--border-strong)` |

**Layer 2 — tone ramps.** Any component tinted by intent (success, warning,
destructive, info, accent, muted) reads its ramp:

```css
/* Chip with success tone — all four values come from tokens. */
.chip-success {
  color: var(--tone-success-fg);
  background: var(--tone-success-bg);        /* soft tint, ~14% */
  border: 1px solid var(--tone-success-border);
}
.chip-success:focus-visible {
  box-shadow: 0 0 0 3px var(--tone-success-ring);
}
```

Do **not** write `color-mix(in oklch, var(--success) 14%, transparent)` in
component CSS. That formula is in tokens.css once — change one line, every
chip/badge/alert/status-dot inherits it across every theme.

Tones: `default`, `muted`, `info`, `success`, `warning`, `destructive`,
`accent`, `primary`. Each exposes `-fg`, `-bg` (soft), `-bg-solid`,
`-border`, `-ring`.

Components that accept a `tone` attribute should import `toneVars` from
`./util` and then read `var(--tone-fg)` / `--tone-bg` / `--tone-border` /
`--tone-ring`. The helper routes the host's `[tone='…']` attribute to the
matching ramp automatically.

**Layer 3 — typography roles, motion, density.** These are theme-aware
roles, not font/curve/size primitives.

| Intent | Token |
|---|---|
| Heading / panel head / card header / dialog title | `font-family: var(--heading-font); font-weight: var(--heading-weight); letter-spacing: var(--heading-tracking);` |
| Body text | `var(--body-font)` |
| Monospace | `var(--mono-font)` |
| Eyebrow label / group head | `var(--label-font); font-weight: var(--label-weight); letter-spacing: var(--label-tracking);` |
| Soft motion for surface changes | `var(--ease-out-soft)` |
| Tactile spring for buttons | `var(--ease-spring)` |
| Expressive default | `var(--ease-out)` |
| Row padding — density-aware | `padding: var(--density-row-py) var(--density-row-px)` |
| Icon size — density-aware | `width: var(--density-icon); height: var(--density-icon);` |

A theme can set `--heading-font: var(--font-body)` to demote display
typography (the `paper` theme already does this). Never hardcode
`"Syne"` or `"Inter"` — use the role.

**Layer 4 — primitives.** Spacing scale, radii, shadow ramp, typography
scale, duration primitives. Reach for these only when the higher layers
don't fit. `var(--space-3)` is fine; `12px` is a smell.

**Shadows.** Read from the semantic shadow tokens:

| Intent | Token |
|---|---|
| Resting card | `var(--shadow-xs)` |
| Raised surface at rest | `var(--shadow-sm)` |
| Interactive card hover | `var(--shadow-hover)` |
| Dropdown / popover | `var(--shadow-md)` |
| Dialog | `var(--shadow-lg)` |
| Toast / topmost overlay | `var(--shadow-xl)` |
| Sticky header bottom cue | `var(--shadow-sticky)` |
| Primary-tinted focus glow | `var(--shadow-accent)` |

Dark-mode and theme-specific depth intensities are already tokenised —
you get the right recipe automatically.

**Density.** A view can set `<html data-density="compact">` or
`"spacious"` and every component that reads `--density-*` adapts. When
authoring a new list-like component, pull padding and icon size from
the density channel, not the base `--space-*` scale.

### Shadow DOM conventions

- Lit `static styles = css\`...\`` for every component.
- Always include `:host { display: ... }` — pick `block`, `inline-block`,
  `contents`, or `flex` explicitly. Never leave host display undefined.
- Use `exportparts` on nested custom elements when their parts need to
  be reachable from outside.
- Never import token fallbacks in TS. Use bare `var(--foo)`. The tokens
  CSS is always loaded at the document root.
- No inline `<style>` injection via `innerHTML`. Use Lit's `css` template.

## File discipline

- One component per file. File name = kebab tag name (`ship-button.ts`
  → `class ShipButton`).
- Tests colocated: `ship-button.test.ts` next to `ship-button.ts`.
- Target 100–150 lines per component. **No hard cap.** If a component
  genuinely needs 200 well-composed lines to match visual parity and
  composition rules, that's fine. Small is good because the component is
  simple — not because a cap said so.
- If a component passes ~220 lines, review: is it actually two
  components? Could a helper extract cleanly?

## Testing

Every component has at least:

1. A render test — mounts, renders initial content, reflects attrs.
2. An attribute reactivity test — changing an attr re-renders correctly.
3. An event test for any interactive component — dispatches the
   documented `CustomEvent` with the documented detail.
4. Where relevant, a keyboard test (Enter/Space on button, Escape on
   dialog, Arrow on tabs).

Use `@open-wc/testing` or Vitest with `happy-dom`, whichever the package
is already configured for. Don't introduce a second test runner.

## Anti-slop rules

These are explicit prohibitions. They codify the mistakes from
yesterday's first pass.

1. **No hardcoded glyph maps.** If a component wants an icon, use a
   slot. Do not ship an `ICON_GLYPHS: { inbox: '⌯' }` micro-registry.
2. **No domain concepts leaking into core names or attrs.**
   A git-diff row with `status="M"` is a git-specific concept; it
   belongs in `domain/` (e.g. composed inside `ship-git-diff`), not
   in `core/`. A core list-tile uses `selected`, not `status`.
3. **No fake enums that encode display.** A `variant="primary"` for a
   badge is fine (it's the tone). A `variant="dispatched"` is a leak —
   that's a Ship domain concept; map it to a tone at the caller.
4. **No attrs where slots belong.** Content is slots. State is attrs.
   If you find yourself writing `title`, `description`, `body` as attrs
   for arbitrary text, switch to slots.
5. **No fallbacks inside `var(--foo)`.** Tokens are guaranteed loaded.
   `var(--primary, oklch(...))` is a smell — delete the fallback.
6. **No parallel token systems.** Colors, radii, fonts come from
   `packages/ship-set/src/tokens.css`. No TS token constants. No
   per-component color tables that duplicate the canonical palette.
7. **No inline styles via `innerHTML`.** Use Lit `css` templates.
8. **No un-tested components.** PR is incomplete without colocated
   tests. Applies equally to core, islands, domain, and viz.
9. **No components without a header comment** describing: what it is,
   its attrs, its slots, its events. One block comment at the top of
   the file. Used by documentation generators and by future agents.
10. **No new pnpm workspace packages** without explicit design approval.
    Everything web-component lives in `@ship/set`.

## Polish pass — bringing an existing component to shadcn/Radix quality

Use this when the job is "make this component production-tier." Proven
pattern from the ship-button polish (see git history for `ship-button.ts`).

### Approach

1. **Read the existing component** and its tests. Note the public API
   (attributes, slots, events, parts) — do NOT change it.
2. **Look at shadcn/ui and Radix Themes** for the equivalent component.
   Match their visual weight.
3. **CSS custom-property channels** for tone × variant × size:
   - Per-host tone channel: `--tone`, `--tone-fg`, `--tone-contrast` set
     once at `:host([tone=…])`; variant CSS reads from the channel.
     Adding a tone = one `:host([tone=new])` block.
   - Per-host size channel: `--btn-h`, `--btn-px`, `--btn-fs`, etc.
   - Variants read from channels, not hardcoded per variant.
4. **Micro-interactions** — hover/active/focus use `var(--duration-fast)`
   + `var(--ease-out)`. Hover lifts 1px + `shadow-sm`; active returns to
   0 + `shadow-xs`. Focus-visible uses stacked box-shadow ring.
5. **States must be visually distinct:**
   - Loading: content fades (opacity + scale), spinner overlays in
     `currentColor`, `aria-busy` set, interactions disabled.
   - Disabled: muted bg, desaturated filter, dimmed text, no affordances,
     `cursor: not-allowed`.
6. **Premium CTA treatment** — for `tone=accent` + `variant=solid`, subtle
   gradient + colored glow on hover. Primary actions feel premium.
7. **Optical corrections** — 0.5px padding nudges for descenders, inset
   highlights on solid variants so fills don't look flat.
8. **Respect `prefers-reduced-motion`** — import `reducedMotion` from
   `util.ts`, transforms/transitions collapse automatically.
9. **Don't break tests.** Public API and event emission unchanged.

### Verification loop (required)

After changes, run the full loop:

1. **Unit tests**: `cd packages/ship-set && pnpm test <component>`
2. **Bundle + reload daemon:**
   ```bash
   /workspaces/ship/node_modules/.pnpm/@esbuild+linux-x64@0.25.12/node_modules/@esbuild/linux-x64/bin/esbuild \
     packages/ship-set/src/web-components/index.ts --bundle --format=esm --target=es2020 \
     --outfile=apps/shipd/assets/ship-elements.js --log-level=warning
   cp apps/shipd/assets/ship-elements.js apps/shell/public/vendor/ship-elements.js
   cd /workspaces/ship && cargo build -p shipd
   pkill -9 shipd; sleep 2
   cd /workspaces/ship && ./target/debug/shipd --port 9315 2>/tmp/shipd.log &
   sleep 3
   ```
3. **Playwright verification** (use the `web-qa` skill):
   - Build a gallery HTML page exercising the component across all
     tones/variants/sizes/states. Inject it via
     `await page.setContent(html)` or render it inline into localhost:3000.
   - Load scripts from the running daemon at `http://localhost:9315`
     (`/api/runtime/assets/view-tokens.css`,
     `/api/runtime/assets/ship-elements.js`).
   - Capture console errors: target 0.
   - Screenshot the gallery at 1400×1800, deviceScaleFactor: 2.
   - Screenshot hover and focus states separately.
   - Verify keyboard navigation works (Tab + Enter/Space).
   - Write a short report to `.artifacts/web-qa/<run>/` with screenshots
     + observations.
4. **Update any views** that were using the old attribute names or
   patterns the component emits (e.g. ship-select went from emitting
   `ship-select-change` to `ship-change` — templates had to update).
5. **Commit** only when: tests pass, zero console errors, screenshots
   show each state deliberately.

### What "production-tier" means

Not: "colors match the tokens." That's a floor.

Yes:
- Hard to distinguish from shadcn/Radix equivalent in polish
- Every state (hover, focus, active, disabled, loading) is deliberate
- Typography is optically balanced, not just mathematically sized
- Micro-interactions feel snappy, not sluggish
- The component photographs well (looks good in a docs screenshot)

## ship-list-tile — the canonical list primitive

`core/ship-list-tile.ts` is the single primitive for every row- or
tile-shaped pattern in the library. Earlier iterations shipped a
family of narrow row elements; they were all absorbed into this one
tile, and the library has no dedicated row tags beyond it. Reach for
`ship-list-tile` for list items, log lines, event rows, history
entries, spec summaries, settings rows, and dashboard cards.

**API:**

| Axis | Values |
|---|---|
| `variant` | `row` (default) — inline, hover background, thin divider. `card` — elevated surface, radius, subtle shadow. |
| `density` | `compact` \| `normal` (default) \| `relaxed` — scales vertical/horizontal padding. |
| `interactive` | boolean — role=button, tabindex=0, cursor:pointer, Enter/Space dispatches `ship-select`. |
| `selected` | boolean, reflected — reflects `aria-selected` and visual highlight. |
| `disabled` | boolean, reflected — dims and blocks interaction. |
| `tone` | shared Tone vocabulary — tints the left-border accent. |

**Slots:** `leading` (icon/dot/avatar), default (primary content), `meta`
(secondary line below primary — legacy name preserved), `trailing`
(right-aligned badges, timestamps, chevrons).

**Event:** `ship-select` (CustomEvent, bubbles, composed) when
`interactive` and the user clicks or presses Enter/Space.

**Parts:** `button`, `row`, `primary`, `secondary`, `trailing`.

### Example — a settings row (label left, control right)

Settings rows are the archetypal `ship-list-tile` composition: primary
label, sub-label in `meta`, interactive control in `trailing`.

```html
<ship-list-tile variant="card" density="relaxed">
  <ship-icon slot="leading" name="bell"></ship-icon>
  Notifications
  <span slot="meta">Email me when a pipeline fails</span>
  <ship-switch slot="trailing" checked></ship-switch>
</ship-list-tile>

<ship-list-tile variant="card" density="relaxed" interactive>
  <ship-icon slot="leading" name="key"></ship-icon>
  API token
  <span slot="meta">Last rotated 14 days ago</span>
  <ship-button slot="trailing" variant="ghost" size="sm">Rotate</ship-button>
</ship-list-tile>
```

Use the same pattern for agent rows, event rows, skill rows, job cards,
dashboard entries. If you find yourself wanting a new `<ship-X-row>`
element, re-read the distribution test above.

## Adding a new component — checklist

Before writing code:

1. **Pass the distribution test.** If the element doesn't, it's a
   view-local Jinja composition, not a library element.
2. **Confirm the subdir.** `core/`, `islands/`, `domain/`, or `viz/`?
   Apply the decision rule above.
3. **Check the fixed vocabulary.** Does an existing tone/variant/size
   value fit? Don't invent.
4. **Sketch attrs and slots.** Attrs for state, slots for content. Write
   them in the header comment before writing the class.
5. **Identify the wrapped native element.** If interactive, there should
   be one.
6. **Identify events.** What verbs does the user do? Name them
   `ship-<verb>`.
7. **Visual parity.** Find the existing UI this component replaces. Match
   it. Padding, radius, hover, focus — all of it.

Then write the component, then write the tests, then wire the export
through the appropriate subdir index (`core/`, `islands/`, `domain/`,
or `viz/`) and up through `packages/ship-set/src/web-components/index.ts`
and `packages/ship-set/src/index.ts`.

## Element catalog — two tiers

### Compact catalog (agent default)

Start here. Machine-generated, 66 elements in ~4KB. One line per
element: tag, attrs (enums as `attr=a|b|c`, booleans as bare names),
slots, events.

- **Repo:** `packages/ship-set/catalog.md`
- **Daemon URL:** `GET /api/runtime/assets/ship-elements.md`

Agents authoring daemon views should read this first. It's small
enough to sit in-context at all times. For ~95% of emission work
(pick a tag, pick attr values, slot content) this is all you need.

### Full CEM (depth reference)

When you need types, defaults, descriptions, or CSS custom properties,
load the full manifest.

- **Repo:** `packages/ship-set/custom-elements.json` (~190KB)
- **Daemon URL:** `GET /api/runtime/assets/custom-elements.json`

IDE tooling (lit-plugin, lsp-cem, VS Code) consumes this for
attribute autocomplete and type checking.

### Regenerate

Both files regenerate from source automatically on build:

```
pnpm -F @ship/set build      # full pipeline: tsc → manifest → catalog
pnpm -F @ship/set manifest   # CEM only
pnpm -F @ship/set catalog    # compact catalog only (requires fresh CEM)
```

## Daemon-rendered views (minijinja + Datastar SSE)

First-party views (`apps/shipd/templates/<view>/`) render `<ship-*>`
elements from the daemon via minijinja, then patch live updates over
Datastar SSE. The pattern is two endpoints per view:

- `GET /api/runtime/view/<name>` — full shell. Extends
  `_page_shell.html.j2`, overrides `title`/`head`/`body_class`/`body`.
  Returns text/html.
- `GET /api/runtime/view/<name>/sse` — `ViewSse` response. On connect
  it renders the content partial and emits
  `vs.patch("#<name>-content", html)`. It then subscribes to kernel
  events via `state.kernel().spawn_actor(...)` and re-renders on each
  event. A `vs.heartbeat_every(Duration::from_secs(15))` handle keeps
  the connection alive; drop it when the task ends.

Write new templates under `apps/shipd/templates/<view>/`:

```
<view>/shell.html.j2             # extends _page_shell, full page
<view>/content.html.j2           # tab dispatcher, target of SSE patches
<view>/partials/<tab>.html.j2    # one per tab
<view>/partials/<card>.html.j2   # reusable row/card fragments
```

Register each template in the `TEMPLATES` array of
`apps/shipd/src/views/templates.rs` (alphabetized). Render with
`templates::render("<view>/<file>.html.j2", &ctx)`; context structs
derive `Serialize`.

State reads go through `crate::views::state`: `list_pipeline_defs`,
`list_pipeline_runs`, `partition_runs`, `status_token`, plus the
query-deserializable `Filter`. Do not re-walk the filesystem inside
handlers.

Filter persistence is URL-based: tab switches and chip clicks emit
`@get('/api/runtime/view/<name>?tab=...&status=...')`. The server
re-renders and Datastar morphs the page; the browser URL updates from
`@get`'s fetch. No client-side state machine.

For wire-level details (SSE framing, selectors, signal patches) see
`.ship/skills/datastar-views/SKILL.md`.

## When this charter changes

- Changes require a design discussion, not an inline edit.
- New fixed-vocabulary values need a charter amendment first, then
  propagation to all existing components that use the affected axis.
- The distribution test and the core-vs-islands-vs-domain-vs-viz
  decision rule are the most sensitive parts. Propose, discuss, then
  amend.
