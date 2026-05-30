---
title: "The Daemon Is the Host — Deck Serves Everything, One Way"
description: "The hub is daemon-served like spaces: one bundler (deck-bundler/Rolldown), one serve path (deck), one router primitive. 'Offline boot' and 'trust isolation' are rejected as reasons to make the hub a separate static SPA — Ship is the daemon, and the daemon is the trust root."
group: ADRs
order: 51
audience: internal
status: accepted
date: 2026-05-27
supersedes: []
tags: [deck, hub, spaces, serving, bundler, tauri, architecture]
---

# The Daemon Is the Host — Deck Serves Everything, One Way

## Context

The hub (`apps/shell`) was served differently from spaces (`apps/spaces`, daemon-
served per-app windows):

- **Hub:** a pre-bundled static SPA. Built by `shell-app.ts` (a `Bun.build`
  script — a *second* bundler engine, distinct from the Rust Rolldown
  `deck-bundler`), shipped in the Tauri binary, loaded by the webview from
  `dist/` on local disk. Framework + chrome + first-party views baked in. Some
  views bundled into the hub (`SHELL_LOCAL_VIEWS`), others daemon-served — a
  split maintained by membership checks, devtools 404-guards, and dual HMR.
- **Space:** daemon-served. The Tauri window navigates to `http://<daemon>/
  space/<id>`; deck serves the scaffold + `/_sui` + data, same-origin.

This was justified by two claims (see the prior `.artifacts/serving-story.md`):
**offline boot** (the hub must launch with the daemon down) and **trust
isolation** (first-party hub unconfined, third-party spaces confined).

## Decision

**Reject both justifications. Deck serves the hub the same way it serves spaces.**

- **"Offline boot" is not a requirement.** Without the daemon, Ship does
  nothing — the daemon *is* Ship. A daemon-down state is a bug, not a supported
  mode. The correct response is a static "daemon unavailable / report a bug"
  page, **not** shipping a full offline SPA. We will not contort the bundling,
  serving, or routing architecture to make the hub runnable without its backend.
- **"Trust isolation" does not require a separate serve path.** The daemon is
  Ship's **most trusted source** — serving the hub *from* it is not a trust
  downgrade. Spaces still get per-window scoped tokens because they're
  third-party; that is **auth scoping**, not a different toolchain or serve path.

Therefore:

- **One bundler.** `deck-bundler` (Rolldown) gains an `app` build target and
  builds the hub. `shell-app.ts` (`Bun.build`) is retired. One engine for
  framework + views + hub-app + space-scaffold.
- **One serve path.** Deck serves the hub (daemon-served, same-origin) like a
  space. No separate static `dist/`, no vite in the prod path (vite is at most a
  personal dev convenience). The Tauri main window navigates to the daemon URL,
  exactly as space windows do.
- **One router primitive.** The hub stays a client-rendered SPA — that's a fine
  *rendering* choice — using the shared `@ship/sui-runtime/route`. CSR vs SSR is
  downstream of nothing; it does not imply a separate serving/bundling path.
- The `SHELL_LOCAL_VIEWS` / bundled-vs-daemon-served split inside the hub
  collapses: all views are daemon-served.

## Consequences

- **Collapses the accidental glue** (~12 of the ~20 hub/spaces decision points):
  the second bundler, the `SHELL_LOCAL_VIEWS`/`chromeSurface` split, the devtools
  source/schema 404-guards, vite-as-prod, the "bundle source port
  (embedded|cdn|local)" machinery.
- **`gen-routes` folds into deck-bundler's app build** — deck builds the hub, so
  deck emits its typed route table from the app's view tree (resolves the
  `gen-routes`-doesn't-belong-in-`scripts/` smell correctly: deck owns it by
  *building the hub*, not by the daemon serving studio).
- **Coordination:** the Tauri main-window-navigates-to-daemon change is the
  desktop/window lane (`feat/window-actors`, which owns the space/window model).
  The bundler unification (deck-bundler `app` target, retire `shell-app.ts`) is
  the framework lane and lands independently first.
- A static daemon-down page is the only offline concession.

Supersedes the hub half of `.artifacts/serving-story.md` (the trust/lifecycle
axis stays for *spaces* provenance + per-window auth; the "hub = offline static
bundle" conclusion is withdrawn).
