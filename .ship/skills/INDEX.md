# Skills Index

Active impl skills bundled with this project. Each one is a single
`SKILL.md` (plus optional `references/`, `scaffolds/`, `assets/`,
`scripts/`) under `.ship/skills/<id>/` and is loaded into agent
context when its trigger matches.

For doc skills (the `docs/ship-*/` tree), see `.ship/skills/writing-docs/`.

Publish-ready, generic-purpose skills live in the top-level `skills/`
directory — those load when their trigger fires regardless of which
project the agent runs in. Current publish set:
`collection-authoring`, `ship-canvas`, `ship-components`,
`ship-design-system`, `ship-views`, `tdd`, `web-qa`. The
`trigger-authoring` skill used to live there too but moved back to
`.ship/skills/` because Ship triggers aren't launch-ready yet.

## Authoring contract

Active framework contracts agents use when writing first-party code.

| Skill | One-liner |
| --- | --- |
| [`no-slop`](no-slop/SKILL.md) | Ten commandments for building software with AI agents. Hard-won, opinionated, concise. |
| [`ship-views`](../../skills/ship-views/SKILL.md) | (publish-ready) Author Ship views in SUI — `.sui` templates with optional paired `.controller.ts`, AOT-compiled, server-rendered first, hydrated by `attach()`. |
| [`ship-components`](../../skills/ship-components/SKILL.md) | (publish-ready) Author Ship components in SUI — `<sui-*>` primitives compiled AOT from `.sui` source with optional paired `.controller.ts`. |
| [`ship-design-system`](../../skills/ship-design-system/SKILL.md) | (publish-ready) Charter for Ship's design-system layer — tokens, themes, surfaces, tone ramps, motion, density. Read when editing `packages/ui-components/src/tokens/tokens.css`/`themes.css` or adding a theme. |
| [`writing-docs`](writing-docs/SKILL.md) | Write reference docs for a doc skill (`docs/ship-*/references/docs/`). Covers Markdown + ship-elements, frontmatter, audience filtering, multi-page structure. |
| [`write-adr`](write-adr/SKILL.md) | Capture a technical/product decision for future agents — context, decision, alternatives, consequences, success measure. |
| [`trigger-authoring`](trigger-authoring/SKILL.md) | Author Ship triggers — JSONC files in `.ship/triggers/*.jsonc` that match a Ship event and run actions. Not yet launch-ready; lives here until the surface stabilises. |

## Orchestration

Commander loop and dispatch — how work gets planned, fanned out, and
gated.

| Skill | One-liner |
| --- | --- |
| [`cmdr`](cmdr/SKILL.md) | Ship Commander — captures intent, dispatches specialist agents, monitors via mesh, gates before merge. |
| [`plan`](plan/SKILL.md) | Author structured job specs in `.artifacts/specs/` that the daemon can launch as jobs. |
| [`spawn-agent`](spawn-agent/SKILL.md) | Dispatch a job to a specialist agent in a git worktree — write spec, create worktree, launch agent. |
| [`configure-agent`](configure-agent/SKILL.md) | Pick the agent profile, permission preset, scope, and worktree path when standing up a workspace. |
| [`setup-workspace`](setup-workspace/SKILL.md) | Configure a multi-agent terminal pod from `.artifacts/pod.md` — one tab per agent in its worktree. |
| [`gate`](gate/SKILL.md) | Review a completed job branch against acceptance criteria — pass/fail with evidence. Now blocks on missing `web-qa` artifacts for any branch touching UI surfaces. |

## Review

| Skill | One-liner |
| --- | --- |
| [`code-review`](code-review/SKILL.md) | Structured review for PRs and branch diffs — security, correctness, test coverage, architecture drift. |
| [`diff-review`](diff-review/SKILL.md) | Render a git diff or PR as an interactive review surface in Ship Studio's session canvas. |
| [`web-qa`](../../skills/web-qa/SKILL.md) | **Gate-required** for UI work. Every interactive element pressed, multi-theme + multi-viewport, console + network captured, visibility asserted (not just DOM-presence). Output to `.artifacts/web-qa/<surface>/<timestamp>/`. Lives in the publish-ready `skills/` tree; loaded by commander, web-lane, ship-designer, gatekeeper. |

## Workflow

| Skill | One-liner |
| --- | --- |
| [`design-workflow`](design-workflow/SKILL.md) | Design pipeline — brainstorm HTML mockup, preview in browser, extract spec, dispatch implementation. |

## Dev tools (session-rendered)

Each starts a local server and surfaces its UI in the Ship Studio
session canvas via a `.url` artifact.

| Skill | One-liner |
| --- | --- |
| [`drizzle-studio`](drizzle-studio/SKILL.md) | Browse, query, and edit the database visually via Drizzle Studio. |
| [`mcp-inspector`](mcp-inspector/SKILL.md) | Debug MCP servers — launch the Inspector UI, connect to any server, watch SSE notifications. |
| [`vite-inspect`](vite-inspect/SKILL.md) | Inspect the Vite bundle — plugins, module graph, transform chains, build performance. |
| [`vitest-ui`](vitest-ui/SKILL.md) | Run, filter, and watch tests visually via Vitest's interactive runner. |
