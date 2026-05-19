# Job Spec: Missing SUI primitives — tag-input, combobox, autocomplete, date-picker, color-picker

**Branch:** job/missing-primitives
**Agent:** ship-designer
**Mode:** autonomous

## Goal

Build v1 of five missing `<sui-*>` primitives so the design system covers the
shapes a real product app needs (single-select with filter, free-text with
suggestions, chip input, date selection, color selection). Keep each primitive
tight — the existing `<sui-*>` set is the bar for visual quality, API shape,
and accessibility.

## File scope

Modify only:
- `packages/components/primitives/src/sui-tag-input/`     (new)
- `packages/components/primitives/src/sui-combobox/`      (new)
- `packages/components/primitives/src/sui-autocomplete/`  (new)
- `packages/components/primitives/src/sui-date-picker/`   (new)
- `packages/components/primitives/src/sui-color-picker/`  (new)
- `packages/components/primitives/test/sui-tag-input.test.ts`     (new)
- `packages/components/primitives/test/sui-combobox.test.ts`      (new)
- `packages/components/primitives/test/sui-autocomplete.test.ts`  (new)
- `packages/components/primitives/test/sui-date-picker.test.ts`   (new)
- `packages/components/primitives/test/sui-color-picker.test.ts`  (new)
- `packages/components/primitives/src/index.ts`                   (register the 5 tags)
- `crates/sui/sui-compiler/primitives.json`                       (regenerated, see Verify)

## What to change

Per-primitive shape (each is its own directory with `<name>.sui` + optional
`<name>.controller.ts` co-located, mirroring existing primitives like
`packages/components/primitives/src/sui-input/`, `sui-popover/`, `sui-chip/`).

1. **`sui-tag-input`** — chip composition. Type text → press Enter → add a chip.
   Backspace on empty input → remove the last chip. Pre-existing tags via `value`
   attribute (comma-separated string). Emits `sui-change` with the current array.
   Compose with the existing `<sui-chip>` for each chip.

2. **`sui-combobox`** — single-select where the user types to filter the option
   list. Options come from a default slot whose children carry `value` + label
   text (`<option value="x">Label</option>` shape, parsed by the controller),
   OR an `options` string prop containing JSON (the compiler only knows
   bool/number/string for `<prop>` types — parse JSON in the controller). ARIA:
   `role="combobox"` on the host, `aria-expanded`, `aria-controls`,
   `aria-activedescendant`. Keyboard: ↑ ↓ move highlight, Enter commits,
   Escape closes.

3. **`sui-autocomplete`** — text input with a suggestion dropdown. Differs from
   combobox: the value is free-text, suggestions are hints rather than the only
   valid values. Same keyboard model. ARIA: `role="combobox"` with
   `aria-autocomplete="list"`.

4. **`sui-date-picker`** — input field showing the formatted date + a popover
   with a month-grid calendar. Compose `<sui-popover>` for the popover surface.
   Value is ISO 8601 (`YYYY-MM-DD`). `min` / `max` props. Keyboard inside the
   popover: arrow keys to navigate days, Enter to select, Escape to close.

5. **`sui-color-picker`** — input field showing hex + a popover with: a
   swatch palette (8-12 colors derived from design tokens), an HSL slider trio
   (H 0-360, S 0-100, L 0-100), and a hex text input. Emit `sui-change` with the
   hex value. Single-color use — don't reinvent gradients or alpha.

DO NOT build: form orchestrator, data grid. Those need design decisions the
user is driving directly.

## Architectural context

- Authoring contract: `skills/ship-views/SKILL.md` + `skills/ship-components/SKILL.md`.
  The three-file shape, the `<prop>` declaration syntax, the rule that templates
  declare and controllers do.
- Token system: `skills/ship-design-system/SKILL.md` and
  `packages/ui-components/src/tokens/tokens.css`. All color/spacing/typography
  through `var(--token)` — the `sui-css-literal` compiler rule rejects raw hex /
  rgb / px values in themed properties as a hard error.
- Canonical enum vocab: `tone="default|primary|info|success|warning|destructive|accent|muted"`,
  `variant="solid|soft|outline|ghost"`, `size="xs|sm|md|lg|xl"` (or a sensible
  subset per primitive). The `sui-prop-value` compiler rule rejects unknown
  values as a hard error.
- Layout: compose `<sui-stack>`, `<sui-cluster>`, `<sui-grid>`, `<sui-center>`
  instead of writing bespoke flex CSS.
- Test pattern: mirror `packages/components/primitives/test/sui-stack.test.ts` —
  assert registration, shadow root, slot wiring, attribute reflection. Skip
  computed-style assertions; jsdom doesn't compute shadow CSS.
- Active ADRs: none specific to primitives.
- Key constraints: file cap 300 lines per source file. Split before exceeding.

## Acceptance criteria

1. All 5 primitive directories exist with `<name>.sui` + (if needed)
   `<name>.controller.ts`, each registered in `packages/components/primitives/src/index.ts`
   (alphabetical insertion).
2. Each primitive uses only design tokens for themed properties — no raw hex,
   rgb, hsl, or named colors. The `sui-css-literal` rule is Error severity
   today, so violations fail compile.
3. Each primitive declares enum-typed props using canonical vocabulary. The
   `sui-prop-value` rule is Error severity, so drift fails compile.
4. Five vitest specs exist at the test paths above, each at minimum asserting
   registration, shadow-root presence, default slot wiring, and reflected
   attribute round-trip.
5. `cargo run -p sui-lint --example regen_primitives` re-generates
   `crates/sui/sui-compiler/primitives.json` with the 5 new entries.
6. `timeout 90 cargo test -p sui-compiler --test shape_conformance --quiet`
   passes (every primitive compiles + typechecks against the runtime .d.ts).
7. `cd packages/components/primitives && timeout 60 npx vitest run test/sui-tag-input.test.ts test/sui-combobox.test.ts test/sui-autocomplete.test.ts test/sui-date-picker.test.ts test/sui-color-picker.test.ts`
   passes.

## Build hint

If the SUI compiler binary isn't already at `target/debug/sui-compiler`, build
it first with `cargo build -p sui-compiler --bin sui-compiler`. The
`packages/components/primitives/vitest.config.ts` shim compiles `.sui` files
on demand once that binary exists.

---

> Execute this job yourself. Do not delegate the implementation to a subagent
> — subagents are only for read-only parallel research, never for writing the
> code this job asks for.
>
> If you notice a bug or problem outside your file scope, log it via
> `mcp__ship__log_progress` and continue your work.
>
> **When all acceptance criteria are met:**
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary of what
>    changed.
> 2. Call `mcp__ship__mesh_broadcast` with
>    `{ "topic": "job.completed", "workspace_id": "<id>", "summary": "<same summary>" }`.
>    If shipd is not running, skip step 2 — step 1 is sufficient.
