# Build a `<sui-counter>` component

Write the SUI primitive `<sui-counter>` under
`packages/components/primitives/src/sui-counter/`. The component shows a
numeric counter with **+** and **−** buttons and emits `change` events.

## Surface

- **Props:**
  - `value: number` (reflect), default `0` — the current count.
  - `min: number`, default `Number.NEGATIVE_INFINITY` — the lower bound.
  - `max: number`, default `Number.POSITIVE_INFINITY` — the upper bound.
  - `step: number`, default `1` — increment/decrement amount.
- **Events:**
  - `change` with `detail: { value: number, previous: number }` —
    fires on every change, both via buttons and external prop set.
- **Slots:**
  - Default slot — optional label rendered between the buttons.
- **Style:**
  - Inline-flex row, 8px gap between elements, `var(--tone-fg)` color.
  - Tone prop (`default | primary | accent`) drives `--tone-fg` per
    the canonical pattern (see `sui-icon` or `sui-button` for parity).

## Behavior

- Clicking **+** increments by `step`, clamped to `max`.
- Clicking **−** decrements by `step`, clamped to `min`.
- External `value` writes (via `<sui-counter value={n}>`) flow through
  the same event surface.
- When at the bound, the corresponding button is `disabled`.

## Acceptance

- `packages/components/primitives/src/sui-counter/sui-counter.sui` exists
  and compiles via `sui-compiler` (component mode auto-detected from path).
- Paired `sui-counter.controller.ts` is optional — only if the
  bound-disabled state needs derived state. Templates can declare
  `disabled={value() <= min()}` directly without a controller.
- The `change` event fires when buttons clicked; payload shape matches.
- `bun run --filter @ship/primitives build` succeeds.
- No `setAttribute('value', …)` calls in any controller (sui-lint
  catches; runtime error if it slipped through).
- No `state.x.set(…)` writes (sui-lint catches; runtime error post-#179).

## Write to

`packages/components/primitives/src/sui-counter/sui-counter.sui`
(and `sui-counter.controller.ts` only if derived state warrants it).
