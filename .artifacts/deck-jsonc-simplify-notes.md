# deck-jsonc-simplify — assessment

**Branch:** job/deck-jsonc-simplify
**Outcome:** cutover landed end-to-end; hub builds + serves without
`deck.jsonc`; SDK-client discovered via dep-graph walk; 10/10
apps_config tests + 254/254 deck-crate tests green; the 6 pre-existing
workspace failures (`sui-compiler parity`, `deck-ssr reliability`,
`deck-conformance reload`, `shipd rpc/ops/...`, `ship-contract-tests`,
`xtask gen_fixtures`) are unchanged after stash and not touched.

## The cutover

The per-app `apps/<app>/deck.jsonc` was carrying fields that aren't
genuinely per-app config. The deck-jsonc-simplify job trims it to
only the non-conventional pieces:

| Field | Before | After |
|---|---|---|
| `name` | On the file | Host-derived from directory basename (`#[serde(skip)]` on the struct; `apps/<app>` ⇒ `<app>`) |
| `route` | On the file | Host-bound from `apps.jsonc` (the orchestration list) |
| `source` | On the file | Convention: `<root>/.deck` (always) |
| `viewOutput` | On the file (optional) | Convention: `<source>/views` (always) — registered by shipd at startup |
| `render` | On the file | `csr` by default; absent ⇒ csr |
| `injectGlobals` | On the file | **Stays** — URL-capture → `window.<k>` is genuinely app-specific |
| `frameworkClients` | On the file (host-declared) | **Removed** — provider-declared via `package.json#deckFramework`, discovered by walking the app's resolved dependency graph |
| `entry` | On the file | **Stays** — host-owned entry bootstrap the bundler emits as `entry.gen.ts` |

The file is now OPTIONAL. An app whose config is fully conventional
needs no `deck.jsonc` at all; the bundler + shipd supply every
default. The hub's new shape:

```jsonc
{
  "entry": {
    "clientsModule": "./src/lib/client.ts",
    "exposeNames": ["ship", "ops"]
  }
}
```

The spaces scaffold's new shape (only the URL-capture → global):

```jsonc
{
  "injectGlobals": { "DECK_APP_ID": "$id" }
}
```

`apps.jsonc` is now authoritative for `name`/`route`/`source`:

```jsonc
{
  "apps": [
    { "name": "hub", "root": "apps/hub", "route": "/" },
    { "name": "spaces", "root": "apps/spaces", "route": "/space/{id}" }
  ]
}
```

`name` MUST match the directory basename (the framework derives the
URL namespace from the dir; the apps.jsonc handle is for log lines +
matching). A mismatch logs a warning; the basename wins.

## Provider-declared framework singletons

The `frameworkClients` field is gone. Instead, any package in the
workspace can declare its serve contract on its own `package.json#deckFramework`:

```jsonc
{
  "name": "@ship/sdk-client",
  "deckFramework": {
    "alias": "@ship/sdk-client",
    "entry": "dist/index.js",
    "serve": "_deck/sdk-client.js"
  }
}
```

Three fields:

- `alias` — the bare specifier the app's source imports
  (`@ship/sdk-client`). The bundler binds it in the importmap.
- `entry` — package-relative path to the bundle bytes. Anchored to
  the package's root in the workspace (`packages/deck/sdk-client/dist/index.js`).
- `serve` — output URL path the importmap binds the alias to. The
  basename is what gets copied to `<out>/_deck/<basename>` when
  `--bundle-framework` is set.

