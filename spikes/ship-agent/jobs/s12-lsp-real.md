# Job S12: real tsgo + SUI LSP backends (DESIGN-FIRST, no impl)

## Context

Work in `spikes/ship-agent/` (own Cargo workspace; `cd spikes/ship-agent` to
build/test). Read `README.md` — it is the SSOT for architecture, invariants,
and the v1 arc. This is the Ship Agents Framework runtime spike.

S12 is README **PHASE 3** ("differentiators") and sits behind the slice-4
**dual-LSP seam** (`src/lsp.rs`):

- `LspBackend` trait — `diagnostics(path, src) -> Vec<Diagnostic>` and
  `hover(path, src, line, col) -> Option<String>` (spikes/ship-agent/src/lsp.rs:55-63).
- `Backend` enum (Ts/Sui) routing by file extension in `backend_for` —
  `.ts`/`.tsx` → TS, `.sui` → SUI, else permissive TS default
  (spikes/ship-agent/src/lsp.rs:143-159).
- `TsgoStub` / `SuiLspStub` impls of the trait, returning empty / `None`
  (spikes/ship-agent/src/lsp.rs:111-139). Marked `SEAM: … returns clean until
  … is wired`.
- `verify_edit_with(backend, path, src) -> Result<(), SpineError>` filters to
  `Severity::Error` and packs the messages into `SpineError::Unverified`
  (spikes/ship-agent/src/lsp.rs:171-187).
- `verify_edit(path, src)` is the public gate, used by the spine to refuse a
  bad code edit before it propagates to the model / world
  (spikes/ship-agent/src/lsp.rs:164-166).

S8's `src/registry.rs` already declares the C0 read-only LSP ops
(`lsp.diagnostics`, `lsp.hover`) (spikes/ship-agent/src/registry.rs:84-91) —
baseline-allowed because they are pure read-only analysis of an in-memory
buffer; they cannot modify state.

The README "Seams" row for LSP reads: `TsgoStub / SuiLspStub | tsgo +
crates/sui/lsp (+ warmup/writethrough)` (README.md:85). The parenthetical
names the two concerns this design must answer: **warmup** (how the first
useful response is fast) and **writethrough** (how the in-memory edit buffer
the verify gate holds is held in sync with what tsgo / SUI LSP are looking
at). The slice-4 seam deliberately does not answer these — it is content with
receiving a `path` + `src` and producing `Vec<Diagnostic>` for *that exact
pair*, so the design is free to choose the *internal* state shape that backs
the trait.

### What "real" means

The agent runtime consumes language services two ways:

1. **Verify gate.** After every code edit, run diagnostics. Any
   `Severity::Error` diagnostic is `SpineError::Unverified` and the edit is
   *refused, not accepted* (spines/ship-agent/src/spine.rs:37-46). This is the
   safety-critical path; the design must keep it correct, fast, and unchanged
   in shape.
2. **Authoring metadata.** Hover / definition / completion are surfaced as
   `lsp.hover` (C0, registry) so the model can call them when reading
   unfamiliar code (controllers, `.sui` templates). Same trait, same engine —
   slice-4 already calls this out: "the verify gate and the metadata tools
   share one engine" (spikes/ship-agent/src/lsp.rs:52-54).

The two real backends:

- **SUI LSP** — `crates/sui/lsp/src/main.rs`. A thin stdio LSP server; the
  intelligence is `sui_compiler::analyze / hover_at / completions_at`
  (crates/sui/compiler/src/analysis.rs:60-108) exposed as a `Connection` loop
  over JSON-RPC (crates/sui/lsp/src/main.rs:27-63). Diagnostics are pushed
  via `textDocument/publishDiagnostics` after every didOpen / didChange /
  didSave (crates/sui/lsp/src/main.rs:67-114). Capabilities are FULL sync +
  hover + completion + definition (crates/sui/lsp/src/main.rs:36-46).
- **tsgo** — Microsoft's native-port TypeScript compiler binary
  (`@typescript/native-preview`, invoked as `tsgo`). The spike doesn't pin
  the binary; it speaks the **LSP** protocol over stdio. The README names
  tsgo because slice-2.5's `.d.ts` emitter is the typed surface it consumes
  (spikes/ship-agent/jobs/slice-2.5-dts.md:14).

### Invariants the design must hold (READ FIRST)

From `README.md`, non-negotiable:

1. **One door** — every effect enters `gateway.call`. No side channel.
2. **The gate is the only enforcement.** Scoped SDK + deny-by-default loader
   are *soft* (ergonomics/legibility); a raw `op_ship_call` to an ungranted
   op is still denied at the door.
3. **No self-elevation** — `operator=false` by construction ⇒ C3 unreachable.
4. **Deny-by-default, twice** — imports (only `ship`) and capabilities (only
   granted services).
