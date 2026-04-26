# SDK + SFC roadmap

Plan for taking the `.ship` SFC format from its current 7/10 to the platform Ship deserves. Authored after writing seven non-trivial components against the format and merging the framework's gap-fix wave.

The framework today is gap-driven (commits literally name "gap #N"). That's the right discipline; the work below extends it with structural moves the gap-list won't surface on its own.

## Themes

Five buckets, ranked by leverage on real authoring friction:

1. **Type story** — biggest single jump. Authors today get zero static knowledge about props, signals, or events.
2. **Composition** — property bindings + context API. Right now everything routes through string attributes, which caps what components can be composed of.
3. **Authoring DX** — auto-loader, script HMR, devtools. Compounds across sessions.
4. **Migration** — converging `ship-set` LitElement primitives onto the SFC format. Long tail.
5. **Ecosystem** — testing harness, examples, pack story. Earns its place once 1–4 reduce friction.

Each bucket below: items in priority order, cost (S/M/L), what unlocks.

---

## 1. Type story

**Why this is #1:** every other framework solved this. Authors expect autocomplete on `props.X`, type-checked attributes at the call site, and red squiggles when a component receives the wrong shape. Ship today has none. This is the biggest perceived-quality gap.

### 1.1 Prop schema declaration (S, ~2 days)

A typed declaration block in `<script>` the framework + LSP both read:

```js
// Option A: TypeScript-style
const props = defineProps({
  agentId: { type: String, required: true },
  refreshMs: { type: Number, default: 2000 },
  readonly: { type: Boolean, default: false },
})

// Option B: JSDoc + plain destructure (less invasive)
/**
 * @prop {string} agentId      required, agent id to attach to
 * @prop {number} [refreshMs=2000]
 * @prop {boolean} [readonly]
 */
const { agentId, refreshMs = 2000, readonly = false } = props
```

The compiler:
- Emits `static observedAttributes` from the declared list (no more guess-from-destructure)
- Emits a typed decoder per prop (Number → `Number(attr)`, Boolean → presence check)
- Emits a JSON schema sidecar at `/_ship/components/<tag>.schema.json` for the LSP

**Unlocks:** 1.2, 1.3, eventual call-site validation.

### 1.2 LSP call-site validation (M, ~4 days, blocked on 1.1)

When the LSP sees `<app-agent-picker default-agent="..." default-provider="...">` in a `.html.j2` or `.ship` template, it:

- Marks unknown attributes red
- Type-checks values where decidable (string literals, Jinja-static expressions)
- Suggests completions from the schema
- Surfaces the prop's docstring on hover

This is what makes "I'm authoring against this component" feel like authoring against a real API. Today it's all stringly-typed.

### 1.3 Event payload typing (S, ~1 day, blocked on 1.1)

Same pattern for emit:

```js
/** @event select {agentId: string, provider: string} */
emit('select', { agentId, provider })
```

Listening side gets autocomplete on `event.detail.agentId`. Catches refactor regressions where you rename `agent_id` → `agentId` and forget to update three consumers.

### 1.4 Signal type inference (M, ~3 days, optional)

The compiler already knows initial values for `signal(0)`, `signal('')`, `signal([])`. Surface that to the LSP so `$count++` in a Datastar attribute lights up if `$count` was declared as a string. Lower priority — same-file authoring catches this fast.

**Bucket cost: ~10 days, large ergonomic jump.**

---

## 2. Composition

### 2.1 Property bindings, not just attribute bindings (S, ~2 days)

Today: `data-attr:session-id="$sessionId"` sets a DOM attribute (string). Object values JSON-stringify and the child has to re-parse. Booleans are presence-encoded. Lit and every modern framework support setting properties directly.

Add `data-prop:<name>="$value"` (or auto-detect: if the target element has a property with that name, set the property; else fall back to attribute). The runtime is one `Object.defineProperty` away.

```html
<my-list data-prop:items="$items"></my-list>
<!-- vs: data-attr:items="$items" → "[object Object]" disaster -->
```

### 2.2 Context API (M, ~4 days)

Provider/consumer model. Modal publishes `session_id`, `agent_id`, `provider` once; thread + composer + future siblings consume by key without anyone manually threading attributes down.

```js
// Provider
const { provide } = ctx
provide('session', { sessionId: '...', agentId: '...' })

// Consumer
const { inject } = ctx
const $session = inject('session')  // signal that updates when provider changes
```

Implementation: a per-tree WeakMap of context maps, walked from consumer up to find the nearest provider. Standard pattern.

**Why this matters more than it sounds:** today every component in the agent-ui hierarchy is parameterized by `sessionId` because there's no shared ambient way to express "the active session." Context eliminates the prop-drilling.

### 2.3 Slot props (S, ~1 day)

Format docs already mention native `<slot>` with `item="item"`. Make this typed in 1.1's wake — slot author declares the slot's props, consumer destructures them in the slot template. Already half-there; surface it.

### 2.4 First-class composition primitive: `<ship-portal>` (S, ~1 day)

Render a child outside the component's DOM subtree (modal overlays, toasts, popovers). I worked around this by mounting `<app-agent-modal>` at the shell root. A portal primitive moves that into the component model itself.

**Bucket cost: ~8 days, unlocks composition patterns Ship doesn't have today.**

---

## 3. Authoring DX

### 3.1 Auto-loader (S, ~2 days)

Component manifest. Page declares what it uses (or the daemon discovers it from rendered template parsing); the daemon emits the script tags.

```html
<!-- author writes -->
<app-agent-modal></app-agent-modal>

<!-- daemon serves: detects the tag, injects -->
<script type="module" src="/_ship/components/app-agent-modal.js"></script>
```

