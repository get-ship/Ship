# Layer Shared — Audit Shape

Shared crates are consumed by multiple layers: `build-info`, `mcp-framework`,
`ship-pipelines`, `cli-framework`, `ship-types`, and similar cross-cutting
utilities. The defining constraint is **library discipline**: a shared crate
must not import from L1 or L0 domain crates (only from peer shared crates and
workspace infrastructure). Audits at this layer are caller-graph-centric.

## What shared audits cover

- **Caller graph** — which crates consume this shared crate, and from which
  call sites? An undocumented consumer is a dependency boundary that may
  constrain future restructuring.
- **Library discipline** — is every `pub` item actually consumed somewhere?
  Shared crates accumulate dead exports that silently widen the API surface.
- **Layer-direction** — `shared → L0` or `shared → L1` imports are violations.
  Shared crates must be bottom-of-graph leaves (or import only from peer shared).
- **Semver risk** — if the crate is or could become externally published
  (`publish = false` → check), API changes carry compatibility cost.

## Section shape (9 sections)

### 1. File inventory
Table: `File | LOC | Role`. Note cap status per file. Shared crates tend to be
small; a soft-cap violation here is notable.

### 2. Crate role
Paragraph: purpose, what it replaces or consolidates, what it enables. If this
is a `[parallel-systems-consolidation]` (multiple prior parallel impls retired
into one crate), say so explicitly and cite the retired implementations.

### 3. Public surface
Table: `Item | Site | Verdict`. Verdict values: `contract` (has callers),
`internal-leaked` (pub but no external callers), `dead` (pub, no callers
anywhere).

State the total: "0 dead, 0 leaked, N scaffolding." Note the cleanest shared
crates (`build-info` is the reference — 0 dead, 0 leaked, 0 violations).

### 4. Caller graph
Enumerate every direct Rust consumer (by crate) with at least one call-site
citation. Group: production crates vs dev/test-only. Note any doc-only
references (imports only in doc-comments — these are not real dependencies).
Check for a 4th consumer if the crate is being considered for expansion.

### 5. Layer-direction
`grep -n "use ship_" src/lib.rs` (or equivalent for the crate). Confirm:
- No imports from L0 domain crates (`ship_substrate::`, `ship_kernel::`,
  `ship_supervisor::`) unless this shared crate is specifically a peer of L0.
- No imports from L1 domain crates (`ship_platform::`, `ship_analytics::`, etc.).
- Workspace/infrastructure deps only: `anyhow`, `serde`, `tokio`, etc.

If the crate has zero Ship-internal deps, state: "Strict bottom-of-graph leaf."
Layer-direction violations are `[layer-direction: shared→L1]` severity `high`.

### 6. API discipline
- `#[doc(hidden)]` usage — correct for macro support modules; check it is not
  hiding load-bearing stable API.
- Macro hygiene — does the macro use `$crate::` qualified paths? Required to
  prevent import-order breakage.
- Derive traits — `Copy + Clone + Debug` for typed-value semantics; `Send + Sync`
  if used across thread boundaries.
- Allocation in hot paths — `&'static str` fields prevent allocation on read path;
  `String` fields are acceptable for infrequent operations.
- Soundness — `unsafe`, `panic!`/`unwrap`/`expect` in production paths.

### 7. Cross-cutting observations
Bullet per observation with tag string from the collection vocabulary. Common
shared-layer tags:
- `[parallel-systems-consolidation]` — first sighting (only promoted once a
  second crate ratifies the pattern)
- `[layer-direction: shared→L1]` — violation; severity `high`
- `[over-applied-pub]` — pub items with no external callers
- `[dead-crate]` — the crate is imported but none of its exports have live callers

### 8. Open calls
Genuine structural questions only. Common shared-layer open calls:
- `__private` module location convention (keep vs rename vs move)
- Doc-only references — should the referencing crate actually import?
- Build-script path assumptions that break outside the workspace layout
- `publish = false` correctness — is this intentionally workspace-only?

### 9. Security findings
Shared crates rarely have security surface (no auth, no file access, no network).
State explicitly: "No security surface. The crate captures only [build metadata /
type definitions / macro expansion / …]." If findings exist, use the SF-N format.

## Shared-specific checks

### Self-enforcing layer tests
Some L0/shared crates assert their own dependency graph in tests
(`include_str!("../Cargo.toml") + assert !contains("sqlx")`). This is a novel
guard pattern. Tag first sighting as `[candidate: layer-purity-enforcement]`.

### Macro support modules
`__private` or `__macro_support` modules should be tagged `#[doc(hidden)]` and
use `$crate::` qualification. Verify the macro won't break under import reordering.

### `build-info` as the reference shape
`crates/build-info/src/lib.rs` is the canonical clean shared crate:
- 0 dead, 0 leaked, 0 layer violations
- Self-contained: zero Ship-internal deps
- Two surfaces (build-side `emit_build_env()` + runtime-side `build_info!()`)
- `#[doc(hidden)] mod __private` correctly scoped to macro plumbing
- 3 tests covering the public surface

Use it as the comparison baseline when evaluating other shared crates.

### Parallel-systems-consolidation
When a shared crate explicitly documents that it replaced N prior parallel
implementations (e.g., `build-info` replacing 4 prior identity-capture impls),
this is a positive finding — the opposite of most shared observations (which
uncover parallel systems). Tag `[parallel-systems-consolidation]` first sighting
as `candidate`; a second crate ratifying the pattern promotes it to `confirmed`.
