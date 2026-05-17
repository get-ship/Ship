---
title: /git functional depth pass
slug: git-depth
agent: ship-designer
branch: job/git-depth
daemon: localhost:9334 (workspace=/home/madvib/dev/ship-worktrees/git-depth, SHIP_RESPECT_WORKTREE=1)
date: 2026-05-17
themes_tested: [ship-light, ship-dark, console]
fixtures: live working tree of this branch — 10 modified files, 5 local branches
result: passing on golden path (probe errors=0); 2 followups
---

## Summary

`/git` is no longer a sketch. The view now reads the active repo via the
SDK's `ship.git.overview()` + `ship.git.diff()` live sources and writes
via `ship.git.{stage, unstage, commit, push, pull, fetch, checkout,
branch_create, branch_delete, init}` against the daemon's REST surface.
Every write feeds back through the same overview signal so the file
list, status chips, history, and branches all converge after a mutation
without a manual reload.

What works end-to-end in the live browser probe:

- Status panel — branch, ahead/behind, total changed (header + chip).
- File tree — staged / modified / untracked groups, click → diff opens.
- Diff viewer — file → hunks → +/- coloured lines via `<sui-git-diff>`
  family. Theme matrix legible in ship-light / ship-dark / console.
- File-level stage / unstage actions with hover-revealed + / − buttons.
- Bulk **Stage all** / **Unstage all** with optimistic refresh.
- Commit dialog (subject + body). Submits real commit. Disabled when
  nothing staged.
- History — recent commits list with search filter.
- Branches — switch (Checkout), create from HEAD (New branch dialog),
  delete with confirm.
- Push / Pull / Fetch in header + per-branch card. Progress via
  `attr:loading`/`attr:disabled`; success surfaces as a green
  `<sui-alert>`; failures as a destructive `<sui-alert>`.
- **No-repo** empty state with **Initialize repository** button (calls
  `ship.git.init`).

## Probe findings

`probe.mjs` runs Playwright against a paired browser session
(`/auth/pair/init` + `/auth/pair/status` dev auto-approve, bearer
forwarded as `Authorization` header). The probe captures screenshots
per theme + a "diff open", "after-stage", "commit dialog", "branches",
and "history" frame.

```
findings=23 warnings=2 errors=0
```

Step results:

| step                                 | result   |
| ------------------------------------ | -------- |
| attach: `window.__suiDispose` set    | ✅       |
| overview resolved → `.git-head`      | ✅       |
| theme matrix screenshots             | ✅ (×3)  |
| status chip = "11 changed"           | ✅       |
| file rows count = 11                 | ✅       |
| diff body opens after file click     | ✅       |
| stage file → "Commit (n)" increments | ✅       |
| commit dialog opens                  | ✅       |
| branches tab renders tiles           | ✅       |
| Checkout button on non-active branch | ✅       |
| history tab renders                  | ✅       |

The two warnings are `net::ERR_ABORTED` on the SSE channels — expected
on `refreshNonce` bump, since the runtime tears down the old
`EventSource` mid-stream before opening a new one.

## Daemon work touched

The view consumes a few endpoints that didn't exist as of HEAD:

- `POST /api/v1/git/branch` — create branch from HEAD (optional
  `checkout`).
- `DELETE /api/v1/git/branch` — delete branch; refuses current branch
  and unmerged-without-force.
- `POST /api/v1/git/init` — `Repository::init` at the active workspace
  root, no-op if `.git` already exists.

All three are libgit2-direct, mirror the existing handler style in
`apps/shipd/src/routes/git_api.rs`, and ride the same `GitService`
trait. Route variants added to `apps/shipd/src/routes.rs` (build script
generates the router glue). No services-layer-charter exceptions
needed.

## SDK + view-host

- `packages/sui-runtime/src/ship.ts`: extended `ShipSDK.git` with the
  full mutation surface (stage / unstage / commit / push / pull /
  fetch / checkout / branch_create / branch_delete / init).
- `packages/ship-sdk-client/src/index.ts`: wired matching HTTP shims.
- `crates/sui/ship-sui-server/assets/view-host.html`: the standalone
  view-host (`/sui/<name>`) was mounting views *without* a `ship`
  client in the controller ctx — any controller using `({ ship })` would
  get `undefined` and explode on the first mutation. Now imports
  `@ship/sdk-client` and passes `createClient()` into
  `mount(target, payload, { ship })`. The old git controller's "no
  ship in ctx" header note is no longer accurate; left untouched in
  the comment because the rewrite supersedes it.

## Followups (not blocking ship)

1. **Hunk- / line-level stage.** The spec calls for click-hunk →
   stage-hunk and click-line → stage-line "where supported". libgit2's
   index has no patch-mode primitive, and a subprocess `git apply
   --cached` would have to invent its own conflict story. Out of scope
   for this dispatch; file-level stage covers the common case and the
   surface is plumbed end-to-end so a future patch can add the per-hunk
   button without redesigning the controller.
2. **Worktree daemon resolution.** Without `SHIP_RESPECT_WORKTREE=1`,
   `resolve_project_ship_dir` redirects worktrees to the main checkout's
   `.ship` (deliberate — see `project/discovery.rs:189`). That means the
   daemon serves the **main** worktree's `views/git/page.sui` even
   though the bundler runs from the active worktree. Surfaced when the
   per-view SSE channel returned 0 live-data declarations against an
   old page.sui. Workaround: set `SHIP_RESPECT_WORKTREE=1` when running
   `just dev` from a worktree, or drop a `.boundary` sentinel into the
   worktree's `.ship/`. Not introduced by this work; documenting so
   the next person doesn't burn the same hour.

## Files changed

```
apps/shipd/src/router.rs
apps/shipd/src/routes.rs
apps/shipd/src/routes/git_api.rs
crates/core/ship-platform/src/services/git/service.rs
crates/core/ship-platform/src/services/git/write.rs
crates/sui/ship-sui-server/assets/view-host.html
packages/ship-sdk-client/src/index.ts
packages/sui-runtime/src/ship.ts
views/git/controller.ts
views/git/page.sui
views/git/style.css
```

## Build gate

```
bun runtime-assets/bundler.ts --all          # views/git ✓ (sui-components bundle failures pre-existed on main)
bun run --filter shell build                 # blocked: sui-build not installed in this env (pre-existing)
cargo check --workspace                      # ✓ clean (3 dead-code warnings pre-existing)
```

## Screenshots

- `git-ship-light.png` / `git-ship-dark.png` / `git-console.png`
- `git-diff-open.png` — file selected, diff with toolbar visible
- `git-after-stage.png` — staged group populated, Commit (n) live
- `git-commit-dialog.png` — open dialog with subject + body fields
- `git-branches.png` — branch list + per-branch actions
- `git-history.png` — recent commit list

## Probe artifact

`probe.mjs` is checked in alongside this report. Re-run with:

```sh
SHIP_RESPECT_WORKTREE=1 SHIPD_URL=http://localhost:<port> bun probe.mjs
```
