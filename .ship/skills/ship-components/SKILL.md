---
name: ship-components
stable-id: ship-components
description: Charter for Ship's web-component library. Covers composition model, naming, props vocabulary, shadow DOM conventions, tier split (primitives vs domain), and anti-slop rules. Load this whenever authoring, reviewing, or extending `<ship-*>` elements in packages/ship-set/src/web-components/.
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

## Tier split

Two tiers. Both tiers are `<ship-*>` custom elements — consumers can't
tell them apart at usage time. The library organization is strict.

### Primitives

**Location:** `packages/ship-set/src/web-components/` (top level).

**Rules:**
- Domain-free. Names are generic nouns from HTML/UI tradition.
- No Ship concepts (no `pipeline`, `spec`, `job`, `agent` in primitive
  names, attrs, or events).
- Zero knowledge of the daemon API. Pure DOM.
- Built on Lit.

**Examples:** `ship-button`, `ship-card`, `ship-list-item`, `ship-stat`,
`ship-input`, `ship-tabs`, `ship-empty-state`.

### Domain components

**Location:** `packages/ship-set/src/web-components/domain/`.

**Rules:**
- Ship-specific. Named for the Ship concept they represent.
- Compose primitives. Never reimplement primitive behavior.
- MAY know about Ship's data shapes (attrs like `state="running"` for a
  pipeline).
- Still free of direct daemon API calls — receive data via attrs / props.

**Examples:** `ship-pipeline-step`, `ship-pipeline-run`,
`ship-pipeline-definition`, `ship-pipeline-dag`, `ship-history-row`,
`ship-spec-row`.

### Decision rule

When considering a new component, ask in order:

1. **Is there a Ship concept in the name, attrs, or events?** If yes →
   domain. If no → primitive.
2. **Does it compose existing primitives?** Domain components MUST.
   Primitives MAY but usually don't.
3. **Could an unrelated project use it?** If yes → primitive. If it only
   makes sense inside Ship → domain.

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
2. **No domain concepts leaking into primitive names or attrs.**
   `ship-file-row` with `status="M"` is a git-specific component; it
   belongs in domain, not primitives. A primitive list-item uses
   `selected`, not `status`.
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
   tests. Applies equally to primitives and domain.
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

## Reviewing existing components

When this charter lands, five components already exist in
`packages/ship-set/src/web-components/`: `badge.ts`, `empty-state.ts`,
`file-row.ts`, `step-card.ts`, `tabs.ts`. Disposition:

| File | Action |
|---|---|
| `badge.ts` | Rewrite API: rename `variant` → `tone`, default to `muted`, keep `color-mix` implementation approach. |
| `empty-state.ts` | Rewrite: delete `ICON_GLYPHS` map, delete `title`/`description` attrs, go slot-only. |
| `file-row.ts` | **Move and rename.** `domain/file-row.ts` or absorb into a Studio git view composite. The generic replacement is a new primitive `ship-list-item.ts`. |
| `step-card.ts` | **Move.** Becomes `domain/pipeline-step.ts` (rename tag to `ship-pipeline-step`), simplify to compose `ship-card` + `ship-status-dot` + `ship-spinner`. |
| `tabs.ts` | Extend: add `variant="underline" \| "segmented"`, add `count` attr to `ship-tab`. Preserve child-composition pattern. |

## Adding a new component — checklist

Before writing code:

1. **Confirm tier.** Primitive or domain? Apply the decision rule above.
2. **Check the fixed vocabulary.** Does an existing tone/variant/size
   value fit? Don't invent.
3. **Sketch attrs and slots.** Attrs for state, slots for content. Write
   them in the header comment before writing the class.
4. **Identify the wrapped native element.** If interactive, there should
   be one.
5. **Identify events.** What verbs does the user do? Name them
   `ship-<verb>`.
6. **Visual parity.** Find the existing UI this component replaces. Match
   it. Padding, radius, hover, focus — all of it.

Then write the component, then write the tests, then wire the export
through `packages/ship-set/src/web-components/index.ts` and
`packages/ship-set/src/index.ts`.

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
- The decision rule for primitive-vs-domain is the most sensitive part.
  Propose, discuss, then amend.