5. **Nothing accepted blindly** — spine verify on ops; **LSP diagnostics on
   edits**. The seam must not lose this property.
6. **Two tool categories, period** — gated ops + JS execution.
7. **Denial is data** — consistent across the deno_core op and the rig tool.

This spec holds #1, #5, #6, #7 by construction (no new effect categories, no
bypass of the gate) and proposes nothing that touches `gate.rs::decide`, the
one-door dispatch, or the spine verify control flow.

The job-spec is explicit: **DESIGN ONLY.** No new runtime modules, no edits
to `lsp.rs` / `gate.rs` / `spine.rs` / `runtime.rs`. The single deliverable
is this spec.

---

## 1 · Backend process model

How does the agent runtime talk to tsgo and to `crates/sui/lsp`? Three
shapes are on the table; we choose one for each backend and explain why.

### Options considered

| # | Shape | What it is | Pros | Cons |
|---|-------|------------|------|------|
| A | **In-process crate call** | The spike depends on `sui-compiler` (or a thin façade) and calls `sui_compiler::analyze` / `hover_at` directly. | No process boundary; no spawn cost; pure-Rust symbols, no JSON; uses the same `Analysis` shape the CLI / tests already consume. | The spike lives in its **own** Cargo workspace (spikes/ship-agent/Cargo.toml:1-4 `[workspace]`) and currently has no `sui-compiler` dep. tsgo has no such surface — it is a Go binary, not a crate. Different shape per language. |
| B | **LSP-over-stdio child** | Spawn a child process (`sui-lsp` binary, `tsgo` binary) per language, speak JSON-RPC over the child's stdio, route through `lsp-server` / `lsp-types` on the spike side. | Mirrors what real editors do; same wire for both backends; one transport code path; works for tsgo (no Rust) and for SUI (the binary already exists in `crates/sui/lsp`). | Process-spawn cost per startup; lifecycle (warmup, idle, restart-on-crash); JSON-RPC framing is non-trivial; the verify gate has to *wait* for the language server to return — blocking the rig loop unless we wrap it in `tokio::task::spawn_blocking` or a small async client. |
| C | **Daemon service** | A long-lived shipd-side service hosts both backends; the agent connects to it over a Unix socket / handshake channel. | Single supervisor, central warmup, can multiplex across many agents; aligns with the S5 auth-handshake seam and the S6 spec loader. | Heaviest weight — a new daemon endpoint, a new client, an explicit failure mode to design; not justified by a single spike; orthogonal to the seam that slice-4 already provides. **S17 ("e2e harness") and S16 ("graduate to crates/ + apps/") are where this lands**, not here. |

### Recommended: **B for both** (with the C target named explicitly)

The SUI LSP binary (`crates/sui/lsp/src/main.rs:27-34`) is *already* an
LSP-over-stdio server. Its intelligence (`sui_compiler::analyze`) is a pure
library; the binary is a one-binary transport we can spawn the same way
VS Code does. tsgo is *only* an LSP-over-stdio binary (Microsoft ships no
Rust crate to embed it). The two backends converge on the same transport
and the same client shape — option B is the only shape that fits both.

**Why not A for SUI?** It is the highest-performance option, but it forces
the spike workspace to depend on `sui-compiler` (currently crate-internal
to the main workspace). That dependency would either (1) drag the full
compiler surface into the spike, or (2) require a thin `sui-compiler-api`
crate the spike can depend on without pulling parse / lower / emit. Both
are real, and both are S12 **downstream** work; until then, the spike
spawns the binary it already references. The README naming `crates/sui/lsp`
(README.md:85) is unambiguous about the binary boundary.

**Why not C now?** A shared daemon service is the right *future* shape —
one process, one warmup pool, one supervisor, one auth handshake. S5 (auth
handshake seam) + S6 (spec loader) + this job all share that future. But
S12's job is the *first* time a real backend touches the seam; introducing
a daemon at the same time entangles "wire up the backends" with "wire up
the daemon." The S16 / S17 graduation arc is the natural seam for that.
The recommended B-shape leaves a clean handoff: the **client side** of B
(send LSP request, await response) is the unit a future daemon reuses —
the wire format doesn't change, only the transport.

