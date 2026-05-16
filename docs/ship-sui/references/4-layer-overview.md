# SUI — Four-Layer Overview

SUI splits cleanly into four layers. Each owns one concern and exposes a narrow contract to the next.

```
page.sui + controller.ts     (authoring — what humans write)
    ↓ crates/sui/sui-compiler/ (AOT compile — emits ESM + .d.ts + .ssr.html)
.js + .sui.d.ts + .ssr.html  (compiler output — gitignored under dist/)
    ↓ packages/sui-runtime/  (reactive primitives + DOM bindings + attach())
runtime in the browser       (signals, control-flow, transitions, error-boundary)
    ↓ crates/sui/ship-sui-server (daemon view-host — resolver + assets + SSR + SSE)
HTTP /_sui/* + /view-host    (apps/shipd routes wire it together)
```

## Layer 1 — Compiler (`crates/sui/sui-compiler/`)

Pure transformation. No filesystem walk, no daemon, no network. Library entry: `crates/sui/sui-compiler/src/lib.rs`.

```rust
pub fn compile(source: &str, opts: CompileOpts) -> Result<CompileOutput, CompileError>
```

`CompileOpts` knobs: `setup_url` (paired `.ts` import), `mode` (`View | Component | Auto`), `source_path` (diagnostic spans), `component_meta` (tag name + shadow flag), `styles` (component-only external CSS), `emit_dts`, `emit_ssr`.

`CompileOutput` carries:

| Field | What it is |
|---|---|
| `js` | ESM module that imports from `@ship/sui-runtime`. Constructs the render tree directly — no parser ships to the client. |
| `css` | Extracted / passed-through CSS (component `<style>` blocks, `styles` opt), present when the source carries styles. |
| `cem` | Custom Elements Manifest (components only) — fuel for editor tooling. |
| `schema` | Prop schema JSON — read by `decode.ts` to validate runtime prop values. |
| `dts` | Typed `Scope` for the paired controller. Includes both `<data>` names and `scope:` factory keys. |
| `meta` | `.sui.meta.json` — binding kinds / exprs / deps / renderKey. Developer-tooling fuel; the runtime never loads it. |
| `ssr_html` | Static SSR body fragment (view mode only). Written to `<repo-root>/.ship/sui-out/<src-rel>/<stem>.ssr.html`. |

Pipeline (`src/lib.rs:165` `compile()`): `parse::parse` → `check::run` → `bindings::build` → `lower::lower` → `emit::js::emit_module` (+ `emit::cem`, `emit::schema`, `emit::dts`, `emit::meta`, `emit::ssr`). Diagnostics carry byte spans (`src/diagnostics.rs`).

Binary: `crates/sui/sui-compiler/src/bin/sui-compiler.rs` (the `sui-compiler` Cargo bin). Used by `@ship/sui-build` and editor extensions for one-shot compiles.

## Layer 2 — Runtime (`packages/sui-runtime/`)

Public API barrel: `packages/sui-runtime/src/index.ts`. Anything not re-exported there is internal.

| Surface | Symbols |
|---|---|
| Reactivity | `signal`, `derived`, `effect`, `batch`, `untrack`, `createScope`, `createChildScope`, `onCleanup` |
| DOM bindings | `bindText`, `bindShow`, `bindAttr`, `bindClass`, `bindProp`, `bindValue`, `bindChecked`, `bindHtml`, `bindRawHtml` |
| Events | `on`, `matchesKey` |
| Control flow | `bindFor`, `bindForSignal`, `bindIf`, `bindAwait` |
| Slots | `resolveSlots`, `snapshotChildren`, `slotFilled`, `bindSlotEmpty` |
| Components | `defineComponent`, `component`, `prop`, `camelOf` |
| Prop decoders | `decodeString`, `decodeNumber`, `decodeBoolean`, `decodeJson`, `decodeEnum` |
| Data | `data`, `live`, `unwrap`, `unwrapOr`, `wireFormMutation` |
| State helpers | `state`, `debounce`, `useFilteredList`, `useSelection`, `useForm` |
| Shared context | `shared`, `isShared`, `provide`, `useShared`, `pickShared` |
| Host API | `emit`, `expose`, `attachHostApi`, `bindNativeEvent`, `slotted`, `bindHostAttr` |
| Transitions | `registerTransition`, `runIn`, `runOut`, `runEnter`, `runLeave`, `cancelTransitions`, `fade`, `slide`, `scale` |
| Morph / boundary | `morph`, `attachBoundary`, `detachBoundary`, `findNearestBoundary`, `runProtected` |
| While-states | `bindWhileLoading`, `bindWhileError` |
| Controller | `controller`, `resolveController`, `LIFECYCLE_KEY`, `view` |
| Template hooks | `registerTemplateHook`, `getTemplateHook`, `clearRegistry`, `listRegisteredHooks` |
| Mount | `mount(target, payload, opts) -> MountHandle` |
| Styles | `suiTonesSheet`, `suiBaseSheet`, `adoptSuiSheets` |
| Generated types | `AgentSummary`, `GitOverview`, `LintFinding`, `ManifestRail`, `StatusBuckets`, `ViewListEntry`, `ViewManifest`, … (daemon ↔ runtime contract, authored by `cargo xtask gen-types`) |

The dev-only leak detector lives at the subpath `@ship/sui-runtime/dev` so it doesn't ship in production bundles.

`<sui-if>`, `<sui-for>`, `<sui-match>` custom elements register on module load (`import './boundary-elements'` at the bottom of `index.ts`).

