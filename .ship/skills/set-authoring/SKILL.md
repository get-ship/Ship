---
name: set-authoring
stable-id: set-authoring
description: Author single-file `.set` components and route pages — Ship's reactive web-component framework. Use this when creating or editing files under `packages/ship-set-sail/components/` or `.ship/sail/`, or when authoring a `/sail/<tag>` route page. Covers the consolidated directive surface (`<:host>`, `data` for live server signals, `prop`/`state`/`derived`, slot helpers, form lifecycle), injected scope, recipes, and active gotchas. Distinct from `ship-views` — that skill is for Jinja+Datastar `.html.j2` views; this one is for the SFC component framework.
tags: [set, sail, components, sfc, reactive, ports, framework]
authors: [ship]
---

# set — Component + Page Authoring

A `.set` file is a single-file component (script + template + style) that compiles to a custom element. Authoring is HTML-shaped with Solid-style fine-grained reactivity and a server-truth wire model (`data X = ship.foo.bar()` + SSE patches).

**Use this skill when:** opening or creating a file under `packages/ship-set-sail/components/`, `.ship/sail/`, or any `<tag>.set` file. For Jinja `.html.j2` views, use `ship-views` instead.

## First principles — the agent-one-shot filter

Every directive on this page earns its slot by passing the filter:

1. **Agent one-shot success** above all else. Authors who don't know the framework should produce correct output by writing the obvious thing.
2. **Use the platform.** Custom elements, native attribute names (`onclick={fn}`, not `on:click`), light DOM, `<slot>`.
3. **One way to do each thing.** No duplicate idioms.
4. **Auto-omit nothing meaningful.** Empty / null / undefined / `false` attribute values disappear from the DOM silently — write the obvious thing, get correct ARIA.
5. **Vertical integration where it pays.** `data X = ship.foo.bar()` is the headline differentiator; don't use `fetch()` for server data.

## The `.set` file shape

```html
<script>
component formAssociated         <!-- optional: declares ElementInternals -->
prop X: T = default              <!-- typed prop -->
state Y = default                 <!-- local signal -->
derived Z = expr                 <!-- memoized computed -->
data items = ship.foo.bar() <!-- server-sourced + SSE-subscribed signal -->

function helper() { … }

mount { … }                       <!-- one-time setup, after DOM in place -->
effect { … }                      <!-- runs on dep changes, auto-tracked -->
on form_reset { … }               <!-- form-associated lifecycle -->
on form_restore(state) { … }      <!-- payload arg matches detail key -->
event NAME: T                     <!-- typed CustomEvent declaration -->
</script>

<template>
  <:host role="img" aria-label={maybeLabel} />        <!-- reactive host attrs -->
  <div class="thing" onclick={fn}>{message}</div>
  {#if cond}…{:else if c2}…{:else}…{/if}
  {#for x of items}<li>{x.name}</li>{/for}
  {#await promise}loading…{:then v}…{:catch e}…{/await}
  <slot name="leading">{#if hasIcon}{@html svgString}{/if}</slot>
  {@html sanitizedHtml}
</template>

<style>                            <!-- inline; auto-scoped to host -->
.thing { color: var(--foreground); }
</style>

<!-- OR external stylesheet (split when file approaches 300 LOC): -->
<style src="./my-component.css"></style>
```

## Canonical surface — directives & declarations

### Script-block declarations

| Form | Semantics |
|---|---|
| `prop X: T = default` | Typed component prop. Becomes attribute (kebab-cased). Read as `X()` in script, written as `X.set(v)`. |
| `state X = default` | Local signal. Same access shape as prop. |
| `derived X = expr` | Memoized — re-runs when read deps change. Auto-tracked. |
| `data X = expr` | Server-sourced signal — implicitly **live**. SSE-subscribed via `/_sail/data?expr=...`. The expression is a server-side schema-source call (e.g. `ship.agents.summaries()`), NOT a JS expression. Initial value is `null`; first SSE patch delivers real data. **For a one-shot local seed, use `state X = expr`.** |
| `function fn(…) { … }` | Plain JS in component scope. |
| `mount { … }` | One-time setup. `el`, `internals`, `props` injected. |
| `effect { … }` | Runs on every reactive read inside the body. Auto-tracked — do NOT use `effect on X { … }`. |
| `on form_reset { … }` | Lifecycle hook for `component formAssociated` reset. |
| `on form_restore(state) { … }` | Restore hook. The param name pulls from `e.detail.<param>`. |
| `on form_disabled(disabled) { … }` | Disabled hook. |
| `event NAME: T` | Typed CustomEvent declaration. |
| `component formAssociated` | Element-level config. Causes the runtime to attach `ElementInternals`. |

