// Extension entry for the ship repo's own source space — POINTER manifest.
// Per format-migration Wave 4 (phase A) + Wave 5 + dot-ship close-out slices
// 2b + 2c + 3 (A3+A4, 2026-07-11): rules, collections, triggers, MCP, pipelines,
// agents are NOT inlined in this file. The manifest is a directory pointer;
// each noun's source lives in a per-noun dir beside this file:
//
//   agents/<id>.ts         → defineAgent(id)    (7 files)
//   collections/<id>.ts    → defineCollection(id) (2 files)
//   skills/<id>/SKILL.md   → on-disk skill     (14 dirs, walked from
//                                             defineConfig({ skills_dir }))
//   rules/<id>.md          → on-disk rule      (7 files, walked from
//                                             defineConfig({ rules_dir }))
//   hooks/<event>.<ext>    → on-disk hook      (1 file, walked from
//                                             defineConfig({ hooks_dir }))
//
// MCP server registrations live under
// `defineConfig({ agents: { mcp: {…} } })` — slice 2c close-out.
// Named permission presets live under
// `defineConfig({ agents: { permissions: { presets: {…} } } })` —
// slice 3 close-out (A3+A4), replaces the per-preset sections of the
// dead `.ship/permissions.jsonc`.
//
// The hooks + permissions blocks don't carry per-entry content; the
// manifest is a pointer manifest, the same way skills + rules are
// on-disk files detected by the resolver's `walk_hooks_dir` + the
// `defineConfig.agents.permissions.presets` map harvested by
// `merge_ts_declarations`. See `data/blueprint/subsystems/ship-stack-format.md`
// for the manifest-nature contract; see
// `killmap-dot-ship-closeout.md` A3+A4 + `job-spec-dotship-residue.md`
// for the cutover that produced this shape.
//
// File loaded by the Phase-2 resolver from the space root
// (`space_loader::read_ship_config_ts`); see also `decision-space-scan.md`
// for the rename from `ship.ts` to `ship.config.ts`.
import { defineConfig } from "ship";

export default defineConfig({
  dependencies: {
    "github.com/better-auth/skills": "main",
  },
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
    permissions: {
      presets: {
        "ship-readonly": {
          default_mode: "plan",
          tools_allow: [
            "Read",
            "Glob",
            "Grep",
            "Bash(ship *)",
            "mcp__*__read*",
            "mcp__*__list*",
            "mcp__*__search*",
          ],
          tools_deny: [
            "Write(*)",
            "Edit(*)",
            "Bash(rm*)",
            // Platform state protection
            "Bash(rm */.ship/*)",
            "Bash(rm -r*/.ship/*)",
            "Bash(rm -rf*/.ship/*)",
          ],
        },
        "ship-standard": {
          default_mode: "default",
          tools_allow: [
            "Read",
            "Glob",
            "Grep",
            "Bash(ship *)",
          ],
          tools_deny: [
            "Write(apps/**/migrations/**/*.sql)",
            "Edit(apps/**/migrations/**/*.sql)",
            // Platform state protection
            "Bash(rm */.ship/*)",
            "Bash(rm -r*/.ship/*)",
            "Bash(rm -rf*/.ship/*)",
          ],
        },
        "ship-autonomous": {
          // Dispatched specialist agents in worktrees. Zero prompts.
          // Deny list is a guardrail, not a security boundary. Real security = worktree isolation + gate review.
          default_mode: "dontAsk",
          tools_allow: [
            "Read", "Write", "Edit", "Glob", "Grep",
            "Bash(*)",
          ],
          tools_deny: [
            "Bash(rm -rf *)",
            "Bash(git reset --hard*)",
            "Bash(git push*)",
            "Bash(cargo publish*)",
            "Bash(npm publish*)",
            "Write(apps/**/migrations/**/*.sql)",
            "Edit(apps/**/migrations/**/*.sql)",
            // Platform state protection
            "Bash(rm */.ship/*)",
            "Bash(rm -r*/.ship/*)",
            "Bash(rm -rf*/.ship/*)",
          ],
        },
        "ship-elevated": {
          // Deploy and release agents. Unlocks git push and publish.
          default_mode: "dontAsk",
          tools_allow: [
            "Read", "Write", "Edit", "Glob", "Grep",
            "Bash(*)",
          ],
          tools_deny: [
            "Bash(rm -rf *)",
            "Bash(git reset --hard*)",
            "Bash(git push --force*)",
            "Write(apps/**/migrations/**/*.sql)",
            "Edit(apps/**/migrations/**/*.sql)",
            // Platform state protection
            "Bash(rm */.ship/*)",
            "Bash(rm -r*/.ship/*)",
            "Bash(rm -rf*/.ship/*)",
          ],
        },
      },
    },
  },
});