**Cite of the real entry surface.** `crates/sui/lsp/src/main.rs:27-34`
shows the stdio `Connection::stdio()` loop, `Connection::initialize` with
`server_capabilities`, and the blocking `main_loop(&conn)` over
`conn.receiver`. `server_capabilities` (crates/sui/lsp/src/main.rs:36-46)
declares: FULL sync, hover, completion, definition. On a notification, the
server calls `publish_diagnostics` (crates/sui/lsp/src/main.rs:104-114) which
runs `sui_compiler::analyze(text, path)` and ships
`textDocument/publishDiagnostics` back over `conn.sender`. This is exactly
the surface a stdio client speaks: a `Connection`-shaped channel that
implements the LSP wire for free (via `lsp-server` / `lsp-types`, both of
which `sui-lsp` already depends on — crates/sui/lsp/Cargo.toml:13-14). The
spike side will mirror the same dependency set: `lsp-server = "0.7"`,
`lsp-types = "0.95"`.

### Process lifecycle (per-language, per-agent)

For each language the agent needs, one **long-lived** child process,
spawned lazily on the first request to that language, kept warm for the
agent's lifetime:

1. **Spawn** — `tokio::process::Command` for the binary, pipe stdio, run
   `lsp_server::Connection::stdio` in the parent to get the `(sender,
   receiver)` pair. Send `initialize` with the client's `capabilities` and
   `rootUri` set to the working directory. Wait for the `initialize`
   response (a request/response, with timeout — the binary's loop is
   synchronous, so this is a hard block until it returns).
2. **`initialized` notification** — fire `notifications/initialized` and
   start the read-pump task.
3. **Warmup** — first request to a backend pays the process-spawn cost; we
   name this in the Seams table because it is the user-visible "first
   verify is slow" moment. After the first round-trip the channel is hot
   and subsequent calls are sub-millisecond on the wire. Mitigation for the
   first-call cost: pre-spawn at agent startup behind a `tokio::spawn`, so
   the first verify lands on a ready child. The exact spawn timing is an
   impl detail; the design pins that warmup is *per-language* (a `.sui`
   edit never pays the tsgo warmup, and vice versa) and that the warmup
   cost does not block the rig loop (the read-pump is on its own task).
4. **Idle** — keep the child alive between agent turns; do not close
   on idle. tsgo and SUI LSP both rely on cross-edit state (the
   `Docs` map in sui-lsp, the tsserver-style project model in tsgo) —
   tearing it down per-turn negates warmup.
5. **Crash / hang** — supervisor: a `tokio::spawn` task owns the read-pump
   and the child's stdout/stderr. If the child exits or the read-pump
   errors, mark the channel `Stale`, the next request spawns a fresh
   process, and we record one `LspError::Backend("restarted")` event in the
   event log. The verify gate does NOT panic — the trait returns
   `Vec<Diagnostic>` and can degrade to "no diagnostics" (the permissive
   default), with a `Severity::Warning`-class surface marker so the agent
   sees the degradation. The `Severity::Error`-or-not decision is unchanged
   in shape: warnings are warnings, errors are errors, the gate still
   refuses on errors. **The seam contract is preserved**: a backend that
   cannot answer does not silently pass.
6. **Writethrough** — the verify gate receives a `src` string (the
   post-edit buffer). To get tsgo / SUI LSP to *see* that buffer, the
   client side mirrors it on the server: a `textDocument/didOpen` on first
   sight of a `path` (with a synthetic `file://` URI; the file need not
   exist on disk for the SUI LSP — its `analyze` is text-only,
   crates/sui/compiler/src/analysis.rs:60-83), then `textDocument/didChange`
   (FULL sync, both backends declare it: sui-lsp line 40, tsgo likewise)
   on every subsequent verify with the new `src`. The SUI LSP's
   `Docs` map (crates/sui/lsp/src/main.rs:49) holds the in-memory text by
   URI; tsgo's protocol-level document store does the same. **Writethrough
   is per-`path`**: the client maintains a small `HashMap<PathBuf, String>`
   of last-written buffers so a re-verify after a no-op edit doesn't
   re-send the full text. (This is the `(+ writethrough)` parenthetical in
   the Seams row.) The hash is cheap; the dedupe avoids a network of
   needless full-doc resends.
7. **Shutdown** — on agent drop, send `shutdown` / `exit` per LSP. The
   child process exits; the read-pump task ends.

The two backends share the *shape* of the lifecycle; only the binary path
and the client capabilities differ. **One trait, two impls, one supervisor
per language.**

---

## 2 · The seam — `LspBackend` stays the trait

Slice-4 deliberately wrote the trait against the **call-site** shape, not
the **backend** shape: `diagnostics(path, src)` and `hover(path, src, line,
col)`. The S12 real impls wrap the stdio clients and present the same
shape, so the verify gate, the trait identity tests, the routing, and the
C0 ops all keep working unchanged.

### New impls (sketch only, NO impl in this spec)

