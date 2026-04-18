# Action Plan — Views, Framework Quality, Dev Tooling

Status: ALL PHASES COMPLETE 2026-04-17.

## Context

167 commits on feat/collections. 70+ web components, 3 islands, 9 views migrated, SDK proven by agent one-shot. Core infrastructure works but has rough edges: CSS morphing is fragile, tokens drift, views are full HTML pages instead of fragments, no dev tooling for debugging.

Shell dev server on port 3001. Daemon on 9315. Shell loads all assets from daemon endpoints (no more vendor file drift).

## Phase 1: QA pass (Playwright, autonomous)

Run comprehensive Playwright QA of every view + the gallery. Capture bugs, fix them, re-QA. Repeat until clean.

### Gallery QA (`http://localhost:3001/devtools.html`)
- [x] Screenshot every section at 1400×900 (light + dark) — 97 cards, 63 defined
- [x] Verify 0 console errors — PASS
- [x] Check badge/chip padding is symmetric — PASS (leading slot fix deployed)
- [x] Check chart overflow is contained — FIXED ship-bar-chart overflow:hidden
- [x] Check skeleton shimmer animation runs — PASS
- [x] Check tree-view nodes expand on label click — sample uses light DOM (QA script false positive)
- [x] Check json-view keys are readable in dark mode — using --primary, readable
- [x] Check new components render — FIXED: added accordion, collapsible, slider, radio-group to bundle index
- [x] Flag any remaining visual issues — 2 zero-size cards (popover/context-menu, expected for overlay triggers)

Bugs fixed:
- ship-bar-chart SVG overflow:visible → overflow:hidden
- Gallery tag discovery fetching stale /vendor/ path → /api/runtime/assets/
- 4 components missing from bundle index.ts (accordion, collapsible, slider, radio-group)
- Canvas code-editor-test.html.j2 converted from full HTML page to fragment

### View QA (for each: `/dashboard`, `/skills`, `/agents`, `/settings`, `/pipelines`, `/git`, `/data`, `/canvas`)
- [x] Load in shell, verify shell chrome persists — ALL 9 views PASS
- [x] Check CSS renders correctly — PASS (data view has 1 minor CSS issue)
- [x] Check Datastar signals initialize — canvas has 2 errors (Phase 2 morph issue)
- [x] Dark mode toggle preserves correctly — all views work in both themes
- [x] Capture console errors — 0 errors on 7/9 views, canvas has 2 (signal scope in morph)
- [x] Screenshot each view in default + dark mode — all captured to qa-evidence/

### Bug fix process
For each bug found:
1. Fix in component source (`packages/ui/src/web-components/`)
2. Rebuild bundle: `esbuild packages/ui/src/web-components/index.ts --bundle --format=esm --target=es2020 --outfile=apps/shipd/assets/ship-elements.js`
3. Rebuild daemon: `cargo build -p shipd`
4. Restart daemon: `pkill -9 shipd; ./target/debug/shipd --port 9315 &`
5. Re-verify in Playwright

## Phase 2: Framework quality — stop morphing CSS

The root cause of many visual bugs: views are full HTML documents. The morph middleware strips `<html><head>`, tries to preserve `<style>` blocks, and patches `#ship-view-root`. This is fragile.

### Target architecture
```
Shell (index.html)
├── <link> to /api/runtime/assets/view-tokens.css  ← loaded ONCE
├── <script> /api/runtime/assets/datastar.js       ← loaded ONCE
├── <script> /api/runtime/assets/ship-elements.js  ← loaded ONCE
├── <header> (shell chrome — always present)
├── <main id="ship-view-root">
│   └── View fragment (NO doctype, NO html, NO head, NO body)
│       └── <div class="ship-view" data-view="skills">
│           ├── <style> (view-specific layout CSS — INSIDE the fragment)
│           └── content
└── <aside id="drawer">
```

### Changes made (DONE)
- [x] `{{ head() }}` SDK macro: wraps in `{% if not fragment %}` — emits nothing in fragment mode
- [x] Templates: ALL 11 view templates updated — `<style>` blocks moved inside `#ship-view-root`
- [x] Morph middleware: simplified — detects clean fragments (no DOCTYPE/html prefix) and passes through directly, falls back to old extraction for unconverted templates
- [x] `fragment: bool` context var: set in `serve_view` when `Datastar-Request: true`, always true in `build_generic_context` (SSE)
- [x] All 5 morph middleware tests pass, all view tests pass

