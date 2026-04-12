---
name: ship-components
stable-id: ship-components
description: Charter for Ship's web-component library. Covers composition model, naming, props vocabulary, shadow DOM conventions, tier split (primitives vs domain), and anti-slop rules. Load this whenever authoring, reviewing, or extending `<ship-*>` elements in packages/ui/src/web-components/.
tags: [ui, components, web-components, lit, design-system, frontend]
authors: [ship]
---

# Ship Component Charter

The component library is a strategic surface. It must be extremely easy to
understand, sensibly named, and compositionally consistent. This document
is the rulebook. Every component in `packages/ui/src/web-components/`
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
- Colors — use the canonical OKLCH tokens (`packages/ui/src/tokens.css`);
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

**Location:** `packages/ui/src/web-components/` (top level).

**Rules:**
- Domain-free. Names are generic nouns from HTML/UI tradition.
- No Ship concepts (no `pipeline`, `spec`, `job`, `agent` in primitive
  names, attrs, or events).
- Zero knowledge of the daemon API. Pure DOM.
- Built on Lit.

**Examples:** `ship-button`, `ship-card`, `ship-list-item`, `ship-stat`,
`ship-input`, `ship-tabs`, `ship-empty-state`.

### Domain components

**Location:** `packages/ui/src/web-components/domain/`.

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
   `packages/ui/src/tokens.css`. No TS token constants. No
   per-component color tables that duplicate the canonical palette.
7. **No inline styles via `innerHTML`.** Use Lit `css` templates.
8. **No un-tested components.** PR is incomplete without colocated
   tests. Applies equally to primitives and domain.
9. **No components without a header comment** describing: what it is,
   its attrs, its slots, its events. One block comment at the top of
   the file. Used by documentation generators and by future agents.
10. **No new pnpm workspace packages** without explicit design approval.
    Everything web-component lives in `@ship/ui`.

## Reviewing existing components

When this charter lands, five components already exist in
`packages/ui/src/web-components/`: `badge.ts`, `empty-state.ts`,
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
through `packages/ui/src/web-components/index.ts` and
`packages/ui/src/index.ts`.

## Machine-readable catalog (Custom Elements Manifest)

The full, current list of `<ship-*>` elements — with every attribute's
type and default, every slot, every event's `detail` shape, and every
exposed CSS custom property — is generated at build time and lives at:

- **Repo:** `packages/ui/custom-elements.json`
- **Daemon URL:** `GET /api/runtime/assets/custom-elements.json`

This is the canonical reference for both humans and agents. IDE tooling
(VS Code, JetBrains, lit-plugin, lsp-cem) consumes it for attribute
autocomplete and type checking of HTML. Agents authoring daemon views
should read it to ensure emitted markup is valid.

**Regenerate** after any change to a `<ship-*>` element:

```
pnpm -F @ship/ui manifest
```

The `build` script runs this automatically.

The manifest is bundled into this skill's context so agents reading the
skill can answer "what attrs does `ship-button` accept?" without a
network round-trip. When the manifest diverges from what you see here,
trust the manifest — it's the source of truth.

## When this charter changes

- Changes require a design discussion, not an inline edit.
- New fixed-vocabulary values need a charter amendment first, then
  propagation to all existing components that use the affected axis.
- The decision rule for primitive-vs-domain is the most sensitive part.
  Propose, discuss, then amend.
