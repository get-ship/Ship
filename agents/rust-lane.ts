// agents/rust-lane.ts — per-job source dir for the `rust-lane` agent.
// Migrated from `ship.config.ts`'s inline `defineAgent` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; no field changes.
import { defineAgent } from "ship";

defineAgent({
  "agent": {
    "id": "rust-lane",
    "name": "Rust Lane",
    "icon": "simpleicons:rust",
    "version": "0.1.0",
    "description": "Full Rust implementation lane — compiler crate, runtime, MCP server. Use for cross-crate work.",
  },
  "skills": {
    "refs": [],
  },
  "mcp": {
    "servers": ["ship"],
  },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(git push*)", "Bash(*cargo publish*)", "Bash(*npm publish*)", "Bash(* ~/.ship/*)", "Bash(rm -rf *)"],
  },
  "rules": {
    "inline": "Scope: Rust surface — crates/core/, apps/mcp/, apps/shipd/, apps/ship-cli/. May also touch scripts/, contrib/, .github/workflows/ when the job spec requires it. Compiler is pure transformation — no side effects. Runtime owns DB migrations. Use `just build`, `cargo test -p <crate>`. Never run_in_background for builds. Job spec overrides scope when it lists a specific file scope.",
  },
});
