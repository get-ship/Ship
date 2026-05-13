# SUI Naming Decisions — Lit→SUI Port Gaps

**Branch:** job/sui-naming-decisions  
**Date:** 2026-05-12

These four items were flagged in `lit-port-plan.md` as "apparent gaps that are actually naming mismatches." Each section below compares the Lit component's surface with its SUI counterpart and issues a concrete porting decision.

---

## 1. ship-tree-node vs sui-tree-item

**Verdict: Already done. sui-tree-item fully covers ship-tree-node.**

`ship-tree-node` (Lit) and `sui-tree-item` (SUI) are the same concept under different names. The rename follows the SUI namespace convention (`ship-` prefix → `sui-`) and normalizes "node" to the more semantically accurate "item" (the ARIA spec uses `treeitem`, not `treenode`).

Full prop parity:

| Lit prop | SUI prop | Status |
|---|---|---|
| `label` (string, `''`) | `label` (string, `''`, reflect) | ✓ |
| `expanded` (bool, `false`) | `expanded` (bool, `false`, reflect) | ✓ |
| `selected` (bool, `false`) | `selected` (bool, `false`, reflect) | ✓ |
| `leaf` (bool, `false`) | `leaf` (bool, `false`, reflect) | ✓ |

Full event parity:

| Lit event | SUI event | Status |
|---|---|---|
| `ship-toggle { expanded }` | `sui-toggle { expanded }` | ✓ (prefix renamed) |
| `ship-select {}` | `sui-select {}` | ✓ (prefix renamed) |

Slot parity: `(default)` nested items, `icon` (leading), `trailing` — identical.  
Part parity: `row`, `caret`, `label`, `children` — identical.  
ARIA: `role=treeitem` on host, `aria-expanded` (non-leaf only), `aria-selected` — identical.  
Keyboard: Enter/Space → select, ArrowRight → expand, ArrowLeft → collapse — identical.

SUI adds one capability the Lit version lacks: the `prop:node={n}` data-binding path in the controller, which derives `nodeLabel` and `nodeChildren` from a `TreeNode` object. This is a superset, not a gap.

**No follow-up port task needed.** The port is complete.

---

## 2. ship-progress-ring vs sui-progress

**Verdict: Different shapes of the same concept. Port ship-progress-ring as `sui-progress/sui-progress-ring.sui`.**

`sui-progress` is linear-only (CSS width-animated bar). `ship-progress-ring` is circular-only (SVG stroke-dashoffset arc). They share the `progressbar` ARIA role and the 0–100 value model but diverge completely in rendering:

| Attribute | sui-progress | ship-progress-ring |
|---|---|---|
| Shape | Linear bar | Circular arc (SVG) |
| `value` | 0..`max` (default 0) | 0..100 (implicit max) |
| `max` | configurable (default 100) | fixed 100 |
| `size` | enum: `sm\|md\|lg` | arbitrary px (default 48) |
| `thickness` | not applicable | stroke-width in px (default 4) |
| `tone` | union vocabulary, reflect | same vocabulary |
| `indeterminate` | explicit bool prop | not supported |
| Slot | none | `(default)` → centered label |
| `label` | maps to `aria-label` | maps to `aria-label` |

A single `variant=` merge on `sui-progress` is impractical because the prop shapes are incompatible: the ring needs `size` in raw px and a `thickness` prop, while the bar uses an enum `size` and has an `indeterminate` mode with no ring equivalent. Forcing these into one component would produce a prop surface that is partially meaningless depending on which variant is active.

The right pattern is co-location: add `sui-progress-ring.sui` inside `packages/ui-components/src/sui-progress/`. This mirrors how `sui-toast` and `sui-toast-container` live in the same directory as two related but structurally distinct components. The shared `sui-progress/` directory signals the conceptual family without conflating the implementations.

**New file needed:** `packages/ui-components/src/sui-progress/sui-progress-ring.sui` (plus paired `controller.ts`).

