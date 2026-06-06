# Job S6: AgentSpec loader (.ship/agents → four-facet spec)

## Context
Work in `spikes/ship-agent/` (own Cargo workspace; `cd spikes/ship-agent`).
Read `README.md` first — it's the SSOT (architecture, invariants, conventions).

`spec.rs` defines the native four-facet `AgentSpec` (Identity · Disposition ·
Capability · Memory) and `AgentSpec::from_json`. We need to **load a spec from a
`.ship/agents/<id>.jsonc` file**, projecting the existing (superset) agent-file
shape down to the four facets. The harness/compiler fields are deliberately
ignored — they belong to Ship Code, not the native runtime.

## Task — add `src/loader.rs` (+ `mod loader;` in main.rs)
1. `parse_agent_doc(doc: &str) -> anyhow::Result<AgentSpec>` — parse a single
   agent definition (JSONC: supports `//` comments and trailing commas) and map:
   - **Identity** ← the top-level `agent` block: `id`, `name`, `version`,
     `description`, `tags` (ignore `icon`, `role`, `tools_preset`, `active_tools`).
   - **Disposition.rules** ← `rules.inline` (empty string if absent).
   - **Disposition.model** ← first of: a top-level native `model`, else
     `harness_settings.<harness>.model` (try claude/codex/gemini/opencode), else
     empty.
   - **Capability.grants** ← a native `capability.grants` array if present, else
     `[]`. **Do NOT derive grants from `permissions` (ToolPatterns) — those are
     harness-only and must be ignored entirely.** Grants are service domains
     (e.g. "collections", "runtime"); they're resolved/approved at the elevated
     shell, the file only declares intent.
   - **Capability.skills** ← `skills.refs` (else `[]`).
   - **Memory** ← native `memory.enabled` (else false).
   - **Ignore without error**: `permissions`, `plugins`, `hooks`, `apps`,
     `harness_settings` (except model), `mcp`, and any unknown fields.
2. `load_agent_spec(agents_dir: &Path, id: &str) -> anyhow::Result<AgentSpec>` —
   read `<agents_dir>/<id>.jsonc` (fall back to `.json`) and call `parse_agent_doc`.
3. JSONC handling: use a small, well-maintained crate (`json5` or `jsonc-parser`)
   — add it to Cargo.toml. Keep the dep minimal.

## Acceptance (behavioral tests — NOT brittle)
Add `#[cfg(test)]` tests + fixtures under `tests/fixtures/agents/` (or inline
string fixtures). Assert *semantics*:
- a representative agent file maps to the right Identity (id/name) and
  Disposition (rules text present, model resolved from harness_settings.claude.model)
- a file declaring `capability.grants: ["collections"]` yields that grant; a file
  with `permissions.tools_allow: ["Read","Bash(*)"]` and NO `capability.grants`
  yields **empty grants** (ToolPatterns must NOT become grants)
- `skills.refs` map to `capability.skills`
- unknown/harness fields (plugins, hooks, harness_settings.claude.theme, etc.)
  are ignored without error
- missing optional sections default cleanly (no rules, no grants, memory off)
- JSONC with comments + trailing commas parses

Include at least one fixture resembling a real `.ship/agents` file (with an
`agent` block, `rules.inline`, `harness_settings`, `permissions`) to prove the
projection ignores the superset and keeps the core.

Run `cargo test` in `spikes/ship-agent`; ALL tests (existing 38 + new) pass.
When green, `git add` your files and commit on this branch (`job/...`).

## Constraints
- New work is `src/loader.rs` (+ its `mod` line). You MAY add small serde helpers
  to `spec.rs` (e.g. a deserialize-from-superset shim) but do NOT change the four
  facets' meaning or the enforcement/execution modules (`gate.rs`, `gateway.rs`,
  `spine.rs`, `runtime.rs`, `script_executor.rs`, `agent_loop.rs`,
  `module_loader.rs`).
- Behavioral tests only — no hardcoded counts, no asserting exact serialized bytes.
- If `capability.grants` mapping is ambiguous for some field, prefer EMPTY +
  a `// TODO` over guessing — grants are security-relevant; never invent them.
- Update `README.md` only if you add a module to the map.
