<!--
  HANDOFF TEMPLATE — copy this whole folder to design/domains/<service>/ when a
  platform service completes upstream. Fill every section. This file is the
  contract: an agent should be able to read ONLY this file and know the entire
  job. Keep it tight — link to fixtures and mocks, don't inline them.
-->

# Handoff — `<service>`

> **Status:** `pending` · `in-design` · `in-review` · `approved`
> **Upstream service:** `crates/.../  <service>`  ·  **RPC pulled:** `<sha>` / `<date>`
> **Owner (eng):** `@<name>`  ·  **Designer:** `@<name>`

One sentence: what this service is and what the user does with it.

---

## 1. RPC surface (the input)

The methods this design must speak to. Names + shapes only — no implementation.

| Method | Request → Response (shape) | Notes |
|---|---|---|
| `<service>.list` | `{ filter? } → Item[]` | |
| `<service>.get` | `{ id } → Item` | |
| `<service>.create` | `{ … } → Item` | optimistic? |
| `…` | | |

Streamed events (if any): `<service>.item_created`, `…`

## 2. Requirements (what the design must do)

- [ ] …interaction / UX requirement…
- [ ] …state the user must be able to reach…
- [ ] …constraint (latency, empty state, error, permission gate)…

Out of scope for this handoff: …

## 3. Components requested

What this view needs from the packs. Mark each: **exists** (use as-is),
**extend** (pack has it, needs a variant), or **new** (design in `pack/`).

| Component | Pack | Status | Use |
|---|---|---|---|
| `sui-list` | primitives | exists | row container |
| `sui-<x>` | `<service>` (domain) | new | … |

## 4. Fixtures

Data shaped like daemon responses lives in [`./fixtures/`](./fixtures/).
Mirror upstream `fixtures/data/*.json` where it exists — **no invented fields**.

- `fixtures/<service>.list.json` — …
- `fixtures/<service>.get.json` — …

## 5. Related view

- Route: `/<service>` → `views/<service>/page.sui` (if one exists upstream)
- Host scaffold: `shell` · `spaces`
- Mock(s) in this folder: `list.html`, `detail.html`, …

---

## Deliverable & approval

Design is **ready to implement** when:

1. Mock(s) here compile against the fixtures with real `<sui-*>` tags.
2. Any `new` / `extend` components are designed in [`./pack/`](./pack/).
3. The deliverable mock is registered in the **review pane** and marked
   **approved** (the approval gate — this status field mirrors it).

Engineering owns the wire-up to live data and tests. We hand off UX, the
component set, and the visual definition — not the running implementation.
