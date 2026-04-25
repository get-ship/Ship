---
title: "Setup View"
description: "First-run wizard view served at /setup. Four steps — welcome, workspace, provider, done — gated by workspace existence."
sidebar:
  label: "Setup View"
  order: 5
---
The setup view is the browser-side first-run wizard. The daemon serves it at `GET /setup` when no workspace exists; once any workspace is created, `/setup` redirects to `/shell/` (the Studio dashboard) so the wizard cannot accidentally re-prompt or wipe state.

The view lives at `.ship/views/setup/` and is rendered by `apps/shipd/src/views/setup.rs`. The file structure:

```
.ship/views/setup/
├── index.html.j2           # frame + step container
├── manifest.yaml           # name, type
└── partials/
    ├── step-welcome.html.j2
    ├── step-workspace.html.j2
    ├── step-provider.html.j2
    └── step-done.html.j2
```

State is purely client-side Datastar signals — there is no SSE channel or stream. The frame declares signals (`$step`, `$ws_mode`, `$ws_name`, `$ws_path`, `$ws_error`, `$provider`, `$api_key`, `$provider_test_result`, `$provider_test_error`); each step reads or writes them. Mutations that must hit the server use `@post` and patch response signals back into the form.

## Step 1 — Welcome

Static introduction. Sets expectations: Ship runs locally, the next two steps pick a project folder and a provider. One button advances `$step` to 2.

## Step 2 — Pick a project folder

Segmented control with two modes:

- **Create new project** — captures `$ws_name`; default location `~/Documents/Ship/<name>`.
- **Open existing project** — captures `$ws_path`; folder must exist and not already be a Ship workspace.

The Continue button optimistically advances `$step` to 3 and posts `{ mode, name, path }` to `/api/projects`. On error the response patches `$ws_error` and an inline `<ship-alert>` displays the message; on success the workspace is registered and any later `GET /setup` will redirect to `/`.

## Step 3 — Connect an AI provider

Four interactive provider cards (`claude`, `gemini`, `codex`, `opencode`) act as a radio group bound to `$provider`. Selecting a provider reveals an API-key input and a Test connection button.

The Test button posts `{ provider, key }` to `/api/providers/test` with `$provider_test_result = 'testing'`. On the response:

- HTTP 2xx — `$provider_test_result = 'ok'`; the green Connected badge renders and the Save & continue button enables.
- Non-2xx — `$provider_test_result = 'failed'`; `$provider_test_error` carries the message and the destructive badge renders.

Save & continue advances to step 4 and posts `{ provider, key }` to `/api/settings`, which writes to `~/.ship/library/settings.jsonc`. A Skip, configure later button bypasses provider selection entirely; users can configure providers from the settings view at any time.

## Step 4 — Done

Three resource cards link to docs, community, and the registry store at `getship.dev`. The Open Ship button posts `setup.completed = true` to `/api/settings` and navigates to `/`, where the daemon now redirects to the Studio shell.

## Provider selection is asked exactly once

The wizard's provider step is the only place the canonical first-run path prompts for a provider. `ship init` accepts `--provider <id>` as a CLI flag and defaults to `claude` non-interactively; it does not prompt. Once `ship init` (or the wizard's step 2) creates a workspace, `/setup` becomes unreachable for the rest of the daemon's lifetime, so no second prompt can fire.

If you need to change provider preferences after first run, use the settings view at `/views/settings/` or edit `~/.ship/library/settings.jsonc` directly.

## Routing rules

| Route | When no workspace exists | When a workspace exists |
|-------|--------------------------|-------------------------|
| `GET /` | 303 → `/setup` | 303 → `/shell/` |
| `GET /setup` | 200 (wizard HTML) | 303 → `/` |

`/setup` is unauthenticated by design — a freshly installed daemon has not yet paired with any browser or CLI session, so the wizard must be reachable without a token. Once setup completes and the wizard is gated behind a workspace, all other routes require the daemon token.