Props to carry over: `value` (0–100), `size` (px), `thickness` (px), `tone`, `label` (→ `aria-label` on SVG).  
Slot to carry over: `(default)` → centered label.  
Note: omit `max` (ring hardcodes 100); add `label` as an explicit prop (Lit reads it from the attribute but doesn't declare it as a `static properties` entry — the SUI port should formalize it).

---

## 3. ship-step-segments vs sui-stepper

**Verdict: Different concepts. Port ship-step-segments as a new `sui-step-segments` directory.**

`sui-stepper` is a wizard-flow indicator: it shows "step N of T" with dots where the active position is a pill. Its data model is `(total, current)` — a single position in an ongoing sequence. Its ARIA role is `progressbar`.

`ship-step-segments` is a pipeline-outcome visualizer: it renders a compact row of colored rectangles showing the pass/fail/running status of each step in a completed (or in-progress) pipeline run. Its data model is a per-step outcome array: `values="done,done,failed,pending"`. Its ARIA role is `img` with a label like "4 step outcomes."

The concepts are orthogonal:

| Dimension | sui-stepper | ship-step-segments |
|---|---|---|
| Use case | Where am I in an ongoing flow? | What happened in each step? |
| Data model | `total` + `current` (position) | `values` (outcome array) |
| States | `active`, `done`, `pending` | `done`, `failed`, `pending`, `running` |
| `failed` state | none | yes |
| Animation | none | `running` → pulse |
| Interaction | read-only | read-only |
| ARIA role | `progressbar` | `img` |

Folding step-segments into stepper as a `variant=` prop would be wrong for three reasons: (1) the data models cannot share props — `total + current` vs an outcome string are mutually exclusive inputs; (2) the ARIA role changes between variants, which is not a valid use of variant; (3) the semantics are unrelated — one answers "where are you?" and the other answers "what happened?".

**New directory needed:** `packages/ui-components/src/sui-step-segments/sui-step-segments.sui`.

Props to carry over: `values` (comma-separated `done|failed|pending|running`), `size` (`sm|md`).  
Behavior to carry over: unknown tokens → `pending`; empty/absent `values` → render nothing; `running` → pulse animation.  
ARIA: `role=img`, `aria-label="{N} step outcomes"`.

---

## 4. ship-view-chrome scope

**Verdict: General platform primitive. Port as `sui-view-chrome`.**

`ship-view-chrome` is a headless "chrome contribution" element: a view declares one `<ship-view-chrome>` and the Shell paints the topbar and statusbar on its behalf. On `connectedCallback` it teleports its slot children into shell-owned containers (`#view-topbar-actions`, `#view-status-start`, `#view-status-end`) and writes text into shell-owned nodes (`#breadcrumbs-title`, `#status-branch`, `#status-ab`, `#status-dot`). On `disconnectedCallback` it withdraws everything. The component itself renders nothing visible.

The component explicitly documents a "no-shell fallback": when the target containers do not exist, the element is invisible and no-ops — safe to include in previews or tests. This defensive design confirms it was written as a reusable protocol, not as an internal shortcut.

The doc comment is the strongest signal: *"Keeps first-party and third-party views symmetric."* This is a platform API. Every view that runs inside Ship's shell — both L2 built-in views and L3 user apps — needs a way to contribute breadcrumb, branch status, and action buttons to the topbar without reimplementing the teleportation logic themselves. If `sui-view-chrome` were not ported, L3 developers would have no way to participate in standard shell chrome.

The tight coupling to specific DOM IDs (`#view-topbar-actions`, etc.) is a feature, not a smell — the IDs are the shell's public contract with views, analogous to a slot API. The SUI port should preserve that contract exactly.

**Port as `sui-view-chrome` in `packages/ui-components/src/sui-view-chrome/`.** The controller replaces `LitElement.createRenderRoot()` + manual DOM ops with `mount`/`unmount` lifecycle callbacks, which map cleanly onto the existing `_contribute()` and `_withdraw()` logic. The `<style>` block is empty (the element renders nothing). Props to carry over: `title`, `branch`, `ahead`, `behind`, `tone`, `busy`. Slots to carry over: `actions`, `status-start`, `status-end`.

---

## Decision Table

| Item | Decision | Action |
|---|---|---|
| tree-node | Already done — sui-tree-item is a complete, superset port of ship-tree-node | None. Port is complete. |
| progress-ring | Different shape of the same concept; merge into one component is impractical | Add `packages/ui-components/src/sui-progress/sui-progress-ring.sui` + controller. |
| step-segments | Different concept from sui-stepper; data model, ARIA role, and semantics are incompatible | New `packages/ui-components/src/sui-step-segments/sui-step-segments.sui` + controller. |
| view-chrome | General platform primitive used by all views (L2 + L3); doc confirms third-party symmetry intent | Port as `packages/ui-components/src/sui-view-chrome/sui-view-chrome.sui` + controller. |

### Justifications

**tree-node → done.** Every prop, event, slot, part, and ARIA behavior in `ship-tree-node` is present in `sui-tree-item` with only a namespace prefix rename. The SUI version adds a `prop:node` data-binding path on top. No gap exists.

**progress-ring → sui-progress/sui-progress-ring.sui.** The ring and the bar share the progressbar role and a 0–100 value concept, but their render primitives, sizing models, and prop shapes are incompatible for a single-component merge. Co-locating inside `sui-progress/` keeps the conceptual family together without polluting either component's prop surface.

**step-segments → new sui-step-segments.** The state vocabulary (`failed`, `running`) and data model (outcome array) have no counterpart in `sui-stepper`. Their ARIA roles differ. They serve different product moments. A variant merge would produce dead props in both directions.

**view-chrome → port as sui-view-chrome.** The component is documented and designed as a first/third-party symmetry mechanism. Its no-shell fallback makes it safe outside the shell context. L3 developers cannot participate in Shell chrome without a SUI equivalent.