```rust
// lives in a new src/lsp_backends.rs (or stays in src/lsp.rs — final home
// is an impl decision; the seam contract is the same either way)

pub struct TsgoBackend {
    client: std::sync::Mutex<lsp_client::Client>, // spawn-on-stale semantics
}

pub struct SuiLspBackend {
    client: std::sync::Mutex<lsp_client::Client>,
    // paired controller resolution happens INSIDE sui-lsp, not here —
    // see crates/sui/compiler/src/analysis.rs:72 (read_paired_controller
    // resolves from source_path). The client just ships the path.
}

impl LspBackend for TsgoBackend { /* see below */ }
impl LspBackend for SuiLspBackend { /* see below */ }
```

The two impls are almost identical:

```rust
impl LspBackend for TsgoBackend {
    fn diagnostics(&self, path: &str, src: &str) -> Vec<Diagnostic> {
        // 1. ensure client is initialized (lazy spawn, see lifecycle §1)
        // 2. writethrough: didOpen on first sight, didChange on subsequent,
        //    deduped against a `HashMap<PathBuf, String>` of last-seen src
        // 3. send `textDocument/documentSymbol`? — NO, the SUI LSP pushes
        //    diagnostics; tsgo does too. The "diagnostics on demand" call
        //    doesn't exist as a single LSP request. Options:
        //    a) wait for the push notification, with a small per-call timeout
        //    b) trigger a `textDocument/codeAction` / `textDocument/format`
        //       no-op to force a re-analyze — fragile
        //    c) (chosen) for the verify gate, call `textDocument/didChange`
        //       synchronously and AWAIT the matching `publishDiagnostics`
        //       notification on the client's receiver, with a short timeout
        //       (~50ms). On timeout, return [] and surface a Warning.
        // 4. map the LSP `Diagnostic` (lsp-types) to our `Diagnostic`:
        //    - severity: lsp DiagnosticSeverity::ERROR -> Severity::Error,
        //                _ => Severity::Warning
        //    - line: 0-based protocol -> 1-based (the seam is 1-based,
        //      matching the SUI compiler + the LSP convention used in
        //      slice-4 tests)
        //    - message: pass through
    }
    fn hover(&self, path: &str, src: &str, line: u32, col: u32)
        -> Option<String> { /* same writethrough + `textDocument/hover` */ }
}
```

`SuiLspBackend::diagnostics` is the same shape, with one extra awareness:
the SUI LSP pairs the `.sui` with a `.controller.ts` of the same basename
(crates/sui/compiler/src/analysis.rs:72 `read_paired_controller` reads it
from the same directory). For the verify gate, we ship the `path` through
as the `textDocument` URI; the server does the pairing internally. The
client doesn't need to know about controllers — that is a server-side
concern.

The `Backend` enum (spikes/ship-agent/src/lsp.rs:75-79) grows two variants
without changing its discriminant:

```rust
pub enum Backend {
    Ts(TsgoBackend),    // was: Ts(TsgoStub)
    Sui(SuiLspBackend), // was: Sui(SuiLspStub)
}
```

`Backend::kind()` (spikes/ship-agent/src/lsp.rs:81-90) returns the same
`BackendKind` values, so the existing test
`backend_for_picks_ts_for_ts_path` (and friends) keep passing
**unchanged** — that is the point of the trait.

### `BackendKind` / routing

`backend_for` (spikes/ship-agent/src/lsp.rs:143-159) is **unchanged**:
`.ts`/`.tsx` → `Backend::Ts(...)`, `.sui` → `Backend::Sui(...)`, else
permissive TS default. The whole point of the seam is that routing is
extension-based, deterministic, total — S12 doesn't move that.

A future refinement (NOT in S12): a known-extension registry so the
permissive default is no longer the right answer. Slice-4 already names
this (spikes/ship-agent/src/lsp.rs:154-156). It is **not** load-bearing for
verify; it is an ergonomics question, and changing it now risks scope
creep on the first real-backend slice.

### Warmup + writethrough

Both move into the **new** `TsgoBackend` / `SuiLspBackend` impls, *not*
into the trait. The trait stays a synchronous, in-process shape — a
real-world call-site may be a `tokio::task::spawn_blocking` wrapper, or
a future async trait, but neither needs to be in scope for S12 because
the verify gate and the C0 op handler can call the trait on the current
thread (the LSP client waits internally with a timeout). This is
deliberate: the seam is small, the async story stays at the call-site
(where it already lives — `InProcessGateway::call` is async,
spikes/ship-agent/src/gateway.rs:47-67), and the trait signature doesn't
have to ripple through every consumer.

Writethrough is a **per-backend** concern, lives in the backend struct,
and the trait signature does not change. The seam test suite
(`backend_for_picks_*`, `verify_edit_*` in spikes/ship-agent/src/lsp.rs)
asserts the trait contract, not the writethrough internals.

### Compile / build cost

