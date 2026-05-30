# Layer L0 — Audit Shape

L0 is the substrate: `ship-kernel`, `ship-substrate`, `ship-supervisor`,
`ship-types`. These crates define invariants that every layer above must
respect. Audits at this layer are invariant-centric and soundness-heavy.
Correctness is the primary signal; wire exposure is usually absent by design.

## What L0 audits cover

- **Invariants enforced** — explicit guarantees the crate makes to callers.
  Each invariant gets a name (I1, I2, …), a statement, an enforcement citation
  (`file:line`), and a test citation. If there is no test, say so.
- **Write barriers** — can the ordering of persist and deliver be subverted?
  Can projections corrupt each other? Are event-type registrations idempotent?
- **API-stability surface** — which `pub` items are the real contract vs
  `internal-leaked` (pub because a peer crate uses `inner.*` access)?
- **Soundness** — `unsafe` blocks, `panic!`/`unwrap`/`expect` in production
  paths, blocking ops in async fns, cancellation safety, `Send + Sync` bounds.
- **Layer-direction** — L0 must not import from L1 or above. Violation is
  architectural; flag as `[layer-direction: L0→L1]` severity `critical`.

## Section shape (10 sections)

### 1. File inventory
Table: `File | LOC | Role`. Note cap status per file. Call out test-bulk that
inflates LOC without violating production caps.

### 2. Crate role + invariants enforced
One subsection per invariant: name, statement, enforcement citation,
test citation. If the crate provides zero invariants, say so explicitly.

### 3. API-stability surface (the L1 contract)
How caller counts were determined: qualified-path grep method, crates checked.
Table: `Symbol | Export path | Defined at | External callers (cited) | Verdict`.

Verdict values: `contract` (actively used by the layer above), `internal-leaked`
(pub for `inner.*` access only, no external callers), `dead` (pub, no callers
anywhere).

Breakdown summary: `contract N | internal-leaked N | dead N`.

### 4. Write barriers + invariant-protection
Can L1 reorder events? Can a projection corrupt another? Are event-type
registrations idempotent? Schema migration / versioning correctness. Write
barrier on mutable operations.

### 5. Soundness / safety surface
`unsafe` blocks (count + locations). `panic!`/`unwrap`/`expect` in production
paths. Blocking ops in async fns. Cancellation safety. `Send + Sync` analysis.
Silent drops (mailbox full, channel send failure).

### 6. Layer-direction discipline
Imports from peer crates (list each). Confirm no L1+ imports. Cite the
`Cargo.toml` dependency list. Self-enforcing layer tests (e.g. `!contains("sqlx")`)
are noteworthy — tag `[candidate: layer-purity-enforcement]` first sighting.

### 7. Event types defined here
If none: "this crate defines 0 event type constants — dual-side check not
applicable." If some: table with `Event type | Defined at | Producer sites
(cited) | Consumer sites (cited) | Liveness verdict`.

### 8. Cross-cutting observations
Bullet per observation tagged with the pattern tag. Link to `_observations.md`.
First-sighting tags use `[candidate: ...]`; established tags use verbatim
vocabulary from the collection schema.

### 9. Open calls for sign-off
Genuine architectural questions surfaced by the pass. Each must be a real
question, not punted ambiguity. Format: numbered list, question + citation +
resolution options.

### 10. Security findings
L0 often has none (no auth, no file access, no network). State this explicitly:
"No security findings. This crate is [pure in-memory router / pure build-time
utility / …]." If there are findings, use the SF-N format with severity.

## L0-specific checks

- **Self-test on dependency graph** — does the crate assert its own layer purity
  in tests? (`include_str!("../Cargo.toml")` + assert `!contains("sqlx")`).
  Tag first sighting as `[candidate: layer-purity-enforcement]`.
- **Silent-drop sites** — `let _ = tx.try_send(...)` in routing/delivery code.
  Event is durable (persist ran first) but the in-memory actor misses delivery.
  Tag `[candidate: silent-mailbox-drop]`.
- **`internal-leaked` methods** — methods callable via `inner.*` field access
  that are nominally `pub` but have zero external callers. These could be
  `pub(crate)`. Tag `[over-applied-pub]`.

## Empty-section protocol

If a section has nothing to report, state it explicitly as a first-class
verdict. Examples:
- "This crate emits 0 events. Dual-side check: not applicable."
- "Zero `unsafe` blocks."
- "Zero production `unwrap()` / `expect()`."

Absence is the architecture claim being verified. Do not fabricate placeholders.
