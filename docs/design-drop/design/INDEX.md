# design/ — INDEX

The one accurate map of local design work. If something here is wrong, fix it
here first. No other doc gets to describe this folder.

## The model

Ship = **2 host scaffolds** + **first-party views** + **component packs**, all
driven by the **platform services** (RPC). We design these; the services drive
*when* and *what*. Top-level buckets, each with one job:

```
design/
  scaffolds/   the 2 host scaffolds + chrome composition (cross-cutting)
  domains/     ONE folder per platform service — first-party views (the spine)
  components/  component-library design — sui/ + ship/ (mirrors the repo)
  mocks/       legacy bucket; currently holds the marketing track only
  _archive/    abandoned work — moving a file here IS "abandoned"
  INDEX.md     ← you are here (the only accurate map)
```

At repo root, *outside* `design/`: **`sui/`** is the built runtime our mocks load
(tokens: `sui/ship.css`); **`tools/`** is our shared design tooling (the catalog
viewer — a tool, not a mock); **`reference/`** is read-only repo source we only
read; **`site/`** is the separate marketing product. Design iteration lives in
`design/`.

## The unit of work is a domain

When a platform service completes upstream, it lands a **handoff**. Copy
`domains/_TEMPLATE/` → `domains/<service>/` and fill `handoff.md`. Everything
for that job then lives in one folder:

```
domains/<service>/
  handoff.md     the contract — RPC surface · requirements · components · status
  fixtures/      daemon-shaped sample data (no invented fields)
  pack/          domain-pack components designed here (the new/extend ones)
  view/          optional related first-party view mock
  *.html         the interaction / UX mocks
```

**An agent doing a task reads `domains/<service>/handoff.md` and nothing else
to orient.** That's the whole point — one read, one folder, no hunting.

## Status — the system

Every deliverable has one status, drawn from a fixed vocabulary:

`draft · in-review · approved · changes · shipped · abandoned`

Three surfaces, each unambiguous:

1. **Review pane** — the verdict you click. Registered deliverables appear there
   as your queue: `needs-review → approved` / `changes-requested`. This is the
   live approve action.
2. **`_archive/`** — moving a file there *is* `abandoned`. Git remembers.
3. **The board below + each `handoff.md` `Status:` line** — the human ledger.
   Update the row when status changes. "approved" = ready for eng to implement;
   "shipped" = ported to `madvib/Ship`, local mock can be archived/deleted.

## Deliverables — status board

The at-a-glance ledger. Edit the Status when you review (and approve in the pane).

| Deliverable | File | Status | In review pane |
|---|---|---|---|
| Artifacts — components | `domains/artifacts/components.html` | **in-review** | per-component status |
| Artifacts view (composition) | `domains/artifacts/artifacts.html` | in-review | needs-review |
| Spaces interaction model | `scaffolds/spaces/interaction-model.html` | in-review | needs-review |
| Shell — nav frame | `scaffolds/shell/nav-frame.html` | draft | needs-review |
| Home dashboard | `domains/home/desktop.html` | draft | needs-review |
| Marketing landing | `mocks/marketing/landing.html` | in-review | needs-review |
| Collections — components | `domains/collections/components.html` | **in-review** | per-component status |
| Collections Data Studio (view) | _not built_ | staged | — |
| Components — sui overview | `components/sui/overview.html` | **in-review** | per-component status |
| Components — forms pack | `components/sui/forms/` | **in-design** ← active | per-component status |

_Versions (`landing-v1…v5`, `nav-frame-v1`) are compare history, not separate
deliverables. Abandon one by moving it to `_archive/`._

## Domain roster (work queue)

Source of routes: `reference/views/manifest.json`. Status hand-maintained here.

| Domain | Route | Handoff | Status |
|---|---|---|---|
| home | `/` | — | mock exists (`domains/home/desktop.html`), no handoff yet |
| artifacts | `/artifacts` (+`[id]`) | **2026-06-01** | **in-design** — mock built (`domains/artifacts/artifacts.html`), in review |
| collections | `/collections` | **2026-06-01** | **staged** — spec + fixtures in; Data Studio not built |
| events | `/events` | — | pending |
| agents | `/agents` (+create/[id]/settings/wire) | — | pending |
| sessions | `/sessions` | — | pending |
| pipelines | `/pipelines` (+focus/run/history) | — | pending |
| git | `/git` | — | pending |
| mesh | `/mesh` | — | pending |
| canvas | `/canvas` | — | pending |
| skills | `/skills` | — | pending |
| specs | `/specs` | — | pending |
| settings | `/settings` | — | pending |
| ship-ui-studio | `/ship-ui-studio` | — | pending |
| theme-studio | `/theme-studio` | — | pending |
| setup | `/setup` | — | pending |

_Fill in the Handoff column with a date when a service lands one._

## Component library (not domains)

The `sui-*` / `ship-*` packs are the framework's own library — not platform
services, so they don't fit `domains/`. Their design lives in **`components/`**,
mirroring the repo's `packages/components/{sui,ship}` split. Each pack folder has
three surfaces — **overview** (catalog of states × 6 themes × verdict),
**iteration canvas** (design directions side-by-side), **handoff** (decision →
coding agents). See `components/INDEX`-level READMEs:

| Where | What |
|---|---|
| `components/sui/overview.html` | cross-pack overview of the new `sui-*` packs |
| `components/sui/review.md` | the written design review (4 axes, prioritized) |
| `components/sui/forms/` | **active** — forms pack; combobox in iteration |
| `components/ship/` | stub — `ship-*` tier, designed after `sui` settles |

Catalog tooling is shared from `/tools/catalog/` (`viewer.js` · `viewer.css` ·
`ship-ui.css`). Verdicts persist per component (localStorage).

## Scaffolds & marketing (not domains)

| What | Where | Notes |
|---|---|---|
| Spaces / hub scaffold | `scaffolds/spaces/` | `interaction-model.html` (open this) + scene-*.jsx |
| Shell scaffold | `scaffolds/shell/` | `mock.html`, `nav-frame.html`, `home/`, `studio/` |
| Marketing | `mocks/marketing/` | `landing.html` + v1–v5, self-contained `sui/`. Path-fragile — left in place. |

## Path rule (don't break mocks when moving them)

Every mock loads `sui/` by relative path keyed to its **depth**. Moving a mock
between equal-depth folders is safe; changing depth breaks `../../../sui/`.

| Mock location | Path to `sui/` |
|---|---|
| `design/<bucket>/<file>.html` (depth 2) | `../../sui/` |
| `design/<bucket>/<route>/<file>.html` (depth 3) | `../../../sui/` |
| `design/<bucket>/<route>/<sub>/<file>.html` (depth 4) | `../../../../sui/` |

Shared tooling is at repo root `/tools/catalog/` — reach it with the same number
of `../` as `/sui/` from any mock (e.g. depth-3 mock: `../../../tools/catalog/`).

## The SUI rule

No daemon here, so `ship.*` SDK calls don't resolve. Mocks must: use real
`<sui-*>` tags + real prop names, feed them static fixtures shaped like daemon
responses, and skip live runtime state. A mock that would compile to a working
`page.sui` is the only mock worth making.
