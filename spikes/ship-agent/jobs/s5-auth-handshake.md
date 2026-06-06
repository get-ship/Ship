# Job S5: auth handshake seam

## Context
Work in `spikes/ship-agent/` (own workspace; builds anywhere; first build slow —
wait it out). Read `README.md` (SSOT). Today `AuthContext` is hand-constructed
locally (spike-only). In production the agent **presents a credential to shipd
and receives its grants** — the agent does not decide its own grants. This slice
builds that SEAM behind a trait + a fake; the real peer-cred-socket handshake to
shipd is a later refactor-gated swap.

## Task — add `src/handshake.rs` (+ `mod handshake;` in main.rs)
1. `trait AuthHandshake { fn authenticate(&self, agent_id: &str) -> anyhow::Result<crate::auth::AuthContext>; }`
   — the seam: identity in, an `AuthContext` (with grants) out.
2. `struct FakeHandshake` configured with a map of `agent_id -> granted service
   domains`. `authenticate` returns `AuthContext::in_process(agent_id, grants)`
   for a known agent (use the EXISTING `AuthContext::in_process` constructor — do
   NOT modify `auth.rs`). Unknown agent → `Err`.
   - INVARIANT: the produced `AuthContext` always has `operator_cred == false`.
     The handshake can grant service domains; it can NEVER mint an operator.
3. Doc-comment the real seam: the production impl performs a peer-cred handshake
   over shipd's socket and shipd returns the grants — note it as a `// TODO(swap)`.

## Acceptance (behavioral tests — NOT brittle)
- a known agent authenticates to an `AuthContext` carrying its configured grants.
- the produced `AuthContext` has `operator_cred == false` (assert it — the
  no-self-elevation invariant holds across the handshake).
- an unknown agent yields `Err` (not an empty-grant success, not a panic).
- the grants returned match what the handshake was configured with (the agent
  does not get to choose its own grants).

## Process (IMPORTANT)
Run `cd spikes/ship-agent && cargo test` in the FOREGROUND and WAIT for the final
`test result:` line before reporting. Do NOT background the build or return early.
When green (existing 87 + new), `git add` + `git commit` on the current branch.

## Constraints
- New file `src/handshake.rs` (+ its mod line) ONLY. Do NOT modify `auth.rs`
  (use `AuthContext::in_process`), enforcement/execution modules, or any other
  slice's files, or `Cargo.toml` (no new deps).
- Behavioral tests only. Update `README.md` seams table (auth row) / module map
  if appropriate.
