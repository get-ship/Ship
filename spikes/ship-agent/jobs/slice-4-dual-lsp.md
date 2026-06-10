# Job: dual-LSP tool + diagnostics-fed verify (slice 4)

## Context
Work in `spikes/ship-agent/` (its own Cargo workspace; `cd spikes/ship-agent` to
build/test). This is the Ship agent runtime spike. The differentiator is **tool
quality** — and the flagship tool is semantic code intelligence over BOTH of
Ship's languages:
- **TypeScript** (`.ts`/`.tsx`) — controllers, the scoped SDK, agent code → backed by **tsgo**
- **SUI templates** (`.sui`) — views/components → backed by the **SUI LSP** (`crates/sui/lsp` in the main repo)

The agent consumes LSP two ways: (1) authoring metadata (hover/definition), and
(2) the **verify gate** — after an edit, run diagnostics; a new error means the
edit is `UNVERIFIED`, caught *before* execution.

This spike has no real tsgo/SUI-LSP wired yet (like the gateway, it's a SEAM).
Build the seam + routing + verify wiring; real backends drop in later.

## Task — create `src/lsp.rs`
1. Types: `Diagnostic { severity, message, line }` with a `Severity` enum
   (at least `Error`, `Warning`); `LspError`.
2. `trait LspBackend { fn diagnostics(&self, path: &str, src: &str) -> Vec<Diagnostic>; fn hover(&self, path: &str, src: &str, line: u32, col: u32) -> Option<String>; }`
3. **Routing by file type**: `fn backend_for(path: &str) -> Backend` choosing the
   TS backend for `.ts`/`.tsx` and the SUI backend for `.sui`. Two stub impls
   (`TsgoStub`, `SuiLspStub`) — seams returning empty/canned data, clearly marked
   as where the real tsgo / `crates/sui/lsp` wiring lands.
4. **Verify wiring**: `fn verify_edit(path: &str, src: &str) -> Result<(), crate::spine::SpineError>`
   — routes to the right backend, runs `diagnostics`; if any `Severity::Error`,
   return `SpineError::Unverified` with the messages; else `Ok(())`. This is the
   spine's post-edit gate for code, using the SAME engine that serves metadata.

## Optional (only if clean)
Register `lsp.diagnostics` / `lsp.hover` as **C0** ops in the registry
(`sdkgen::ops_for` "lsp" + `gateway::blast_of` C0) so the agent can call them
through the one door. LSP is read-only analysis → C0, baseline-allowed. If this
balloons, skip it and leave a `// TODO(lsp-ops)` note — the core deliverable is
`lsp.rs` + verify routing.

## Acceptance (behavioral tests — NOT brittle)
Add `#[cfg(test)]` tests asserting *semantics*:
- `backend_for` picks the TS backend for a `.ts` path and the SUI backend for a
  `.sui` path (assert the chosen backend's identity/kind, not internals)
- `verify_edit` returns `Unverified` when the routed backend reports an
  `Error`-severity diagnostic, and `Ok` when it reports none (use a stub backend
  that you can make report an error vs. clean — do NOT hardcode tsgo output)
- routing is total: an unknown extension has a defined, tested behavior

Run `cargo test` in `spikes/ship-agent`; ALL tests (existing 25 + new) must pass.

## Constraints
- Primary work is the NEW `src/lsp.rs` (+ its `mod lsp;` line in `main.rs`).
- Do NOT change `gate.rs::decide`, the spine's core logic, `auth.rs`,
  `runtime.rs`, `module_loader.rs`, or `agent_loop.rs`. (The optional op
  registration may touch `gateway.rs`/`sdkgen.rs` for registry entries ONLY —
  never the gate decision or dispatch logic.)
- Behavioral tests only — no hardcoded counts, no asserting exact generated
  strings. If a test pins a count or a literal line, rewrite it to assert behavior.
- Keep it a seam: do not try to invoke a real tsgo or SUI LSP binary.
