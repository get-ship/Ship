// Helmd Compile integration test fixture — format-migration Wave 5 + slice 2b.
// Agent `test-agent` and skill `example-skill` are declared via
// `defineAgent` (in this file) + on-disk `skills/example-skill/SKILL.md`
// (Wave 6 / slice 2b close-out). The leaf's `.ship/agents/*.jsonc` walk
// is gone; the test-agent ships via `defineAgent`. The `.ship/skills/<id>/SKILL.md`
// walk is gone too (Wave 5 + slice 2b): the skill body is on disk, the
// resolver's `merge_ts_declarations` walks `skills/` from the extension
// root. The MCP server entrypoint (test-server) is declared via
// `defineMcpServer` for the same reason — pre-Wave-5 it lived in
// `.ship/mcp.jsonc`.

import { defineAgent, defineMcpServer } from "ship";

defineMcpServer({
  id: "test-server",
  server_type: "stdio",
  command: "npx",
  args: [],
  env: {},
  url: null,
  scope: "project",
});

defineAgent({
  agent: {
    id: "test-agent",
    name: "Test Agent",
    version: "0.1.0",
    description: "An agent used by the helmd Compile integration test.",
  },
  skills: {
    refs: ["example-skill"],
  },
  mcp: {
    servers: ["test-server"],
  },
  rules: {
    inline: "Test rule body for helmd Compile integration test.",
  },
});

;
