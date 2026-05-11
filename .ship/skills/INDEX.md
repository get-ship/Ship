# Skills Index

Active impl skills bundled with this project. Each one is a single
`SKILL.md` (plus optional `references/`, `scaffolds/`, `assets/`,
`scripts/`) under `.ship/skills/<id>/` and is loaded into agent
context when its trigger matches.

For doc skills (the `docs/ship-*/` tree), see `.ship/skills/writing-docs/`.
For archived skills covering legacy stacks that still ship in the
project, see `_archive/README.md`.

## Authoring contract

Active framework contracts agents use when writing first-party code.

| Skill | One-liner |
| --- | --- |
| [`no-slop`](no-slop/SKILL.md) | Ten commandments for building software with AI agents. Hard-won, opinionated, concise. |
| [`ship-views`](ship-views/SKILL.md) | Author Ship views in SUI — `.sui` templates with optional paired `.controller.ts`, AOT-compiled, server-rendered first, hydrated by `attach()`. |
| [`ship-components`](ship-components/SKILL.md) | Author Ship components in SUI — `<sui-*>` primitives compiled AOT from `.sui` source with optional paired `.controller.ts`. |
| [`ship-design-system`](ship-design-system/SKILL.md) | Charter for Ship's design-system layer — tokens, themes, surfaces, tone ramps, motion, density. Read when editing `packages/ship-set/src/tokens.css`/`themes.css` or adding a theme. |
| [`writing-docs`](writing-docs/SKILL.md) | Write reference docs for a doc skill (`docs/ship-*/references/docs/`). Covers Markdown + ship-elements, frontmatter, audience filtering, multi-page structure. |
| [`write-adr`](write-adr/SKILL.md) | Capture a technical/product decision for future agents — context, decision, alternatives, consequences, success measure. |

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
| [`gate`](gate/SKILL.md) | Review a completed job branch against acceptance criteria — pass/fail with evidence. |

## Review

| Skill | One-liner |
| --- | --- |
| [`code-review`](code-review/SKILL.md) | Structured review for PRs and branch diffs — security, correctness, test coverage, architecture drift. |
| [`diff-review`](diff-review/SKILL.md) | Render a git diff or PR as an interactive review surface in Ship Studio's session canvas. |

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
| [`storybook`](storybook/SKILL.md) | Develop or review UI components in isolation. |
| [`vite-inspect`](vite-inspect/SKILL.md) | Inspect the Vite bundle — plugins, module graph, transform chains, build performance. |
| [`vitest-ui`](vitest-ui/SKILL.md) | Run, filter, and watch tests visually via Vitest's interactive runner. |

## Archived

`.ship/skills/_archive/` holds skills covering legacy stacks that
still ship in the project but should not be loaded by default. See
`_archive/README.md` for the table and migration pointers.

- `ship-components-lit` — Lit `<ship-*>` web components. Superseded by
  `ship-components` (`<sui-*>` SUI components).
- `ship-views-j2` — Jinja `.html.j2` views. Superseded by `ship-views`
  (`.sui` views with paired controller).