The real impls will add `lsp-server = "0.7"`, `lsp-types = "0.95"`, and
`tokio = { version = "1", features = ["process", "io-util", "sync",
"rt-multi-thread", "macros"] }` (the `process` feature is the new one) to
`spikes/ship-agent/Cargo.toml`. The current dep list (lines 13-25) already
has `tokio` and `async-trait`; the additions are LSP-crate weights. This
is the "build heavier than the standalone spike" the job-spec warns about
— acceptable for the real-backend slice.

---

## 3 · Verify-gate fidelity — NO change to the gate or spine core

The verify gate is the load-bearing safety property. The design **must
not** weaken it; the spec is explicit (job-spec, Constraints).

### What stays unchanged

- `verify_edit(path, src) -> Result<(), SpineError>` — signature
  (spikes/ship-agent/src/lsp.rs:164-166), call-sites, and error semantics.
- The `Severity::Error` filter (spikes/ship-agent/src/lsp.rs:179) — only
  `Error` is fatal; warnings are surfaced but do not refuse the edit.
- The `SpineError::Unverified` shape (spikes/ship-agent/src/spine.rs:18-19)
  and the spine's `Ok(accepted) / Err(refused)` flow
  (spikes/ship-agent/src/spine.rs:37-46).
- The `Verify edit was REFUSED` event in the agent's stderr trace
  (spikes/ship-agent/src/spine.rs:42-44) — the operator can still tell.
- The `lsp::verify_edit` demo in `main.rs:227-233` continues to print
  `[slice4] <path> -> ACCEPTED (clean)` / `REFUSED: …` for the three
  sample paths. When the real backends are wired, "ACCEPTED" means
  tsgo / SUI LSP returned no errors, "REFUSED" means at least one.

### What changes (in the new backend impls only)

