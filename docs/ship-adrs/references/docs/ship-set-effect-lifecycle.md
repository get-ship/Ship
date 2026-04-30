---
title: "Ship Set — No Separate `effect` Controller Section (Mount + Hooks Cover It)"
description: "ADR: deliberate deferral of a 7th `effect` controller section. The five-section shape (derived/on/use/mount/unmount) plus the `useEffect` controller hook covers the use cases without expanding the framework's surface."
group: ADRs
order: 57
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, controller, lifecycle, framework, deferred]
---

# Ship Set — No Separate `effect` Controller Section

## Context

The five-section controller (`derived` / `on` / `use` / `mount` / `unmount`) handles state, computation, behavior, composition, and lifecycle. The framework's docs flagged a long-standing question: should there be a 7th — `effect` — section for "side effects that run when a signal changes"?

The motivating use case: an author wants to do something side-effecty (write to localStorage, log analytics, call a debounced fetch) when a particular state value changes. Today's options:

1. Read the signal inside `mount`'s closure body. The signal's reactivity tracks the read, and the closure re-runs when the signal changes. But cleanup of intermediate state is awkward.
2. Wrap the side effect in a `derived` whose value is irrelevant. Side effects in deriveds are an anti-pattern; the framework's discipline rejects this.
3. Build a `useEffect` controller hook that takes a thunk + deps and handles re-run + cleanup. Composes via the `use:` section.

Option 3 is what React does (with hooks). Option 1 is what Solid/Svelte do. Option 2 is what nobody should do.

## Decision

**Defer the 7th `effect` section. The framework keeps five sections.** The combination of `mount` + closures + a (to-ship) `useEffect` controller hook covers the documented use cases.

The `useEffect` controller hook becomes part of the v1 hook surface alongside `useFilteredList`, `useSelection`, `useForm`. Its shape:

```ts
import { useEffect } from '@ship-set/runtime'

controller<Scope>({
  use: ({ count }) => ({
    persist: useEffect(() => {
      const v = count()
      localStorage.setItem('count', String(v))
      return () => { /* optional teardown */ }
    }, [count]),
  }),
})
```

The hook handles the re-run-on-dep-change semantics; cleanup runs before each re-execution and at unmount. No new framework section.

## Rationale

1. **The five-section shape is freshly stabilized.** The declarative-state ADR just collapsed six sections to five; adding a 7th immediately would re-expand the surface that just got pruned.
2. **`useEffect` as a hook composes the existing primitives.** Hooks can read signals, register cleanup, run on changes — everything the proposed `effect` section would do — without adding a section.
3. **Composability matters more than convenience.** A hook can be combined with other hooks in `use: { ... }`. A section can't compose; it stands alone. The hook shape is more powerful for the same expressiveness cost.
4. **Authors who think "effect" should reach for `useEffect`.** The naming maps onto the React intuition; the implementation differs (signals vs render dependency tracking) but the API surface is familiar.
5. **Deferral is reversible.** If a real view shows that `useEffect`-as-hook is consistently more verbose than a hypothetical `effect` section, reopen the ADR and add the section. The framework can always grow.

## Alternatives considered

### Ship `effect` as the 7th section now

Add `effect: { someSideEffect: ({ count }) => { /* runs when count() changes */ } }`.

**Rejected** because:
- The five-section spec is the freshly-shipped reduction. Re-expanding it without a forcing function is regression.
- The same expressiveness comes from `useEffect` as a hook, with strictly more composability (multiple effects chain via `use:`; multiple section keys don't compose at all).
- Section authors can't share effects across controllers; hook authors can. Hook is the right shape.

### Defer entirely; document mount + closures as the only path

Don't ship `useEffect` either. Tell authors to read signals inside `mount`'s closure for re-run-on-change behavior.

**Rejected.** The closure-only path doesn't handle clean teardown across re-runs cleanly. `useEffect` handles it; the gap is real.

### Add `effect` AND `useEffect`

Both, for redundant ergonomics.

**Rejected.** Two ways to do the same thing violates the discipline. Pick one. The hook is the right one.

## Consequences

### Positive

- Five-section controller stays five.
- Effect behavior is composable via `use: { ... }`.
- Authors familiar with React's `useEffect` find the API immediately recognizable.
- Cleanup semantics are explicit (the returned function from `useEffect`'s thunk) — no implicit teardown ordering.

### Negative

- A `useEffect` declaration is slightly more verbose than a hypothetical `effect` section entry. The trade is tolerable; verbosity is bounded by the hook's parameter shape.
- Authors who intuit "effect lifecycle" may look for an `effect` section first and need to be redirected to the hook. Documentation must address this.

### Implementation status

`useEffect` is **not yet shipped** as part of `@ship-set/runtime`. Adding it is a small follow-up (similar shape to `useSelection` / `useFilteredList`). The shipping target lands as part of Arc D's "v1.5 hooks" or the next reactivity arc.

### Open questions

- **Reopen criteria.** If three or more shipping controllers each have ≥2 `useEffect` invocations AND the verbosity vs `effect` section becomes a measurable readability problem, reopen this ADR. Document the forcing function in the new ADR.
- **Effect ordering.** Multiple `useEffect` calls in one controller — does ordering matter? The hook's deps array isolates each effect's dependency graph; the framework runs them in declaration order. This is a hook-implementation detail, not a framework decision.

## References

- [Three-role architecture ADR](./ship-set-three-role-architecture.md) — discipline that prefers hooks over new sections.
- [Declarative-state ADR](./ship-set-declarative-state.md) — the five-section shape that stays five.
- [Controller doc](../../../ship-set-framework/references/docs/controller.md) — current section vocabulary.
- [Hooks doc](../../../ship-set-framework/references/docs/hooks.md) — controller-hook authoring contract.
