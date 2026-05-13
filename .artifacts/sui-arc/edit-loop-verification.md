# Edit-loop verification — shell2 / SUI HMR

**Question asked:** "does the shell fucking load pages?"

**Short answer:** It loads pages on first navigation and on manual reload. The
in-place hot-reload path — author edits a `.sui` / `.controller.ts` / `.css`
and the change appears in the running shell without a reload — is **broken end
to end** in `apps/shell2` today. Every layer downstream of the watcher works;
the shell never subscribes to the HMR SSE because of a one-line bug in
`apps/shell2/src/hmr.ts`.

## How this was measured

- Probe: `apps/shell2/edit-loop-probe.mjs` — Playwright + real fs edits +
  `pnpm -F shell2 build`. Restores every mutated file in a `finally` block.
- Daemon: stock `just dev-restart` against this worktree's daemon
  (`/home/madvib/.ship/machine/daemons/a25049ff`, port 9367).
- Shell2 vite: `pnpm -F shell2 dev` on the matching port 3252.
- Bundler: `~/.ship/runtime/bundler.ts`, freshly synced from
  `runtime-assets/bundler.ts` via `just dev-runtime-update`. The pre-existing
  installed copy was from an older `@ship/ui-runtime` era of the repo
  (`packages/ui-runtime/*` paths that no longer exist) and was producing bad
  bundles — a sub-finding worth flagging on its own (see Gap 5).

Run the probe yourself: `SHELL2_URL=http://127.0.0.1:3252 node apps/shell2/edit-loop-probe.mjs`.

## Per-step result

| # | Scenario | Result | Smallest gap-closer |
|---|---|---|---|
| 1 | Edit `views/home/page.sui` → see change without reload | **FAIL** | Fix `isDev()` in `apps/shell2/src/hmr.ts` (Gap 1). After that, the in-place re-mount in `defaultOnChanged()` already does the right thing. |
| 2 | Edit `views/home/controller.ts` → see change without reload | **FAIL** | Same root cause as #1. The bundler treats `controller.ts` as part of the `views/home/view` target, the rebuild emits `module-changed`, but no client listens. |
| 3 | Edit `views/home/style.css` → see change without reload | **FAIL** | `.css` is not in the bundler watcher's extension list and view CSS is read fresh from disk only at payload-fetch time. New CSS becomes visible on the next navigation-away-and-back (probe verified `cssApplied=true` after a goto `/dashboard` → goto `/home` round-trip). See Gap 2. |
| 4 | Add `views/edit-probe-tmp/{page.sui,controller.ts,manifest.jsonc}` | **PARTIAL** | `/api/v1/views` lists it immediately (handler rescans the views dir per request). Direct nav to `/edit-probe-tmp` renders because `serve_sui_view_setup` falls through to on-the-fly compile when the bundler asset is missing. The **rail** is hardcoded in `apps/shell2/index.html` (lines 103–113) — a new view never appears in the rail without an index.html edit. See Gap 3. |
| 5 | Dev → prod build (`pnpm -F shell2 build`) | **PASS** | Vite build exits 0, produces `apps/shell2/dist/index.html` (~14.7 KB) plus the hashed bundle tree. Note: this only validates the shell entry point. The daemon's production mode (`shipd` without `--dev`) is exercised separately by `cargo build --release -p shipd`; it disables the HMR routes (see the existing tests in `apps/shipd/src/views/sui_hmr.rs` lines 287–319). |

## Findings (ordered by what most blocks the user's question)

### Gap 1 — shell2's `isDev()` returns false in vite dev

**File:** `apps/shell2/src/hmr.ts` lines 81–86

```ts
function isDev(): boolean {
  const meta = import.meta as ImportMeta & { env?: { DEV?: boolean } }
  if (meta.env?.DEV === true) return true
  const w = window as Window & { __SHIP_DAEMON_MODE?: string }
  return w.__SHIP_DAEMON_MODE === 'development'
}
```

Vite's env transform only injects the runtime `import.meta.env` object on
modules that reference `import.meta.env` **as a literal sequence**. Aliasing
through `const meta = import.meta` defeats the transform — `meta.env` is
plain `undefined` at runtime. `isDev()` returns false, `initHmr()` returns
`null`, and the shell never opens an `EventSource` to `/_sui/dev/hmr`.