### Template directives

| Form | Notes |
|---|---|
| `{expr}` text | Reactive interpolation. |
| `attr={expr}` | Reactive attribute. **Empty / null / undefined / `false` auto-omit.** `true` → presence-only. |
| `onclick={fn}`, `oninput={fn}`, `onkeydown={fn}` | Platform attribute names. Any `on*` attr with an interpolated value becomes an event listener. **Do NOT use `on:click={fn}`** — the `on:` prefix is deprecated. |
| `bind:value={sig}`, `bind:checked={sig}` | Two-way binding for form inputs. The runtime owns the input listener — don't pair with `oninput={…}`. |
| `{#if cond}…{:else if c2}…{:else}…{/if}` | Mustache control flow. |
| `{#for x of xs}…{/for}` | Optional `key={expr}` for stable identity. |
| `{#await promise}loading{:then v}…{:catch e}…{/await}` | |
| `{@html expr}` | Raw HTML (author owns sanitization). |
| `<slot>`, `<slot name="X">fallback</slot>` | Light-DOM slot. |
| `<:host attr={expr} role={…} />` | Reactive bindings on the component's host element. Replaces `mount + effect on X + el.setAttribute` rituals. Self-closing only. |

### File-level

| Form | Notes |
|---|---|
| `<style src="./X.css"></style>` | External stylesheet. Inlined at compile time. Use when CSS pushes the file over the 300-LOC cap. |

### Removed (don't use)

| Removed | Replacement |
|---|---|
| `on:click={fn}` | `onclick={fn}` |
| `effect on X { … }` | `effect { … }` (auto-tracks) |
| `<sail:window onkeydown={fn} />` | `mount { window.addEventListener('keydown', fn); cleanups.push(() => window.removeEventListener('keydown', fn)) }` |
| `<sail:document>` | Same as `<sail:window>` |
| `attr?={expr}` (proposed) | Just `attr={expr}` — auto-omit on empty handles it |
| `data live X = expr` | `data X = expr` (live is implicit; the legacy `live` keyword is rejected by the parser) |
| `data X = expr` (static) | `state X = expr` for a one-shot local seed |
| `<extends>`, `<block>`, `<fill>`, `<include>` | Compose with components + named slots. For shared chrome, define an `app-shell.set` component with `<slot name="…">` regions and use it as the page root. |

## Injected scope

These identifiers are available everywhere in the script block (mount, effects, handlers, functions). Do not redeclare.

| Identifier | Type | Notes |
|---|---|---|
| `el` | `HTMLElement` | The host element. Use for `el.dispatchEvent`, `el.setAttribute`, `el.querySelector`, etc. |
| `internals` | `ElementInternals \| undefined` | Set when `component formAssociated` is declared. Used for `internals.setFormValue(...)`. |
| `props` | read-only proxy | `props.X` is `X()` — same value, alternate access. |
| `cleanups` | `(() => void)[]` | Push disposers for `mount`-time setup that needs teardown. The runtime calls them on unmount. |
| `slotFilled(host, name)` | function | Returns true iff the named slot received caller-provided content (vs. fell back). |

## File cap

300 lines per `.set` file. Split CSS to a sibling via `<style src="./X.css"></style>` if you're approaching it. Behavior should never get cut to fit — if 300 lines isn't enough, log to `.artifacts/set/gaps.md` and propose a cap-raise.

## Agent-one-shot recipes

