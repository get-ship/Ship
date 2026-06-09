# Layer L1 — Audit Shape

L1 is the platform layer: `ship-platform` modules, `ship-ops-rpc`, `ship-analytics`,
`ship-gateway-client`, and similar platform primitives. L1 crates
consume L0 through `ship-substrate`'s re-exports and expose higher-level domain
services (agents, sessions, rules, skills, pipelines, etc.). Audits at this
layer are events-and-wire-centric — the core question is whether each capability
reaches the wire and whether events have both producers and consumers.

## What L1 audits cover

- **Events emitted** — table of all event types this module produces, with emit
  citations, helper-fn resolution, wire exposure, consumer-side liveness, and
  classification (`{scope, retention}`).
- **Wire integration** — which public capabilities are reachable via proto op or
  SDK skill, and whether there is drift between the proto shape and the function
  signature.
- **Trait surface vs bypass** — does the proto handler route through the service
  trait, or does it reach `state::*` directly? Bypasses are acceptable for reads
  when documented; undocumented bypasses are `[trait-bypass-read]` observations.
- **Dual-side liveness** — every `pub fn` must have at least one outside caller;
  every event-type must have both a producer count and a consumer count ≥ 1.

## Section shape (10 sections)

### 1. File inventory
Table: `File | LOC | Role | Cap status`. Note test percentage. Flag files
exceeding the 300-line soft cap or 400-line hard cap.

### 2. Module headline (header block)
Short paragraph: module purpose, total pub fn count, event count, wire endpoint
count. Cite file:line for the headline claim.

### 3. Public API table
| Fn / type | file:line | Signature (abridged) | R/W | Caller-count (in-module / outside-prod) |

Verdict per row (append as a note, not a column): `contract` / `internal-leaked`
/ `dead` / `test-only`. Flag `[over-applied-pub]` for pub items with 0
outside-production callers.

### 4. Events emitted
| event-type | file:line of emit | Helper-resolved (yes/no) | Wire-exposed (proto field) | Consumer-side liveness | Classification {scope, retention} |

If the module emits 0 events: "Zero. Verified: `rg '.emit(|EventEnvelope::new'`
in `<path>` returns empty." This is a first-class verdict. Note if it ratifies a
cross-module pattern (`[mr-02 silent-write]`, `[event-producer-primary]`).

### 5. Wire integration
| Public capability | Reachable via proto op (cite) | SDK skill (cite) | Sync subset (yes/no) | Drift (proto shape ≠ fn signature) |

Note if the module has dual wire surfaces (proto + registry) — tag
`[dual-wire-surface]` per sighting.

### 6. Liveness — dual-side
For every `pub fn`: caller-count outside the module (production callers only,
not tests) AND outside the test set. For every event-type: producer count AND
consumer count. Summarize liveness verdicts. Flag:
- `[parallel-entry-point]` — if a majority of pub fns are only reachable from
  tests while production uses a different code path.
- `[dead-emit: <event-name>]` — if an event has a producer but 0 consumers.

### 7. Trait surface vs bypass
Which read operations route through the service trait vs call `state::*`
directly? Classify: `trait-first` (all ops go through trait), `half-bypass`
(reads bypass, writes go through trait), `full-bypass` (nothing goes through
trait). Tag bypass instances as `[trait-bypass-read: <location>]`.

### 8. Meta-observations (for `_observations.md`)
Bullet per cross-cutting observation with tag, sighting count, and whether it
promotes from candidate → confirmed → promoted. Ratification notes (e.g.,
"5th/final mr-02 sighting — pattern fully locked").

### 9. Open calls for sign-off
Genuine architectural questions only. Each must state: the question, its
citation, and 2–3 resolution options. No "improve X" items — only real binary
choices.

### 10. Security findings (if any)
Format: `| ID | file:line | Finding | Severity | Tracking |`. Use severity labels
from the collection schema: `info` / `low` / `medium` / `high` / `critical`.
C1-WORKSPACE and C2-ELEVATED blast-radius notes belong here.

## L1-specific checks

### mr-02 wrapper pattern
Service modules (rules, mcp, permissions, skills, agents) follow the mr-02
shape: `trait <Domain>Service` + `<Domain>ServiceImpl` over `FilesService`,
`arc` ctor for production, `with_roots` ctor for tests, `state` layer, `wire_types`,
`fixtures`. Check each component:
- Trait defined? Impl over FilesService? `arc` ctor present?
- `with_roots` test ctor isolated from production?
- `state::find` pub visibility: should be `pub(super)`, not `pub`.
- `fixtures::examples()` — does an xtask consumer exist?

Tag: `[mr-02-wrapper]` per sighting. `[over-applied-pub]` for `state::find` /
`write_body` / `delete_body` visible outside the module.

### Dual wire surface
All five mr-02 wrappers expose both proto `<Domain>Service` AND registry ops.
Verify:
- Proto endpoints reach the trait.
- Registry ops route through `state::*` directly (bypass is expected but must
  be noted as `[trait-bypass-read]`).
- Writes go through the command dispatcher and are gated.
Tag: `[dual-wire-surface]` when two parallel wire surfaces confirmed.

### Helper-fn resolution
If the "Events emitted" section has dynamic emit sites (`emit_artifact_event`,
`emit_skill_event`, etc.), walk the helper's call sites explicitly. Each helper
covers a finite number of call sites. Do not mark them "dynamic" and discard.

### Cross-module duplicate detection
Common duplicate shapes in L1:
- `[duplicate-utility: strip_jsonc_comments]` — multiple crates define their own
- `[duplicate-utility: gen_nanoid]` — compiler + substrate both define
- `[duplicate-internal: content_hash_hex]` — agents module: 3 copies
- `[duplicate-internal: validate_id]` — mr-02 modules: each defines its own
- `[duplicate-internal: parse_scope]` — shipd RPC + command files each define
  private `parse_scope` with identical match arms

Check for these by name when auditing modules in those neighborhoods.
