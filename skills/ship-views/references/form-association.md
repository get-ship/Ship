# Form-aware SUI components

Depth reference for `ship-views` and `ship-components`. The contract lives in those skills' SKILL.md files.

## When to use this

When a `<sui-*>` component is itself a form control (input, textarea, checkbox, switch, select, file upload, …) the browser needs to discover it as a form participant so:

- `new FormData(form)` includes its value
- `form.reset()` resets it
- bfcache / back-forward-cache restoration repopulates it
- HTML5 validation (`:invalid`, `setCustomValidity`) flows through it
- The form's `disabled` state cascades

Without form association, none of that happens — `<sui-input>` inside `<form>` is invisible to submission.

## The contract

In the component's `.sui` file, mark the value prop with `form-value` and (optionally) the disabled prop with `form-disabled`:

```html
<prop name="value" type="string" default="" form-value />
<prop name="disabled" type="bool" default="false" reflect form-disabled />
```

That's the entire author-side surface.

## What the compiler emits

The compiler generates `component({...})` with these additions:

```ts
component('sui-input', {
  formAssociated: true,        // ← emitted because of form-value
  formValue: 'value',          // ← which prop drives setFormValue
  formDisabled: 'disabled',    // ← which prop tracks form-disabled-cascade
  props: { /* … */ },
  setup(ctx) { /* … */ },
})
```

## What the runtime auto-wires

When `defineComponent` sees `formAssociated: true`, it sets `static formAssociated = true` on the generated host class, calls `host.attachInternals()` once on connect, and installs the four platform lifecycle methods:

| Hook | What it does |
|---|---|
| Per-change effect | Calls `internals.setFormValue(String(<value>))` whenever the marked value prop's signal changes |
| `formResetCallback()` | Resets the value prop to its declared `default` |
| `formStateRestoreCallback(state, mode)` | Decodes `state` through the prop's decoder and writes it back to the signal |
| `formDisabledCallback(disabled)` | Writes `disabled` into the marked disabled prop's signal so descendants react |

## Controller access to `internals`

The setup ctx still exposes `internals?: ElementInternals` when the component is form-associated. Controllers can use it for paths the auto-wiring doesn't cover:

```ts
// sui-input.controller.ts
import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-input.sui'

export default controller<Scope>({
  mount(state, host) {
    // internals is available because <prop form-value> is declared
    const internals = (host as any)._internals as ElementInternals
    effect(() => {
      const v = state.value()
      if (state.required() && !v) {
        internals.setValidity({ valueMissing: true }, 'Required')
      } else {
        internals.setValidity({})
      }
    })
  },
})
```

`setValidity` / `checkValidity` are not auto-wired — they need component-specific logic. The runtime owns the four platform lifecycle hooks; controllers own custom validity.

## Constraints (compiler-enforced)

- **One value prop per component.** Multiple `form-value` directives is a compile error.
- **Form-value type:** `string` / `number` / `bool` / `union`. Rejected on `object` / `array` (you can't submit those as a single value).
- **Form-disabled type:** `bool` only.
- The two flags can coexist on the same component but on different props.

## Worked examples

- `packages/components/ui-components/src/sui-input/sui-input.sui` — single value, basic case.
- `packages/components/ui-components/src/sui-textarea/sui-textarea.sui` — same shape.
- `packages/components/ui-components/src/sui-checkbox/sui-checkbox.sui` — `form-value` on `checked` (bool); submits `'true'` / `'false'`. Hosts wanting HTML-checkbox semantics (value present only when checked) call `internals.setFormValue` themselves.
- `packages/components/ui-components/src/sui-switch/sui-switch.sui` — same pattern as checkbox.
- `packages/components/ui-components/src/sui-select/sui-select.sui` — `form-value` on `value` plus light-DOM `<option>` sync.

## Implementation reference

Internals on the framework side:

- `crates/sui-compiler/src/ir/directive.rs` — `form_value` / `form_disabled` IR fields on `PropDirective`.
- `crates/sui-compiler/src/lower/directives.rs` — lowering + validation rules.
- `crates/sui-compiler/src/emit/js.rs` — emits `formAssociated: true` + `formValue` / `formDisabled` keys.
- `crates/sui-compiler/src/emit/schema.rs` / `emit/dts.rs` / `emit/cem.rs` — parallel emitters mirror the contract.
- `packages/sui/runtime/src/component-factory.ts` — reads the keys.
- `packages/sui/runtime/src/component-form.ts` — installs the lifecycle hooks.
- `packages/sui/runtime/src/component.ts` — `defineComponent` sets the static flag on the generated class.
