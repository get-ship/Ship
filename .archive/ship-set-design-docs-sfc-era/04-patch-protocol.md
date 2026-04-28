# Sail patch protocol

The wire format for SSE-driven updates. **The killer feature.** This is what lets a server-side agent update a canvas, a chart, a list, or any reactive surface in a Sail app in real time without re-emitting state — only changes.

## Why this matters

The use case driving the design: an agent owns a canvas (drawing tool, spreadsheet, code editor, diagram). The agent emits "I added a circle at (20, 30)" or "I changed cell B7 to '=A1+A2'" as a patch over SSE. The browser applies the patch in <16ms — no full re-render, no flicker, no token cost on the agent for re-emitting state.

The token economy multiplier:
- A 10,000-shape canvas update: full state re-emit ≈ 50,000+ tokens. Patch ≈ 50 tokens.
- A 1000-row table sort: full state re-emit ≈ 5,000 tokens. Patch ≈ 30 tokens.
- A real-time chat thread: each message is one append patch. ~10 tokens per turn vs hundreds for re-state.

This protocol is **not an optimization for canvas.** It's the *primary* mutation rail between daemon and client. Every component uses it. A pipeline board updating one step status, a chat thread getting a new turn, a mesh view showing one new actor — all patches.

## Wire format

Patches arrive over SSE on an EventSource the runtime opens for each `data live` declaration. Two event types:

### Event: `signal:patch`

A targeted mutation to a specific signal.

```
event: signal:patch
data: {"signal":"agents","op":"append","value":{"id":"shipwright","name":"Shipwright"}}
```

The runtime resolves `signal` to the appropriate signal handle on the subscribing component, applies the operation. If the component has unsubscribed (unmounted), the patch is ignored.

### Event: `element:patch`

A full HTML element morph (less common; for hypermedia views where partial server re-render is the right tool).

```
event: element:patch
data: {"target":"#main","html":"<main>...new server-rendered HTML...</main>"}
```

The runtime morphs `target` against the new HTML using idiomorph (vendored). Preserves focus, scroll, animation state where DOM identity is preserved.

## Patch operations

The full set. Every operation is targeted by a JSON path within the signal's value (or empty path = root).

### `set` — replace at path

```json
{ "op": "set", "path": "title", "value": "New title" }
{ "op": "set", "path": "shapes[7].color", "value": "#ff0000" }
{ "op": "set", "path": "", "value": { "completely": "new" } }
```

Replaces the value at `path` with the provided value. If `path` is empty, replaces the entire signal value.

### `merge` — shallow object merge at path

```json
{ "op": "merge", "path": "metadata", "value": { "dirty": true, "lastEdit": "2025-01-01" } }
```

Merges `value` into the object at `path`. Only changed keys re-trigger reactivity. Errors if the target isn't an object.

### `append` — array append

```json
{ "op": "append", "path": "shapes", "value": { "id": "sh-12", "kind": "circle" } }
```

Pushes `value` onto the array at `path`. Errors if the target isn't an array.

### `prepend` — array prepend

```json
{ "op": "prepend", "path": "log", "value": { "ts": 12345, "text": "..." } }
```

Inserts `value` at the start of the array at `path`.

### `insert` — array insert at index

```json
{ "op": "insert", "path": "shapes", "value": { ... }, "index": 3 }
```

Inserts `value` at `index` in the array at `path`. `index` may equal length (== append).

### `remove` — remove at path

```json
{ "op": "remove", "path": "shapes[3]" }
{ "op": "remove", "path": "metadata.legacy_field" }
```

Removes the value at `path`. For arrays, splices out the element. For objects, deletes the key.

### `move` — array reorder

```json
{ "op": "move", "path": "shapes", "from": 7, "to": 2 }
```

Moves the element at index `from` to index `to` within the array at `path`. The keyed-list reconciler in the runtime preserves DOM identity across moves (focus / animation / scroll).

### `splice` — array splice

```json
{ "op": "splice", "path": "rows", "index": 5, "count": 2, "value": [{...}, {...}] }
```

Splices the array at `path`: removes `count` elements starting at `index`, inserts `value` (an array) in their place. Mirrors `Array.prototype.splice` semantics.

### `inc` — numeric increment

```json
{ "op": "inc", "path": "counter", "by": 1 }
{ "op": "inc", "path": "shapes[7].opacity", "by": -0.1 }
```

Atomically increments the numeric value at `path`. Helpful for counters, atomic accumulation, optimistic-update scenarios where the client and server independently increment.