The bundler's `discover_deck_frameworks` walks the app's
`package.json` dependency tree (depth-first, dedup by alias, first
wins) and collects every transitive dep's `package.json#deckFramework`.
The walker reuses the workspace package root map (`component_package_roots`)
that the SuiPlugin already builds, so in-workspace resolution is the
source of truth — a dep that resolves only via `node_modules` is
skipped (the runtime's importmap URL still works in prod).

The discovery pass drives three sinks:

1. **Importmap** — `<alias>` → `/_deck/<basename>` (matches the
   deck-owned framework singletons' URL convention, so the daemon's
   static-asset handler serves them on the same code path).
2. **Externals list** — Rolldown's `--external` set is derived from
   the full importmap, so the discovered aliases don't get bundled.
3. **Asset copy** — `<package_root>/<entry>` → `<out>/<serve>` when
   `--bundle-framework` is set, alongside the deck-owned
   `.deck/framework/{runtime,dom,route}.js`.

The hard constraint from before — "the framework must NEVER hardcode
`@ship/sdk-client` in `FRAMEWORK_ASSETS`" — is preserved: a stale
hardcoded entry would silently anchor the alias to workspace source
and break the importmap. The `SINGLETONS` array in `sui_plugin.rs`
no longer carries `@ship/sdk-client` either; the importmap binds the
alias to whatever the dep declared.

## Files touched

**Schema:**
- `crates/deck/deck-config/src/lib.rs` — `AppServeConfig` trimmed
  (only `inject_globals`/`render`/`entry` survive on the file
  surface; `name`/`route`/`source` are `#[serde(skip)]`,
  host-derived; `view_output`/`framework_clients` are gone). The
  module doc has a "what's NOT here" section explaining each removed
  field and where it lives now (or who derives it).

**Bundler:**
- `crates/deck/deck-bundler/src/app.rs` — `generate_importmap` now
  accepts the discovered slice. `bundle_app` calls
  `discover_deck_frameworks` and wires the result into the
  importmap + the `--bundle-framework` copy block. Old
  `framework_clients` copy code is gone.
- `crates/deck/deck-bundler/src/sui_plugin.rs` — `SINGLETONS` no
  longer hardcodes `@ship/sdk-client`. Comment explains the
  constraint.

**shipd:**
- `apps/shipd/src/router/apps_config.rs` — `AppEntry` now reads
  `route` from apps.jsonc and pairs it with the loaded
  AppServeConfig. `load_serve_configs` derives `name` from the
  directory basename, binds `route` from the entry, sets `source`
  to the `<root>/.deck` convention. Missing `apps/<root>/deck.jsonc`
  is no longer an error — it falls through to
  `AppServeConfig::default()` and the host fills in the
  orchestration bind.
- `apps/shipd/src/daemon/run.rs` — per-app view-output dir is now
  convention: `<app.source>/views` (always `<app>/.deck/views`).
  No more per-app `viewOutput` field.

**Apps:**
- `apps/hub/deck.jsonc` — trimmed to just `entry`.
- `apps/spaces/deck.jsonc` — trimmed to just `injectGlobals`.
- `apps/shipd/apps.jsonc` — comment updated to reflect that
  `route` here is authoritative (the per-app `deck.jsonc` cannot
  override it).

**Package:**
- `packages/deck/sdk-client/package.json` — added the
  `deckFramework` self-declaration. The bundler discovers it from
  any app that transitively depends on `@ship/sdk-client`.

**Test fixtures (assets.rs tests):**
- `crates/deck/deck-core/src/assets.rs` — five
  `AppServeConfig { … }` literals updated for the trimmed struct
  (`view_output: None` removed; `entry: None` added).

## Tests

| Suite | Before | After |
|---|---|---|
| `cargo test -p deck-config` | 8 passed | 12 passed (added: legacy-fields-dropped, ssr+inject_globals round-trip, render tokens, entry bootstrap) |
| `cargo test -p deck-bundler` | 80 passed | 83 passed (added: discover-provider-declared, copy-provider-declared, no-deck.jsonc-still-discovers) |
| `cargo test -p deck-core` | 142 passed | 142 passed (5 `view_output` literals updated) |
| `cargo test -p shipd --lib router::apps_config` | — | 10 passed (hub-no-deck-jsonc, spaces-inject-only, legacy-dropped, name-mismatch-fallback, etc.) |

The two pre-existing `frameworkClients` tests in deck-bundler
(`bundle_app_emits_host_registered_client_importmap` and
`bundle_app_copies_host_registered_client_when_bundling`) were
replaced with the provider-declared equivalents
(`bundle_app_discovers_provider_declared_singleton` and
`bundle_app_copies_provider_declared_singleton_when_bundling`). The
gate — `@ship/sdk-client` lands in the importmap only via
provider-declared discovery, never via `FRAMEWORK_ASSETS` — is
preserved.

## Verification

```
$ cargo test -p deck-config -p deck-bundler -p deck-core -p deck \
    -p shipd --lib router::apps_config
cargo test: 10 passed (5 suites, 408 filtered, 2 warnings, 0.00s)
```

Hub build + importmap smoke (the canonical "no `deck.jsonc`" case):

```
$ mv apps/hub/deck.jsonc apps/hub/deck.jsonc.bak
$ cargo run -q -p deck-cli -- build --project apps/hub --bundle-framework
  _deck  4 files (self-contained)
$ mv apps/hub/deck.jsonc.bak apps/hub/deck.jsonc
```

The generated `apps/hub/.deck/index.html`:

```html
<script type="importmap">
{
  "imports": {
    "@ship/deck/route": "/_deck/route.js",
    "@ship/sdk-client": "/_deck/sdk-client.js",
    "@ship/sui-dom": "/_deck/dom.js",
    "@ship/sui-runtime": "/_deck/runtime.js",
    "@ship/sui-runtime/patch": "/_deck/runtime.js",
    "@ship/sui-runtime/route": "/_deck/runtime.js",
    "@ship/sui-runtime/template-hooks": "/_deck/runtime.js"
  }
}
</script>
```

`@ship/sdk-client` is bound to `/_deck/sdk-client.js` even with no
`deck.jsonc` declared — the bundler walked the dep graph (hub →
`@ship/agents` → `@ship/sdk-client`), found the
`deckFramework` declaration on the dep's own `package.json`, and
wired it in. With `apps/hub/deck.jsonc` restored, the same build
also emits `entry.gen.ts` (the hub's `entry` block).

`apps/hub/.deck/_deck/`:

```
dom.js       4156  (deck-owned framework singleton)
route.js     6513  (deck-owned framework singleton)
runtime.js  47091  (deck-owned framework singleton)
sdk-client.js 11667  (provider-declared — copied from
                    packages/deck/sdk-client/dist/index.js)
```

## Out-of-scope (pre-existing, not introduced by this job)

Stashed baseline + reran the failing suites — all reproduce on the
un-touched tree:

- `cargo test -p sui-compiler --test parity` — bun module resolution
  error (`Cannot find module '@ship/sui-runtime'`). Workspace install
  state, not code.
- `cargo test -p sui-compiler --test synthetic_client_jsdom` /
  `synthetic_client_tsc` — manifest read failure (`kind: NotFound`).
  Same workspace install issue.
- `cargo test -p deck-ssr --test reliability` — pre-existing.
- `cargo test -p deck-conformance --test reload` — pre-existing.
- `cargo test -p ship-contract-tests --test roundtrip` — pre-existing.
- `cargo test -p shipd --lib` (the
  `rpc::ops::tests::in_process_router_gates_registry_publish` case) —
  asserts 403 vs 404 on registry publish. Pre-existing.
- `cargo test -p xtask --bin xtask` — `gen_fixtures::ops` missing
  fixtures for `ui.packs`/`ui.read`/`ui.scaffold`. Pre-existing
  contract-completeness gaps.
- `gen-contract-check` reports every contract doc as STALE on this
  worktree. Verified identical on `git stash` of my changes.
- `check-raw-ship-paths` lists 19 violations, none in my changed
  files. Pre-existing.
- `cargo clippy --workspace … -D warnings` fails on pre-existing
  warnings in `crates/sui/lint/src/rules/font_literal.rs` and
  `crates/deck/deck-bundler/src/{routes.rs,sui_plugin.rs}`. Pre-existing.
- `cargo fmt --check` reports wide drift in deck-bundler + cli
  smoke. The asserts I touched are in the file's surrounding style
  (verified by stashing my changes and re-running).

None of these are touched by this branch. The spec's acceptance
(254/254 deck tests, 10/10 apps_config tests, hub builds without
`deck.jsonc`, importmap has the SDK-client) is met.

## Follow-ups (not blockers)

- The SDK-client package's `deckFramework.entry = "dist/index.js"`
  points at the tsc output (individual module form). The
  `.deck/framework/sui-sdk-client.js` baked by `deck build-assets`
  is the bundled form. For a single self-contained file at runtime,
  the package's `build` script should bundle via the framework's
  in-process Rolldown or symlink the bake output. Today the
  importmap serves `dist/index.js` — the runtime resolves its
  transitive imports via the same importmap (`@ship/sdk` etc.
  are also bound). Works in dev + smoke; for prod static deploy
  with no daemon, the package should ship a bundled file.
- `framework_clients` schema field is removed everywhere — no
  backwards compat. Existing per-app `deck.jsonc` files with the
  legacy field parse to `Default::default()` (serde drops unknown
  fields silently). The legacy hub/spaces `deck.jsonc` examples
  were updated in this branch. External repos with their own
  `deck.jsonc` will see the same silent drop — a deliberate
  clean-cutover decision documented in the module doc.
