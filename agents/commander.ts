// agents/commander.ts — per-job source dir for the `commander` agent.
// Migrated from `ship.config.ts`'s inline `defineAgent` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; no field changes,
// no inline-rules extraction. The full TS body is what the resolver
// harvests into `ExtensionLibrary::agent_profiles`.
import { defineAgent } from "ship";

defineAgent({
  "agent": {
    "id": "commander",
    "name": "Commander",
    "icon": "simpleicons:starship",
    "version": "0.2.0",
    "role": "commander",
    "description": "Ship orchestrator — captures intent, dispatches specialists, monitors via mesh, gates before merge",
  },
  "skills": {
    "refs": ["cmdr", "spawn-agent", "web-qa"],
  },
  "mcp": {
    "servers": ["ship", "helm"],
    "active_tools": [],
  },
  "permissions": {
    "preset": "ship-standard",
    "tools_deny": ["Bash(* ~/.ship/*)", "Bash(rm -rf *)"],
    "tools_ask": ["Bash(git reset*)", "Bash(git push --force*)"],
  },
  "rules": {
    "inline": "Hold intent, dispatch specialists, gate before merge. Do not write source code — dispatch an agent.",
  },
});