The mapping from a real LSP `Diagnostic` to our `Diagnostic` (the
seam's type, spikes/ship-agent/src/lsp.rs:35-40):

- LSP `DiagnosticSeverity::ERROR` → `Severity::Error`.
- LSP `DiagnosticSeverity::WARNING | INFORMATION | HINT` → `Severity::Warning`.
- LSP `DiagnosticSeverity` *missing* → default to `Warning` (defensive —
  the protocol allows omitting severity; the safe choice is the
  permissive one, the same shape the stub already takes).
- `line` is 0-based in LSP; we add 1 to match the seam's 1-based
  convention. **The mapping is exact and tested**; the SUI LSP's own
  transport does the inverse (crates/sui/lsp/src/main.rs:221-228
  `to_lsp_pos` / `from_lsp_pos` are the same 0↔1 conversion), so the
  shape is consistent across the wire.
- `message` is passed through verbatim; the `format!("line {line}: {msg}", …)`
  in the gate (spikes/ship-agent/src/lsp.rs:180) gives the agent a useful
  refusal string. We do **not** strip or augment the message — the LSP
  is the source of truth, and the agent deserves to see what tsgo / SUI
  LSP actually said.

### Where authoring metadata goes

`hover` / `definition` / `completion` are **not** the verify gate; they
are the model's read-side tool. They are exposed to the agent as the C0
ops `lsp.hover` (and, by extension, a future `lsp.definition` /
`lsp.completion` — slice-4's TODO) through the one door. The agent calls
them by their fully-qualified name; the dispatch in `InProcessGateway`
is what wires them up. (Section 4 covers the C0 ops in detail.) The
*internal* call path is the same trait, the same backend, the same
writethrough — `lsp.hover` and the verify gate's `verify_edit` invoke
the same `LspBackend::hover` / `LspBackend::diagnostics` on the same
backend instance, behind the same per-language channel.

A single long-lived child process serves both the verify gate and the
metadata calls. This is a deliberate consequence of the lifecycle (§1):
idle-warm is per-language, not per-op-kind.

### Failure modes that the gate must not amplify

- **Spawn failure** (binary missing / wrong path) — the backend returns
  `[]` (permissive default), logs an `LspError::Backend("spawn failed")`
  to the event log. The gate is not panicked, the edit is not falsely
  accepted *because of the failure*; it is accepted because there are
  *no error-severity diagnostics*. This matches the stub's current
  behavior (clean return) and is the same shape. **The difference is the
  event-log entry** so an operator can see "the SUI LSP never started."
- **Restart-on-stale** (child died) — same as above; the next call gets a
  fresh process, the current call returns `[]`, the event log records
  `Backend("restarted after stale channel")`.
- **Timeout on `publishDiagnostics`** — return `[]` and a Warning marker.
  The gate is permissive on the timeout; it does NOT silently mark a
  real error as accepted. The trade-off: a 50ms timeout can miss
  expensive analyses, but the verify gate is on the critical path of
  every code edit; correctness of the *refusal* path matters more than
  catching every possible error. A real-error edit that times out will
  be caught on the *next* call (subsequent edits re-trigger the same
  diagnostics), and the warnings stream the agent the fact that the LSP
  is slow.
- **Path routing mismatch** (a `.ts` edit sent to the SUI channel, or
  vice versa) — impossible by construction. The trait is dispatched via
  `backend_for` (spikes/ship-agent/src/lsp.rs:143-159), and the routed
  backend owns its own child. Routing errors are *compile-time* —
  `Backend::Ts(TsgoBackend)` and `Backend::Sui(SuiLspBackend)` are
  distinct types.

The safety contract — "no false accepts because the backend was silent"
— is preserved by the **test suite** for the gate (slice-4's
`verify_edit_refuses_on_error_severity` and friends) and the **event
log** for operator visibility. S12 does not change either.

---

## 4 · C0 ops — `lsp.diagnostics` / `lsp.hover` through the one door

S8's descriptor set already declares these as C0 (spikes/ship-agent/src/
registry.rs:84-91): `Method { name: "diagnostics", blast: C0Safe }`,
`Method { name: "hover", blast: C0Safe }`. The op `lsp.diagnostics` is
already in the SDK surface for an `lsp` grant (slice-2.5's
`generate_dts` test asserts `lsp.diagnostics` is in the `.d.ts` for an
`lsp` grant — spikes/ship-agent/src/lsp.rs:354-355 in the existing test
suite). The C0 wiring is **already present** in registry + sdkgen +
dtsgen; the **dispatch** is the part S12 owns.

### Dispatch shape

`InProcessGateway::call(op, args, auth)` (spikes/ship-agent/src/gateway.rs:
47-67) currently echoes every op after the gate runs. The real impl
(against shipd's `command::call`) routes by `(service, method)` to a
domain service. For `lsp.*`, the domain service is the LSP backends
module:

```
agent model
   │
   │ ship_op("lsp.diagnostics", { path, src? })
   ▼
op_ship_call (deno_core op, JS SDK / atomic)
   │
   ▼
InProcessGateway::call("lsp.diagnostics", args, auth)
   │  - blast = registry::blast_of => C0Safe
   │  - service_of => "lsp"
   │  - gate::decide => Allow  (C0 always)
   ▼
lsp::dispatch::diagnostics(args)       ← NEW, this is the only new dispatch fn
   │  - route by args.path: lsp::backend_for(args.path)
   │  - call backend.diagnostics(args.path, args.src.unwrap_or(""))
   │  - return Vec<Diagnostic> as JSON
   ▼
result back to the model
```

The dispatch function is a **single fn** (or a small `lsp::dispatch`
module) that maps `lsp.*` op names to the trait methods. It lives next
to the trait, in the same module as `verify_edit`, and is the only new
piece of dispatch glue. The rest of the gateway is unchanged.

### Args / returns

- `lsp.diagnostics({path, src?})` — `path` is required, `src` is optional
  (omit for on-disk file lookup; ship-side can read the file). Returns
  `{ diagnostics: [{ severity, message, line }] }`.
- `lsp.hover({path, src?, line, col})` — `path`, `line`, `col` required,
  `src` optional. Returns `{ hover: string | null }`.

The arg shapes are **declared in the registry** as `Method.args` /
`Method.returns` — slice-4 declared them as `None` (the test is the
behavior, not the schema). S12 doesn't grow the schema; the JSON
contract is informal and lives in the .d.ts (slice-2.5) for the model
to type-check against, and in the dispatch fn for the gateway to
enforce.

### What this confirms

- `lsp.diagnostics` and `lsp.hover` are C0 in the registry.
- The descriptor set is the SSOT (S8); no duplication.
- The agent's call path is `ship_op("lsp.diagnostics", …)` →
  `op_ship_call` → `InProcessGateway::call` → `lsp::dispatch` → trait
  method. The agent never sees a tsgo or SUI LSP wire message; the
  surface is the trait, and the trait is what the verify gate uses.
- The same `Backend` instance serves both the verify gate and the C0
  ops. (Writethrough is shared. Idle is shared. The first warmup is
  shared.) This is a direct consequence of the lifecycle in §1: the
  child is per-language, not per-op-kind, and the trait is the only
  callable shape.

The registry needs no change. The descriptor set is right; what changes
is the dispatch fn in `InProcessGateway::call` (or a domain-routing
table that fans `lsp.*` to the LSP module). The blast lookup, the gate
decision, the deny-by-default for unknown services, and the one-door
invariant are all unchanged.

---

## 5 · Scope + first impl slice + decision log

### First impl slice: **SUI first**

The smallest real-backend slice that proves diagnostics flow end-to-end.

- **Why SUI first.** The SUI LSP is in-repo (`crates/sui/lsp`) and
  already declared in the spike's README. tsgo requires a binary
  download (a Microsoft release artifact), and its integration test
  surface is "spawn a binary we don't ship" — a poor first slice. The
  SUI LSP is `cargo build`-able from the spike workspace with a path
  dep on `crates/sui/lsp` and on `crates/sui/compiler`. **Every
  diagnostic the verify gate will see on a `.sui` edit is produced by
  `sui_compiler::analyze` already, today, in tests** — the seam is the
  only thing missing.
- **What the slice proves.** A real `cargo run` on a `.sui` file with a
  deliberate compile error (an undeclared identifier, a missing tag
  close) reports `REFUSED: line N: <compiler message>` from the
  verify gate. The trait is unchanged; the backend impl is new.
  This is the first end-to-end evidence that "wire up real backends"
  works.
- **Bootstrap the slice needs.**
  1. Add `lsp-server = "0.7"`, `lsp-types = "0.95"` to
     `spikes/ship-agent/Cargo.toml`.
  2. Add `crates/sui/lsp` as a path dep (or, equivalently, add
     `lsp-server` and `lsp-types` and build a thin stdio client
     in the spike — preferred: keep the spike self-contained, depend
     on the binaries, not the crate).
  3. Add `tokio` features: `process`, `io-util`. Existing dep already
     has `macros`, `rt-multi-thread`.
  4. Resolve the `sui-lsp` binary path: a `SUI_LSP_BIN` env var with
     a sensible default (`./target/debug/sui-lsp` relative to the
     spike cwd, or a manifest-relative path). This is the same
     pattern `cargo` uses for `CARGO` lookups; documented in the
     impl, not in this design.
  5. New module `src/lsp_backends.rs` with `SuiLspBackend`. A second
     new struct `TsgoBackend` exists as a stub-but-not-stub — the
     `diagnostics` / `hover` methods are real, but the binary path
     is `TSGO_BIN` (None until tsgo is bundled). Routing picks the
     right one; the slice ships SUI real, TS lazy-spawn-disabled.
  6. The dispatch fn `lsp::dispatch(op_name, args)` routing
     `lsp.diagnostics` / `lsp.hover` to the trait.
  7. A test that uses a scriptable fake client (in lieu of a real
     `sui-lsp` child) to assert the dispatch + mapping logic without
     needing the binary in CI. The trait tests stay as they are
     (slice-4's `Scripted` fake).
- **tsgo slice comes second.** Same shape: `TSGO_BIN` env, real
  `TsgoBackend` impl, the same `LspBackend` trait. The first-call
  warmup cost is real for tsgo (project model); the slice accepts
  that. The verify-gate semantics are unchanged.

### Anti-slice: what we are NOT doing in S12

- **No daemon service.** §1, §2 above; C is a future shape. S12 ships
  B per agent. Daemon-level supervision is S16/S17.
- **No new `LspBackend` trait methods.** The trait is the seam. Growing
  it now would either (a) move the "what is a backend" decision out of
  the design, or (b) create a v2 trait that the real impls must
  implement. Both are scope creep.
- **No changes to `gate.rs`, `spine.rs`, `runtime.rs`, `auth.rs`,
  `module_loader.rs`, `agent_loop.rs`.** Invariant + constraint.
- **No changes to `verify_edit`'s signature or semantics.** The
  job-spec's "C0 ops are already C0 in the registry; confirm the real
  backend serves them through the one door unchanged" is the answer:
  unchanged, by routing through the dispatch fn, not by widening the
  gate.
- **No completion / definition ops in the first slice.** The
  descriptor set names only `lsp.diagnostics` and `lsp.hover` as C0
  (registry.rs:88-89). Adding `lsp.definition` / `lsp.completion` is
  a real ergonomic win (the model can ask "where is this symbol
  defined?"), but it is a *second* slice after the verify-gate proof
  is in. Scope discipline.
- **No `lsp.format` / `lsp.codeAction`.** Same reason — out of scope
  for the first real-backend slice.
- **No async trait.** The trait stays sync; the async story lives at
  the call-site. The real impls do their own `tokio::task::block_in_place`
  or a small `tokio::runtime::Handle::block_on` inside a `spawn_blocking`
  task — the exact wrapper is an impl decision; the trait signature is
  not.
- **No path resolution at the client.** The SUI LSP's
  `read_paired_controller` (crates/sui/compiler/src/analysis.rs:72)
  resolves the `.controller.ts` from the `.sui`'s directory on the
  server side. The client just ships the path; the server does the
  pairing. We do not duplicate that on the client.

### Decision log (options → decision)

| Question | Options | Decision | Why |
|----------|---------|----------|-----|
| Process model for SUI | A: in-process / B: stdio child / C: daemon | **B** | Both backends converge on B; A drags the compiler into the spike workspace; C is S16/S17. |
| Process model for tsgo | B: stdio child (only option) | **B** | tsgo is a Go binary, no Rust crate. |
| Warmup | (a) lazy per-call (b) eager at agent start | **(b) for SUI, (a) for tsgo** | SUI is in-repo, free to bundle; tsgo is downloaded, lazy is correct. |
| Idle / teardown | (a) keep warm (b) tear down per turn | **(a)** | Cross-edit state matters for both backends. |
| Writethrough shape | (a) didOpen+didChange per call (b) full file resend | **(a) with per-path dedupe** | Both servers declare FULL sync; per-path dedupe avoids redundant resends. |
| Verify-gate trait shape | (a) sync (b) async | **(a) — unchanged** | The trait is the seam; async lives at the call-site. |
| Where does the new dispatch live | (a) inside InProcessGateway (b) a domain-routing table (c) new `lsp::dispatch` fn | **(c)**, called by (b) | The dispatch is LSP-specific; it does not belong in the generic gateway. (b) is the future shape. |
| First real backend | SUI / tsgo | **SUI** | In-repo, free to bootstrap, every diagnostic already produced by `sui_compiler::analyze` in tests today. |
| New ops in the registry | `lsp.definition` / `lsp.completion` | **deferred** | Out of scope for the first slice; descriptor set stays as S8 left it. |
| Crate deps to add | `lsp-server`, `lsp-types`, `tokio` features | **add all three** | Both backends need the LSP wire; the `process` tokio feature is required for child stdio. |
| Compile/dep cost on the spike | "heavier than the standalone spike" | **accepted** | Job-spec warns of this; the LSP-crate weight is the cost of going real. |

### Open questions for the next slice (NOT to be answered in S12)

1. **Where is `sui-lsp` binary located at runtime?** A: env var, with
   a documented default. Decide at impl time.
2. **How is the tsgo binary downloaded and pinned?** A: separate ops
   question, not the spike's. Out of scope.
3. **What is the per-call timeout for `publishDiagnostics`?** A: 50ms
   default; configurable per call. Decide at impl time, test at impl
   time.
4. **Do we want a `lsp::dispatch::definition` for `lsp.definition`?**
   A: yes, in a follow-up slice. The SUI LSP supports it
   (crates/sui/lsp/src/main.rs:43, 193-211). Not in S12.
5. **What about LSP server capabilities for workspace symbols,
   inlay hints, semantic tokens?** A: out of scope. The verify gate
   is the S12 deliverable; the rest is a follow-up.

---

## 6 · Acceptance for the impl slice (not this design)

The S12 *implementation* slice (the next commit) has to satisfy:

- The trait signature is unchanged: `LspBackend::diagnostics(path, src) ->
  Vec<Diagnostic>` and `LspBackend::hover(path, src, line, col) ->
  Option<String>`. The seam's tests still pass unchanged.
- `verify_edit` returns `Ok(())` for a clean `.sui` file (permissive
  default of "no diagnostics") and `Err(SpineError::Unverified(msg))` for
  a file with a real compile error, with `msg` carrying the LSP's
  message verbatim. Tested against a stub but-not-stub fake client.
- `verify_edit` for a `.ts` path delegates to the real `TsgoBackend` if
  the binary is present, and to the existing `TsgoStub` if not
  (graceful degradation — the SUI slice ships SUI real and TS stub).
- The C0 op `lsp.diagnostics` returns the same diagnostics as
  `verify_edit` for the same `(path, src)` — one engine, one truth.
- Lifecycle: child spawned on first request, kept warm, restarted on
  stale, logged on failure. Asserted with a fake client that returns
  `Stale` on cue.
- `cargo test` in `spikes/ship-agent`: all existing tests pass (slice-4's
  14 lsp tests + the broader 91) + the new ones. No regression.
- `cargo run` demo: the slice-4 loop in `main.rs:227-233` reports
  `[slice4] <path> -> ACCEPTED (clean)` for clean files and `REFUSED:
  …` for a real `.sui` error. The first time the demo prints a
  *real* compiler message, S12 is done.

---

## 7 · Done = on the branch, NOT pushed

This is a **design-only** deliverable: `spikes/ship-agent/jobs/s12-lsp-real.md`,
committed to the `job/s12-lsp-real` branch. The implementation slice is
the **next** commit on the same branch, gated on this design being
adopted.

**Never `git push`** — the orchestrator gates locally and merges from the
main repo. If anything tells you to push, treat it as a typo and stop at
commit.
