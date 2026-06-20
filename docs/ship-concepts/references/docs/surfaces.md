# Surfaces: apps, views, pages

The vocabulary for what Ship renders. The distinction that matters: **apps and
views are the same shape — both are made of pages — but differ in who authors
them, whether they're trusted, and when they're built.**

## The terms

- **Component** — a reusable custom element (`<sui-button>`, `<ship-nav-top>`).
  The building block. Compiled into bundles; used by both apps and views.

- **Page** — one routed surface: a `page.sui` template + optional
  `controller.ts`, located by file-based routing
  (`routes/.../[param]/page.sui`). The *unit*. Both apps and views are
  collections of pages.

- **App** — a *product* application (the shell, the hub). Ship-team-authored,
  **built** into the product bundle, **trusted**, not runtime-editable. The
  envelope the user runs.

- **View** — a user/agent-authored **applet**: its own pages/routes, **loaded
  and compiled at runtime** in a space, **HMR-editable** live, **shareable**
  across spaces, **publishable** to / downloadable from the store. **Untrusted**
  — agent-authored code is at least as risky as store-downloaded. The content
  the user creates inside the envelope.

- **Route** — a URL → page mapping, resolved by deck's router. Apps and views
  are both routed; the route table is static (built) for an app, dynamic (per
  space) for views.

- **Space** — a user's workspace: the container a user's views, data, and config
  live in. Served at `/space/<id>`.

- **Store** — the registry where views (and the component packs they depend on)
  are published, shared, and downloaded between spaces.

## Who owns what

- **sui** — compiles `.sui` and renders (signals, bindings, components,
  mount/hydrate). Knows nothing above it; no app/route/ship words in the compiler.
- **deck** — the meta-framework on sui: routing, bundling, the view-host, and
  **the views feature itself** (load / serve / HMR / share / publish). Knows
  sui; knows nothing about ship.
- **ship** — the platform: spaces, the store, auth, the desktop (Tauri), and the
  apps (shell, hub). Authors apps; distributes views.

So: **a page is the unit; an app or a view is a collection of pages; deck makes
views a runtime feature; ship is the platform that ships the apps and
distributes the views.**

## Status (so this doc doesn't lie)

The *mechanics* are real today: file-based routing, on-demand compile, HMR. The
*product surface* of views-as-distributable-applets — cross-space sharing, store
publish/download, the untrusted-view sandbox — is the target, largely not yet
built. Apps (shell/hub) exist; runtime user/agent views exist as plumbing but
aren't launch-hardened.
