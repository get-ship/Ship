# User-app authoring — views outside the Ship repo

**Branch:** `job/user-app-authoring`
**Scope:** Path A from the job spec; Path B is the next iteration.

## Problem

The view resolver and listing endpoints were hardcoded to scan one of:

- `<repo-root>/views/<name>/page.sui` — built-in apps tree
- `<workspace>/.ship/views/<name>/index.sui` — project-scoped

If a user dropped `~/my-project/views/my-view/page.sui` into a workspace
that wasn't the Ship repo, the daemon never found it. `/sui/my-view`
returned 404 and the view never appeared in `/api/v1/views`.

The cause was twofold:

1. **Daemon resolver** (`ship-sui-server::resolver`) only consulted
   `SHIP_DEV_PROJECT_DIR` and a cwd-ancestor walk. With no env and a
   workspace lacking `.ship/`, there was no project root.
2. **Listing handlers** (`apps/shipd/src/views/handlers.rs`) only scanned
   `<ship_dir>/views/` (i.e. `.ship/views/`). The top-level `views/`
   directory under the workspace was never scanned for the registry.

## What changed (this PR)

### Resolver discovery (`crates/ship-sui-server/src/resolver.rs`)

`resolve_project_root` now consults:

1. `SHIP_DEV_PROJECT_DIR` env var — tests + the shipd `--workspace`
   bridge already write this on startup (`apps/shipd/src/lib.rs:171`).
2. `<daemon_dir>/workspace` file — `daemon_dir_for(cwd)` lands on the
   `machine/daemons/<hash>/` slot that shipd writes its workspace file
   into via `daemon::write_workspace_file`. This is the spec's
   canonical handoff: any in-process resolver call (axum handler,
   embedded compile, etc.) reads the same path the daemon committed to
   at boot, regardless of cwd drift.
3. The pre-existing cwd-ancestor `.ship/` walk remains the fallthrough
   for tools running with neither env nor a daemon record (CLI scripts).

No changes were needed to the underlying `ship-sui-host::resolver`. Once
`project_root` is correct, it already tries both
`<project_root>/views/<name>/page.sui` and
`<project_root>/.ship/views/<name>/index.sui`.

### Discovery module (`apps/shipd/src/views/discovery.rs`)

New module — single source of truth for "where do views live". Returns
an ordered list of `ViewSearchDir { path, source }`:

1. `<workspace>/views/`        → source: `workspace`
2. `<workspace>/.ship/views/`  → source: `project`
3. `<global>/views/`           → source: `global`

First hit per view-name wins so workspace-rooted user apps can shadow
built-ins by id without renaming. `workspace_root()` normalizes the
`get_project_dir` return value, which is sometimes the `.ship` dir (cwd
walk) and sometimes the workspace root (env override) — see the helper
doc comment for the full rationale.

### Listing + manifest resolution

`views/handlers.rs::list_views` and `views/resolve.rs::resolve_view_dir`
both delegate to `discovery::view_search_dirs()`. Net effect:

- `/api/v1/views` lists user-authored views from `<workspace>/views/`
  AND `<workspace>/.ship/views/` (plus global).
- Manifest discovery (icons + rail label via `manifest.jsonc`) flows
  through the same scan, so rail metadata Just Works once the file is
  on disk.

### Bundler (`runtime-assets/bundler.ts`)

`discoverViewBundles` now scans both `<workspace>/views/<name>/page.sui`
and `<workspace>/.ship/views/<name>/index.sui`. The file watcher also
includes `.ship/views/` so edits under that root trigger HMR like
top-level edits already did. `repoRoot` continues to be `SHIP_REPO_ROOT
?? process.cwd()` — shipd sets `SHIP_REPO_ROOT=<workspace>` when it
spawns the worker, so all paths are workspace-rooted.

## What the user does

A user with a non-Ship-repo workspace at `~/my-project/`:

```
~/my-project/
├── views/
│   └── my-view/
│       ├── page.sui                 # required — view source
│       ├── page.controller.ts       # optional — JS setup
│       ├── style.css                # optional — view-scoped CSS
│       └── manifest.jsonc           # optional — rail label/icon, type
```

Then:

```
shipd --workspace ~/my-project
```

Hitting `http://localhost:<port>/sui/my-view` renders the page. The
view shows up in `/api/v1/views` with `source: "workspace"`. If
`manifest.jsonc` declares a `rail:` block, the listing carries the
label + icon for Path B's dynamic-rail consumer.

A user preferring colocation with `.ship/` can instead place:

```
~/my-project/.ship/views/my-view/index.sui
```

Both layouts are first-class; pick whichever you prefer. Top-level
`views/` wins by precedence when both exist.

## Acceptance check vs. job spec

| Acceptance criterion                                                   | Status |
|------------------------------------------------------------------------|--------|
| View under a NON-repo workspace path renders at `/sui/<name>`          | ✅ Verified by `apps/shipd/tests/user_app_views.rs` |
| Audit doc covers design + remaining gaps                               | ✅ This file |
| `cargo` + tests pass                                                   | ✅ `cargo test -p ship-sui-server` green; full workspace test under `just test` |

## Gaps / next iteration

1. **Path B — dynamic rail.** Shell2's rail is still hardcoded in
   `apps/shell2/index.html`. The data source (`/api/v1/views`) now
   carries `rail.label` / `rail.icon` for every view including
   user-authored ones, so the shell-side work is "fetch on boot, render
   `<sui-rail-item>` per entry, filter to those that opt into the
   rail." Out of scope for Path A by the spec.

2. **Bundler output for non-Ship-repo workspaces.** The bundler still
   writes view bundles into `<repoRoot>/crates/ship-sui-server/assets/`.
   For a user workspace that isn't the Ship repo, that path doesn't
   exist and the bundle write fails. This is benign today because the
   daemon's compile-on-demand path (`serve_sui_view_setup` →
   `compile_view_js`) serves the view module without needing a
   pre-built artifact. Pre-building user views for prod-style serving
   would require relocating the output (e.g. `<workspace>/.ship/dist/`)
   and teaching `read_asset` about that location.

3. **Resolver caching.** `workspace_from_daemon_file` reads the daemon's
   `workspace` file once per `resolve()` call. The cost is negligible
   (small file, OS cache) but a `OnceLock` cache would shave a syscall
   per view payload. Skipped to keep the resolver stateless and
   trivially testable.

4. **`page.sui` controller naming.** `compile_view_js` infers
   `controller.ts` (for `page.sui`) vs `index.controller.ts` (for
   `index.sui`). User apps using `page.sui` should name their setup
   file `controller.ts`. Documented in
   `docs/ship-sui/references/authoring-patterns.md`.