### `text` — string mutation

```json
{ "op": "text", "path": "doc.body", "splice": [{ "at": 100, "remove": 5, "insert": "hello" }] }
```

Targeted edit on a string. Useful for collaborative editing of long text fields (more efficient than `set` for small edits to large strings). MVP: single splice op; future: full operational-transform support.

## JSON path syntax

Paths use a subset of JSONPath:

- `field` — root-level field
- `field.subfield` — nested object access
- `field[3]` — array index
- `field[3].sub` — chained
- `` (empty) — root

No wildcards, no slicing, no recursive descent in v1. Paths are deterministic, fast to resolve.

```ts
function atPath(value: any, path: string): any {
  if (!path) return value
  // Walk: split on `.` and `[N]`
  const tokens = path.match(/[^.[\]]+/g) ?? []
  let v = value
  for (const t of tokens) {
    if (v == null) return undefined
    v = v[t]
  }
  return v
}
```

The runtime does immutable updates: each patch returns a new value at the root, with copy-on-write for objects/arrays along the path, leaving siblings untouched. Reference equality is preserved for unchanged subtrees — important for `derived` recomputation efficiency.

## Server-side emission

The Ship daemon emits patches via a standardized envelope:

```rust
pub struct SignalPatch {
    pub signal: String,                // matches client's data declaration name
    pub op: PatchOp,                   // set/merge/append/.../inc/text
    pub path: Option<String>,          // empty = root
    pub value: Option<Value>,
    pub index: Option<usize>,
    pub from: Option<usize>,
    pub to: Option<usize>,
    pub count: Option<usize>,
    pub by: Option<Number>,
    pub splice: Option<Vec<TextSplice>>,
}

pub fn emit_patch_to_subscribers(channel: &str, patch: SignalPatch) {
    // Serialize as JSON, push to all subscribers' SSE streams
    // for the matching channel.
}
```

For Ship's daemon, agents emit patches via:
```
POST /api/v1/signals/<channel>/patch
Body: SignalPatch JSON
```

The daemon validates the patch against the channel's published schema (which value paths are valid, which ops are allowed), persists to the event log (for replay + audit), broadcasts to subscribers.

## Schema validation

Patches are validated against the destination signal's TypeScript type:

- `set` at path: value must match the type at that path
- `append`/`prepend`/`insert`: value must match the array element type
- `merge`: value's keys must match the object's allowed keys + types
- `move`/`remove`/`inc`: paths must reference valid existing fields

Validation is a compile-time concern (the schema source publishes the signal's type; the patch wire format is checked against it at the daemon's emit site). Runtime-side validation is optional; trusts the daemon to send well-formed patches.

## Subscription channel

A `data live agents = ship.agents.list({ workspace })` declaration becomes an SSE subscription on a channel like:

```
GET /api/v1/signals/ship.agents.list/subscribe?workspace=feat-x
```

The channel name encodes the data path + relevant args. Ship's daemon maintains channels via the existing kernel actor system (each subscriber registers as a kernel-router actor; patches are kernel events with `target_actor_id` set to that actor).

For non-Ship backends (OpenAPI, GraphQL adapters), the schema source defines its own channel naming and SSE endpoint format.

## Optimistic updates and rollback

Authors can opt into optimistic mutation:

```ts
state count = 0

function inc() {
  // Optimistic — apply locally first
  const prev = count
  count += 1

  // Then sync to server
  fetch('/api/v1/counter/inc', { method: 'POST' })
    .then(r => r.ok ? null : count = prev)  // rollback on failure
}
```

For server-owned signals (`data live`), optimistic patches are a first-class operation:

```ts
data live shapes = ship.canvas.shapes(workspace)

function addShape(s: Shape) {
  // Optimistic: apply locally
  patch(shapes, { op: 'append', value: s, optimistic: true })
  // Send to server
  fetch('/api/v1/canvas/add', { method: 'POST', body: JSON.stringify(s) })
    .then(r => r.ok ? confirmOptimistic(shapes) : rollbackOptimistic(shapes))
}
```

The runtime tracks optimistic patches in a pending queue. When the server confirms (sending back its own patch with the agreed-upon shape), the optimistic patch is reconciled. On rollback, optimistic patches are reverted in reverse order.

Detail design TBD — optimistic-update ergonomics are a v1.5 feature.

## Versioning the protocol

Patch protocol is versioned independently of the language spec.

```
event: signal:patch
data: {"v":1, "signal":"...", "op":"..."}
```

