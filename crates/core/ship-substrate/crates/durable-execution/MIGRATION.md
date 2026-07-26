# H4 — migrate `durable-execution` → ship-exec

**Status:** prepared (docs only). Crate **not** moved yet — multi-repo
coordination required.

**Helm SOT:** `helm/data/blueprint/subsystems/durable-execution.json` (+ `.md`)

## Target

| | |
|---|---|
| **Repo** | [`ship-exec`](https://github.com/get-ship/ship-exec) |
| **Path** | `ship-exec/crates/durable-execution` |
| **Layout** | sibling of `ship-exec/crates/ship-js` |
| **Consume from ship** | git dep (same style as `ship-js` today) |

```
ship-exec/
  crates/
    ship-js/              # already here
    durable-execution/    # H4 target
  Cargo.toml              # members = ["crates/ship-js", "crates/durable-execution"]
```

## Why

- Thin leaf: only `serde` / `serde_json` / `thiserror` / `async-trait` /
  `futures-util` + `ship-js` (`default-features = false`).
- Already consumes ship-js from **git ship-exec** — belongs next to its contract.
- Keeps V8/dispatch evolution and journal-engine evolution in one exec plane.
- Substrate should not host pure execution leaves (sqlx adapters stay in ship).

## Current location

```
ship/crates/core/ship-substrate/crates/durable-execution/
```

Workspace member (ship root `Cargo.toml`).

### Reverse deps (must cut over)

| Consumer | Manifest | Current |
|---|---|---|
| **floki** | `crates/core/ship-platform/crates/floki/Cargo.toml` | `path = "../../../ship-substrate/crates/durable-execution"` |
| **journal-sqlite** | `crates/core/ship-substrate/crates/journal-sqlite/Cargo.toml` | `path = "../durable-execution"` |
| **shipd** | `apps/shipd/Cargo.toml` | `path = "../../crates/core/ship-substrate/crates/durable-execution"` |

No other `Cargo.toml` path-deps the crate (grep before delete).

### Does **not** move

- `journal-sqlite` (sqlx backend) — stays in ship-substrate
- shipd floki dispatch / runtime.db journal paths
- floki program wiring

## Steps

Do in this order. Two repos; land ship-exec first so ship can pin git.

### 1. ship-exec — add the crate

1. Copy the crate tree:
   ```sh
   cp -a ship/crates/core/ship-substrate/crates/durable-execution \
         ship-exec/crates/durable-execution
   ```
   Drop this `MIGRATION.md` from the copy after cutover (or keep a short
   “migrated from ship-substrate” note in the ship-exec README).

2. Edit `ship-exec/crates/durable-execution/Cargo.toml`:
   - Use `version.workspace = true`, `edition.workspace = true`,
     `license.workspace = true` (match `ship-js` style).
   - Point `ship-js` at **path** `../ship-js` with
     `default-features = false` (in-repo; no git self-dep).
   - Pull shared deps from `[workspace.dependencies]`.
   - Remove `[lints] workspace = true` **or** add
     `[workspace.lints]` to ship-exec root (ship-js today has no workspace
     lints — pick one posture for both crates).

3. Extend `ship-exec/Cargo.toml`:
   ```toml
   members = [
       "crates/ship-js",
       "crates/durable-execution",
   ]
   ```
   Add any missing workspace deps used by the engine:
   - `futures-util` (not currently in ship-exec workspace)
   - `serde` with `features = ["derive"]` if not already
   - Align **`thiserror`**: ship workspace uses `2`, ship-exec workspace
     currently pins `1`. Prefer one major across ship-exec crates; verify
     compile either way before publish.

4. Verify in ship-exec:
   ```sh
   cargo check -p durable-execution
   cargo test -p durable-execution
   cargo check --workspace
   ```

5. Commit + push `feat/next` on ship-exec so ship git deps resolve.

### 2. ship — retarget consumers

For each reverse dep, replace the path dep with git (mirror ship-js):

```toml
durable-execution = {
  git = "https://github.com/get-ship/ship-exec.git",
  branch = "feat/next",
}
```

Files:

1. `crates/core/ship-platform/crates/floki/Cargo.toml`
2. `crates/core/ship-substrate/crates/journal-sqlite/Cargo.toml`
3. `apps/shipd/Cargo.toml`

Optional local-dev only (never commit absolute paths into shared
manifests — see **Risk**):

```toml
# .cargo/config.toml or a personal [patch] — not for CI/worktrees
```

### 3. ship — delete the old member

1. Remove from ship root `Cargo.toml` `members` (and `default-members` if
   listed):
   `"crates/core/ship-substrate/crates/durable-execution"`
2. Delete the directory:
   `crates/core/ship-substrate/crates/durable-execution/`
3. Refresh lockfile:
   ```sh
   cargo check -p floki -p journal-sqlite -p shipd
   # or broader:
   cargo check --workspace
   ```
4. Grep for stragglers:
   ```sh
   rg 'durable-execution|durable_execution' --glob 'Cargo.toml'
   rg 'ship-substrate/crates/durable-execution'
   ```

### 4. Blueprint / docs follow-up

1. Flip helm subsystem gaps in
   `helm/data/blueprint/subsystems/durable-execution.json`
   (`req-home-ship-exec`, `req-consumers-git-dep`,
   `req-removed-from-ship-substrate`, `req-worktree-safe-deps` → `gap: false`).
2. Update `ship-exec/README.md` layout section to list durable-execution.
3. Update this file’s status line to **done** only if the success criteria
   hold (or delete this file with the crate).

## Risk

### Path vs git / worktrees (load-bearing)

- **ship-exec is a shared external repo** — it is **not** worktreed with ship.
  Only `ship` is worktreed under `Ship/worktrees/<n>/`.
- Relative path from a ship worktree → sibling `Ship/ship-exec` has a
  **different `..` depth** than from `Ship/ship/`. Relative path deps break
  helmd worktrees. This is why `ship-js` already uses **git**, and why the
  old absolute-path experiment was documented in this crate’s Cargo.toml
  comments (absolute path “works” on one machine, fails portability and is
  the wrong long-term pin).
- **Rule for H4:** committed consumer deps = **git ship-exec**, same as
  ship-js. Do not reintroduce absolute or relative path deps to ship-exec
  in ship manifests.
- Local operator override for unpushed ship-exec work: personal
  `[patch."https://github.com/get-ship/ship-exec.git"]` or a private
  `.cargo/config.toml` — not something the monorepo commits.

### Version / workspace skew

- `thiserror` 1 (ship-exec) vs 2 (ship) — resolve when adding the crate.
- Missing `futures-util` on ship-exec workspace until step 1.3.
- ship-exec may have unrelated WIP on `feat/next` (check `git status`
  before landing H4 on top).

### Multi-repo atomicity

- There is no single commit across ship-exec + ship. Prefer:
  1. ship-exec lands + is fetchable
  2. ship switches deps and deletes the member in one ship PR
- Avoid a window where ship deletes the path member before git source
  has the crate (breaks every consumer build).

### journal-sqlite coupling

- After the move, `journal-sqlite` is a ship crate that git-deps an
  external engine. That is intentional: heavy sqlx stays out of ship-exec.
- Integration tests that assumed a path workspace member still compile
  via git; first `cargo check` after cutover will fetch.

## Success criteria

- [ ] `ship-exec/crates/durable-execution` exists and is a workspace member
- [ ] `cargo test -p durable-execution` green in ship-exec
- [ ] floki, journal-sqlite, shipd depend on git ship-exec (no path to
      ship-substrate for this crate)
- [ ] **No** `durable-execution` under `ship/crates/core/ship-substrate/`
- [ ] ship root `Cargo.toml` does not list the old member
- [ ] `cargo check -p floki -p journal-sqlite -p shipd` green from a
      clean ship worktree (proves git resolution, not main-checkout path)
- [ ] helm subsystem gaps for H4 closed

## Explicit non-goals (this mission)

- Renaming floki → yoke
- Moving `journal-sqlite` or runtime.db schema
- Publishing to crates.io / private registry (git branch pin is enough
  until a registry cutover)
- Changing the public engine API

## Rollback

1. Revert the ship PR (restores path member + consumer paths).
2. Leave or delete the ship-exec copy; an unused member in ship-exec is
   harmless if ship no longer depends on it.

If ship-exec already released the crate and external consumers exist,
prefer a revert on ship only and keep ship-exec as the canonical home
(re-point ship at the last good rev).
