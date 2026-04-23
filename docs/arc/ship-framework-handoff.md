# Ship Framework — Handoff

Context dump for the next thread. The daemon-decoupling arc (first-party views → file-routed framework) is complete except for two views the user explicitly deferred. Below is state + what's next.

## Where we are

**Principle:** the daemon exposes a generic render surface (`/api/v/{*path}` + `/api/sse/{*path}`), reads templates from `.ship/views/` (project) + `~/.ship/views/` (user), and templates pull data via the `ship.<domain>.<op>()` SDK backed by a single dispatcher (`services::registry::call`). No per-view Rust. First-party and third-party views use the same surface.

**Arc:** `feat/ship-set-architecture`, starting at `7a3ae5fa` (generic routed handler shipped). Every commit since named the view it migrated. Use `git log --oneline` to reconstruct.

## Migrated views

All routes below flow through `/api/v/<name>` with no bespoke Rust handlers:

- agents (list + `[id]` + `[id]/settings` + `[id]/wire` + `create`)
- events
- settings
- skills (+ `[id]`)
- git (query-mode: `?commit=`, `?file=`)
- specs (query-mode: `?id=`)
- pipelines (list + `focus/[name]` + `run/[slug]` + `history`)
- dashboard (already file-routed; latent `.append()` bug patched)

## Deferred (user's call)

- **chat** — first-party chrome, explicit exception. `views_chat.rs` + `ViewChat{Shell,Sse,Send}` routes remain. User said an exception is fine "if absolutely necessary." If you migrate, think about how non-chrome extensions would use a message-passing convention with the shell.
- **overview** — canvas-heavy. User flagged the canvas itself as "a great web component to build." Don't migrate until the web component exists; treat the current `overview.rs` as placeholder.

## Framework primitives you should know

### `services::registry` (apps/shipd/src/services/registry.rs)

Single dispatcher. `pub fn call(name, args, state) -> Result<Value, RegistryError>`. Dotted names: `agents.detail`, `git.diff`, etc. Sync. Read-only — writes go through HTTP endpoints under `/api/<domain>/*`. `names()` drives codegen + MCP + lint. **This file is ~570 lines and past the 300 cap** — split by domain when you next touch it.

### `ship.*` template namespace (apps/shipd/src/services/ship_ns.rs)

Minijinja `Object` that exposes registry calls plus request context (`ship.route`, `ship.query`, `ship.signals`, `ship.session`, `ship.workspace`, `ship.env`). Pattern for growing a domain namespace:
```jinja
{% set data = ship.agents.detail(id=ship.route.id) %}
```
Bad id → `not_found` error bubbles as a render 500. **Acceptable:** the URL is the input contract; bad input = broken link, not a template-level concern.

### `views::routed_handler` + `views::routed_sse`

Entry points for `/api/v/{*path}` and `/api/sse/{*path}`. `render_routed` is shared between both so kernel-event SSE re-renders use the exact same pipeline as initial GET.

### `views::packs` — per-view bundle injection

Manifest `packs: [editor, terminal, git]` → `<script type="module">` tags injected into `<head>` (or prepended for fragment responses). **Fixed in `68f96f76`** — the routed handler was silently dropping packs before. Any view declaring `packs:` in `manifest.{yaml,jsonc}` now loads bundles correctly.

### File routing (`views::router`)

Next.js conventions: `[id]` dynamic, `[...rest]` catch-all, `[[opt]]` optional. Precedence: static > dynamic > catch-all > optional. 13 tests in `router_tests.rs` — read those before inventing new patterns.

## Hard-won lessons (read these before you start)

### minijinja 2.x has NO mutation

No `list.append()`. No `namespace(n=0); {% set n.n = ... %}`. Any data shaping that needs mutation goes in the registry op, not the template. This burned two subagents; don't repeat it.

**Diagnostic:** if a template renders `HTTP 500 "could not render include: error in foo (in __routed__:N)"` with no inner detail, the culprit is almost always `.append` or namespace mutation. Minijinja's `{e}` formatter elides the chain — a framework improvement worth making: walk error chain and emit the innermost location.

### The 3-commit phased migration pattern

For bigger views (pipelines ~1024 lines, agents subroutes):
1. **Scaffold** — add registry ops + service `view_state.rs` submodule. Keep old handler wired. Build green.
2. **Templates** — rewrite `.ship/views/<name>/` to consume `ship.*`. Update shell router. Both paths functional.
3. **Delete** — rm bespoke `views_<name>.rs`, route variants, wiring, embedded `apps/shipd/templates/<name>/`. Build green, curl all routes 200.

Skills/git/specs used one commit because scope allowed. Larger migrations NEED phasing — subagents that try to do everything in one shot produce broken code.

### Subagent execution gotchas

