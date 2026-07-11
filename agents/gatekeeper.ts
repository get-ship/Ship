// agents/gatekeeper.ts — per-job source dir for the `gatekeeper` agent.
// Migrated from `ship.config.ts`'s inline `defineAgent` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; no field changes.
import { defineAgent } from "ship";

defineAgent({
  "agent": {
    "id": "gatekeeper",
    "name": "Gatekeeper",
    "icon": "shield-check",
    "version": "0.2.0",
    "description": "Gates job branches against acceptance criteria and reviews PRs for security, correctness, and architecture drift.",
    "tags": ["gate", "review", "security", "pr", "code-quality"],
  },
  "skills": {
    "refs": ["gate", "code-review", "web-qa"],
  },
  "mcp": {
    "servers": ["ship"],
  },
  "permissions": {
    "preset": "ship-standard",
    "tools_deny": ["Bash(* ~/.ship/*)", "Bash(rm -rf *)"],
    "tools_ask": ["Bash(git push*)", "Bash(git reset*)"],
  },
  "rules": {
    "inline": "Gate job branches or review PRs. Never self-approve. Evidence required. Don't fix — return the verdict.",
  },
});
