---
group: Set Framework
order: 8
title: Implementation status
description: What's stable, what's in active development, what's not yet implemented. Honest accounting of the gap between the architecture in this skill and the code on disk.
---

# Implementation status

The architecture documented in this skill is the **target** shape. Several pieces are not yet in code, several others exist in a precursor form that doesn't match the doc, and a few are stable. This page is the honest map.

Updated as the implementation moves. If you find a drift between this page and reality, fix the page or fix the code.

## Stable (matches the doc)

- **`.html.set` template directive set** — `<state>`, `<data>`, `<component>`, `<prop>`. Parsed by `crates/ship-set-compiler/`.
- **Control flow** — `{#if}`, `{#for key=}`, `{#match}`, `{#await}`. Parsed and emitted; runtime support in `@ship-set/runtime` and `@ship-set/dom`.
- **Bindings** — `on:event`, `bind:value`, attribute interpolation, text interpolation. Stable.
- **Slots** — light-DOM resolution at mount. Stable.
- **`<while-loading>` / `<while-error>`** — convenience boundaries. Stable.
- **AOT compiler** — `crates/ship-set-compiler/` emits ESM + CEM + JSON schema. Stable.
- **Live envelope shape** — `LiveState<T> = { kind: 'loading' } | { kind: 'error', error: Error } | { kind: 'ok', value: T }`. Stable in `@ship-set/runtime` (`live.ts`) and `@ship-set/sdk-client`; consumed by views.
- **Envelope unwrap helpers** — `unwrap(sig): T | undefined` and `unwrapOr(sig, fallback): T` exported from `@ship-set/runtime`. Replaces the per-derived `s.kind === 'ok' ? s.value : DEFAULT` boilerplate.
- **Reactive `<data>` source args** — daemon `/_ship-set/data?expr=...` accepts JSON-object args (`ship.git.diff({"commit":"abc"})`); SDK methods take typed args and serialize them into the URL.
- **Reactive re-subscribe** — `<data live>` source thunks receive `(ship, route, $scope)`; reading any signal inside the thunk is tracked by `live()`. When a tracked dep changes, the prior SSE subscription disposes and a new one opens with the updated args. Authors write `<data source="ship.X.Y({id: $scope.selectedId()})" live />` and the channel re-binds automatically when `selectedId` changes. Hydrate orders state-mount before data-mount so state signals are in scope by the time `<data>` evaluates.
- **Controller writable-proxy diagnostic** — when a handler writes `s.X` for a key not declared as `<state name="X">` in the template, the runtime warns explicitly (was silent failure). Catches the mix-up at first run instead of debug session #3.
- **Declarative-only state** — `<state name="X" value="..." />` in the template is the single state primitive; the controller's `state:` section was removed (see [the ADR](../../../ship-adrs/references/docs/ship-set-declarative-state.md)). Five-section controller, one declaration site, SSR walker reads initial scope from the IR alone.
- **Daemon auto-links `view.css`** — the HTML shell at `/set/<name>` includes `<link rel="stylesheet" href="/_ship-set/views/<name>/view.css">`. Authors no longer hand-write the URL in their templates.
- **Template-scope generic on `controller<Scope>`** — declare the template-declared identifiers once as a `Scope` type, pass as the single generic, every section's destructure is typed without per-section annotations. `MutableState<Scope>` and `ReadonlyState<Scope>` are derived from `Scope` directly: scope keys whose values are `Signal<T>` become the writable state surface. Compiler-emitted `.d.ts` is future work; the manual single-declaration pattern is what ships today.
- **Three-role architecture as a typed contract.** `Controller`, `TemplateHook`, `ControllerHook`, `MutableState`, `ReadonlyState`, `ViewScope` in `@ship-set/runtime` (`controller.ts`). Typed exports at the index. 17 type-level + behavior tests in `controller.test.ts`.
- **Five-section controller (derived / on / use / mount / unmount).** Shape is enforced by `ControllerShape<Scope>`. State is declarative-only — declared in the template via `<state>`, never on the controller. All five sections wired by the resolver and hydrate.
- **Three-tier controller (no controller / object / function).** All three accepted by `controller(def)` and the resolver. Tier 1 (no `.ts`) handled by hydrate's existing template-only path.
- **`controller(def)` runtime helper.** Wraps a def in a `{ setup, __def }` export drop-in compatible with `view()`. Daemon hydrate path unchanged. In `@ship-set/runtime` (`controller-resolver.ts`).
- **Resolver — derived / on / use.** `resolveController(def, ctx)` consumes template-state signals from `ctx.scope` (identified by `ctx.stateKeys`), memoizes deriveds, namespaces `use:` hooks, wraps `on` handlers with a writable proxy whose `set` trap calls `.set()` on the underlying state signal and falls through to ambient scope (ship/route/navigate/template-data) on reads. Unit tests in `controller-resolver.test.ts` + e2e through hydrate in `controller-hydrate.test.ts`.
- **Lifecycle — `mount` / `unmount`.** Resolver attaches lifecycle as a non-enumerable `__shipSetLifecycle` property on the bindings record (won't leak into template scope). `hydrate` reads it, calls `mount(state, container)` after render, and calls `unmount(state)` before disposal. Errors in user code are caught and logged. The lifecycle's `state` is the writable proxy — handlers and lifecycle hooks see the same writable shape.
- **Per-section scope destructure.** Each controller section receives the merged scope. `derived` and hook factories see `{ ship, route, navigate, ...templateData, ...stateView, ...derivedView }`. Handlers see a writable proxy with the same fallthrough.
- **Mutation discipline as a type fact.** `derived` signature uses `ReadonlyState<Scope>` (callable getters over template-state keys); `on` handlers use `MutableState<Scope>` (assignable values for the same keys). Both types are computed from the `Scope` generic by selecting the keys whose values are `Signal<T>`. The compiler refuses `s.count = 5` in a derived without writing the right cast.
- **`packages/ship-set-components/`** with **18 ported primitives** — `set-status-dot`, `set-spinner`, `set-badge`, `set-chip`, `set-card`, `set-link`, `set-divider`, `set-skeleton`, `set-stat`, `set-mark`, `set-kbd`, `set-avatar`, `set-progress`, `set-button`, `set-tabs`, `set-tab`, `set-empty-state`, `set-input`. Each is shadow-DOM-by-default with scoped tone-matrix CSS. Bundled at `apps/shipd/assets/ship-set-components.js` and served at `/_ship-set/components.js`; the shell loads it side-effect so every `<set-*>` tag is defined before any view renders. Authors prefix ports with `set-` during the migration so they coexist with the unported Lit primitives — see the [ship-set-prefix-during-migration ADR](../../../ship-adrs/references/docs/ship-set-prefix-during-migration.md).
- **Component port helpers** — `defineComponent(template, { styles? })` from `@ship-set/components` splices CSS into a `<component>` block before hydrate, so authors can keep big stylesheets in sibling `.css` files. Test utilities (`createContainerRegistry`, `mount(reg, tag, attrs, innerHTML?)`, `flush()`) collapse the per-component test boilerplate. Scaffolder script at `scripts/scaffold-set-component.mjs <tag>` generates the trio + reminds about the index.ts wiring.
- **Architecture decision records** — three ADRs in `docs/ship-adrs/references/docs/` capture the load-bearing design decisions: `ship-set-three-role-architecture.md` (the spine), `ship-set-prefix-during-migration.md` (set-* naming), `ship-set-deferred-design-decisions.md` (alternatives considered and rejected, so future contributors don't re-litigate without a forcing function).
- **`views/agents/` ported to the new shape.** Subscription declared in template (`<data name="agents" source="..." live />`); controller has `derived: { items, total }` and `on: { newAgent }`. Bundled and served at `/set/agents`.
- **Controller hooks (v1).** `useFilteredList`, `useSelection`, `useForm` ship from `@ship-set/runtime`. 13 unit tests in `test/hooks/`. Each is a load-bearing UI pattern slice — no pre-emptive `usePagination`/`useSort`/`useTable`.
- **Template hooks (v1).** `<use-kbd>`, `<use-click-outside>`, `<use-shell>` ship from `@ship-set/runtime`'s template-hook registry; `@ship-set/dom` lifts `<use-*>` elements at parse time and invokes the registered factories during hydrate (after setup, before render). 6 e2e tests in `test/template-hooks.test.ts`.
- **Tree-sitter grammar + tmLanguage** — at `grammars/tree-sitter-set/` and `editors/vscode/`. Stable.
- **LSP** — `packages/ship-set-lsp/` ships diagnostics, hover, completion. Stable for the directives + bindings it knows about. Recognizes `controller(def)` exports (Form C in `setup.ts`) — pulls section keys (derived / on / use) into the template's scope-resolution. State identifiers come from the template's `<state>` directives via `parsed.state`, not the controller def. Doesn't yet know `use:` *namespacing* (so `list.filtered` reads as a bare identifier reference inside templates).
- **Eval rig** — `packages/ship-set-eval/` runs jsdom probes. Stable for the six task fixtures (easy/medium/hard). Prompt teaches `controller(def)` + the v1 hook surface; a controller-shape few-shot is included.

## In active development

- **Rust compiler awareness of `<use-*>`.** The Rust AOT compiler at `crates/ship-set-compiler/` doesn't yet lift `<use-*>` elements to its IR — only `<data>`, `<state>`, `<component>`/`<prop>` are recognized as directives. Today this isn't an issue because the daemon parses `.html.set` source via the JS dom interpreter, not the AOT output. Becomes a gap when the compiler's output enters the production path.
- **Eval rig grades against the new shape.** Prompt teaches `controller(def)` and the new section vocabulary; few-shots include both pure declarative and a controller-shape example. `usePagination` / `useSort` / `useTable` controller hooks are deferred — added the first time a real view needs them, not before. Same for `<use-title>`, `<use-clipboard>`, `<use-focus-trap>` template hooks.
- **Renderable expression subset.** Documented as a contract; the compiler doesn't yet warn on non-renderable expressions because there's no SSR target to render against.
- **Route-param reactivity.** `route` today is passed to source thunks as a plain `Record<string, string>`. Reading `route.params.id` from a source thunk does NOT track — only `$scope`-scoped signal reads do. To re-fire on route-driven id changes, the daemon shell needs to populate `route` as a record of signals (or a transparent reactive proxy) and update them when navigation occurs. State-driven reactivity already works.
- **Compiler-emitted `.d.ts` per `.html.set`** — the manual `Scope` declaration lands the type-inference win, but the long-term answer is auto-derivation from `<data>` / `<state>` / `<use-* as=>` declarations. Not yet wired.
- **LSP awareness of `controller()` semantics.** LSP recognizes the export shape and extracts top-level section keys (good — `s.count`, `incr`, `total`, `list` all resolve in template scope). It doesn't yet:
  - **Namespace `use:` keys** — `list.filtered` reads as `list` not `filtered-on-list`, so deep references aren't checked.
  - **Refuse `s.count = 5` in a derived** — the controller types reject it but the LSP doesn't surface set-aware diagnostics for it yet.
  - **Complete handler names** — completion lists the section keys but doesn't yet know they're handlers (no signature hint, no return-type hint).

## Not yet implemented

- **Server-side rendering.** The Rust render walker over the IR does not exist. `shipd` today serves an empty HTML shell that loads `@ship-set/runtime` + `@ship-set/dom` + `@ship-set/sdk-client` + the view's compiled module, then parses the `.html.set` source client-side via `@ship-set/dom`. This is strictly worse than the j2 pipeline on first paint, byte size, and architectural coherence; it's a Phase 3 placeholder. The doc's SSR contract is the target.
- **The attach-only client runtime.** Once SSR lands, `@ship-set/dom`'s production role goes away; the runtime becomes attach-only. Today's runtime constructs DOM from parsed source — wrong shape for the target.
- **Marker contract.** The exact serialization between server-rendered HTML and the client attach runtime is undecided. Open candidates: `data-set-*` attributes (Datastar-shape), opaque IDs + sidecar manifest, inline annotations. See "Open questions" below.
- **Custom-element shadow DOM opt-in.** `<component name="X" shadow>` attaches a shadow root and (when an inline `<style>` child is present inside `<component>`) injects the captured styles into it. Light DOM remains the default. The render pipeline appends component body into the shadow root for shadow components and into the host for light ones.

## Removed shapes (formerly here, now gone)

- **`view()` factory.** The original `view<Bindings>(setup)` factory is gone. Every shipped view (just `views/agents/`) is on `controller()`; the precursor wrapper, its tests, and the LSP's Form A type-arg extraction were removed.
- **`.ship` SFC three-block format** and **sail-era script-block syntax** (`prop X: T = default`, `mount {}`, `effect on X`) — never made it to a working compiler under the current framework. See the phased-out `ship-components` skill for what was there.

## Open questions

1. **Marker shape.** Datastar uses `data-*` attributes verbatim; we are not committing to that. Candidates:
   - `data-set-*` attribute family (Datastar-shape with our prefix, simple to author).
   - Opaque element IDs + sidecar JSON manifest (smaller HTML, more indirection, manifest needs to be served).
   - Inline annotations as comment markers (`<!-- s:on:click=increment -->` adjacent to the element).
   We need to pick one before the SSR walker lands.
2. **Renderable expression subset boundaries.** The doc lists what's in and out, but the compiler check is not yet implemented. Edge cases: optional chaining (`a?.b?.c`), nullish coalescing (`a ?? b`), array literals in expressions, object literals. Likely all in; explicit decision pending.
3. **`<data>` reactive `source=`.** Can the `source` attribute reference signals (`source="ship.agents.get(route.params.id)"` where `route.params.id` is reactive)? If yes, the subscription needs to re-fetch on parameter change. Behavior is unspecified.
4. **Controller hook composition helper.** Is there a `compose(...)` runtime helper, or does the runtime infer composition from the `use:` shape directly? Leaning toward "infer," but the API hasn't shipped.
5. **Custom-element shadow DOM opt-in syntax.** `<component name="ship-foo" shadow>` attribute, or a flag in the `<component>` declaration's metadata? Pending.
6. **`event NAME: T` typed CustomEvent declarations.** Documented in earlier ledgers; not yet in the implemented directive set. Open whether this lands in the template or the controller.
7. **Effect lifecycle.** `mount` runs once; `unmount` runs once; what about effects that run when a derived signal changes? Open question on whether to add `effect` (a seventh section) or rely on `derived` alone (deriveds run on dep change; if you want a side effect tied to a change, an effect is what you want, not a derived). Leaning toward deferring: `mount` + closures over signals can express most cases; if a real gap appears, revisit then.

## Phased-out adjacent skills

These docs remain the source of truth for the **current production stack** until views are ported off them. New work for ship-set goes here, not in those skills.

- **`docs/ship-views/`** — j2 templates + Datastar signals. Currently shipping; one view at a time gets ported off.
- **`docs/ship-components/`** — `.ship` SFC three-block format. Format is dead. Components ported to `.html.set` over time; pack-distribution mechanics survive (see ship-component-packs).
- **`docs/ship-component-packs/`** — packs of `.ship` components. Pack distribution survives; the `.ship` shape inside does not. Packs of `.html.set` components are the target.
