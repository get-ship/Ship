# Studio design inventory — what we have / don't have

Durable record so the design discussion survives compaction. Source: 3 handoff
bundles from the designer (claude.ai/design). Latest is **v3**, preserved at
`.artifacts/studio-design-v3/` (the clean reorganized tree).

## Designs we HAVE (from the designer, v3)

| Area | Files | Status in code |
|---|---|---|
| **Shell chrome** | `shell/mock.html` (thin top/bottom + blank canvas), `shell/chrome/switcher.html` | ✅ BUILT + merged (shell-chrome-refactor) |
| **Studio chrome** | `shell/studio/shell.html` (rail + top + content) | ✅ BUILT + merged (studio-surface) |
| **Agents** | `agents/{list,detail,create,wire,settings}.html` | detail ✅ merged (keystone); list in flight (agents-list-polish); create/wire/settings = designs ready, NOT built |
| **Themes** | `themes/{list,detail}.html` | ✅ BUILT + merged (themes-domain) — folds /theme-studio |
| **Components** | `components/{list,detail}.html` | 🔄 in flight (components-domain) — folds /ship-ui-studio |
| **Skills** | `skills/{list,detail,invoke}.html` | designs ready, NOT built — mirrors keystone |
| **Project** | `project/{overview,create,isolation}.html` | overview = BUILDABLE (identity: name/slug/glyph/hue/theme); create = onboarding flow; isolation = DEFERRED (runtime not ready) |
| **Library** | `library/library.html` (installed/marketplace/authored + promote banner) | DEFERRED (no marketplace backend) |
| **Home / desktop** | `shell/home/{home,home-v2,home-v3}.html` | IN DISCUSSION — see below |

## Home — IN DISCUSSION (3 variants, none locked)

- `home.html` — widget grid (Events 2-row, Agents, Pipelines tiles)
- `home-v2.html` — atmospheric (day greeting, "Right now" focus card, "Today" timeline, pinned pill row)
- `home-v3.html` — connection-graph (agents wired to a hero pipeline, node aesthetic)

**Key realization (per user):** WIDGETS ARE JUST SHIP COMPONENTS. The desktop is a
view that places `<sui-*>` instances (sui-stat = counts, sui-list-tile = recent
runs, sui-chart = throughput) and whole views as tiles. There is NO "widget
primitive" to design. Home is therefore a *composition view*, buildable with what
we have — gated on (a) picking a layout we like, and (b) the arrange/edit
interaction IF user-composable. Not gated on infrastructure.

First-launch state today = the blank canvas (already shipped in shell-chrome).

## What's genuinely MISSING from the designer (short, real list)

1. A home/desktop layout we actually like — the 3 variants miss. Taste iteration, not infra.
2. The arrange/edit interaction for the desktop IF it's user-composable (drag/resize/pin tiles). The one new interaction-design question. Can ship fixed-layout-v1 first.

NOT missing (don't re-add these — they were weak/wrong earlier):
- ~~Cross-resource links~~ — that's just navigation (`<a href>`), no design needed.
- ~~Empty/loading/error states~~ — `sui-skeleton/spinner/error-state/empty-state` exist; `{#async}` wires defaults.
- ~~Widget primitive~~ — widgets ARE components; nothing to design.

## DEFERRED (blocked on RUNTIME, not designer) — don't build vestigial UI

- `project/isolation.html` — VM/cloud/container/resource budget. Build when runtime provides the isolation source/API.
- `library/library.html` authoring + promote-to-marketplace — build when marketplace backend exists.

## PATH.md caveat

The designer's `PATH.md` still carries the STALE framing ("Studio is design language,
not a route"). The actual decision (locked with the user): **Studio is shell-internal,
`/studio/*` is a protected route namespace, code lives in `apps/shell/src/studio/`.**
The v3 file tree follows the new framing even though PATH.md text lags. Trust the tree
+ this inventory, not PATH.md's prose.

## Buildable-now queue (not blocked, fully backed)

1. project-overview (identity screen)
2. skills section (list/detail/invoke)
3. agents sub-views (create/wire/settings polish)
4. studio-consolidation (move into apps/shell/src/studio/, after agents-list + components land)
