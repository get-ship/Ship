# apps/shell — Lit/Datastar/Jinja chrome

Frozen as of F1's mount-unification cleanup (see
`.artifacts/design-f1-mount-unification.md`). This shell hosts the
legacy Lit-based chrome — top bar, status bar, side rail, command
palette — and the Lit/Datastar/Jinja-driven views (agents, git,
pipelines, etc.) that haven't been ported to SUI yet.

**No SUI plumbing lives here.** The studio (`/ship-ui-studio`) used
to be the one SUI surface hosted by this shell; F1.6 removed it. SUI
views are accessed standalone at `/sui/<name>` via the daemon's
`view-host` static page until shell 2.0 ports them in.

**Status: maintenance-only.** Bug fixes welcome; no new features.
Anything net-new — new chrome, new views, new components — should
land in shell 2.0 (`apps/shell2/`, not yet created at the time of
this writing) where SUI is the canonical paradigm.

When shell 2.0 reaches feature parity, this directory is deleted in
one commit, along with `lit`, `datastar`, and the Jinja templating
deps it carries.

## Run

```bash
just shell-dev   # starts vite at http://127.0.0.1:3100/
```

## Layout

- `index.html` — boots `ship-elements.js` + `datastar.js` from the
  daemon at `/api/assets/*` (proxied same-origin via Vite in dev)
- `src/mount.ts` — `mountDaemonFragment` primitive (+ `openDaemonSSE`)
- `src/main.ts` — Navigo router, command palette, view mounting

Bundles are built by `pnpm build:bundles` (root) before `cargo build`.