`v` field defaults to 1. Future versions add fields or new ops; old clients gracefully ignore unknown ops (with a warning).

## Test plan

`packages/@ship-set/runtime/test/patch.test.ts`:
- Every op against simple values (root-level set/merge/append)
- Every op against deep paths (`a.b[3].c`)
- Immutability (sibling subtrees keep reference equality)
- Edge cases (out-of-bounds index, missing path, type mismatch)
- Optimistic + rollback ordering

Integration tests (Playwright):
- A canvas component receiving a stream of patches: position, color, count updates render correctly without flicker
- A list component reconciling against insert/remove/move patches: focus and scroll preserved
- A connected modal: SSE drop and reconnect resumes from current state without dropping patches

## Performance budget

End-to-end SSE patch latency target: <16ms (one frame at 60Hz).

Breakdown:
- Network arrival (local daemon → browser): ~0.5ms
- SSE event dispatch + JSON.parse: ~0.2ms
- Patch op + path resolution: ~0.1ms
- Signal update + subscriber notification: ~0.1ms per subscriber
- DOM mutation: ~0.5ms typical
- Total: ~1.5ms typical, ~3ms in load

Well within budget. The runtime can comfortably handle 60+ patches/second per component before frame drops appear.

## Why this is hard to copy

This protocol is what makes Sail unique. To replicate elsewhere:

- **Solid** has signals but no patch protocol. You'd build it on top.
- **Svelte** has stores but the wire format is bring-your-own.
- **Phoenix LiveView** has a similar protocol (Diff format) but it's coarse-grained — emits whole-component HTML diffs, not signal-targeted patches. And it's Elixir-bound.
- **Datastar** has SSE-driven HTML morphing but no signal-targeted patches. The hypermedia rail is good; the fine-grained patch surface is missing.

Sail combines fine-grained signals with a typed patch wire format. The combination is novel.

## Implementation order

1. **Patch operations on plain values** (week 1, in `@sail/runtime`):
   - Implement all ops against root and nested paths
   - Heavy unit tests for path resolution, immutability, edge cases
2. **Signal integration** (week 1):
   - `applyPatch(sig, patch)` reads, transforms, writes
   - Subscribers fire as expected
3. **SSE wiring** (week 2):
   - EventSource subscription per `data live` declaration
   - Reconnect handling
   - Buffered patches during reconnect
4. **Server emission** (week 2-3, in Ship's daemon — Layer 3):
   - `POST /api/v1/signals/<channel>/patch` route
   - Channel registry tied to data sources
   - Patch validation against signal schema
   - Broadcast via kernel router
5. **Optimistic + rollback** (week 3-4, deferred to v1.5 if pressed):
   - Pending queue
   - Confirm/rollback API for authors
   - Reconciliation on server-disagreement

Total: ~3 weeks for the patch protocol end-to-end, integrated.

## Spec compatibility

This protocol mirrors RFC 6902 (JSON Patch) for ops `set`/`merge`/`remove` but adds Sail-specific ops (`append`, `prepend`, `insert`, `move`, `splice`, `inc`, `text`) that are common in reactive UI mutations. Authors can write JSON Patch by hand and Sail will accept it (we treat unknown ops as warnings, not errors, for forward compat).

The richer op set is what saves tokens for agents. JSON Patch's `replace` for an array append requires re-emitting the whole array; Sail's `append` is a single-value patch.

## What's not in v1

- **Concurrent edits / OT.** Multiple writers to the same signal need conflict resolution. Sail's model assumes one writer (the server) per signal. Multi-writer support is a v2 concern.
- **Subscriptions to specific paths.** Today: subscribe to a signal. Future: subscribe to `path.within.signal` for finer fan-out. Useful for very large state trees.
- **Patch batching.** Multiple patches in one SSE event. Useful for atomic multi-field updates. Defer.

These don't block v1; they're future enhancements.

## Why this is the killer feature

When the framework is built around patches as the wire model:

- Agents author canvases at token costs proportional to changes, not state
- The daemon's event log naturally records the patch stream — perfect for replay, undo, audit
- Multi-user sync (if it ever lands) is one server emitting patches to N subscribers, no extra protocol
- Optimistic updates with rollback are first-class
- Real-time UIs (chat, collab editing, dashboards, live data) all use the same machinery

The "Sail" pitch becomes:

> Server-side agents and processes emit small patches over SSE. The framework applies them with no full re-render. Real-time UIs cost tokens proportional to changes, not state. This is the framework for agent-authored real-time apps.

That's a unique product positioning. No other framework has it built-in. We do.