Already listed in caveats as future work. Worth doing before 4 (migration) because it makes ad-hoc composition viable without bookkeeping.

### 3.2 Script-block HMR (M, ~3 days)

Today: edit `<script>` → hard browser refresh. Template + style changes hot-reload via Datastar morphing.

Wire a daemon-pushed SSE event when `.ship` mtime changes. Browser receives the event, reloads the matching `customElements` definition. Existing instances either:
- swap and remount (preserving DOM position via `replaceWith`), OR
- get marked stale and reload on next interaction (cheaper, still nice)

The framework already has the per-instance signal storage and the per-tag namespace. Hot-swap is mechanical, not architectural.

### 3.3 Component devtools (S, ~3 days)

`<ship-devtools-components>` exists in the panel dock — fill it in. Per-tag list, per-instance signal values, recent emit() events, hot-edit prop attributes (`setAttribute` on the live host). The data is already there; this is just UI.

### 3.4 Token-aware authoring (S, ~2 days)

LSP reads `tokens.css`, autocompletes `var(--ship-color-*)` on hover. Errors on undeclared token references. Errors on bare hex literals (with an opt-out comment). Catches regressions that today only surface in design review.

**Bucket cost: ~10 days, compounds across every component author writes.**

---

## 4. Migration

### 4.1 ship-set primitive ports (L, ongoing)

The 200+ Lit components in `packages/ship-set/`. `ship-tabs` is ported as the stress test. Each port:
- Validates the SFC format covers Lit's reactive surface
- Reduces vocabulary mismatch in the SDK
- Is incremental — port what authors hit friction on, not everything

Rough order by author friction: `ship-card`, `ship-button`, `ship-input`, `ship-select`, `ship-toast`, `ship-popover`, `ship-list-tile`. The interactive ones with state are the high-leverage ports; pure-styled ones are mostly cosmetic.

### 4.2 Mixed-vocabulary devtools surface (S, ~2 days)

While migration is partial, the devtools panel shows both shapes — Lit components AND SFC components — with a clear "kind" indicator. So authors learning the SDK don't get confused about which pattern to follow.

**Bucket cost: ongoing, ~2 days/primitive average. Prioritize 1–3 first.**

---

## 5. Ecosystem

### 5.1 Test harness (M, ~4 days)

Vitest + a `mountSFC(path, props)` helper. Renders the component to a DOM, returns a handle for assertion + interaction:

```js
import { mountSFC } from '@ship/sfc-test'

it('picker emits select on commit', async () => {
  const { host, fire, on } = mountSFC('app-agent-picker', { defaultAgent: 'shipwright' })
  const events = on('select')
  // ... assert state, click button, assert event fired
})
```

Without this, every component is a liability. With it, you can refactor framework internals with confidence.

### 5.2 Examples directory (S, ongoing)

`docs/ship-components/examples/` with one SFC per pattern: data-fetch, list, form, modal, command, drag-drop. Today there's `app-typeahead.ship` + `ship-tabs.ship`. Need ~10 more covering the patterns authors actually hit.

### 5.3 Pack publication (M, ~5 days)

The contract-freeze landed `packs/`. SFCs should be packageable. Dependency: 4.x ship-set ports done enough that authors aren't fighting migration mid-pack.

**Bucket cost: ~10 days for 5.1 + 5.3, examples are ongoing.**

---

## Sequencing

If I were running this with two people for two months:

**Sprint 1 (week 1–2):** 1.1 (prop schema) + 3.1 (auto-loader). These are foundational and unblock most other items.

**Sprint 2 (week 3–4):** 1.2 (LSP call-site validation) + 2.1 (property bindings). Now authoring feels typed and composition isn't string-fighting.

**Sprint 3 (week 5–6):** 2.2 (context) + 3.4 (token authoring). Now real composition lands and design system has guardrails.

**Sprint 4 (week 7–8):** 5.1 (test harness) + 3.2 (script HMR). Now refactoring is safe and dev iteration is fast.

After this: 1.3 (event types), 2.3 (slot props), 3.3 (devtools), 4.x (ports) — the long tail.

## What NOT to do

A few items that look attractive but are wrong moves:

- **Don't introduce a new template syntax.** Jinja-on-the-server + Datastar-on-the-client is the right division of labor. A unified expression language (the 7→8 jump in my critique) is tempting but doubles the surface area to teach. Ship's choice to lean on existing tools is principled; reinforce, don't replace.

- **Don't add a virtual DOM.** Datastar's morph + per-instance signals is fast enough for the kinds of UI Ship builds. A vDOM would be a substantial runtime cost in service of marginally better DX.

- **Don't enforce type annotations.** Make 1.1 opt-in. Components without schemas still work; LSP just gives them less help. Pragmatic > purity.

- **Don't migrate `ship-set` wholesale.** The 200+ Lit components work. Each port is a 4-hour commitment that introduces regression risk. Drip-feed by author friction.

- **Don't gate the SFC format behind a build step.** Today: edit `.ship`, save, daemon recompiles, browser sees new bytes on next request. Adding webpack/rollup/vite into that loop kills the magic. The runtime IS the build.

## How to know it's working

Three signals:

1. **Component count growing without framework changes.** Healthy when authors add components without filing framework gaps every other commit. Today: gap-fix wave landing 5/week. Healthy: 1/week or less, with new SFC commits dominating.

2. **Old chat impl + right drawer deletion.** When the agent modal hits parity (see `agents-on-supervisor.md` deletion criteria), and the right drawer audit lands, those go. That's the proof point that the SFC modal can replace the legacy view-rendered surface.

3. **First non-Ship-team author ships an SFC.** Outside contributor lands `<app-something>` without DM-ing the framework team for help. Means the docs + LSP + format are self-serve.

— Plan owner.