Confirmed two ways:
- Direct fetch of the transformed module via `curl /src/hmr.ts` returns
  `if (meta.env?.DEV === true)` with **no leading `import.meta.env = {...}`
  injection** — compare to a sibling file that uses the literal form, which
  Vite prepends with `import.meta.env = {"BASE_URL": "/", "DEV": true, ...}`.
- The probe wraps the page's `EventSource` constructor; after `mountRoute`
  resolves, `window.__esUrls` contains only `/_sui/data?expr=ship.git.overview()`
  — the data live stream — and never the HMR endpoint. Manually invoking
  `subscribeHmr({onChanged: ...})` from devtools immediately opens it and
  receives events as soon as the next bundler rebuild fires.

**Smallest fix (one line):**

```ts
function isDev(): boolean {
  if (import.meta.env?.DEV === true) return true            // direct, transformed by Vite
  return (window as Window & { __SHIP_DAEMON_MODE?: string }).__SHIP_DAEMON_MODE === 'development'
}
```

Not landed: `apps/shell2/src/hmr.ts` is outside this job's allowed file
scope. The job spec lists `apps/shipd/src/views/sui_hmr.rs`,
`packages/sui-runtime/src/mount.ts`, and `runtime-assets/bundler.ts` as the
only client/server hot-paths in scope — the shell glue is not. Flagging this
as the single highest-leverage fix; the entire HMR pipeline downstream of it
already works.

### Gap 2 — bundler watcher ignores `.css`

**File:** `runtime-assets/bundler.ts` line ~371

```ts
const ext = filename.split('.').pop()
if (!['ts', 'tsx', 'js', 'sui'].includes(ext ?? '')) return
```

`.css` is excluded, so saving `views/home/style.css` triggers nothing. The
CSS is also not part of any bundle target — it is read fresh by
`crates/ship-sui-server/src/view_payload.rs:collect_view_css()` only at
payload-serve time (`GET /_sui/views/<name>`). So a manual reload (or
navigation away and back) picks up the new CSS, but no HMR signal nudges the
shell to re-fetch.

**Why a one-line "add `css` to the extensions list" fix is wrong:** the
watcher then calls `targetsForFile(absPath)`, which would return `[]` for a
.css file (no target has a CSS entry) and fall through to `toRebuild = TARGETS`
— a full rebuild on every CSS save. Worse than today.

**Smallest correct fix (≤30 LOC, allowed by the spec):** emit a
new bundler event kind `view_css_changed { view: string }` when a
`views/<name>/style.css` (or `view.css`) save is detected; surface it as a
`view-css-changed` SSE event on `/_sui/dev/hmr`; shell2's hmr glue re-fetches
just the `/_sui/views/<name>` payload and swaps its adjacent `<style>` node.
Out of scope for *this* job (the prerequisite — shell2 HMR running at all —
is Gap 1).

### Gap 3 — the rail is hardcoded; new views don't auto-register

**File:** `apps/shell2/index.html` lines 103–113

The rail is a literal list of `<sui-rail-item>` elements in
`index.html`. Adding `views/<name>/manifest.jsonc` with `rail: { label, icon }`
populates `/api/v1/views` (and the launcher, command bar, and nav-top, which
all read the catalog), but **not** the rail. The probe creates
`views/edit-probe-tmp/`, watches the catalog flip to include it, navigates to
`/edit-probe-tmp` successfully, and confirms the rail still doesn't contain it.

This is more like a deliberate omission than a bug — the rail is the only
chrome surface in shell2 that isn't driven from the catalog. Two fixes are
defensible:

1. Replace the static markup with a controller that fetches `/api/v1/views`
   on mount and renders rail items from manifests whose `rail.label` is set.
   ~30 LOC in `apps/shell2/src/main.ts` or a new
   `packages/ui-components/src/sui-rail/sui-rail-from-catalog.controller.ts`.
2. Or: leave the rail static and document that "new views show up in the
   command bar / launcher; the rail is a curated subset." Then the
   `manifest.rail` field becomes documentation-only.

Out of scope here (no file in the allowed list owns the rail).

### Gap 4 — bundler watcher: virtual entries match every file under repoRoot

**File:** `runtime-assets/bundler.ts` lines 344–353

```ts
function targetsForFile(absPath: string): BundleTarget[] {
  return TARGETS.filter((t) => {
    if (t.precompile?.some((p) => p.path === absPath)) return true
    const entryAbs = resolve(repoRoot, t.entry)
    if (entryAbs === absPath) return true
    const entryDir = dirname(entryAbs)
    return absPath.startsWith(entryDir + '/')
  })
}
```

