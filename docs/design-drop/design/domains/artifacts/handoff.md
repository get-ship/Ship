# Handoff — `artifacts`

> **Status:** `in-design`
> **Upstream service:** `ArtifactsService` · **Contract:** `proto/ship/ops/v1/artifacts.proto`
> **Spec:** [`./spec.md`](./spec.md) · **Fixtures:** [`./fixtures/artifacts.demo.json`](./fixtures/artifacts.demo.json)

Content-addressed blob store + metadata projection. The UI is a **file browser
with previews** — list blobs, filter, preview by mime, promote scratch→durable,
rename, delete. Reactive: workflow-produced artifacts appear live.

---

## 1. RPC surface

| Method | Shape | Notes |
|---|---|---|
| `List` | `{ filter? } → { data: ArtifactRecord[], count }` | page of artifacts |
| `Watch` | stream → `snapshot` frame then `append` / `remove` patches | replaced `ListLive` |
| `Patch` | `{ id, name?, collection_id? } → ArtifactRecord` | rename + reassign collection |
| `Promote` | `{ id } → { id, promoted, retained }` | scratch → durable |
| `Delete` | `{ id } → { id, deleted }` | confirm first |

`ArtifactRecord`: `id · name · mime · size · owner_actor · collection_id · scope · viewer_url · viewer_label · created_at`.

## 2. Requirements

- [x] Browse / filter / preview, **live** (Watch patches applied in place).
- [x] Preview by mime — image inline · markdown rendered · JSON tree · text · binary→download. **mime→viewer mapping is the core logic** (legacy `viewers.ts`).
- [x] Actions: **Promote** (scratch→durable), **Delete** (confirm), **Rename** (Patch).
- [x] Upload (drop / select → create).
- [x] All states: loading · empty · empty-filtered · error · ok · **live**.
- [x] "This just changed" affordance on live append/remove.

Out of scope: blob-bytes substrate port (no UI impact); upload-wire modernization (confirm at build).

## 3. Components — `sui-artifact-*`

| Component | Status | Calls | Use |
|---|---|---|---|
| `sui-artifact-list` | new | `List` + `Watch` | the browser; applies Watch patches in place |
| `sui-artifact-preview` | new | bytes via `viewer_url` | mime→viewer render (the core) |
| `sui-artifact-filters` | new | none (filters result) | mime / owner / age / collection |
| `sui-artifact-actions` | new | `Promote` `Delete` `Patch` | promote · delete · rename |
| `sui-artifact-upload` | new | create path | drop/select |
| `sui-watch-badge` | new (shared w/ collections) | Watch status | live / reconnecting indicator |

## 4. Fixtures

[`./fixtures/artifacts.demo.json`](./fixtures/artifacts.demo.json) — 5 artifacts
(image/markdown/json/binary/text), Patch/Promote/Delete responses, Watch
snapshot→append→remove, empty + error states.

## 5. Related view

- Route `/artifacts` → `reference/views/artifacts/page.sui` (legacy wire — flow + tone reference).
- Host scaffold: `shell`.
- **Primary deliverable:** [`./components.html`](./components.html) — the component catalog
  (each `sui-artifact-*` in isolation, all states, API, per-component status).
- **Composition:** [`./artifacts.html`](./artifacts.html) — the components assembled into the view.

---

## Open questions for eng (surfaced in design)

1. **`List` rows need a durable/scratch signal** so the UI can show the Promote
   affordance without a second call. Add `retained: bool` (and `scope`) to
   `ArtifactRecord` on `List`/`Watch`? The mock assumes this.
2. Rename via `Patch` — does it re-key `viewer_url`? Mock keeps the URL stable.

## Deliverable & approval

Ready to implement when the mock covers every state against the fixtures and the
`sui-artifact-*` decomposition is legible. Registered in the review pane as
**Artifacts view** — approve there to release.