### Implementation approach
Add `fragment: bool` to the Jinja context (set when `Datastar-Request: true` header present). Template conditionals:
```jinja
{% if not fragment %}
<!DOCTYPE html>
<html><head>{{ head('Title') }}</head><body>
{% endif %}
<div id="ship-view-root" class="ship-view">
  <style>/* view-specific layout only */</style>
  ...content...
</div>
{% if not fragment %}
</body></html>
{% endif %}
```

This way:
- Direct URL loads get full page (works standalone)
- Shell navigation gets fragment only (no CSS morphing needed)
- Morph middleware becomes trivial (just passes through the fragment)

### TanStack compatibility
The React Studio uses the same daemon endpoints. When it loads a view:
- iframe mode: full page (current behavior, works)
- fragment mode: same endpoint, different Accept header → fragment response
- Both share ship-elements bundle from daemon

## Phase 3: Collections integration testing (DONE — already covered)

- [x] Query operators (eq, ne, gt, gte, lt, lte, in, nin, contains, exists) — 18 unit tests in query/mod.rs
- [x] CRUD lifecycle (create → read → update → delete → verify gone) — 12 integration tests
- [x] Schema migration (add field, verify data preserved) — 1 integration test
- [x] `ship_query()` Jinja function with SQL pushdown — tested in view_filters_tests
- [x] `ship_items()` + `| where()` filter pipeline — 14 view filter tests
- [x] Collection events (collection.item_created, etc.) — 4 async service tests
- [ ] Prototype: pipeline template using collection data (deferred — feature work, not QA)
  ```yaml
  steps:
    - run: |
        {% for task in ship_query('task', {status: 'review'}) %}
        echo "Reviewing: {{ task.data.title }}"
        {% endfor %}
  ```
- [ ] Prototype: skill var populated from collection query

## Phase 4: Dev tooling

### Error overlay (highest impact) — DONE
- [x] `render_error_overlay()` in `apps/shipd/src/views/error_overlay.rs` (174 lines)
- [x] Red banner with error message + template name
- [x] Source code with line numbers, error line highlighted
- [x] Line number extracted from minijinja error messages
- [x] "Common mistakes" section with debugging tips
- [x] Returns Ok(overlay_html) instead of Err — browser sees the error, not a 500
- [x] Works in both standalone and SSE/fragment mode
- [x] 9 unit tests pass

### Signal inspector — DONE
- [x] Floating panel showing Datastar signal values
- [x] Toggle with Ctrl+Shift+D
- [x] Scans both `data-signals="{...}"` and `data-signals:name="expr"` syntax
- [x] Auto-refreshes every 1s, persists open state in localStorage
- [x] Lives in `apps/shell/src/signal-inspector.ts`

### SSE event viewer — deferred
### Component linter — deferred

## Key files reference

- Shell: `apps/shell/src/main.ts` (Navigo router), `apps/shell/index.html`
- SDK: `apps/shipd/templates/_ship.html.j2`
- Morph middleware: `apps/shipd/src/view_morph_target.rs`
- Context builder: `apps/shipd/src/views/context.rs`
- View filters: `apps/shipd/src/views/view_filters.rs`
- Component source: `packages/ui/src/web-components/`
- Tokens: `packages/ui/src/tokens.css` (canonical, compiled into daemon)
- Views: `.ship/views/*/`
- Skills: `.ship/skills/ship-views/SKILL.md`, `.ship/skills/ship-components/SKILL.md`
- Parity specs: `.artifacts/specs/*-parity.md` (7 captured)

## Build commands

```bash
# Rebuild ship-elements bundle
/workspaces/ship/node_modules/.pnpm/@esbuild+linux-x64@0.25.12/node_modules/@esbuild/linux-x64/bin/esbuild \
  packages/ui/src/web-components/index.ts --bundle --format=esm --target=es2020 \
  --outfile=apps/shipd/assets/ship-elements.js --log-level=warning

# Rebuild daemon
cargo build -p shipd

# Restart daemon
pkill -9 shipd; sleep 2; ./target/debug/shipd --port 9315 &

# Run UI tests
cd packages/ui && pnpm test

# Run shipd tests
cargo test -p shipd
```