Island targets (`sui-islands/sui-*`) use `entry: '<virtual>'`. `resolve(repoRoot, '<virtual>')`
returns `<repoRoot>/<virtual>`, so `entryDir` is `<repoRoot>`, and every file
in the worktree satisfies the `startsWith(entryDir + '/')` test. **Every
single save triggers a rebuild of all 8 island bundles** on top of whatever
real target the file belongs to. Probe trace: editing
`views/home/page.sui` produces 9 `module-changed` events — 1 expected
(views/home/view) + 8 unwanted (sui-islands/*).

Once Gap 1 is fixed, this would cause the shell to call `location.reload()`
on every keystroke (the shell's hmr glue treats any non-`views/<X>/view`
target as a full-reload trigger). So this becomes the **second** thing to
fix.

**Smallest fix (~6 lines, within the spec's 30-LOC budget for bundler.ts):**
ship `discoverIslandBundles()` should attach the island's real source
directory to the target, and `targetsForFile` should skip the directory
heuristic for `virtualEntry` targets and instead check against that recorded
directory.

```ts
// in discoverIslandBundles():
out.push({
  name: `sui-islands/${e.name}`,
  entry: '<virtual>',
  virtualEntry: ...,
  external: [...],
  outFile: ...,
  watchDir: dir,           // ← new
})

// in targetsForFile():
if (t.virtualEntry) {
  return t.watchDir ? absPath.startsWith(t.watchDir + '/') : false
}
```

Not landed in this PR — landing it without fixing Gap 1 first changes nothing
observable, and landing it after Gap 1 is fixed is the right ordering. It is
in-scope (`runtime-assets/bundler.ts` is allowed) and it is the simplest
correct fix; we explicitly chose to defer.

### Gap 5 — installed bundler/runtime can silently drift from the repo

**Files:** `~/.ship/runtime/bundler.ts` vs `runtime-assets/bundler.ts`

The bundler the daemon actually executes lives at `~/.ship/runtime/bundler.ts`,
populated by `just dev-runtime-install` (or `update`). It is not rebuilt by
`just dev-restart`. When the repo file changes — e.g. the
`@ship/ui-runtime` → `@ship/sui-runtime` rename, the
`crates/ship-ui-framework/assets` → `crates/ship-sui-server/assets` move —
the runtime can be left pointing at packages that no longer exist. The
symptom in this worktree was that any controller.ts save produced a
`compile-error` for `views/home/view` (Bun.build "Bundle failed") because
the externals listed packages whose names no longer matched. Fixed by
re-running `just dev-runtime-update`.

Possible mitigations, none landed:

- `dev-restart` could checksum `runtime-assets/bundler.ts` and bump
  `~/.ship/runtime/bundler.ts` when it differs.
- The bundler could announce its version/source-checksum in the `ready`
  event and the daemon could log a warning if it diverges from the repo
  file.

## Code smells noted but not acted on

- `shell2's hmr.ts` chooses `location.reload()` as the recovery path for
  any failure during in-place re-mount, including for ANY non-view target
  change (runtime, ship-elements, sui-components, every island). Once Gap 1
  is fixed, expect one full reload per save of any component file. A
  custom-element re-register dance is documented as v2.
- The data-live `<style>` block lives **inside** the home-hero shadow root,
  so `document.body.innerText` doesn't pick up the hero title — bit me
  during probe development. Worth a one-line note in the SUI authoring
  guide that probes targeting hero/title text should use
  `el.shadowRoot?.textContent`.
- `views/home/page.sui` console errors visible in the probe transcript
  (`__t12 is not defined` in the compiled sui-nav-top island, plus
  `scope.headerEl is not a function` and `scope.bodyEl is not a function`
  in the components bundle) are present on a clean load — pre-existing,
  unrelated to HMR, but worth a follow-up: they imply a stale or
  mis-numbered template slot reference in the sui-nav-top bundle.

## What the user can do today

- First-load navigation: works.
- Hard reload after a save: works (for `.sui`, `.controller.ts`, and `.css`).
- Hot reload while authoring: does not work in shell2 today. Use **shell1**
  (`apps/shell`) if you need a live edit loop, or land the one-line fix to
  `apps/shell2/src/hmr.ts:isDev()` before continuing in shell2.
