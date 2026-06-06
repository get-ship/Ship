# Job S13: file ops + hashline (content-addressed edits)

## Context
Work in `spikes/ship-agent/` (own workspace; builds anywhere; first build slow —
wait it out). Read `README.md` (SSOT). Files are gated ops (`files.read`/
`files.edit`) and edits use **hashline** — content-hash line anchors so an edit
references a line by its hash, not a line number: token-cheap and staleness-safe
(if the file changed under the anchor, the edit is REFUSED, not misapplied — a
compare-and-swap on content).

## Task
1. New `src/hashline.rs`:
   - `fn line_anchors(src: &str) -> Vec<Anchor>` where `Anchor { line: usize, hash: String }`
     — a short stable content hash per line.
   - `fn apply_anchored_edit(src: &str, anchor_hash: &str, new_line: &str) -> Result<String, HashlineError>`:
     find the line whose hash equals `anchor_hash`; if found AND unique, replace
     it with `new_line` and return the new source; if the anchor is absent (file
     drifted) or ambiguous (multiple matches), return an `Err` — never guess.
   - `HashlineError` enum (e.g. `Stale`, `Ambiguous`).
2. Register file ops in the registry (registry entries ONLY — do not change gate
   logic or dispatch): `files.read` = **C0**, `files.edit` = **C1** in
   `gateway::blast_of`, and a `files` service in `sdkgen::ops_for` (ops: `read`,
   `edit`).

## Acceptance (behavioral tests — NOT brittle)
- `line_anchors` gives the same anchor for identical line content and different
  anchors for different content (stability + distinctness — assert the property,
  not specific hash strings).
- `apply_anchored_edit` with a CURRENT anchor replaces exactly that line; the
  rest of the file is unchanged.
- a STALE anchor (compute anchors, mutate the file, then apply the old anchor) is
  REFUSED with `Err` — the CAS property. An ambiguous anchor is also `Err`.
- `files.read` resolves to C0 and `files.edit` to C1 via `gateway::blast_of`;
  `sdkgen::generate_sdk(["files"])` exposes a `files` surface.

## Process (IMPORTANT)
Run `cd spikes/ship-agent && cargo test` in the FOREGROUND and WAIT for the final
`test result:` line before reporting. Do NOT background the build or return early.
When green (existing 87 + new), `git add` + `git commit` on the current branch.

## Constraints
- New file `src/hashline.rs` (+ its mod line). You MAY add registry entries to
  `gateway.rs` (`blast_of`) and `sdkgen.rs` (`ops_for`) — registry ONLY, never the
  gate decision or dispatch logic. Do NOT touch `gate.rs`, `spine.rs`,
  `runtime.rs`, `script_executor.rs`, `agent_loop.rs`, `module_loader.rs`,
  `provider.rs`, `session.rs`, `events.rs`, `skills.rs`, or `Cargo.toml`.
- Behavioral tests only (assert properties, never specific hash literals). Update
  `README.md` module map (add `hashline.rs`).
