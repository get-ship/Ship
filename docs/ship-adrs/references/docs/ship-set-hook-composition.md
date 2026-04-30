---
title: "Ship Set — Hook Composition is the `use:` Record (No `compose()` Helper)"
description: "ADR: the `use: { ... }` controller section IS the composition mechanism. No imperative `compose(...)` combinator ships."
group: ADRs
order: 55
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, hooks, controller, framework]
---

# Ship Set — Hook Composition is the `use:` Record

## Context

Controllers compose hook bundles via the `use: { ... }` section. Each entry is a hook factory; the resolver namespaces each by its key and merges the contributions into scope:

```ts
controller<Scope>({
  use: ({ agents }) => ({
    filter: useFilteredList(agents, (a, q) => a.name.includes(q)),
    sel:    useSelection(agents, (a) => a.id),
  }),
  // ...
})
```

The template references `filter.filtered`, `filter.query`, `sel.selected`, etc. Hooks compose by *coexistence* in the same record.

The open question — flagged in `status.md` for several phases — was whether to ALSO ship an imperative combinator:

```ts
const useFormWithValidation = compose(useForm, useValidation)
```

…that stacks two hooks into one. The `compose()` would let library authors ship higher-order hooks without leaking implementation detail to the consuming controller.

## Decision

**No `compose()` helper.** The `use: { ... }` record is the only composition mechanism the framework provides.

If a library wants to ship a "form with validation" bundle, it ships a single `useFormWithValidation` factory whose body internally composes — by direct invocation, not via a framework helper. The framework's surface stays minimal.

## Rationale

1. **The use-case is hypothetical.** No shipped controller uses more than 2–3 hooks. The number of hooks per controller has not grown to where stacking is repetitive enough to want a helper.
2. **The `use:` record is already composition.** It namespaces, it injects scope, it disposes correctly on unmount. Adding `compose()` would be a parallel system to do the same job in a different shape — the kind of choice the three-role discipline rejects.
3. **Hook authors can compose internally.** A "form with validation" hook that needs both `useForm` and `useValidation` underneath is a regular function call — `useForm` is just a function. No framework primitive needed.
4. **Type inference favors records.** TypeScript infers the contributions of a record-typed `use:` better than it infers the result of a higher-order combinator. Combinators tend to widen types in ways the LSP can't unwind cleanly.
5. **Reversibility.** If a real consumer arrives that genuinely needs `compose()`, adding it is a non-breaking addition. Removing `compose()` once shipped is much harder. Defer until a forcing function exists.

## Alternatives considered

### Ship `compose()` now, optimistically

Provide the helper for symmetry with React's `compose`-style HOCs and Redux middleware. Authors can use it if they want; the cost is just the helper's existence.

**Rejected.** "Cost is just existence" is wrong for framework primitives — they shape author intuition. Two ways to compose hooks (record vs `compose`) means agents and humans both pick one and the framework's discipline drifts. The `set-authoring` skill would have to teach when each is right.

### Add a `compose()` helper but only document `use:`

Ship the helper for advanced cases without putting it in the canonical examples. Lets advanced library authors stack without bloating the framework spec.

**Rejected.** Hidden API is API. Once it exists, it gets used; the framework loses control of its own discipline. Either fully support it or don't ship it.

### Deeper composition via hook-of-hooks shape

Instead of a top-level `compose()`, allow `use:` entries to themselves contribute `use:` entries (nested). The resolver flattens them at the bindings level.

**Rejected.** Adds a runtime expansion step the resolver doesn't need today. The same outcome can be achieved by a hook factory calling another hook factory inside its body — no framework support required.

## Consequences

### Positive

- One way to compose. Scope contributors are flat-merged from the `use:` record's keys; no surprise hidden compositions.
- Framework surface stays minimal. The `use:` section is the only composition concept authors learn.
- Type inference works without the LSP needing to unwind a combinator's type machinery.

### Negative

- Library hooks that internally use 2–3 sub-hooks have to inline the composition in their body. Slight duplication if multiple library hooks share the same internal stack — but the duplication is real-code (a function call) rather than framework-API (a primitive).
- If a consumer arrives that genuinely needs `compose()`, the answer is "ship a higher-order hook directly," which may feel less elegant than the React-style API. Acceptable trade.

### Open questions

- **Reopen criteria.** If three or more shipping controllers stack the same internal hook combination AND the inline composition becomes a measurable readability problem, reopen this ADR. Document the forcing function in the new ADR.

## References

- [Three-role architecture ADR](./ship-set-three-role-architecture.md) — the discipline that rejects parallel mechanisms.
- [Controller doc](../../../ship-set-framework/references/docs/controller.md) — current `use:` semantics.
- [Hooks doc](../../../ship-set-framework/references/docs/hooks.md) — controller-hook authoring contract.