- Subagents often lack shell/git access mid-session. They write files but can't `rm`, `cargo build`, or `git commit`. Expect to do the delete+build+commit yourself. They should hand back exact commands.
- The `services/*/view_state.rs` relocation pattern (lifting view-shaped types out of `views_*.rs` into service submodules) mirrors how agents/skills/git/specs/pipelines are now structured. Keep it consistent.
- File cap: 300 lines. `registry.rs` is the biggest outstanding offender.

## What's left

### Route rename (in progress when thread ended)

Drop the `runtime` segment from all view/view-as-app URLs:

| Old | New |
|-----|-----|
| `/api/runtime/view/{name}/raw` | `/api/view/{name}/raw` |
| `/api/runtime/view/{name}/manifest` | `/api/view/{name}/manifest` |
| `/api/runtime/view/{name}/bundle` | `/api/view/{name}/bundle` |
| `/api/runtime/view/{name}/context` | `/api/view/{name}/context` |
| `/api/runtime/view/{name}/sse` | `/api/view/{name}/sse` (generic per-view SSE — distinct from `/api/sse/*` routed) |
| `/api/runtime/view/{name}/asset/{file}` | `/api/view/{name}/asset/{file}` |
| `/api/runtime/view-sessions/*` | `/api/view-sessions/*` |
| `/api/runtime/view-errors` | `/api/view-errors` |
| `/api/runtime/views` | `/api/views` |
| `/api/runtime/views/{name}/meta` | `/api/views/{name}/meta` |

These are the view-as-app plumbing endpoints — still needed; just losing the `runtime` prefix. Chat routes (`/api/runtime/view/chat*`) also need updating if chat stays bespoke.

**Callers to update in lockstep:**
- `apps/shell/src/main.ts` — `viewRoute()` targets, `getRawUrl()`, pack loader URLs
- `apps/shipd/templates/_urls.html.j2` is build-script-generated — regenerates automatically
- Any remaining `/api/runtime/` hrefs in `.ship/views/` templates (grep + replace; the agent pass already caught dashboard)
- `packages/sdk/src/generated/routes.ts` regenerates automatically

### Known debt

1. **`ManifestData` fields `agents`, `pipelines`, `specs`, `git`, `collections`, `items`** in `views/resolve.rs` — remnants of the old manifest-data context-injection system. Unused by the routed handler but kept so legacy manifests parse. Remove the field list entirely once every view manifest drops them.
2. **`view_filters::ship_agents()`, `ship_collections()`** — minijinja globals from the old context-injection pathway. Still used by dashboard. Either migrate dashboard to `ship.*` SDK or keep these as sugar wrappers.
3. **`apps/shipd/src/views/state.rs`** — grab-bag of pipeline/skill helpers. Some lifted into `services/*/view_state.rs`; the rest should follow once their consumers migrate.
4. **`build_view_context`** in `views/handlers.rs` — legacy context builder, only used by the generic per-view SSE endpoint (`serve_view_sse` via `generic_sse.rs`). Needs review: does the routed SSE handler replace it? If yes, delete. If it has a role for third-party views, document it.
5. **registry.rs at 570 lines** — split into `registry/agents.rs`, `registry/skills.rs`, etc. Keep the dispatcher + `RegistryError` in `registry/mod.rs`.
6. **Error handling in `ship.*`** — spec §11.3 promised structured errors templates can render. Not implemented. Current behavior: service error → `RegistryError` → minijinja `InvalidOperation` → template render fails with 500. Fine for link errors (URL is the contract) but bad for soft-fail cases (e.g., `ship.skills.file()` when file was recently deleted but UI still links to it). Two approaches: (a) a `_opt` suffix convention that returns null instead of raising, or (b) a `ship.try(...)` filter. Punt until you actually feel the pain.
7. **Dry-run / context introspection** — `/api/runtime/view/{name}/context` used to let third-party devs see their template's context as JSON. Useful for extension debugging. Not preserved by the routed handler. Re-implement as `/api/v/{*path}?__dbg=context` or `/api/view/{name}/context`.

### Test coverage drift

- `views_raw_tests.rs::each_sdk_view_renders_its_root` was trimmed to just `chat` + the remaining SDK shell views as first-party views migrated. It's mostly vacuous now — delete when chat migrates, or recycle for testing the routed handler's pack injection.

## Framework stance for next thread

Keep compounding: every migrated view that needed a new primitive (pack injection, dynamic routes, compound filter ops) revealed a real gap. The remaining unknowns are chat (session-stateful + message-passing) and canvas views (overview, agent wire). The bet is that both resolve to "add a registry op + build a web component" — neither requires bespoke Rust per view. If chat turns out to need a new primitive (likely `ship.chat.stream()` or a shell↔extension message bus), that's a framework extension, not a failure.

The single sentence: **no Rust per view.** If you find yourself writing one, stop and add the primitive that makes it unnecessary.
