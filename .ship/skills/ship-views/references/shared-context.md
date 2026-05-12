# Cross-component reactive context (`shared` / `useShared`) and controller tiers

Depth reference for `ship-views`. The contract lives in `../SKILL.md`.

## When to use shared / useShared

For complex components that split into a public outer + internal inner pieces, share signals from parent to descendants without prop-drilling.

## Provider

The parent wraps signals with `shared(...)` inside its `scope:` factory:

```ts
// parent.controller.ts
scope: () => {
  const mode  = shared(signal<BarMode>('find'))
  const value = shared(signal(''))
  const local = signal(0)  // not shared
  return { mode, value, local }
}
```

The runtime auto-publishes the marked signals on the host element under the host's tag name.

## Consumer

Descendants resolve the parent's scope inside their own `scope:` factory:

```ts
// child.controller.ts
import type { Scope as ParentScope } from '../parent/parent.sui'

scope: () => {
  const bar = useShared<ParentScope>('parent-tag-name')
  return { bar }
}
```

Templates bind `bar.mode()`, `bar.value`, etc. — same signal identity, two-way reactivity flows.

## Lookup semantics

`useShared` walks DOM ancestry crossing shadow boundaries. Returns `null` when no provider is found; descendants guard with `{#if bar}`.

Picking by tag name (not by string token) keeps the lookup deterministic: there's exactly one provider tag in any ancestry chain. Components that participate in shared-context have stable tag names.

## Controller tiers — smallest that fits

1. **No controller** — pure `.sui` with `<prop>` / `<data>` and single-expression inline handlers. No state, no derived, no hooks. Pure presentational.
2. **Paired controller** — anything with state, derived, hooks, lifecycle, or multi-statement handlers.

A multi-statement `on:event` in a tier-1 file (no `.controller.ts`) is a compile error. Enforced by `crates/sui-compiler/src/check/handler_complexity.rs`.

For controller-side hooks (`useFilteredList`, `useSelection`, `useForm`) and the live-data envelope, see `controller-tiers.md` (when it exists) or `template-surface.md` § "Controller hooks".