Each recipe links to a real exemplar. **Read the exemplar before authoring** — the structural shape lives there.

### Page composition with live server data

Exemplar: `packages/ship-set-sail/components/ship-agents.set` (~118 LOC + styles).

```html
<script>
data agents = ship.agents.summaries()
derived items = (agents() && agents().items) ? agents().items : []
derived total = items().length
derived hasItems = total() > 0
</script>

<template>
<div class="page-wrap">
  <header><h1>Agents</h1><p>{total()} agent{total() === 1 ? '' : 's'}</p></header>
  {#if hasItems}
    <div class="grid">
      {#for a of items()}
        <a href={`/agents/${a.id}`}>
          <ship-list-tile variant="card">…</ship-list-tile>
        </a>
      {/for}
    </div>
  {:else}
    <ship-empty-state>No agents yet</ship-empty-state>
  {/if}
</div>
</template>
```

`ship.agents.summaries()` resolves through the daemon's service registry (`apps/shipd/src/services/registry.rs`). Every read-side `ship.<domain>.<op>()` registry name works automatically — no custom resolver needed.

### Form-associated component

Exemplar: `packages/ship-set-sail/components/ship-input.set` (~106 LOC + styles).

```html
<script>
component formAssociated
prop value: string = ''
prop name: string = ''
state defaultValue = ''

function syncFormValue() {
  if (internals && typeof internals.setFormValue === 'function') {
    internals.setFormValue(value())
  }
}

mount {
  defaultValue.set(value())
  syncFormValue()
}

on form_reset {
  value.set(defaultValue())
  syncFormValue()
}
on form_restore(state) {
  value.set(state ?? '')
  syncFormValue()
}

effect { syncFormValue() }
</script>

<template>
<input bind:value={value} name={name} />
</template>
```

### ARIA-aware component (host attrs reactive)

Exemplar: `packages/ship-set-sail/components/ship-list-tile.set` (~244 LOC).

```html
<script>
prop interactive: boolean = false
prop selected: boolean = false
prop disabled: boolean = false

function dispatchSelect() { /* … */ }
function onClick() { if (interactive()) dispatchSelect() }
</script>

<template>
<:host
  role={interactive ? 'button' : 'listitem'}
  aria-selected={selected ? 'true' : 'false'}
  aria-disabled={disabled ? 'true' : 'false'}
/>
<div class="tile" onclick={onClick}>…</div>
</template>
```

`<:host>` replaces the entire `mount + effect on X + el.setAttribute` ritual every form-associated and ARIA-aware port used to write.

### Slot with conditional fallback

Exemplar: `packages/ship-set-sail/components/ship-icon.set`, `ship-empty-state.set`.

```html
<script>
prop icon: string = ''
derived showFallback = !slotFilled(el, 'icon') && !!icon()
</script>

<template>
<div class="icon-tile">
  <slot name="icon">
    {#if showFallback}
      <ship-icon name={icon} aria-hidden="true" />
    {/if}
  </slot>
</div>
</template>
```

`slotFilled(el, 'icon')` reports whether the caller provided content. **Note:** `slot=` attributes are stripped from children after `resolveSlots` runs (see "Active gotchas"); query slot fills via `slotFilled` not via `[slot=…]` selectors.

## Active gotchas (port author must know)

These bite real ports today. Workarounds are listed; ledger entries (`.artifacts/set/gaps.md`, `.artifacts/set/ergonomics.md`) track the proposed fixes.

### 1. `slot=` is stripped after distribution
`resolveSlots` removes the `slot="X"` attribute from children at mount time. `el.querySelector('[slot=trigger]')` from `mount {}` returns `null`.

**Workarounds:**
- For "did the slot get filled?": `slotFilled(el, 'name')`.
- For "give me the filled element": wrap each named slot in a stable container in your template (`<div class="trigger-wrap"><slot name="trigger" /></div>`), then `el.querySelector('.trigger-wrap')` and walk its `children`.

