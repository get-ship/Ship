---
title: SDK Roadmap
audience: agent-only
description: Known gaps and in-flight work on the Ship View SDK. Agent-only — not published to the public site.
---

Honest punch list of what's missing or mid-migration in the View SDK.
Agent-facing: use this to decide whether to build around a gap, wait,
or escalate.

## Typed-signal mismatch lint

- **Done.** `signals({...})` emits `data-ship-signal-types` (string /
  int / num / bool / any). Parser reads it at lint time.
- **Blocked on.** AST-lint rule that flags assignments like
  `$count = 'hello'` against an `int` signal is not yet wired. The
  metadata is emitted but no rule consumes it.
- **Next step.** Add `signal-type-mismatch` rule to
  `crates/core/lint`, land tests against `signals({...})` fixtures,
  wire into `ship lint views` and the `lint_view` MCP tool.

## `live_topic` SSE primitive

- **Done.** `live()` and `live_persistent()` subscribe to the view's
  declared event list via `/api/view/<name>/sse`.
- **Blocked on.** A topic-filtered kernel endpoint. Today the view
  re-renders on every event in its manifest list; there's no way to
  subscribe an element to a single kernel topic for signal-only
  patches.
- **Next step.** Design `/api/sse/<topic>` with a
  `datastar-patch-signals` framer, then add a `live_topic(topic,
  signal)` macro. Do not ship the macro before the endpoint — there's
  no workaround that isn't a foot-gun.

## `form` CRUD primitive rewrite

- **Done.** `form(schema, collection)` and `table(schema, items,
  collection)` auto-generate from schema. Good enough for simple
  collections.
- **Blocked on.** A caller-migration plan. Current callers mix
  `form()` output with inline overrides; a cleaner "pass field
  overrides as a dict" API would break them.
- **Next step.** Inventory every `form(…)` call site, write the
  migration shim, then introduce `form_v2(schema, overrides={})` with
  a deprecation window on `form`.

## Raw-Datastar lint warnings

- **Done.** Lint flags `{#- SDK gap: ... -#}` comments and
  `missing-pack-declaration` / `stale-pack-declaration`.
- **Blocked on.** Nothing — small work.
- **Next step.** Add a `prefer-macro` lint rule that detects
  `data-on:click="@post(…)"`, `data-signals:*`, and
  `data-indicator="*"` and suggests the wrapping macro.

## Macro reference rendering in the public site

- **Done.** Machine-readable spec at
  `packages/ship-set/sdk/sdk-macros.json`. Human-readable Markdown at
  `GET /api/assets/jinja-macros.md`.
- **Blocked on.** The static site doesn't render the live spec — the
  [catalog](./catalog.md) page is hand-maintained against the JSON.
- **Next step.** Either (a) generate `catalog.md` as part of
  `build-jinja-docs.mjs` on every SDK regen, or (b) embed a
  `fetch('/api/sdk')` driven component into the Starlight
  page for live-updating reference. (a) is simpler.
