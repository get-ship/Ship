# design/components/

> ⚠️ **START WITH [`HANDOFF.md`](HANDOFF.md).** A prior session built much of this
> workspace BEFORE pulling the real repo, so many `before/after` files are guesses
> and the old per-pack `INDEX.md`/`handoff.md` (now deleted) overstated completeness.
> `HANDOFF.md` + [`repo-reconciliation.md`](repo-reconciliation.md) + the real source
> in **`_repo-src/`** are the trustworthy artifacts. Invented (non-source) components
> are quarantined in [`sui/proposed/`](sui/proposed/README.md).

Design work for Ship's **component library** — mirrors the repo's
`packages/components/{sui,ship}` split so the two stay legible against each other.

```
components/
  _repo-src/  REAL imported source from job/sui-framework — GROUND TRUTH
  sui/        the framework's reusable primitives  (repo: packages/components/sui)
  sui/proposed/  components that DO NOT exist in source (gap candidates / hallucinations)
  ship/       features / engine-wrapped surfaces    (repo: packages/components/ship)
```

- **`sui/`** — atomic, generic, SDK-pure primitives (forms, data engines, canvas,
  chat, media). Zero runtime dependency → mocks render at exact fidelity.
- **`ship/`** — verticals built from `sui`, or surfaces wrapping a heavy engine
  (CodeMirror, Tiptap, pdf.js, PixiJS). Designed once `sui` underneath is settled.

> Shared catalog tooling lives at **`/tools/catalog/`** (not here) — it's a tool,
> not a mock. Design tokens come from **`/sui/ship.css`** (the built runtime; note
> the bundled copy may be stale vs the branch — see HANDOFF §1).
