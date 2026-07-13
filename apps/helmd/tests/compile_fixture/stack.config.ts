// Helmd Compile integration test fixture — format-migration Wave 5 + slice
// 2b + slice 2c close-out.
//
// Slice 2c: skills live on disk at `skills/<id>/SKILL.md`; agents live
// in per-noun source dir `agents/<id>.ts`; rules live on disk at
// `rules/<id>.md`; MCP server registrations live under
// `defineConfig({ agents: { mcp: { ... } } })` in the manifest. The
// `defineSkill` / `defineRule` / `defineMcpServer` declare-forms are
// retired (slice 2c).

import { defineConfig } from "ship";

export default defineConfig({
  agents: {
    mcp: {
      "test-server": {
        id: "test-server",
        server_type: "stdio",
        command: "npx",
        args: [],
        env: {},
        url: null,
        scope: "project",
      },
    },
  },
});