### 2. JS-reserved-word props (`for`, `class`, `is`, `default`, `case`)
Prop name → attribute name kebab-cases unconditionally; `prop for: string = ''` would emit `const for = ...` which is a JS syntax error.

**Workaround:** declare `state forValue = ''` instead, read via `el.getAttribute('for')` in `mount`, and watch with a `MutationObserver` if the attr might change. See `ship-label.set` for the canonical shape.

### 3. Trivial boolean → ARIA-string deriveds
ARIA needs literal `"true"` / `"false"` — runtime auto-omit handles `false` (drops the attr) but stringifies `true` as `""` (presence-only). For ARIA semantics, use explicit ternaries:
```
aria-selected={selected ? 'true' : 'false'}
aria-disabled={disabled ? 'true' : 'false'}
```

### 4. Per-instance unique IDs
Module-level `let counter = 0` is not yet emitted by the compiler. Use a random suffix in `mount`:
```
state instanceId = ''
mount {
  instanceId.set(`thing-${Math.random().toString(36).slice(2, 9)}`)
}
```

### 5. MutationObserver for child / attribute syncing
The framework has no `observe { children { ... } }` primitive yet. Hand-roll:
```
mount {
  const obs = new MutationObserver(() => syncSeparators())
  obs.observe(el, { childList: true })
  syncSeparators()
  cleanups.push(() => obs.disconnect())
}
```
Exemplars: `ship-breadcrumb.set` (childList), `ship-label.set` (attribute filter).

### 6. `bind:value` + `oninput` double-wire
`bind:value` already attaches an input listener. Don't also write `oninput={fn}` on the same element — the framework owns the listener.

If you need to dispatch a custom event on input (e.g., `ship-input`), do it from a separate `effect on value { dispatch(...) }` or from the `oninput` handler exclusively (and drop `bind:value`).

### 7. CSS comments in scoped styles
Top-level CSS comments between rules emit invalid prefixed selectors after host scoping (`[data-component="x"] /* ... */ .foo`). Browser silently drops them. Compiler-side fix tracked in `ergonomics.md` row 10.

### 8. No CSS module sharing between components
`ship-error-state` couldn't import `ship-empty-state`'s `<style>` block. The CSS was duplicated. If you find yourself doing this, file a gap row (it's not yet tracked).

## Verification

Before declaring a port done:
- `pnpm -F @sail/ship-set test <name>` passes.
- `cargo test -p sail-compiler --lib` still 138/138.
- `.ship/sail/<name>.set` symlink exists (`ln -sf ../../packages/ship-set-sail/components/<name>.set .ship/sail/<name>.set`).
- For pages: `cargo build -p shipd --features unstable` clean, then probe via Playwright or browser at `/sail/<tag>` — no console errors, no request failures.

## Where things live

| Path | What |
|---|---|
| `packages/ship-set-sail/components/<name>.set` | Component + page sources |
| `packages/ship-set-sail/test/<name>.test.ts` | Per-port jsdom tests |
| `packages/ship-set-sail/test/helpers.ts` | `loadSailComponent` test infrastructure |
| `.ship/sail/<name>.set` | Symlinks the daemon serves at `/_sail/components/<name>.js` |
| `crates/sail-compiler/src/` | The Rust compiler |
| `packages/sail-runtime/src/` | The TS runtime (signals, bindings, slot resolution, patch protocol) |
| `apps/shipd/src/views/sail*.rs` | Daemon-side compile+serve, `data` resolver dispatch |
| `apps/shipd/src/services/registry.rs` | Service registry — every `data X = ship.<domain>.<op>()` resolves through here |
| `.artifacts/set/ergonomics.md` | Running list of authoring frictions (gitignored, append-only) |
| `.artifacts/set/gaps.md` | Running list of framework gaps (gitignored, append-only) |
| `packages/ship-set/sdk/sdk-framework.json` | Framework manifest — also exposes a per-component prop catalog (`components: [{ tag, props: [{ name, type, required }], description }]`) regenerated by `cargo xtask gen-sdk` from the corpus above; consumed by the Rust prop-existence rule and the LSP completion provider. |
