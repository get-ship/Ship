# sui/proposed/ — components that do NOT exist in source

These were built with `before/after` pairs during a session that **had not pulled the
real repo**. None of them exist in `madvib/Ship@job/sui-framework` (`primitives`, `charts`,
`canvas`, `chat-elements`, or `media` registries). They were moved here — not deleted —
because some may be **real gaps** worth proposing back, and the rest are useful to keep as
a record of what was hallucinated.

Per the repo owner: *"a few of these were recently scrubbed from primitives seeing as they
are mostly just 'kinds' for cards, buttons etc."* — i.e. several are not missing components,
they're variants/props of an existing primitive.

> ⚠️ The `.before.sui` here are guesses too. Treat everything in this dir as a sketch.

## Triage (recommended — confirm with owner)

| Proposed | Verdict | Rationale |
|---|---|---|
| `divider` | **DROP — rename** | Real component is **`separator`**. Pure duplicate. |
| `segmented` | **DROP — rename** | Real component is **`toggle-group`**. |
| `media-generic` | **DROP — wrong shape** | Repo splits into `sui-media-image` / `-csv` / `-av`. |
| `component-viewer` | **DROP — wrong tier** | Lives in the `@ship/viewer` router pack, not a primitive. |
| `list-tile` | **likely "kind of card"** | A row/list layout — probably a card variant + `stack`/`cluster`. |
| `metachip` | **likely "kind of chip/badge"** | Looks like a `chip` tone variant. |
| `stat` | **likely "kind of card"** | A stat card — card + type tokens. |
| `app-icon` | **likely "kind of icon/avatar"** | Tinted icon tile — `icon`/`avatar` variant. |
| `kbd` | **likely "kind of code/badge"** | A key cap — mostly CSS on `code`/`badge`. |
| `mark` | **likely markdown concern** | `<mark>` highlight — belongs to `markdown` prose styles. |
| `loading` | **redundant** | Overlaps `spinner` + `skeleton`. |
| `step-segments` | **redundant** | Overlaps `progress` (proposed) + `pagination`. |
| `toolbar` | **likely composition** | A `cluster`/`stack` arrangement, not its own tag. |
| `annotation-marker`, `annotation-list` | **belong to canvas/board** | Domain of `sui-board`, not standalone primitives. |
| **`progress`** | **POSSIBLE REAL GAP** | Registry has `spinner`/`skeleton` but no determinate progress bar. Worth pitching. |
| **`breadcrumb`** | **POSSIBLE REAL GAP** | No breadcrumb in nav primitives. Confirm it's not intentionally omitted. |
| **`stepper`** | **POSSIBLE REAL GAP** | No multi-step indicator; confirm `pagination`/`tabs` don't cover the use case. |
| `copy-button`, `timestamp` | **possible chat/util gap** | Common in chat UIs — check `chat-elements` need; may be utils not components. |

## If pitching a gap
Build it as a real `sui-<name>/sui-<name>.sui` against the actual token + API conventions in
`../../_repo-src/` (study `sui-button`, `sui-badge`, `sui-card` first), not from these sketches.
