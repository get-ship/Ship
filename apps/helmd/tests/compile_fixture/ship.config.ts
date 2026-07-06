// Extension entry for the helmd Compile integration test fixture.
// Format-migration Wave 4 phase A: mcp servers + rules arrive via
// `defineMcpServer` / `defineRule` declare-forms here; the fixture's
// `.ship/mcp.jsonc` and `.ship/rules/01-style.md` are gone.

import { defineMcpServer, defineRule } from "ship";

defineMcpServer({
  id: "test-server",
  server_type: "stdio",
  command: "npx",
  args: ["-y", "@test/mcp"],
  env: {},
  url: null,
  scope: "project",
});

defineRule({
  id: "01-style",
  content: `# Style

This rule ensures the Compile RPC emits context content for harnesses that
produce a context file (CLAUDE.md / AGENTS.md).`,
  applyScope: "always",
});