## Layer 3 — Daemon view-host (`crates/sui/ship-sui-server/`, `apps/shipd/`)

Crate layout (`crates/sui/ship-sui-server/src/`):

| File | Owns |
|---|---|
| `resolver.rs` | Walks `views/` + `.ship/views/` + project mirror trees, picks the manifest for a route, resolves `[param]` segments. |
| `view.rs` | Serves `.sui` source + compiled `.js` + scoped `.css` (`serve_sui_view_template`, `serve_sui_view_setup`, `serve_sui_view_css`). |
| `view_payload.rs` | Static SSR HTML payload (`serve_view_payload`, `serve_view_payload_subpath`). |
| `view_host.rs` | The view-host HTML shell that hydrates the payload (`serve_view_host`). |
| `assets.rs` | Runtime / DOM / sdk-client / components / lucide-icons / island bundles. |
| `bundle.rs` | Bundle resolution against the on-disk build artifacts. |
| `render_helpers.rs` | Shared SSR helpers (rendering manifest-driven shells). |

The crate exposes `DaemonMode::{Development, Production}` — drives cache headers, HMR availability, and the SSR cache.

Daemon-side route table (`apps/shipd/src/router.rs`) mounts the surface:

| Route | Handler |
|---|---|
| `GET /_sui/runtime.js` / `dom.js` / `sdk-client.js` / `components.js` / `lucide-icons.json` | `crates/sui/ship-sui-server/src/assets.rs` |
| `GET /_sui/sui-components/{name}.js` | separate-bundle component |
| `GET /_sui/views/{name}/view.sui` / `view.js` / `view.css` | `crates/sui/ship-sui-server/src/view.rs` (route segment stems are `view.*` regardless of on-disk `page.sui`/`controller.ts`/`style.css`) |
| `GET /_sui/views/{name}` | `crates/sui/ship-sui-server/src/view_payload.rs` |
| `GET /_sui/views/{name}/sse` | `apps/shipd/src/views/sui_patch_channel.rs` |
| `GET /_sui/hmr` / `hmr-client.js` / `dev/hmr` | `apps/shipd/src/views/sui_hmr.rs` |
| `GET /_sui/data` | `apps/shipd/src/views/sui_data.rs` (the `<data live>` SSE channel) |
| `POST /_sui/mutation/{name}` | `apps/shipd/src/views/sui_mutation.rs` |
| `POST /_sui/views/{name}/patch-elements` | `apps/shipd/src/views/sui_patches.rs` |
| `GET /_sui/source/{tag}` | `apps/shipd/src/views/sui_source.rs` |

The router wiring is in `apps/shipd/src/router.rs`. Signal-backed sources (tighter wake-up than the default `config` kernel namespace) live under `apps/shipd/src/views/sui_data_resolvers/` and `apps/shipd/src/views/git_signal.rs`; `sui_sse.rs::is_signal_backed_expr` picks them up by registry name.

## Layer 4 — Authoring (`views/<name>/`, `packages/ui-components/src/<tag>/`)

Directory shapes:

```
views/<name>/                    # first-party views
  page.sui                       # required
  controller.ts                  # optional
  style.css                      # optional
  manifest.jsonc                 # required
  dist/                          # compiler output (gitignored)
  _components/<view>-<tag>/      # view-private widgets
    <view>-<tag>.sui
    <view>-<tag>.controller.ts

.ship/views/<name>/              # project-local extension views; same shape

packages/ui-components/src/<tag>/   # first-party components
  <tag>.sui
  <tag>.controller.ts            # optional
  <tag>.css                      # optional

packages/ui-components/src/<tag>/   # separate-bundle (lazy) components also live here;
                                    # they opt in via `<bundle separate />` in their .sui
```

Five rules carried over from the `ship-views` skill:

1. **`page.sui` is the route entry.** Filename stem says what the file IS — a page, not a reusable widget.
2. **`controller.ts`, no stem prefix.** The directory IS the view; pairing is unambiguous.
3. **`style.css` for page-scoped styles.** Use `<style>` blocks inside `.sui` for component-scoped styles.
4. **`manifest.jsonc` is required for views.** JSONC, not YAML.
5. **`_components/` for view-private widgets.** Tag MUST prefix with the view name (`home-hero` for `views/home/`) so two views can each ship their own widget without colliding on `customElements.define`.

Every first-party view is on this shape — `views/agents/`, `views/git/`, `views/home/`, etc. all ship `page.sui` + `controller.ts` + `style.css` + `manifest.jsonc`. The router (`packages/ship-set/src/router/`) walks only manifest-bearing directories.

## How the layers compose

A request for `/agents` resolves end-to-end:

1. Browser hits `GET /agents` → daemon serves the view-host HTML shell (`serve_view_host`).
2. Shell loads `/_sui/views/agents` → SSR payload (compiled HTML + manifest).
3. Shell loads `/_sui/views/agents/view.js` → compiled ESM module.
4. Module imports from `/_sui/runtime.js` (the SUI runtime barrel) and `/_sui/sdk-client.js`.
5. `attach()` walks the SSR DOM, binds reactive identifiers from the module's controller, opens `<data live>` SSE channels to `/_sui/data?expr=…`.
6. Edits in dev mode emit `module-changed` on `/_sui/dev/hmr`; `apps/shell2/src/hmr.ts` cache-busts the rebuilt `view.js` and re-mounts in place.
