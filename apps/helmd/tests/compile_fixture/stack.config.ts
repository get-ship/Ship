// Helmd Compile integration test fixture — format-migration Wave 5 +
// slice 2b + slice 2c close-out + rules-format cut.
//
// Slice 2c: skills live on disk at `skills/<id>/SKILL.md`; agents live
// in per-noun source dir `agents/<id>.ts`; MCP server registrations
// live under `defineConfig({ agents: { mcp: { ... } } })` in the
// manifest. The `defineSkill` / `defineMcpServer` declare-forms are
// retired (slice 2c).
//
// Rules-format cut: the on-disk `rules/<id>.md` convention died.
// Rule bodies live on the agent composition as
// `rules: { inline: "..." }` — `agents/test-agent.ts` carries the
// rule body for this fixture.

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
