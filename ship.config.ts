// Extension entry for the ship repo's own source space — POINTER manifest.
// Per format-migration Wave 4 (phase A) + Wave 5 + dot-ship close-out slices
// 2b + 2c: rules, collections, triggers, MCP, pipelines, agents are NOT
// inlined in this file. The manifest is a directory pointer; each noun's
// source lives in a per-noun dir beside this file:
//
//   agents/<id>.ts         → defineAgent(id)    (7 files)
//   collections/<id>.ts    → defineCollection(id) (6 files)
//   pipelines/<id>.ts      → definePipeline(name) (6 files)
//   triggers/<id>.ts       → defineTrigger(id)  (1 file)
//   skills/<id>/SKILL.md   → on-disk skill     (15 dirs, walked from
//                                             defineConfig({ skills_dir }))
//   rules/<id>.md          → on-disk rule      (7 files, walked from
//                                             defineConfig({ rules_dir }))
//
// MCP server registrations live under `defineConfig({ agents: { mcp: {…} } })`
// — slice 2c close-out. The per-call `defineMcpServer` form died with this
// wave; the registration route is the manifest block below. Same fields
// (`id`, `server_type`, `command`, `args`, `env`, `url`, `scope`), same
// defaults, same id-collision policy (the first `defineConfig.agents.mcp.<id>`
// claim wins, mirroring the dead per-call form's merge contract).
//
// File loaded by the Phase-2 resolver from the space root
// (`space_loader::read_ship_config_ts`); see also decision-space-scan.md
// for the rename from `ship.ts` to `ship.config.ts`.
import { defineConfig } from "ship";

export default defineConfig({
  agents: {
    mcp: {
      antigravity: {
        id: "antigravity",
        server_type: "stdio",
        command: "antigravity",
        args: ["mcp"],
        env: {},
        url: null,
        scope: "project",
      },
      "rust-docs": {
        id: "rust-docs",
        server_type: "http",
        command: null,
        args: [],
        env: {},
        url: "http://127.0.0.1:7771/mcp",
        scope: "project",
      },
      "frontend-docs": {
        id: "frontend-docs",
        server_type: "http",
        command: null,
        args: [],
        env: {},
        url: "http://127.0.0.1:7772/mcp",
        scope: "project",
      },
      "mcp-docs": {
        id: "mcp-docs",
        server_type: "http",
        command: null,
        args: [],
        env: {},
        url: "http://127.0.0.1:7773/mcp",
        scope: "project",
      },
      helm: {
        id: "helm",
        server_type: "http",
        command: null,
        args: [],
        env: {},
        url: "http://localhost:3000/mcp",
        scope: "project",
      },
      github: {
        id: "github",
        server_type: "stdio",
        command: "npx",
        args: ["-y", "@modelcontextprotocol/server-github"],
        env: { GITHUB_TOKEN: "${GITHUB_TOKEN}" },
        url: null,
        scope: "project",
      },
      playwright: {
        id: "playwright",
        server_type: "stdio",
        command: "npx",
        args: ["@playwright/mcp@latest"],
        env: {},
        url: null,
        scope: "project",
      },
    },
  },
});
