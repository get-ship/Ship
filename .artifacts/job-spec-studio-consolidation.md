# Job Spec: Studio consolidation — move domain views into the shell, claim `/studio/*`

**Branch:** `job/studio-consolidation`
**Agent:** web-lane (touches apps/shell + routing) or rust-lane if router config is Rust-side
**Mode:** autonomous
**CONDITIONAL:** dispatch ONLY after these have merged into `job/sui-framework`:
- `studio-surface` (Studio chrome + router at `apps/shell/src/studio/`)
- `agent-detail-keystone` (polished `views/agents/[id]/`)
- `agents-list-polish` (polished `views/agents/`)
- `themes-domain` (ported into `apps/shell/src/studio/themes/`)
- `components-domain` (ported into `apps/shell/src/studio/components/`)

This is the closing act that makes Studio shell-internal and the `/studio/*` namespace authoritative. Supersedes the earlier `old-routes-cleanup` spec (folded in here).

## Goal

Make Studio's framing physically true: all configuration-domain views live in `apps/shell/src/studio/`, reachable only via the protected `/studio/*` route namespace. No top-level domain routes (`/agents`, `/skills`, `/theme-studio`, `/ship-ui-studio`) survive. `views/` keeps only first-party runtime/workspace surfaces (home, sessions, events, git, mesh, canvas, artifacts, collections, pipelines-runtime) and user-authored views.

## What to change

### 1. Move agents + skills into the shell

```
views/agents/        →  apps/shell/src/studio/agents/
views/skills/         →  apps/shell/src/studio/skills/
```

These are wholesale moves (the polish jobs already landed the content; this just relocates). Fix every import path, controller reference, paired-file resolution, and `.sui` relative reference.

### 2. Delete the now-redundant old studios

`themes-domain` and `components-domain` already PORTED the logic into `apps/shell/src/studio/{themes,components}/`. So:

```
views/theme-studio/    →  DELETE (logic already at apps/shell/src/studio/themes/)
views/ship-ui-studio/  →  DELETE (logic already at apps/shell/src/studio/components/)
```

### 3. Route namespace

- The `/studio/*` namespace is claimed by `apps/shell/src/studio/`. Every section: `/studio/agents`, `/studio/agents/[id]`, `/studio/agents/[id]/{wire,settings}`, `/studio/agents/create`, `/studio/skills`, `/studio/skills/[id]`, `/studio/themes`, `/studio/themes/[id]`, `/studio/components`, `/studio/components/[id]`, plus the Project + Library sections.
- Top-level domain routes are removed. If external links to `/agents` etc. matter, add 301 → `/studio/agents`; otherwise drop them entirely. (Recommend: 301 for one release, then drop. Decide based on whether anything external links in.)
- `/theme-studio` → 301 `/studio/themes`. `/ship-ui-studio` → 301 `/studio/components`.

### 4. Catalog + bundler

- Update `bun packages/deck/bundler/index.ts` view-discovery to find the studio sections at their new `apps/shell/src/studio/` location.
- Update the SDK catalog generation if it enumerates view routes.
- The bundler currently bundles `views/<name>/view` — verify the studio sections build from the new path.

### 5. Sweep references

```
git grep "views/agents"        # update all
git grep "views/skills"
git grep "views/theme-studio"
git grep "views/ship-ui-studio"
git grep "/agents\b" / "/skills\b" / "/theme-studio" / "/ship-ui-studio"  # route refs in nav, links, docs, tests
```

Every hardcoded path / route reference gets updated to `/studio/<section>` or `apps/shell/src/studio/<section>/`.

## Architectural context

- This realizes the decision: **Studio ships with the shell.** It's not user content. `/studio/*` is protected.
- `views/` remains the home for first-party non-Studio surfaces AND user-authored views. Don't touch those.
- The runtime side of pipelines (`views/pipelines/` — the run monitor, history) stays in `views/`. ONLY pipeline *configuration* would eventually move to `/studio/pipelines`, and that's a future job (pipelines config isn't in this round).

## Acceptance criteria

1. `apps/shell/src/studio/{agents,skills,themes,components}/` all exist with their content.
2. `views/{agents,skills,theme-studio,ship-ui-studio}/` all deleted.
3. `git grep "views/agents\|views/skills\|views/theme-studio\|views/ship-ui-studio"` returns zero (outside this spec / migration notes).
4. `/studio/agents`, `/studio/agents/[id]`, `/studio/skills`, `/studio/themes`, `/studio/components` (and sub-routes) all resolve and render.
5. Old top-level routes either 301 to `/studio/<section>` or are gone — navigating to `/agents` does NOT render a standalone copy.
6. `bun run build` passes everywhere. `bun packages/deck/bundler/index.ts --all` succeeds with all studio sections building from the new location.
7. `cargo build --workspace` passes (routing config compiles).
8. All view test suites pass at their new locations.
9. Visual QA artifact at `.artifacts/web-qa/studio-consolidation/<timestamp>/`. Verify `/studio/agents`, `/studio/themes`, `/studio/components` all render inside the Studio chrome across ship-light + ship-dark.

## Out of scope

- Moving runtime views (home, sessions, events, git, etc.) — they stay in `views/`.
- Pipelines config → `/studio/pipelines` (future, pending design).
- MCP / Permissions sections (in-flight features, no vestigial UI).

---

> CONDITIONAL on the five prerequisite jobs merging. Verify with `git log job/sui-framework` before starting.
>
> Commit per logical move: agents move, skills move, old-studio deletions, route claims, catalog updates, sweep.
>
> When done: complete_workspace + mesh_broadcast.
