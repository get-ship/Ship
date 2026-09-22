// Extension entry for the ship repo's own source space — POINTER manifest.
// Per format-migration Wave 4 (phase A) + Wave 5 + dot-ship close-out slices
// 2b + 2c + 3 (A3+A4, 2026-07-11): rules, collections, triggers, MCP, pipelines,
// agents are NOT inlined in this file. The manifest is a directory pointer;
// each noun's source lives in a per-noun dir beside this file:
//
//   agents/<id>.jsonc      → AgentProfile       (shipwright; A5 JSONC)
//   skills/<id>/SKILL.md   → on-disk skill     (authoring + frontend skills)
//   hooks/<event>.<ext>    → on-disk hook      (1 file)
//
// The dev MCP registrations, ship-* permission presets, internal
// agents, workflow skills, and rules live in the ship-meta stack
// (a peer member of the dev space) — this stack keeps only
// publish-intended content.
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
});
