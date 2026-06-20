# Job Spec: ship-substrate dead-code cull (audit phase1 — PILOT)

**Branch:** job/substrate-deadcode
**Agent:** rust-lane
**Mode:** autonomous

## Goal

Remove the *verified-dead* items from the substrate phase-1 audit
(`.artifacts/rust-readonly-audit/phase1-dead-code-substrate.md`). That audit was
**static-only** (it could not run the compiler) — so it is a list of HYPOTHESES. Your
job is to re-verify each against the compiler + a caller search, delete the ones that
are truly dead, and STOP+note any that aren't. Do NOT trust the audit blindly; it has
been wrong before in sibling audits.

## File scope
Only the files named in the DELETE list below. Touch nothing else.

## DELETE (verify each, then remove)
For each: (a) ripgrep the whole workspace for callers, (b) confirm zero, (c) remove the
item + its `#[allow(dead_code)]`, (d) confirm the build still passes. If a caller exists
or a build breaks, STOP, leave it, and write `note(out-of-scope): <item> not dead — <why>`.

1. `crates/core/ship-platform/crates/agent-def/src/harnesses.rs:36` —
   `fn is_supported_harness` (dead wrapper over `adapter_for(id).is_ok()`).
2. `crates/core/ship-platform/crates/ship-isolation/src/net/proxy.rs:267` —
   `async fn drain_until_eof` (private, zero callers).
3. `crates/core/ship-platform/crates/ship-isolation/src/runtime/sidecar_backend.rs:168` —
   `pub async fn force_kill` (doc claims a future caller that was never written; zero callers).
4. `crates/core/ship-platform/crates/ship-agents/src/hooks/mod.rs:38` —
   `struct ProviderHook` (forward-compat deser shim, no producer; module comment confirms
   dispatch is by `tool_name` alone).
5. `crates/core/compiler/src/compile/tests/fixtures.rs:10` —
   `pub fn all_harness_ids` (dead test fixture fn, zero callers).
6. `crates/core/ship-platform/crates/ship-auth/src/attest.rs:71` —
   `alg: Option<String>` JWK field (captured on deserialize, never read; algorithm comes
   from curve + JWT header). If you find it IS read anywhere, keep it + add a one-line
   doc instead of deleting.

## DO NOT TOUCH — these look dead but are LIVE / by-design
- **MEDIUM #5–#10 (feature-gated, NOT dead):** `ship-cloud::sync_service` (`unstable`),
  `ship-js::deno` (`deno`), `ship-agents::cancel` + `pub use ship_code as runtime`
  (`harness-drivers`), `ship-substrate-host` Linux/unix impls (`target_os`/`unix`). These
  are alive under their features — deleting them breaks feature builds.
- **LOW #12, #13:** the `const _AGENT_DEF_LINK` / `_check` type-witness patterns — intentional,
  keep.
- Everything in the audit's "Cross-references — items that look suspicious but are LIVE"
  section (inventory re-export, `EventPayload` derive, pipeline types, `Noop`,
  `SandboxOwner`, `StepExecutor`, account stores, `AgentDefIndex`, ops-rpc `OPS`, etc.).

## Constraints
- Verify-then-delete. The audit is evidence, not a plan. Never guess — STOP + note on doubt.
- Pre-launch: delete, don't gate/comment-out.
- Never widen scope beyond the 6 items. Never push.

## Acceptance
1. `cargo check --workspace --exclude ship-host --all-targets --features unstable` green
   (the audit's own gate).
2. Each deleted item confirmed caller-free before removal (note the ripgrep result in the
   commit body).
3. Any item you did NOT delete because verification failed is recorded as a
   `note(out-of-scope)` line — do not silently skip.
4. Commit cleanly. DO NOT push.

---
> Execute yourself. Read-only research subagents are fine; do not delegate the edits.
> Commit when acceptance passes. DO NOT push.

## Verification results

**Deleted (caller-free):**
1. `crates/core/ship-platform/crates/agent-def/src/harnesses.rs:36` `is_supported_harness` — `rg is_supported_harness` returned only the definition. Removed the fn + its `#[allow(dead_code)]`.
2. `crates/core/ship-platform/crates/ship-isolation/src/net/proxy.rs:267` `drain_until_eof` — `rg drain_until_eof` returned only the definition. Removed fn + its `#[allow(dead_code)]`. `AsyncReadExt` import kept (used by test module `sock.read` at lines 345/383/389).
3. `crates/core/ship-platform/crates/ship-isolation/src/runtime/sidecar_backend.rs:168` `pub async fn force_kill` (free fn) — `rg force_kill` only matched this fn and an unrelated `Session::force_kill` method in `bin/ship-sidecar/src/session.rs:476` (different item). The free fn had zero callers; its doc-stated future caller was never written. Removed fn + its `#[allow(dead_code)]` + now-unused `ProcessKillRequest` import.
6. `crates/core/ship-platform/crates/ship-auth/src/attest.rs:71` `alg: Option<String>` JWK field — `rg '\.alg\b'` confirms never read. Algorithm is hard-coded to `Algorithm::EdDSA` at line 138 and key type is selected by `kty=="OKP"` + `crv=="Ed25519"` (line 131); JWT header `alg` is independent. Removed the field + its `#[allow(dead_code)]`.

**Not deleted (out-of-scope):**
4. `note(out-of-scope): ProviderHook not dead — struct is the deser target for `handle_before_tool` (mod.rs:61) and `handle_after_tool` (mod.rs:97); `tool_name`, `tool_input`, `tool_response` fields are read post-deser. Only `hook_event_name` and `mcp_context` are unused after deser (the "forward-compat" claim in the spec is correct for THOSE two fields, not the whole struct). Wholesale deletion breaks both handlers.
5. `note(out-of-scope): all_harness_ids not dead — used by `others()` in the same file (line 22), which is in turn called by `crates/core/compiler/src/compile/tests/claude_tests.rs:441` and `crates/core/compiler/src/compile/tests/codex_tests.rs:230`. The `#[allow(dead_code)]` is stale — `cargo check -p compiler --all-targets` produces no warning for it. Removing it would break the no-leak integration tests.
