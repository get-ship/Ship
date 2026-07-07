// Helmd Compile integration test fixture — format-migration Wave 5.
// Source authoring for `test-agent` and `example-skill` arrives via
// `defineAgent` + `defineSkill` declare-forms here, replacing the
// pre-Wave-5 `.ship/agents/test-agent.jsonc` and
// `.ship/skills/example-skill/SKILL.md` files. Per
// `census-format-migration.md` §4 row 3 (last paragraph), the leaf's
// `.ship/agents/*.jsonc` and `.ship/skills/<id>/SKILL.md` walks are
// gone; the only source-authoring surface for those nouns is
// `ship.config.ts`. The MCP server entrypoint (test-server) is
// declared via `defineMcpServer` for the same reason — pre-Wave-5 it
// lived in `.ship/mcp.jsonc`.

import { defineAgent, defineMcpServer, defineSkill } from "ship";

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

defineSkill({
  frontmatter: {
    "name": "example-skill",
    "description": "Example skill for the helmd Compile integration test.",
  },
  body: "# Example Skill\n\nThis skill exists to verify the Compile RPC reads SKILL.md and emits a\n`.claude/skills/example-skill/SKILL.md` file in the response.\n",
});
