# Skills Index

Active impl skills bundled with this project. Each one is a single
`SKILL.md` (plus optional `references/`, `scaffolds/`, `assets/`,
`scripts/`) under `.ship/skills/<id>/` and is loaded into agent
context when its trigger matches.

For doc skills (the `docs/ship-*/` tree), see `.ship/skills/writing-docs/`.
The conceptual/onboarding doc skills remain (`ship-concepts`,
`ship-architecture`, `ship-contributing`, `ship-agents`, `ship-skills`);
the code-mirroring ones (kernel, runtime, http-api, compiler, registry,
collections, artifacts, pipelines, triggers, schema/cli reference, sui,
themes, stack) were retired — that truth now lives in per-crate READMEs +
code comments. The convention for writing it is `.ship/skills/ship-docs/`.
Their published reference pages under `docs/ship-*/references/docs/` are
untouched and still feed the docs site. New agents start from
`.ship/skills/ship-onboarding/`.

Publish-ready, generic-purpose skills live in the top-level `skills/`
directory — those load when their trigger fires regardless of which
project the agent runs in. Current publish set:
`collection-authoring`, `ship-canvas`, `ship-components`,
`ship-design-system`, `ship-views`, `tdd`, `web-qa`. The
`trigger-authoring` skill used to live there too but moved back to
`.ship/skills/` because Ship triggers aren't launch-ready yet.

## Grounding

Read first when new to the codebase.

| Skill | One-liner |
| --- | --- |
| [`ship-onboarding`](ship-onboarding/SKILL.md) | What Ship is — the four-layer model, the scope model (Space/Account/Runtime, ownership=sync), fat-substrate/thin-daemon, and where to go next. |
| [`memento`](memento/SKILL.md) | (experimental) Record durable decision records so future agents don't relitigate. Local sqlite prototype; product home is a Space-scoped `decisions` collection. |

## Authoring contract

Active framework contracts agents use when writing first-party code.

| Skill | One-liner |
| --- | --- |
| [`no-slop`](no-slop/SKILL.md) | Ten commandments for building software with AI agents. Hard-won, opinionated, concise. |
| [`ship-views`](../../skills/ship-views/SKILL.md) | (publish-ready) Author Ship views in SUI — `.sui` templates with optional paired `.controller.ts`, AOT-compiled, server-rendered first, hydrated by `attach()`. |
| [`ship-components`](../../skills/ship-components/SKILL.md) | (publish-ready) Author Ship components in SUI — `<sui-*>` primitives compiled AOT from `.sui` source with optional paired `.controller.ts`. |
| [`ship-design-system`](../../skills/ship-design-system/SKILL.md) | (publish-ready) Charter for Ship's design-system layer — tokens, themes, surfaces, tone ramps, motion, density. Read when editing `/home/madvib/dev/sui//home/madvib/dev/sui/packages/sui/tokens/src/tokens.css`/`themes.css` or adding a theme. |
| [`writing-docs`](writing-docs/SKILL.md) | Write reference docs for a doc skill (`docs/ship-*/references/docs/`). Covers Markdown + ship-elements, frontmatter, audience filtering, multi-page structure. |
| [`ship-docs`](ship-docs/SKILL.md) | The documentation convention — comment density, what a per-crate README holds, and when to use README vs ADR vs memento vs nothing. |
| [`trigger-authoring`](trigger-authoring/SKILL.md) | Author Ship triggers — JSONC files in `.ship/triggers/*.jsonc` that match a Ship event and run actions. Not yet launch-ready; lives here until the surface stabilises. |

## Orchestration

Commander loop and dispatch — how work gets planned, fanned out, and
gated.

| Skill | One-liner |
| --- | --- |
| [`cmdr`](cmdr/SKILL.md) | Ship Commander — captures intent, dispatches specialist agents, monitors via mesh, gates before merge. |
| [`spawn-agent`](spawn-agent/SKILL.md) | Dispatch a job to a specialist agent in a git worktree — write spec, create worktree, launch agent. |
| [`configure-agent`](configure-agent/SKILL.md) | Pick the agent profile, permission preset, scope, and worktree path when standing up a workspace. |
| [`setup-workspace`](setup-workspace/SKILL.md) | Configure a multi-agent terminal pod from `.artifacts/pod.md` — one tab per agent in its worktree. |
| [`gate`](gate/SKILL.md) | Review a completed job branch against acceptance criteria — pass/fail with evidence. Now blocks on missing `web-qa` artifacts for any branch touching UI surfaces. |
| [`sui-eval`](sui-eval/SKILL.md) | Run the SUI authoring eval — dispatch lane agents to build a view, poll until they commit, grade compile + lint + structure, record to `sui-eval-runs`. Off-daemon (dispatch.sh, not pipelines). |

## Review

| Skill | One-liner |
| --- | --- |
| [`code-review`](code-review/SKILL.md) | Structured review for PRs and branch diffs — security, correctness, test coverage, architecture drift. |
| [`diff-review`](diff-review/SKILL.md) | Render a git diff or PR as an interactive review surface in Ship Studio's session canvas. |
| [`web-qa`](../../skills/web-qa/SKILL.md) | **Gate-required** for UI work. Every interactive element pressed, multi-theme + multi-viewport, console + network captured, visibility asserted (not just DOM-presence). Output to `.artifacts/web-qa/<surface>/<timestamp>/`. Lives in the publish-ready `skills/` tree; loaded by commander, web-lane, ship-designer, gatekeeper. |

## Dev tools (session-rendered)

Each starts a local server and surfaces its UI in the Ship Studio
session canvas via a `.url` artifact.

| Skill | One-liner |
| --- | --- |
| [`vite-inspect`](vite-inspect/SKILL.md) | Inspect the Vite bundle — plugins, module graph, transform chains, build performance. |
