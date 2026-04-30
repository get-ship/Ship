---
title: "Ship Set — State is Declarative-Only (`<state>` is the Single Primitive)"
description: "ADR: state lives in the template via `<state>` directives. Controllers do not have a `state:` section. Six controller sections become five."
group: ADRs
order: 54
audience: internal
status: accepted
date: 2026-04-30
supersedes: []
tags: [ship-set, framework, controller, state, ssr]
---

# Ship Set — State is Declarative-Only

## Context

The framework currently has two ways to declare local state:

1. **Template `<state name="count" value="0" />`** — a directive that contributes a writable signal to the view scope. Stable; supported by the Rust compiler's IR.
2. **Controller `state: { count: 0 }`** — a section of the controller object that declares writable signals via the resolver.

The two paths overlap. Resolution when both declare the same key is unspecified — the docs flag it as an open question — and a writable-proxy diagnostic exists at runtime to nudge authors when a handler writes to a key that lives in template `<state>` rather than controller `state:`. That diagnostic is itself evidence of a smell: the framework has to explain to authors which declaration site to use.

Two larger forces push the decision now:

1. **The three-role discipline says "templates declare; controllers do."** State *is* a declarative concern — a name and an initial value. It belongs on the template side of that line; placing it in the controller crosses the boundary.
2. **The SSR walker (Phase D) needs initial scope values from the IR.** When state lives in the template `<state value="...">`, the Rust compiler already has the defaults — the IR carries them. When state lives in the controller's `state: { ... }` literal, the Rust compiler would have to parse the paired `.ts` file (or run a separate JS extraction step, or embed a JS engine in `shipd`) to read the defaults. Eliminating that path is a major SSR simplification.

## Decision

**The `<state>` directive is the only state primitive.** The controller has no `state:` section.

The controller's six sections collapse to five:

- `derived` — memos over scope
- `on` — handlers (the only mutators)
- `use:` — composable hook bundles
- `mount` — post-render lifecycle
- `unmount` — pre-disposal lifecycle

State always lives in the template. Both the no-controller tier and the with-controller tier declare state the same way:

```html
<!-- view.html.set -->
<state name="count" value="0" />
<state name="filter" value="''" />
```

The controller reads from scope (already true), derives over it, and mutates via handlers — but does not declare state.

### Computed-initial cases

Cases where the initial value isn't a literal — `crypto.randomUUID()`, an SDK call, a derivation over `route` — move to `mount`:

```html
<state name="id" value="''" />
```

```ts
export default controller({
  mount: ({ state }) => {
    state.id = crypto.randomUUID()
  },
})
```

This is one extra line versus the controller `state: { id: () => crypto.randomUUID() }` shape, and it's the right place for it: non-literal defaults are imperative, and `mount` is where the controller's imperative work runs. The previous shape blurred the line between "default value" (declarative) and "computed value" (imperative).

### What the writable-proxy diagnostic now warns about

The runtime warning that today fires when a handler writes a key not in controller `state:` continues to exist; its message is rephrased to reference the `<state>` directive instead. It now says: "handler tried to write `s.X = …` but `X` is not declared as `<state name="X">` in the template; the assignment was ignored."

## Rationale

1. **One way to do things.** The two declaration paths created an authorship choice that didn't carry information. Authors (especially agent authors) had no signal for picking one over the other; the result was scattered conventions.
2. **Cleaner mental model.** "Templates declare data and structure; controllers declare behavior." State is data. It belongs on the data side of the seam.
3. **SSR walker simplification.** The Rust render walker reads `<state value="...">` directly from the IR. No TS parsing, no JS engine in `shipd`, no separate AOT extraction step. Initial scope is one IR walk away.
4. **Composition is unaffected.** Hooks declare their own internal signals via `useFoo()`'s setup function. Those were never controller `state:` keys. The `useForm`, `useFilteredList`, `useSelection` v1 surface is unchanged.
5. **Resolution ambiguity disappears.** No "what if both declare the same key" question because there is one declaration site.

## Alternatives considered

### Keep both declaration sites with an explicit precedence rule

Decide that controller `state:` wins when both sites declare the same key (or template wins, or it's an error). Document the rule.

**Rejected.** Even with a clear rule, the two-path shape persists; authors still face the same authorship choice with no informational basis for picking. The rule becomes one more thing to remember without buying coherence. Worse, the SSR-walker simplification — the second-largest reason for this decision — is unavailable as long as controller-side state exists.

### Drop template `<state>`, keep only controller `state:`

The opposite direction. State always lives in the controller; templates with no controller can't have local state.

**Rejected.** Breaks the no-controller tier — the small declarative view (the `counter.html.set` archetype) becomes impossible without a paired `.ts`. The framework's "tiered controller" promise (no controller / object / function) loses its first tier. And the SSR walker would still need a way to extract defaults from controller TS, which is exactly the path this ADR is trying to remove.

### Add a third path: a top-level `state` block in `.html.set` adjacent to `<data>` etc.

A single `<states>` section listing all state keys, distinct from individual `<state>` elements. More like Vue's `<script setup>` but in the template.

**Rejected.** Solves nothing the existing `<state>` directive doesn't already solve; introduces a new shape; makes ports between views less mechanical (every state migration becomes a new section to add). Yet another concept where one already worked.

## Consequences

### Positive

- **Five-section controller is mentally simpler.** `derived / on / use / mount / unmount` is the entire shape.
- **SSR walker reads initial scope from the IR alone.** No TS toolchain dependency.
- **No resolution rule needed.** One declaration site.
- **Aligns with the three-role discipline.** "Template owns data; controller owns behavior" is more literally true.

### Negative

- **Computed-initial cases require a `mount` line.** An `id: () => crypto.randomUUID()` shorthand becomes a `<state name="id" value="''" /> + state.id = crypto.randomUUID()` pair. One extra line per such case. Acceptable; the case is uncommon, and the line is in the imperative section where it belongs.
- **Existing controllers using `state:` need migration.** Today the only ported view (`views/agents/`) uses `controller<{}, Scope>` (empty state) so no shipping view is affected, but the eval-rig prompt, examples doc, hook tests, and resolver tests all reference the section. Migration is mechanical but touches several files.
- **Type signatures change.** `MutableState<S>`, `ReadonlyState<S>`, `MountSection<S>` all parameterize over the state shape; with state declarative-only, that shape is now derived from the template's `<state>` declarations rather than a controller-side type argument. The `Scope` generic on `controller<S, Scope>` collapses to `controller<Scope>`. LSP and compiler-emitted `.d.ts` (when it lands) need to reflect this.

### Open questions

- **Shorthand for boolean state.** `<state name="open" value="false" />` is the canonical shape. Should there also be `<state name="open" boolean />` or similar? Defer until the ergonomics show up as a real complaint; the value="..." form works.
- **State scoped to a `{#for}` iteration.** Each iteration gets its own `<state>` instance? Today this isn't formalized; defer to the iteration-state ADR when one is needed.

## References

- [Three-role architecture ADR](./ship-set-three-role-architecture.md) — the framework spine.
- [Architecture](../../../ship-set-framework/references/docs/index.md) — the three-role split.
- [Controller doc](../../../ship-set-framework/references/docs/controller.md) — to be updated to reflect the five-section shape.
- [Implementation status](../../../ship-set-framework/references/docs/status.md) — Phase D (IR render walker) is the SSR-side beneficiary.
- Marker contract ADR: `ship-set-marker-contract.md` — the SSR seam this decision unblocks.
